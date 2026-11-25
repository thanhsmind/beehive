use crate::jsjson;
use crate::session_identity::locate_caller;
use crate::verbs::record_timing;
use crate::verbs::reservations::{keys_known, parse_flags, FlagV, Flags};
use crate::verbs::state_group::{
    acquire_state_lock, ensure_workflow_record_for_feature, start_lane, validate_route_set_flags,
};
use crate::verbs::status_full::find_granted_worktree_for_feature;
use crate::verbs::workflow_store::{
    acquire_named_lock, find_live_workflow, lane_lock_name, list_lanes, list_workflows,
    read_lane_strict, rebuild_lane_projection, update_workflow_assuming_lock, wf_id, write_lane,
};
use crate::verbs::worktree::{
    build_instruction, build_session_transition, check_shared_nested_checkout_guard,
    create_feature_worktree, emit_pi_transition_marker_if_present, feature_slug_ok, p, prelude,
    CErr, CompanionSpec, Pre,
};
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

const DEFAULT_WIP_LIMIT: usize = 5;

const WIP_OPEN_PHASES: &[&str] = &[
    "exploring",
    "planning",
    "swarming",
    "compounding",
    "scribing",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct WipWarning {
    count: usize,
    limit: usize,
    oldest: Vec<String>,
}

fn wip_limit_from_config(config: &Map<String, Value>) -> usize {
    match config.get("wip_limit") {
        Some(Value::Number(n)) => n
            .as_u64()
            .and_then(|v| usize::try_from(v).ok())
            .unwrap_or(DEFAULT_WIP_LIMIT),
        _ => DEFAULT_WIP_LIMIT,
    }
}

fn evaluate_wip_warning(
    lanes: &[Map<String, Value>],
    started_feature: &str,
    config: &Map<String, Value>,
) -> Option<WipWarning> {
    let limit = wip_limit_from_config(config);
    if limit == 0 {
        return None;
    }

    let mut open: Vec<(&str, Option<&str>)> = Vec::new();
    let mut seen = HashSet::new();
    for lane in lanes {
        let Some(feature) = lane.get("feature").and_then(Value::as_str) else {
            continue;
        };
        if feature == started_feature || !seen.insert(feature) {
            continue;
        }
        let Some(phase) = lane.get("phase").and_then(Value::as_str) else {
            continue;
        };
        if !WIP_OPEN_PHASES.contains(&phase) {
            continue;
        }
        if matches!(lane.get("status").and_then(Value::as_str), Some("closed")) {
            continue;
        }
        let created_at = lane.get("created_at").and_then(Value::as_str);
        open.push((feature, created_at));
    }

    let count = open.len();
    if count < limit {
        return None;
    }

    open.sort_by(|(feat_a, ts_a), (feat_b, ts_b)| {
        (ts_a.is_none(), ts_a.unwrap_or(""), *feat_a)
            .cmp(&(ts_b.is_none(), ts_b.unwrap_or(""), *feat_b))
    });

    let oldest = open
        .into_iter()
        .take(3)
        .map(|(feat, _)| feat.to_string())
        .collect();

    Some(WipWarning {
        count,
        limit,
        oldest,
    })
}

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
    if !keys_known(
        &flags,
        &[
            "feature",
            "class",
            "lane",
            "flags",
            "files",
            "same-session",
            "handoff-task",
        ],
    ) {
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
    let mut waiver: Option<String> = None;
    if let Ok(ctrl_root) = crate::verbs::reservations::control_root_for(&main_root_s) {
        let session_id = crate::verbs::reservations::resolve_session_id(None, &ctrl_root).ok().flatten();
        if let Some(sid) = session_id.as_deref() {
            if let Some(merged) = crate::integration_queue::merged_feature_of(Path::new(&ctrl_root), sid)
                && merged != feature
            {
                let config = crate::state::read_config_raw(&main_root);
                let handoff_auto = !matches!(config.get("handoff_auto"), Some(Value::Bool(false)));
                let door = if handoff_auto {
                    crate::verbs::handoff_door::probe_door(&main_root)
                } else {
                    crate::verbs::handoff_door::HandoffDoor::Absent
                };

                if handoff_auto
                    && matches!(door, crate::verbs::handoff_door::HandoffDoor::Present { .. })
                {
                    if flags.get("same-session").is_some() {
                        return Some(ctx.fail(
                            "[handoff_due] --same-session is not accepted while bee can hand off",
                        ));
                    }
                    let owner_text = match flags.get("handoff-task") {
                        Some(FlagV::S(t)) => t.trim().to_string(),
                        _ => String::new(),
                    };
                    if owner_text.is_empty() {
                        return Some(ctx.fail(&format!(
                            "[handoff_due] this session already merged \"{merged}\"; pass --handoff-task \"<owner request and standing orders>\" so bee can hand off \"{feature}\" to a fresh leader session."
                        )));
                    }
                    let flag_str = |k: &str| match flags.get(k) {
                        Some(FlagV::S(s)) => Some(s.as_str()),
                        _ => None,
                    };
                    match crate::verbs::handoff_door::execute_handoff(
                        &door,
                        &main_root,
                        sid,
                        &crate::verbs::handoff_door::HandoffReason::MergedFeature {
                            merged: merged.clone(),
                        },
                        &feature,
                        flag_str("class"),
                        flag_str("lane"),
                        flag_str("files"),
                        flag_str("flags"),
                        &[],
                        &owner_text,
                        None,
                        &[],
                    ) {
                        crate::verbs::handoff_door::HandoffOutcome::Sent { run_id, request_id } => {
                            let _ = crate::verbs::state_group::release_session_record(
                                &main_root, sid,
                            );
                            let note = format!("run {run_id} request {request_id}");
                            if let Err(e) = crate::audit::append(
                                &main_root,
                                "handoff-dispatched",
                                &feature,
                                None,
                                &note,
                                "feature start",
                                None,
                            ) {
                                eprintln!("warning: could not write the handoff-dispatched audit row: {e}");
                            }
                            return Some(ctx.fail(&format!(
                                "[handoff_sent] run {run_id}: this session is released; tell the owner in one line and stop."
                            )));
                        }
                        crate::verbs::handoff_door::HandoffOutcome::Deduped { run_id, status } => {
                            return Some(ctx.fail(&format!(
                                "[handoff_deduped] run {run_id} was already {status}; nothing new was dispatched — rerun bee state handoff send"
                            )));
                        }
                        crate::verbs::handoff_door::HandoffOutcome::Failed { reason } => {
                            return Some(ctx.fail(&format!("[handoff_failed] {reason}")));
                        }
                    }
                }

                match flags.get("same-session") {
                    None => {
                        return Some(ctx.fail(&format!(
                            "[handoff_due] this session already merged \"{merged}\"; write the handoff and start \"{feature}\" in a fresh leader session (playbook: .bee/playbooks/handoff.md), or pass --same-session \"<reason>\" to open it here anyway."
                        )));
                    }
                    Some(v) => {
                        let reason = match v {
                            FlagV::S(r) => r.trim().to_string(),
                            FlagV::Present => String::new(),
                        };
                        if reason.is_empty() {
                            return Some(ctx.fail(
                                "[handoff_due] --same-session needs a non-empty reason: --same-session \"<reason>\".",
                            ));
                        }
                        waiver = Some(reason);
                    }
                }
            } else if !crate::hooks::herding_worker_marker_set() {
                let config = crate::state::read_config_raw(&main_root);
                let handoff_auto = !matches!(config.get("handoff_auto"), Some(Value::Bool(false)));
                if handoff_auto {
                    let session_file = Path::new(&ctrl_root)
                        .join(".bee")
                        .join("sessions")
                        .join(format!("{sid}.json"));
                    if let crate::fsutil::ReadJson::Parsed(Value::Object(session_record)) =
                        crate::fsutil::read_json(&session_file)
                        && let Some(due) =
                            crate::verbs::handoff_door::context_mark_due(&config, &session_record)
                    {
                        let door = crate::verbs::handoff_door::probe_door(&main_root);
                        if matches!(door, crate::verbs::handoff_door::HandoffDoor::Present { .. }) {
                            let owner_text = match flags.get("handoff-task") {
                                Some(FlagV::S(t)) => t.trim().to_string(),
                                _ => String::new(),
                            };
                            if owner_text.is_empty() {
                                let n_k = due.tokens / 1000;
                                let m_k = due.mark / 1000;
                                return Some(ctx.fail(&format!(
                                    "[handoff_due] context {n_k}k is past the {m_k}k mark; run bee state handoff send --task \"<owner request and standing orders>\""
                                )));
                            }
                            let flag_str = |k: &str| match flags.get(k) {
                                Some(FlagV::S(s)) => Some(s.as_str()),
                                _ => None,
                            };
                            match crate::verbs::handoff_door::execute_handoff(
                                &door,
                                &main_root,
                                sid,
                                &crate::verbs::handoff_door::HandoffReason::ContextMark {
                                    tokens: due.tokens,
                                    mark: due.mark,
                                },
                                &feature,
                                flag_str("class"),
                                flag_str("lane"),
                                flag_str("files"),
                                flag_str("flags"),
                                &[],
                                &owner_text,
                                None,
                                &[],
                            ) {
                                crate::verbs::handoff_door::HandoffOutcome::Sent {
                                    run_id,
                                    request_id,
                                } => {
                                    let _ = crate::verbs::state_group::release_session_record(
                                        &main_root, sid,
                                    );
                                    let note = format!("run {run_id} request {request_id}");
                                    if let Err(e) = crate::audit::append(
                                        &main_root,
                                        "handoff-dispatched",
                                        &feature,
                                        None,
                                        &note,
                                        "feature start",
                                        None,
                                    ) {
                                        eprintln!("warning: could not write the handoff-dispatched audit row: {e}");
                                    }
                                    return Some(ctx.fail(&format!(
                                        "[handoff_sent] run {run_id}: this session is released; tell the owner in one line and stop."
                                    )));
                                }
                                crate::verbs::handoff_door::HandoffOutcome::Deduped {
                                    run_id,
                                    status,
                                } => {
                                    return Some(ctx.fail(&format!(
                                        "[handoff_deduped] run {run_id} was already {status}; nothing new was dispatched — rerun bee state handoff send"
                                    )));
                                }
                                crate::verbs::handoff_door::HandoffOutcome::Failed { reason } => {
                                    return Some(ctx.fail(&format!("[handoff_failed] {reason}")));
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Err(msg) = check_shared_nested_checkout_guard(
            &main_root,
            &ctrl_root,
            session_id.as_deref(),
            &feature,
        ) {
            return Some(ctx.fail(&msg));
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
        Err(CErr::Ex) => {
            let msg = lock_busy?;
            return Some(ctx.fail(&msg))
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

    if let Some(reason) = waiver.as_deref()
        && let Err(e) = crate::audit::append(&main_root, "handoff-waived", &feature, None, reason, "feature start", None)
    {
        eprintln!("warning: could not write the handoff-waived audit row: {e}");
    }

    if let Ok(live) = list_workflows(&main_root)
        && let Some(wf) = find_live_workflow(&live, &feature) {
            let mut patch = Map::new();
            patch.insert("route".into(), Value::Object(route_object.clone()));
            let _ = update_workflow_assuming_lock(&main_root, &wf_id(wf), patch);
        }

    let config = crate::state::read_config_raw(&main_root);
    let lanes = list_lanes(&main_root).unwrap_or_default();
    let wip_warning = evaluate_wip_warning(&lanes, &feature, &config);

    if use_json {
        let mut out = json!({
            "worktree": p(&created.worktree_root),
            "instruction": instruction,
        });
        if let Some(ref w) = wip_warning
            && let Some(obj) = out.as_object_mut()
        {
            obj.insert(
                "wip_warning".into(),
                json!({
                    "count": w.count,
                    "limit": w.limit,
                    "oldest": w.oldest,
                }),
            );
        }
        println!("{}", jsjson::stringify(&out));
    } else {
        if let Some(ref w) = wip_warning {
            eprintln!(
                "warning: {} open features (wip_limit {}); oldest: {}",
                w.count,
                w.limit,
                w.oldest.join(", ")
            );
        }
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
    if lane_path.exists()
        && let Ok(content) = std::fs::read_to_string(&lane_path)
            && let Ok(val) = serde_json::from_str::<Value>(&content)
                && let Some(old_phase) = val.get("phase").and_then(|p| p.as_str())
                    && (old_phase == "exploring" || old_phase == "planning") {
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

    struct SessionEnv {
        prior: Vec<(&'static str, Option<std::ffi::OsString>)>,
    }

    impl SessionEnv {
        fn set(id: Option<&str>) -> Self {
            let prior = crate::session_identity::SESSION_ENV_VARS
                .iter()
                .map(|k| (*k, std::env::var_os(k)))
                .collect();
            unsafe {
                for k in crate::session_identity::SESSION_ENV_VARS {
                    std::env::remove_var(k);
                }
                if let Some(id) = id {
                    std::env::set_var("BEE_SESSION_ID", id);
                }
            }
            Self { prior }
        }
    }

    impl Drop for SessionEnv {
        fn drop(&mut self) {
            unsafe {
                for (k, v) in &self.prior {
                    match v {
                        Some(v) => std::env::set_var(k, v),
                        None => std::env::remove_var(k),
                    }
                }
            }
        }
    }

    fn start_argv(feature: &str, same_session: Option<&str>) -> Vec<OsString> {
        let mut argv: Vec<OsString> = ["feature", "start", "--feature", feature, "--class", "feature", "--lane", "standard", "--files", "3"]
            .iter()
            .map(OsString::from)
            .collect();
        if let Some(reason) = same_session {
            argv.push(OsString::from("--same-session"));
            argv.push(OsString::from(reason));
        }
        argv
    }

    fn audit_rows(repo: &Path) -> Vec<Value> {
        std::fs::read_to_string(repo.join(".bee").join("audit.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    #[test]
    fn handoff_after_merge_door_refuses_a_second_feature_and_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _env = SessionEnv::set(Some("sess-merged-1"));
        crate::integration_queue::record_merger(&repo, "sess-merged-1", "first-feat");

        let code = try_native(&start_argv("second-feat", None), Instant::now());

        assert_eq!(code, Some(ExitCode::FAILURE));
        assert!(!tmp.path().join("testrepo--wt--second-feat").exists());
        assert!(!repo.join(".bee").join("lanes").join("second-feat.json").exists());
        assert!(audit_rows(&repo).is_empty());
    }

    #[test]
    fn handoff_after_merge_door_same_session_with_a_reason_succeeds_and_is_audited() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _env = SessionEnv::set(Some("sess-merged-2"));
        crate::integration_queue::record_merger(&repo, "sess-merged-2", "first-feat");

        let blank = try_native(&start_argv("second-feat", Some("  ")), Instant::now());
        assert_eq!(blank, Some(ExitCode::FAILURE));
        assert!(audit_rows(&repo).is_empty());

        let code = try_native(&start_argv("second-feat", Some("owner asked for a back to back run")), Instant::now());

        assert_eq!(code, Some(ExitCode::SUCCESS));
        assert!(tmp.path().join("testrepo--wt--second-feat").exists());
        let rows = audit_rows(&repo);
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert_eq!(rows[0]["kind"], "handoff-waived");
        assert_eq!(rows[0]["feature"], "second-feat");
        assert!(rows[0].to_string().contains("owner asked for a back to back run"), "{}", rows[0]);
    }

    #[test]
    fn handoff_after_merge_door_audit_log_accepts_the_handoff_waived_kind() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let argv: Vec<OsString> = ["audit", "log", "--kind", "handoff-waived", "--feature", "second-feat", "--note", "owner asked"]
            .iter()
            .map(OsString::from)
            .collect();

        let code = crate::verbs::audit::try_native(&argv, Instant::now());

        assert_eq!(code, Some(ExitCode::SUCCESS));
        let rows = audit_rows(&repo);
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert_eq!(rows[0]["kind"], "handoff-waived");
    }

    #[test]
    fn handoff_after_merge_door_a_failed_start_with_same_session_writes_no_audit_row() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _env = SessionEnv::set(Some("sess-merged-3"));
        crate::integration_queue::record_merger(&repo, "sess-merged-3", "first-feat");
        let blocker = tmp.path().join("testrepo--wt--second-feat");
        std::fs::create_dir_all(&blocker).unwrap();
        std::fs::write(blocker.join("keep.txt"), "occupied\n").unwrap();

        let code = try_native(&start_argv("second-feat", Some("owner asked for a back to back run")), Instant::now());

        assert_ne!(code, Some(ExitCode::SUCCESS));
        assert!(audit_rows(&repo).is_empty(), "{:?}", audit_rows(&repo));
        assert!(!repo.join(".bee").join("lanes").join("second-feat.json").exists());
    }

    #[test]
    fn handoff_after_merge_door_lets_a_first_start_and_a_sessionless_start_through() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        {
            let _env = SessionEnv::set(Some("sess-fresh-1"));
            let code = try_native(&start_argv("first-feat", None), Instant::now());
            assert_eq!(code, Some(ExitCode::SUCCESS));
        }
        crate::integration_queue::record_merger(&repo, "sess-other-9", "earlier-feat");
        let _env = SessionEnv::set(None);

        let code = try_native(&start_argv("sessionless-feat", None), Instant::now());

        assert_eq!(code, Some(ExitCode::SUCCESS));
        assert!(tmp.path().join("testrepo--wt--sessionless-feat").exists());
        assert!(audit_rows(&repo).is_empty());
    }

    #[cfg(unix)]
    const FAKE_WAGGLEDANCE: &str = r#"#!/bin/sh
case "$1" in
  list) echo '{"projects":[{"id":"proj-hive","root_path":"__ROOT__"}]}' ;;
  runs)
    if [ "$2" = "show" ]; then
      __SHOW__
    else
      echo '{"runs":[{"id":"run-other","pane_id":"paseo:agent-other"},{"id":"run-mine","pane_id":"paseo:agent-me"}]}'
    fi
    ;;
  dispatch) printf '%s\n' "$@" > '__ARGV__'; __DISPATCH__ ;;
  *) exit 9 ;;
esac
"#;

    #[cfg(unix)]
    #[derive(Clone, Copy, Debug)]
    enum FakeDispatch {
        Sends,
        ExitsNonZero,
        Unparsable,
        DedupedDone,
    }

    #[cfg(unix)]
    struct FakeWaggledance {
        prior_path: Option<OsString>,
        prior_agent: Option<OsString>,
        argv_file: PathBuf,
    }

    #[cfg(unix)]
    impl FakeWaggledance {
        fn install(tmp: &Path, listed_root: &Path, agent: Option<&str>, dispatch: FakeDispatch) -> Self {
            let bin = tmp.join("fake-bin");
            std::fs::create_dir_all(&bin).unwrap();
            let argv_file = tmp.join("dispatch-argv.txt");
            let listed = std::fs::canonicalize(listed_root).unwrap_or_else(|_| listed_root.to_path_buf());
            let body = match dispatch {
                FakeDispatch::Sends | FakeDispatch::DedupedDone => {
                    r#"echo '{"run_id":"run-new-7","status":"queued"}'"#
                }
                FakeDispatch::ExitsNonZero => r#"echo '{"run_id":"run-bad"}'; echo 'dispatch exploded' >&2; exit 3"#,
                FakeDispatch::Unparsable => "echo 'queued, maybe'",
            };
            let show = match dispatch {
                FakeDispatch::DedupedDone => {
                    r#"echo '{"runs":[{"id":"run-new-7","status":"done","created_at":"2026-11-24T00:00:00.000Z"}]}'"#
                }
                _ => r#"echo '{"runs":[{"id":"run-new-7","status":"queued"}]}'"#,
            };
            let script = FAKE_WAGGLEDANCE
                .replace("__ROOT__", &listed.to_string_lossy())
                .replace("__ARGV__", &argv_file.to_string_lossy())
                .replace("__SHOW__", show)
                .replace("__DISPATCH__", body);
            let exe = bin.join("waggledance");
            std::fs::write(&exe, script).unwrap();
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o755)).unwrap();
            let prior_path = std::env::var_os("PATH");
            let prior_agent = std::env::var_os("PASEO_AGENT_ID");
            let mut dirs = vec![bin];
            if let Some(p) = &prior_path {
                dirs.extend(std::env::split_paths(p));
            }
            unsafe {
                std::env::set_var("PATH", std::env::join_paths(dirs).unwrap());
                match agent {
                    Some(a) => std::env::set_var("PASEO_AGENT_ID", a),
                    None => std::env::remove_var("PASEO_AGENT_ID"),
                }
            }
            Self { prior_path, prior_agent, argv_file }
        }

        fn dispatch_argv(&self) -> Option<Vec<String>> {
            std::fs::read_to_string(&self.argv_file)
                .ok()
                .map(|t| t.lines().map(str::to_string).collect())
        }
    }

    #[cfg(unix)]
    impl Drop for FakeWaggledance {
        fn drop(&mut self) {
            unsafe {
                match &self.prior_path {
                    Some(v) => std::env::set_var("PATH", v),
                    None => std::env::remove_var("PATH"),
                }
                match &self.prior_agent {
                    Some(v) => std::env::set_var("PASEO_AGENT_ID", v),
                    None => std::env::remove_var("PASEO_AGENT_ID"),
                }
            }
        }
    }

    #[cfg(unix)]
    fn handoff_argv(feature: &str, task: Option<&str>, same_session: Option<&str>) -> Vec<OsString> {
        let mut argv = start_argv(feature, same_session);
        if let Some(t) = task {
            argv.push(OsString::from("--handoff-task"));
            argv.push(OsString::from(t));
        }
        argv
    }

    #[cfg(unix)]
    fn seed_merged_session(repo: &Path, sid: &str) {
        crate::integration_queue::record_merger(repo, sid, "first-feat");
        let dir = repo.join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{sid}.json")), json!({"id": sid, "status": "active"}).to_string()).unwrap();
    }

    #[cfg(unix)]
    fn session_record(repo: &Path, sid: &str) -> Value {
        serde_json::from_str(&std::fs::read_to_string(repo.join(".bee").join("sessions").join(format!("{sid}.json"))).unwrap()).unwrap()
    }

    #[cfg(unix)]
    fn assert_no_lane_and_no_worktree(tmp: &Path, repo: &Path) {
        assert!(!tmp.join("testrepo--wt--second-feat").exists());
        assert!(!repo.join(".bee").join("lanes").join("second-feat.json").exists());
    }

    #[cfg(unix)]
    fn set_handoff_auto(repo: &Path, on: bool) {
        std::fs::write(
            repo.join(".bee").join("config.json"),
            json!({"commands": {"test": "none"}, "handoff_auto": on}).to_string(),
        )
        .unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn handoff_after_merge_door_with_waggledance_refuses_without_handoff_task() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _env = SessionEnv::set(Some("sess-auto-1"));
        seed_merged_session(&repo, "sess-auto-1");
        let fake = FakeWaggledance::install(tmp.path(), &repo, Some("agent-me"), FakeDispatch::Sends);

        let code = try_native(&handoff_argv("second-feat", None, None), Instant::now());
        let blank = try_native(&handoff_argv("second-feat", Some("   "), None), Instant::now());

        assert_eq!(code, Some(ExitCode::FAILURE));
        assert_eq!(blank, Some(ExitCode::FAILURE));
        assert_eq!(fake.dispatch_argv(), None);
        assert_no_lane_and_no_worktree(tmp.path(), &repo);
        assert!(audit_rows(&repo).is_empty());
        assert_eq!(session_record(&repo, "sess-auto-1")["status"], "active");
    }

    #[cfg(unix)]
    #[test]
    fn handoff_after_merge_door_with_waggledance_refuses_same_session() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _env = SessionEnv::set(Some("sess-auto-2"));
        seed_merged_session(&repo, "sess-auto-2");
        let fake = FakeWaggledance::install(tmp.path(), &repo, Some("agent-me"), FakeDispatch::Sends);

        let code = try_native(
            &handoff_argv("second-feat", Some("owner wants the next feature"), Some("owner asked for a back to back run")),
            Instant::now(),
        );

        assert_eq!(code, Some(ExitCode::FAILURE));
        assert_eq!(fake.dispatch_argv(), None);
        assert_no_lane_and_no_worktree(tmp.path(), &repo);
        assert!(audit_rows(&repo).is_empty(), "{:?}", audit_rows(&repo));
        assert_eq!(session_record(&repo, "sess-auto-2")["status"], "active");
    }

    #[cfg(unix)]
    #[test]
    fn handoff_after_merge_door_with_waggledance_sends_releases_and_follows_up_on_a_pane_match() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _env = SessionEnv::set(Some("sess-auto-3"));
        seed_merged_session(&repo, "sess-auto-3");
        let fake = FakeWaggledance::install(tmp.path(), &repo, Some("agent-me"), FakeDispatch::Sends);

        let code = try_native(
            &handoff_argv("second-feat", Some("Owner: build second-feat. Standing order: keep it small."), None),
            Instant::now(),
        );

        assert_eq!(code, Some(ExitCode::FAILURE));
        assert_no_lane_and_no_worktree(tmp.path(), &repo);
        assert!(!repo.join(".bee").join("HANDOFF.json").exists());
        let record = session_record(&repo, "sess-auto-3");
        assert_eq!(record["status"], "closed", "{record}");
        assert_eq!(record["released"], true, "{record}");
        let rows = audit_rows(&repo);
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert_eq!(rows[0]["kind"], "handoff-dispatched");
        assert_eq!(rows[0]["feature"], "second-feat");
        assert!(rows[0].to_string().contains("run-new-7"), "{}", rows[0]);
        assert!(rows[0].to_string().contains("bee-handoff-sess-auto-3-second-feat-"), "{}", rows[0]);

        let argv = fake.dispatch_argv().expect("dispatch was called");
        let task_file = argv.last().unwrap().clone();
        assert_eq!(
            &argv[..8],
            [
                "dispatch",
                "--project",
                "proj-hive",
                "--leader",
                "--follow-up-of",
                "run-mine",
                "--handoff",
                "--request-id",
            ]
        );
        assert!(argv[8].starts_with("bee-handoff-sess-auto-3-second-feat-"), "{argv:?}");
        assert_eq!(argv[9], "--task-file");
        assert_eq!(argv[10], task_file.as_str());
        let canon_repo = std::fs::canonicalize(&repo).unwrap();
        let canon_task = std::fs::canonicalize(&task_file).unwrap();
        assert!(canon_task.starts_with(canon_repo.join(".bee").join("tmp")), "{task_file}");
        let task = std::fs::read_to_string(&task_file).unwrap();
        assert!(task.contains("first-feat"), "{task}");
        assert!(task.contains("bee feature start --feature second-feat --class feature --lane standard --files 3"), "{task}");
        assert!(task.contains("Owner: build second-feat. Standing order: keep it small."), "{task}");
    }

    #[cfg(unix)]
    #[test]
    fn handoff_after_merge_door_with_waggledance_sends_a_plain_leader_dispatch_without_a_pane_match() {
        for agent in [Some("agent-stranger"), None] {
            let tmp = tempfile::tempdir().unwrap();
            let repo = init_repo(tmp.path());
            let _guard = CwdGuard::enter(&repo);
            let _env = SessionEnv::set(Some("sess-auto-4"));
            seed_merged_session(&repo, "sess-auto-4");
            let fake = FakeWaggledance::install(tmp.path(), &repo, agent, FakeDispatch::Sends);

            let code = try_native(&handoff_argv("second-feat", Some("owner wants second-feat"), None), Instant::now());

            assert_eq!(code, Some(ExitCode::FAILURE));
            let argv = fake.dispatch_argv().expect("dispatch was called");
            assert_eq!(&argv[..4], ["dispatch", "--project", "proj-hive", "--leader"], "{agent:?}");
            assert!(!argv.iter().any(|a| a == "--follow-up-of" || a == "--handoff"), "{agent:?} {argv:?}");
            assert!(argv.iter().any(|a| a.starts_with("bee-handoff-sess-auto-4-second-feat-")), "{argv:?}");
            assert_eq!(audit_rows(&repo)[0]["kind"], "handoff-dispatched");
            assert_eq!(session_record(&repo, "sess-auto-4")["status"], "closed");
        }
    }

    #[cfg(unix)]
    #[test]
    fn handoff_after_merge_door_with_waggledance_failed_dispatch_writes_nothing() {
        for failure in [FakeDispatch::ExitsNonZero, FakeDispatch::Unparsable] {
            let tmp = tempfile::tempdir().unwrap();
            let repo = init_repo(tmp.path());
            let _guard = CwdGuard::enter(&repo);
            let _env = SessionEnv::set(Some("sess-auto-5"));
            seed_merged_session(&repo, "sess-auto-5");
            let fake = FakeWaggledance::install(tmp.path(), &repo, Some("agent-me"), failure);

            let code = try_native(&handoff_argv("second-feat", Some("owner wants second-feat"), None), Instant::now());

            assert_eq!(code, Some(ExitCode::FAILURE), "{failure:?}");
            assert!(fake.dispatch_argv().is_some(), "dispatch was attempted");
            assert_no_lane_and_no_worktree(tmp.path(), &repo);
            assert!(audit_rows(&repo).is_empty(), "{failure:?} {:?}", audit_rows(&repo));
            assert_eq!(session_record(&repo, "sess-auto-5")["status"], "active", "{failure:?}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn handoff_after_merge_door_with_waggledance_for_another_root_keeps_the_waiver() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _env = SessionEnv::set(Some("sess-auto-6"));
        seed_merged_session(&repo, "sess-auto-6");
        let elsewhere = tmp.path().join("elsewhere");
        std::fs::create_dir_all(&elsewhere).unwrap();
        let fake = FakeWaggledance::install(tmp.path(), &elsewhere, Some("agent-me"), FakeDispatch::Sends);

        let refused = try_native(&handoff_argv("second-feat", Some("owner wants second-feat"), None), Instant::now());
        assert_eq!(refused, Some(ExitCode::FAILURE));
        assert!(audit_rows(&repo).is_empty());

        let code = try_native(&handoff_argv("second-feat", None, Some("owner asked for a back to back run")), Instant::now());

        assert_eq!(code, Some(ExitCode::SUCCESS));
        assert_eq!(fake.dispatch_argv(), None);
        let rows = audit_rows(&repo);
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert_eq!(rows[0]["kind"], "handoff-waived");
        assert_eq!(session_record(&repo, "sess-auto-6")["status"], "active");
    }

    #[cfg(unix)]
    #[test]
    fn handoff_after_merge_door_with_handoff_auto_false_keeps_the_waiver() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _env = SessionEnv::set(Some("sess-auto-7"));
        seed_merged_session(&repo, "sess-auto-7");
        set_handoff_auto(&repo, false);
        let fake = FakeWaggledance::install(tmp.path(), &repo, Some("agent-me"), FakeDispatch::Sends);

        let code = try_native(
            &handoff_argv("second-feat", Some("owner wants second-feat"), Some("owner asked for a back to back run")),
            Instant::now(),
        );

        assert_eq!(code, Some(ExitCode::SUCCESS));
        assert_eq!(fake.dispatch_argv(), None);
        assert!(tmp.path().join("testrepo--wt--second-feat").exists());
        let rows = audit_rows(&repo);
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert_eq!(rows[0]["kind"], "handoff-waived");
        assert_eq!(session_record(&repo, "sess-auto-7")["status"], "active");
    }

    #[test]
    fn handoff_auto_cannot_be_changed_through_config_set() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let before = std::fs::read_to_string(repo.join(".bee").join("config.json")).unwrap();

        for extra in [None, Some("--local")] {
            let mut argv: Vec<OsString> = ["config", "set", "--key", "handoff_auto", "--value", "false"]
                .iter()
                .map(OsString::from)
                .collect();
            if let Some(e) = extra {
                argv.push(OsString::from(e));
            }
            let code = crate::verbs::config::try_native(&argv, Instant::now());
            assert_eq!(code, Some(ExitCode::FAILURE), "{extra:?}");
        }

        assert_eq!(std::fs::read_to_string(repo.join(".bee").join("config.json")).unwrap(), before);
        assert!(!repo.join(".bee").join("config.local.json").exists());
        assert!(!crate::state::read_config_raw(&repo).contains_key("handoff_auto"));
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

    fn wip_lane(feature: &str, phase: &str, created_at: &str) -> Map<String, Value> {
        json!({"feature": feature, "phase": phase, "created_at": created_at})
            .as_object()
            .unwrap()
            .clone()
    }

    fn wip_config(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    fn five_open_lanes() -> Vec<Map<String, Value>> {
        vec![
            wip_lane("e-feat", "exploring", "2026-01-05T00:00:00Z"),
            wip_lane("p-feat", "planning", "2026-01-04T00:00:00Z"),
            wip_lane("s-feat", "swarming", "2026-01-03T00:00:00Z"),
            wip_lane("c-feat", "compounding", "2026-01-02T00:00:00Z"),
            wip_lane("x-feat", "scribing", "2026-01-01T00:00:00Z"),
        ]
    }

    #[test]
    fn wip_limit_absent_key_warns_at_five_open_lanes_with_limit_five() {
        let w = evaluate_wip_warning(&five_open_lanes(), "new-feat", &wip_config(json!({})))
            .expect("five open lanes meet the default limit");
        assert_eq!(w.count, 5);
        assert_eq!(w.limit, 5);
    }

    #[test]
    fn wip_limit_four_open_lanes_do_not_warn() {
        let mut lanes = five_open_lanes();
        lanes.pop();
        assert_eq!(evaluate_wip_warning(&lanes, "new-feat", &wip_config(json!({}))), None);
    }

    #[test]
    fn wip_limit_zero_never_warns() {
        let mut lanes = five_open_lanes();
        lanes.extend(five_open_lanes().into_iter().map(|mut l| {
            let f = format!("{}-2", l["feature"].as_str().unwrap());
            l.insert("feature".into(), json!(f));
            l
        }));
        let cfg = wip_config(json!({"wip_limit": 0}));
        assert_eq!(evaluate_wip_warning(&lanes, "new-feat", &cfg), None);
    }

    #[test]
    fn wip_limit_non_integer_value_is_treated_as_absent() {
        let mut four = five_open_lanes();
        four.pop();
        for bad in [json!("2"), json!(2.5), json!(-1), json!(null), json!(true), json!([2])] {
            let cfg = wip_config(json!({"wip_limit": bad.clone()}));
            assert_eq!(wip_limit_from_config(&cfg), 5, "value {bad}");
            assert_eq!(evaluate_wip_warning(&four, "new-feat", &cfg), None, "value {bad}");
            let w = evaluate_wip_warning(&five_open_lanes(), "new-feat", &cfg)
                .unwrap_or_else(|| panic!("value {bad} must fall back to 5"));
            assert_eq!(w.limit, 5);
        }
    }

    #[test]
    fn wip_limit_ignores_lanes_outside_the_five_working_phases() {
        let mut lanes = five_open_lanes();
        lanes.pop();
        for (i, phase) in ["idle", "done", "closed", "abandoned", "reviewing", "Exploring", ""]
            .iter()
            .enumerate()
        {
            lanes.push(wip_lane(&format!("other-{i}"), phase, "2025-01-01T00:00:00Z"));
        }
        let mut no_phase = wip_lane("no-phase", "x", "2025-01-01T00:00:00Z");
        no_phase.remove("phase");
        lanes.push(no_phase);
        assert_eq!(evaluate_wip_warning(&lanes, "new-feat", &wip_config(json!({}))), None);
    }

    #[test]
    fn wip_limit_ignores_a_closed_lane() {
        let mut lanes = five_open_lanes();
        lanes[0].insert("status".into(), json!("closed"));
        assert_eq!(evaluate_wip_warning(&lanes, "new-feat", &wip_config(json!({}))), None);
        lanes[0].insert("status".into(), json!("active"));
        assert!(evaluate_wip_warning(&lanes, "new-feat", &wip_config(json!({}))).is_some());
    }

    #[test]
    fn wip_limit_does_not_count_the_started_feature() {
        let lanes = five_open_lanes();
        assert_eq!(evaluate_wip_warning(&lanes, "e-feat", &wip_config(json!({}))), None);
        assert!(evaluate_wip_warning(&lanes, "new-feat", &wip_config(json!({}))).is_some());
    }

    #[test]
    fn wip_limit_oldest_lists_three_slugs_by_created_at() {
        let mut lanes = five_open_lanes();
        lanes.push(wip_lane("late-feat", "planning", "2026-02-01T00:00:00Z"));
        let w = evaluate_wip_warning(&lanes, "new-feat", &wip_config(json!({}))).unwrap();
        assert_eq!(w.count, 6);
        assert_eq!(w.oldest, vec!["x-feat", "c-feat", "s-feat"]);
    }

    #[test]
    fn wip_limit_custom_limit_is_honored() {
        let mut lanes = five_open_lanes();
        lanes.truncate(2);
        let cfg = wip_config(json!({"wip_limit": 2}));
        let w = evaluate_wip_warning(&lanes, "new-feat", &cfg).unwrap();
        assert_eq!((w.count, w.limit), (2, 2));
        assert_eq!(w.oldest, vec!["p-feat", "e-feat"]);
        lanes.pop();
        assert_eq!(evaluate_wip_warning(&lanes, "new-feat", &cfg), None);
    }

    #[test]
    fn wip_limit_feature_start_warns_and_still_succeeds() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        for lane in five_open_lanes() {
            let slug = lane["feature"].as_str().unwrap().to_string();
            std::fs::write(
                repo.join(".bee").join("lanes").join(format!("{slug}.json")),
                serde_json::to_string(&Value::Object(lane)).unwrap(),
            )
            .unwrap();
        }
        let _guard = CwdGuard::enter(&repo);

        let before = list_lanes(&repo).unwrap_or_else(|_| panic!("list_lanes failed"));
        let cfg = crate::state::read_config_raw(&repo);
        let w = evaluate_wip_warning(&before, "wip-feat", &cfg)
            .expect("the handler's inputs must produce a warning");
        assert_eq!((w.count, w.limit), (5, 5));

        let argv = vec![
            OsString::from("feature"),
            OsString::from("start"),
            OsString::from("--feature"),
            OsString::from("wip-feat"),
            OsString::from("--class"),
            OsString::from("feature"),
            OsString::from("--lane"),
            OsString::from("standard"),
            OsString::from("--files"),
            OsString::from("1"),
        ];
        let code = try_native(&argv, Instant::now());
        assert_eq!(code, Some(ExitCode::SUCCESS));
        assert!(tmp.path().join("testrepo--wt--wip-feat").exists());
        assert!(repo.join(".bee").join("lanes").join("wip-feat.json").exists());

        let after = list_lanes(&repo).unwrap_or_else(|_| panic!("list_lanes failed"));
        let w = evaluate_wip_warning(&after, "wip-feat", &cfg).unwrap();
        assert_eq!(w.count, 5);
    }

    struct HerdingEnv {
        _unherded: crate::hooks::UnherdedEnv,
    }

    impl HerdingEnv {
        fn set(worker: bool) -> Self {
            let unherded = crate::hooks::UnherdedEnv::new();
            if worker {
                unsafe { std::env::set_var("BEE_HERDING_WORKER", "1") };
            }
            Self { _unherded: unherded }
        }
    }

    impl Drop for HerdingEnv {
        fn drop(&mut self) {
            unsafe { std::env::remove_var("BEE_HERDING_WORKER") };
        }
    }

    #[cfg(unix)]
    fn seed_session_at(repo: &Path, sid: &str, tokens: u64) {
        let dir = repo.join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        let now = crate::verbs::reservations::now_iso();
        std::fs::write(
            dir.join(format!("{sid}.json")),
            json!({
                "id": sid,
                "status": "active",
                "started_at": now,
                "last_heartbeat": now,
                "context": {"tokens": tokens, "measured_at": now}
            })
            .to_string(),
        )
        .unwrap();
    }

    #[cfg(unix)]
    fn set_mark_config(repo: &Path, extra: Value) {
        let mut cfg = json!({"commands": {"test": "none"}});
        for (k, v) in extra.as_object().unwrap() {
            cfg[k] = v.clone();
        }
        std::fs::write(repo.join(".bee").join("config.json"), cfg.to_string()).unwrap();
    }

    #[cfg(unix)]
    fn seed_claimable_cell(repo: &Path, id: &str) {
        std::fs::write(
            repo.join(".bee").join("state.json"),
            json!({
                "schema_version": "1.0",
                "phase": "swarming",
                "mode": "standard",
                "feature": "demo",
                "approved_gates": {"context": true, "shape": true, "execution": true, "review": false}
            })
            .to_string(),
        )
        .unwrap();
        let cell = repo.join("cell.json");
        std::fs::write(
            &cell,
            json!({
                "id": id,
                "feature": "demo",
                "title": "t",
                "lane": "standard",
                "role": "docs",
                "action": "act",
                "files": ["docs/a.md"],
                "read_first": [],
                "must_haves": {"truths": ["ok"]},
                "verify": "true",
                "affects_skills": [],
                "affects_specs": []
            })
            .to_string(),
        )
        .unwrap();
        let code = crate::verbs::cells::try_native(
            &[OsString::from("cells"), OsString::from("add"), OsString::from("--file"), cell.into_os_string()],
            Instant::now(),
        );
        assert_eq!(code, Some(ExitCode::SUCCESS));
    }

    #[cfg(unix)]
    fn cell_status(repo: &Path, id: &str) -> Value {
        let v: Value = serde_json::from_str(
            &std::fs::read_to_string(repo.join(".bee").join("cells").join(format!("{id}.json"))).unwrap(),
        )
        .unwrap();
        v["status"].clone()
    }

    #[cfg(unix)]
    fn claim_argv(id: &str) -> Vec<OsString> {
        ["cells", "claim", "--id", id, "--worker", "w1"].iter().map(OsString::from).collect()
    }

    #[cfg(unix)]
    fn send_argv(task: &str) -> Vec<OsString> {
        ["state", "handoff", "send", "--task", task].iter().map(OsString::from).collect()
    }

    #[cfg(unix)]
    #[test]
    fn handoff_context_mark_refusal_names_the_send_verb_at_and_past_the_mark() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _herd = HerdingEnv::set(false);
        let _fake = FakeWaggledance::install(tmp.path(), &repo, None, FakeDispatch::Sends);

        for tokens in [120_000u64, 187_654] {
            seed_session_at(&repo, "sess-mark-r", tokens);
            let msg = crate::verbs::handoff_door::context_mark_refusal(&repo, &repo, Some("sess-mark-r"))
                .expect("refused at or past the mark");
            assert!(msg.starts_with("[handoff_due]"), "{msg}");
            assert!(msg.contains(&format!("context {}k", tokens / 1000)), "{msg}");
            assert!(msg.contains("the 120k mark"), "{msg}");
            assert!(msg.contains("bee state handoff send --task"), "{msg}");
        }
        seed_session_at(&repo, "sess-mark-r", 119_999);
        assert_eq!(crate::verbs::handoff_door::context_mark_refusal(&repo, &repo, Some("sess-mark-r")), None);
        assert_eq!(crate::verbs::handoff_door::context_mark_refusal(&repo, &repo, None), None);
    }

    #[cfg(unix)]
    #[test]
    fn handoff_context_mark_refusal_passes_when_the_door_is_off() {
        let cases: [(&str, Value, bool, bool); 4] = [
            ("mark-zero", json!({"handoff_context_tokens": 0}), true, false),
            ("no-door", json!({}), false, false),
            ("auto-off", json!({"handoff_auto": false}), true, false),
            ("herding-worker", json!({}), true, true),
        ];
        for (label, cfg, door_here, worker) in cases {
            let tmp = tempfile::tempdir().unwrap();
            let repo = init_repo(tmp.path());
            let _guard = CwdGuard::enter(&repo);
            let _herd = HerdingEnv::set(worker);
            set_mark_config(&repo, cfg);
            seed_session_at(&repo, "sess-mark-off", 900_000);
            let elsewhere = tmp.path().join("elsewhere");
            std::fs::create_dir_all(&elsewhere).unwrap();
            let listed = if door_here { repo.clone() } else { elsewhere };
            let _fake = FakeWaggledance::install(tmp.path(), &listed, None, FakeDispatch::Sends);

            assert_eq!(
                crate::verbs::handoff_door::context_mark_refusal(&repo, &repo, Some("sess-mark-off")),
                None,
                "{label}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn handoff_context_mark_door_refuses_feature_start_at_and_past_the_mark() {
        for tokens in [120_000u64, 150_000] {
            let tmp = tempfile::tempdir().unwrap();
            let repo = init_repo(tmp.path());
            let _guard = CwdGuard::enter(&repo);
            let _herd = HerdingEnv::set(false);
            let _env = SessionEnv::set(Some("sess-mark-1"));
            seed_session_at(&repo, "sess-mark-1", tokens);
            let fake = FakeWaggledance::install(tmp.path(), &repo, Some("agent-me"), FakeDispatch::Sends);

            let code = try_native(&start_argv("second-feat", None), Instant::now());

            assert_eq!(code, Some(ExitCode::FAILURE), "{tokens}");
            assert_eq!(fake.dispatch_argv(), None);
            assert_no_lane_and_no_worktree(tmp.path(), &repo);
            assert!(audit_rows(&repo).is_empty());
            assert_eq!(session_record(&repo, "sess-mark-1")["status"], "active");
        }
    }

    #[cfg(unix)]
    #[test]
    fn handoff_context_mark_door_lets_feature_start_through_when_off() {
        let cases: [(&str, u64, Value, bool, bool); 5] = [
            ("below", 119_999, json!({}), true, false),
            ("mark-zero", 900_000, json!({"handoff_context_tokens": 0}), true, false),
            ("no-door", 150_000, json!({}), false, false),
            ("auto-off", 150_000, json!({"handoff_auto": false}), true, false),
            ("herding-worker", 150_000, json!({}), true, true),
        ];
        for (label, tokens, cfg, door_here, worker) in cases {
            let tmp = tempfile::tempdir().unwrap();
            let repo = init_repo(tmp.path());
            let _guard = CwdGuard::enter(&repo);
            let _herd = HerdingEnv::set(worker);
            let _env = SessionEnv::set(Some("sess-mark-2"));
            set_mark_config(&repo, cfg);
            seed_session_at(&repo, "sess-mark-2", tokens);
            let elsewhere = tmp.path().join("elsewhere");
            std::fs::create_dir_all(&elsewhere).unwrap();
            let listed = if door_here { repo.clone() } else { elsewhere };
            let fake = FakeWaggledance::install(tmp.path(), &listed, Some("agent-me"), FakeDispatch::Sends);

            let code = try_native(&start_argv("second-feat", None), Instant::now());

            assert_eq!(code, Some(ExitCode::SUCCESS), "{label}");
            assert!(tmp.path().join("testrepo--wt--second-feat").exists(), "{label}");
            assert_eq!(fake.dispatch_argv(), None, "{label}");
            assert_eq!(session_record(&repo, "sess-mark-2")["status"], "active", "{label}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn handoff_context_mark_door_feature_start_with_handoff_task_sends_with_the_mark_request_id() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _herd = HerdingEnv::set(false);
        let _env = SessionEnv::set(Some("sess-mark-3"));
        seed_session_at(&repo, "sess-mark-3", 140_000);
        let fake = FakeWaggledance::install(tmp.path(), &repo, None, FakeDispatch::Sends);

        let code = try_native(&handoff_argv("second-feat", Some("Owner: build second-feat."), None), Instant::now());

        assert_eq!(code, Some(ExitCode::FAILURE));
        assert_no_lane_and_no_worktree(tmp.path(), &repo);
        let argv = fake.dispatch_argv().expect("dispatch was called");
        assert!(argv.iter().any(|a| a.starts_with("bee-handoff-sess-mark-3-mark-")), "{argv:?}");
        let task = std::fs::read_to_string(argv.last().unwrap()).unwrap();
        assert!(task.contains("140k"), "{task}");
        assert!(task.contains("Owner: build second-feat."), "{task}");
        assert_eq!(session_record(&repo, "sess-mark-3")["status"], "closed");
        let rows = audit_rows(&repo);
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert_eq!(rows[0]["kind"], "handoff-dispatched");
    }

    #[cfg(unix)]
    #[test]
    fn handoff_context_mark_door_refuses_cells_claim_past_the_mark_and_not_below() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _herd = HerdingEnv::set(false);
        let _env = SessionEnv::set(Some("sess-mark-4"));
        seed_claimable_cell(&repo, "demo-1");
        let fake = FakeWaggledance::install(tmp.path(), &repo, None, FakeDispatch::Sends);

        seed_session_at(&repo, "sess-mark-4", 130_000);
        let refused = crate::verbs::cells::try_native(&claim_argv("demo-1"), Instant::now());
        assert_eq!(refused, Some(ExitCode::FAILURE));
        assert_eq!(cell_status(&repo, "demo-1"), "open");

        seed_session_at(&repo, "sess-mark-4", 119_000);
        let claimed = crate::verbs::cells::try_native(&claim_argv("demo-1"), Instant::now());
        assert_eq!(claimed, Some(ExitCode::SUCCESS));
        assert_eq!(cell_status(&repo, "demo-1"), "claimed");
        assert_eq!(fake.dispatch_argv(), None);
    }

    #[cfg(unix)]
    #[test]
    fn handoff_context_mark_door_never_refuses_cells_cap() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _herd = HerdingEnv::set(false);
        let _env = SessionEnv::set(Some("sess-mark-5"));
        seed_claimable_cell(&repo, "demo-1");
        let _fake = FakeWaggledance::install(tmp.path(), &repo, None, FakeDispatch::Sends);
        seed_session_at(&repo, "sess-mark-5", 1_000);
        assert_eq!(crate::verbs::cells::try_native(&claim_argv("demo-1"), Instant::now()), Some(ExitCode::SUCCESS));
        seed_session_at(&repo, "sess-mark-5", 200_000);
        assert!(crate::verbs::handoff_door::context_mark_refusal(&repo, &repo, Some("sess-mark-5")).is_some());

        let cap: Vec<OsString> = [
            "cells", "cap", "--id", "demo-1", "--report",
            "{\"outcome\":\"done\",\"commit\":\"none\",\"files\":[],\"tests\":\"true — green:unit — ok\",\"deviations\":[]}",
            "--files", "docs/a.md", "--inline-reason", "tester caps inline", "--no-mistakes",
        ]
        .iter()
        .map(OsString::from)
        .collect();
        let code = crate::verbs::cells::try_native(&cap, Instant::now());

        assert_eq!(code, Some(ExitCode::SUCCESS));
        assert_ne!(cell_status(&repo, "demo-1"), "claimed");
    }

    #[cfg(unix)]
    #[test]
    fn handoff_send_dispatches_with_the_mark_request_id_releases_and_audits() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _herd = HerdingEnv::set(false);
        let _env = SessionEnv::set(Some("sess-mark-6"));
        seed_claimable_cell(&repo, "demo-1");
        seed_session_at(&repo, "sess-mark-6", 160_000);
        let fake = FakeWaggledance::install(tmp.path(), &repo, Some("agent-me"), FakeDispatch::Sends);

        let code = crate::verbs::state_group::try_native(&send_argv("Owner: finish demo. Keep it small."), Instant::now());

        assert_eq!(code, Some(ExitCode::FAILURE));
        let argv = fake.dispatch_argv().expect("dispatch was called");
        assert_eq!(&argv[..4], ["dispatch", "--project", "proj-hive", "--leader"], "{argv:?}");
        assert!(argv.iter().any(|a| a.starts_with("bee-handoff-sess-mark-6-mark-")), "{argv:?}");
        let task = std::fs::read_to_string(argv.last().unwrap()).unwrap();
        assert!(task.contains("160k"), "{task}");
        assert!(task.contains("120k"), "{task}");
        assert!(task.contains("demo-1"), "{task}");
        assert!(task.contains("Owner: finish demo. Keep it small."), "{task}");
        let record = session_record(&repo, "sess-mark-6");
        assert_eq!(record["status"], "closed", "{record}");
        assert_eq!(record["released"], true, "{record}");
        let rows = audit_rows(&repo);
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert_eq!(rows[0]["kind"], "handoff-dispatched");
        assert!(rows[0].to_string().contains("run-new-7"), "{}", rows[0]);
        assert!(rows[0].to_string().contains("bee-handoff-sess-mark-6-mark-"), "{}", rows[0]);
    }

    #[cfg(unix)]
    #[test]
    fn handoff_send_failed_dispatch_writes_nothing() {
        for failure in [FakeDispatch::ExitsNonZero, FakeDispatch::Unparsable] {
            let tmp = tempfile::tempdir().unwrap();
            let repo = init_repo(tmp.path());
            let _guard = CwdGuard::enter(&repo);
            let _herd = HerdingEnv::set(false);
            let _env = SessionEnv::set(Some("sess-mark-7"));
            seed_session_at(&repo, "sess-mark-7", 160_000);
            let fake = FakeWaggledance::install(tmp.path(), &repo, Some("agent-me"), failure);

            let code = crate::verbs::state_group::try_native(&send_argv("owner task"), Instant::now());

            assert_eq!(code, Some(ExitCode::FAILURE), "{failure:?}");
            assert!(fake.dispatch_argv().is_some(), "{failure:?}");
            assert!(audit_rows(&repo).is_empty(), "{failure:?} {:?}", audit_rows(&repo));
            let record = session_record(&repo, "sess-mark-7");
            assert_eq!(record["status"], "active", "{failure:?}");
            assert!(record.get("released").is_none(), "{failure:?} {record}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn handoff_send_refuses_without_a_door() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _herd = HerdingEnv::set(false);
        let _env = SessionEnv::set(Some("sess-mark-8"));
        seed_session_at(&repo, "sess-mark-8", 160_000);
        let elsewhere = tmp.path().join("elsewhere");
        std::fs::create_dir_all(&elsewhere).unwrap();
        let fake = FakeWaggledance::install(tmp.path(), &elsewhere, Some("agent-me"), FakeDispatch::Sends);

        let code = crate::verbs::state_group::try_native(&send_argv("owner task"), Instant::now());

        assert_eq!(code, Some(ExitCode::FAILURE));
        assert_eq!(fake.dispatch_argv(), None);
        assert!(audit_rows(&repo).is_empty());
        assert_eq!(session_record(&repo, "sess-mark-8")["status"], "active");
    }

    #[test]
    fn handoff_hook_records_context_tokens_for_a_bee_command_only() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let repo = std::fs::canonicalize(&repo).unwrap();
        let _herd = HerdingEnv::set(false);
        let transcript = tmp.path().join("transcript.jsonl");
        std::fs::write(
            &transcript,
            format!(
                "{}\n",
                json!({"message": {"id": "m1", "model": "claude-opus-5-5", "usage": {"input_tokens": 1000, "cache_read_input_tokens": 120000, "cache_creation_input_tokens": 3456, "output_tokens": 10}}})
            ),
        )
        .unwrap();
        let run = |sid: &str, cmd: &str| {
            let payload = json!({
                "hook_event_name": "PreToolUse",
                "session_id": sid,
                "transcript_path": transcript.to_string_lossy(),
                "cwd": repo.to_string_lossy(),
                "tool_name": "Bash",
                "tool_input": {"command": cmd}
            });
            let ctx = crate::hooks::adapter::read_hook_context("write-guard", &[], &payload.to_string());
            let _ = crate::hooks::write_guard::run_native(&ctx);
        };

        run("sess-hook-bee", ".bee/bin/bee cells list");
        run("sess-hook-plain", "ls -la");

        let sessions = repo.join(".bee").join("sessions");
        let record: Value =
            serde_json::from_str(&std::fs::read_to_string(sessions.join("sess-hook-bee.json")).unwrap()).unwrap();
        assert_eq!(record["context"]["tokens"], 124_456, "{record}");
        assert!(record["context"]["measured_at"].is_string(), "{record}");
        assert!(!sessions.join("sess-hook-plain.json").exists());
    }

    #[cfg(unix)]
    #[test]
    fn handoff_send_two_attempts_get_distinct_request_ids_and_task_files() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _herd = HerdingEnv::set(false);
        let _env = SessionEnv::set(Some("sess-two-1"));
        seed_session_at(&repo, "sess-two-1", 160_000);
        let fake = FakeWaggledance::install(tmp.path(), &repo, None, FakeDispatch::Sends);

        assert_eq!(
            crate::verbs::state_group::try_native(&send_argv("first attempt"), Instant::now()),
            Some(ExitCode::FAILURE)
        );
        let argv1 = fake.dispatch_argv().unwrap();
        let req1 = argv1[5].clone();
        let task1 = argv1[7].clone();

        std::thread::sleep(std::time::Duration::from_millis(3));
        seed_session_at(&repo, "sess-two-1", 160_000);
        assert_eq!(
            crate::verbs::state_group::try_native(&send_argv("second attempt"), Instant::now()),
            Some(ExitCode::FAILURE)
        );
        let argv2 = fake.dispatch_argv().unwrap();
        let req2 = argv2[5].clone();
        let task2 = argv2[7].clone();

        assert_ne!(req1, req2);
        assert_ne!(task1, task2);
        let open = crate::verbs::handoff_ledger::read_open_rows(&repo);
        assert_eq!(open.len(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn handoff_send_follow_up_of_wins_over_paseo_and_expect_merge_writes_ledger() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _herd = HerdingEnv::set(false);
        let _env = SessionEnv::set(Some("sess-exp-1"));
        seed_session_at(&repo, "sess-exp-1", 160_000);
        let fake = FakeWaggledance::install(tmp.path(), &repo, Some("agent-me"), FakeDispatch::Sends);

        let argv: Vec<OsString> = [
            "state",
            "handoff",
            "send",
            "--task",
            "finish work",
            "--follow-up-of",
            "run-explicit-77",
            "--expect-merge",
            "wt/alpha",
            "--expect-merge",
            "wt/beta,wt/gamma",
        ]
        .iter()
        .map(OsString::from)
        .collect();
        let code = crate::verbs::state_group::try_native(&argv, Instant::now());
        assert_eq!(code, Some(ExitCode::FAILURE));

        let dispatched = fake.dispatch_argv().unwrap();
        assert_eq!(
            &dispatched[..7],
            [
                "dispatch",
                "--project",
                "proj-hive",
                "--leader",
                "--follow-up-of",
                "run-explicit-77",
                "--handoff",
            ]
        );
        let open = crate::verbs::handoff_ledger::read_open_rows(&repo);
        assert_eq!(open.len(), 1);
        assert_eq!(
            open[0].expects,
            vec![
                crate::verbs::handoff_ledger::Expectation::BranchMerged {
                    branch: "wt/alpha".to_string()
                },
                crate::verbs::handoff_ledger::Expectation::BranchMerged {
                    branch: "wt/beta".to_string()
                },
                crate::verbs::handoff_ledger::Expectation::BranchMerged {
                    branch: "wt/gamma".to_string()
                },
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn handoff_send_deduped_done_run_keeps_session_and_writes_no_ledger_or_audit() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);
        let _herd = HerdingEnv::set(false);
        let _env = SessionEnv::set(Some("sess-dedupe-1"));
        seed_session_at(&repo, "sess-dedupe-1", 160_000);
        let fake = FakeWaggledance::install(
            tmp.path(),
            &repo,
            Some("agent-me"),
            FakeDispatch::DedupedDone,
        );

        let code = crate::verbs::state_group::try_native(&send_argv("retry task"), Instant::now());

        assert_eq!(code, Some(ExitCode::FAILURE));
        assert!(fake.dispatch_argv().is_some());
        assert!(audit_rows(&repo).is_empty());
        assert!(crate::verbs::handoff_ledger::read_open_rows(&repo).is_empty());
        let record = session_record(&repo, "sess-dedupe-1");
        assert_eq!(record["status"], "active");
        assert!(record.get("released").is_none());
    }
}
