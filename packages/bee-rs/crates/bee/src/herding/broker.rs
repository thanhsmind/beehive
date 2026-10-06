use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use serde_json::{json, Map, Value};

use super::{mailbox, resolve_main_root};
use super::mailbox::{MailboxQuestion, MailboxResult, MailboxStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BrokerError {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

impl std::fmt::Display for BrokerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[allow(dead_code)]
pub(crate) enum AdvisorOutcome {
    Done {
        summary: String,
        report_path: Option<String>,
    },
    Blocked {
        reason: String,
        report_path: Option<String>,
    },
    Failed(String),
    NotHerding,
}

pub(crate) trait AdvisorRunner {
    fn run_advisor(
        &self,
        main_root: &Path,
        runtime: &str,
        question: &str,
        question_kind: &str,
    ) -> Result<AdvisorOutcome, String>;
}

pub(crate) struct RealAdvisorRunner;

impl AdvisorRunner for RealAdvisorRunner {
    fn run_advisor(
        &self,
        main_root: &Path,
        runtime: &str,
        question: &str,
        _question_kind: &str,
    ) -> Result<AdvisorOutcome, String> {
        let prepared = match crate::verbs::drivers::prepare_dispatch_with_brief(
            main_root,
            runtime,
            "advisor",
            Some("advisor"),
            None,
            None,
            false,
            None,
            None,
            false,
            None,
            Some(question),
        ) {
            Ok(crate::verbs::drivers::Prepared::Value(v)) => v,
            _ => return Ok(AdvisorOutcome::NotHerding),
        };

        if prepared.get("ok").and_then(Value::as_bool) == Some(false) {
            return Ok(AdvisorOutcome::NotHerding);
        }
        if prepared.get("reason").is_some() {
            return Ok(AdvisorOutcome::NotHerding);
        }

        let tool = prepared.get("tool").and_then(Value::as_str).unwrap_or("");
        if tool != "Bash" {
            return Ok(AdvisorOutcome::NotHerding);
        }

        let payload = match prepared.get("payload").and_then(Value::as_object) {
            Some(p) => p,
            None => return Ok(AdvisorOutcome::NotHerding),
        };

        let command_str = match payload.get("command").and_then(Value::as_str) {
            Some(c) => c,
            None => return Ok(AdvisorOutcome::NotHerding),
        };

        if !command_str.contains("herding run") {
            return Ok(AdvisorOutcome::NotHerding);
        }

        let stdin_content = payload.get("stdin").and_then(Value::as_str).unwrap_or("");

        let mut cmd = match crate::shell::command() {
            Some(c) => c,
            None => Command::new("sh"),
        };
        cmd.arg("-c").arg(command_str);
        cmd.current_dir(main_root);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => return Ok(AdvisorOutcome::Failed(format!("failed to spawn advisor: {e}"))),
        };

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(stdin_content.as_bytes());
        }

        let output = match child.wait_with_output() {
            Ok(o) => o,
            Err(e) => return Ok(AdvisorOutcome::Failed(format!("advisor wait error: {e}"))),
        };

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let parsed_json: Value = match serde_json::from_str(&stdout_str) {
            Ok(v) => v,
            Err(_) => {
                return Ok(AdvisorOutcome::Failed(format!(
                    "advisor non-JSON output: {}",
                    stdout_str.trim()
                )))
            }
        };

        let outcome = parsed_json
            .get("outcome")
            .and_then(Value::as_str)
            .unwrap_or("");
        let summary = parsed_json
            .get("summary")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let report_path = parsed_json
            .get("report_path")
            .and_then(Value::as_str)
            .map(|s| s.to_string());

        if outcome == "done" {
            Ok(AdvisorOutcome::Done { summary, report_path })
        } else if outcome == "blocked" {
            Ok(AdvisorOutcome::Blocked { reason: summary, report_path })
        } else {
            Ok(AdvisorOutcome::Failed(format!("advisor outcome {outcome}: {summary}")))
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SpawnArgs {
    pub(crate) job_id: String,
    pub(crate) task: String,
    pub(crate) cwd: PathBuf,
    pub(crate) agent: Option<String>,
    pub(crate) seat: Option<String>,
    pub(crate) cell_id: Option<String>,
    pub(crate) no_pane: bool,
    pub(crate) inbox_session: Option<String>,
    pub(crate) question_of: Option<String>,
    pub(crate) question_round: u32,
    pub(crate) no_question: bool,
    pub(crate) continue_job: Option<String>,
}

pub(crate) trait JobSpawner {
    fn spawn(&self, main_root: &Path, args: &SpawnArgs) -> Result<(), String>;
}

pub(crate) struct RealJobSpawner;

impl RealJobSpawner {
    pub(crate) fn build_command(main_root: &Path, args: &SpawnArgs) -> Command {
        let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("bee"));
        let mut cmd = Command::new(exe);
        cmd.arg("herding").arg("run");
        if let Some(cont) = &args.continue_job {
            cmd.arg("--continue").arg(cont);
            cmd.arg("--task").arg(&args.task);
            cmd.arg("--cwd").arg(&args.cwd);
            cmd.arg("--main-root").arg(main_root);
        } else {
            cmd.arg("--job-id").arg(&args.job_id);
            cmd.arg("--task").arg(&args.task);
            cmd.arg("--cwd").arg(&args.cwd);
            if let Some(agent) = &args.agent {
                cmd.arg("--agent").arg(agent);
            }
            if let Some(seat) = &args.seat {
                cmd.arg("--seat").arg(seat);
            }
            if let Some(cell_id) = &args.cell_id {
                cmd.arg("--cell-id").arg(cell_id);
            }
            if args.no_pane {
                cmd.arg("--no-pane");
            }
            if let Some(inbox) = &args.inbox_session {
                cmd.arg("--inbox-session").arg(inbox);
            }
            if let Some(qof) = &args.question_of {
                cmd.arg("--question-of").arg(qof);
            }
            cmd.arg("--question-round").arg(args.question_round.to_string());
            if args.no_question {
                cmd.arg("--no-question");
            }
            cmd.arg("--main-root").arg(main_root);
        }
        cmd.current_dir(main_root);
        cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        cmd
    }
}

impl JobSpawner for RealJobSpawner {
    fn spawn(&self, main_root: &Path, args: &SpawnArgs) -> Result<(), String> {
        let mut cmd = Self::build_command(main_root, args);
        match cmd.spawn() {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("failed to spawn child job: {e}")),
        }
    }
}


pub(crate) struct TickOutcome {
    pub(crate) heartbeat: bool,
    pub(crate) claimed: Option<String>,
    pub(crate) notices_sent: usize,
}

fn status_str(status: MailboxStatus) -> &'static str {
    match status {
        MailboxStatus::Done => "done",
        MailboxStatus::Blocked => "blocked",
        MailboxStatus::Question => "question",
    }
}

#[derive(Debug)]
pub(crate) struct AnswerOutcome {
    pub(crate) job_id: String,
    pub(crate) round: u32,
    pub(crate) text: String,
    pub(crate) child_job_id: String,
}

fn clip_text(s: &str, limit: usize) -> String {
    let flat = s.lines().map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= limit {
        flat
    } else {
        let kept: String = flat.chars().take(limit.saturating_sub(1)).collect();
        format!("{}…", kept.trim_end())
    }
}

fn sanitize_question_for_intervention(text: &str) -> String {
    let base = clip_text(text, 400);
    if base.is_empty() {
        "What is the decision?".to_string()
    } else {
        base
    }
}

fn slugify_point_key(raw: &str) -> String {
    let mut out = String::new();
    let mut prev_hyphen = false;
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_hyphen = false;
        } else if !prev_hyphen && !out.is_empty() {
            out.push('-');
            prev_hyphen = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "permission".to_string()
    } else if out.chars().count() > 64 {
        out.chars().take(64).collect::<String>().trim_end_matches('-').to_string()
    } else {
        out
    }
}

pub(crate) fn write_heartbeat(main_root: &Path) -> Result<(), String> {
    let sup_dir = main_root.join(".bee").join("supervisor");
    if let Err(e) = std::fs::create_dir_all(&sup_dir) {
        return Err(format!("failed to create supervisor directory: {e}"));
    }
    let hb_path = sup_dir.join("broker-heartbeat.json");
    let payload = json!({
        "ts": chrono::Utc::now().to_rfc3339()
    });
    crate::fsutil::write_json_atomic(&hb_path, &payload)
        .map_err(|e| format!("failed to write broker-heartbeat.json: {e}"))
}

pub(crate) fn read_max_question_rounds(main_root: &Path) -> u64 {
    let config_path = main_root.join(".bee").join("config.json");
    if let Ok(raw) = std::fs::read_to_string(&config_path) {
        if let Ok(v) = serde_json::from_str::<Value>(&raw) {
            if let Some(n) = v.get("broker").and_then(|b| b.get("max_question_rounds")).and_then(Value::as_u64) {
                return n;
            }
        }
    }
    2
}

pub(crate) fn find_highest_result(job_dir: &Path) -> Option<(u32, MailboxResult)> {
    let rd = std::fs::read_dir(job_dir).ok()?;
    let mut highest: Option<(u32, PathBuf)> = None;
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some(digits) = name.strip_prefix("result-").and_then(|s| s.strip_suffix(".json")) {
            if let Ok(r) = digits.parse::<u32>() {
                if highest.as_ref().map_or(true, |(prev, _)| r > *prev) {
                    highest = Some((r, entry.path()));
                }
            }
        }
    }
    let (round, path) = highest?;
    let raw = std::fs::read_to_string(path).ok()?;
    let result = mailbox::parse_result_text(round, &raw).ok()?;
    Some((round, result))
}

pub(crate) fn start_child_job_for_answered_question(
    main_root: &Path,
    parent_job_id: &str,
    parent_round: u32,
    answer_text: &str,
    spawner: &dyn JobSpawner,
    paseo: &dyn crate::herding::paseo::PaseoCli,
) -> Result<String, BrokerError> {
    let bee_dir = main_root.join(".bee");
    let mbox_dir = mailbox::mailbox_dir(&bee_dir, parent_job_id);
    let job_path = mailbox::job_path(&bee_dir, parent_job_id);

    let job_raw = match crate::fsutil::read_json(&job_path) {
        crate::fsutil::ReadJson::Parsed(Value::Object(v)) => Value::Object(v),
        _ => {
            return Err(BrokerError {
                code: "job_spec_missing",
                message: format!("cannot read job.json for {parent_job_id}"),
            });
        }
    };

    let (_, parent_result) = match find_highest_result(&mbox_dir) {
        Some(res) if res.0 == parent_round => res,
        _ => {
            return Err(BrokerError {
                code: "result_missing",
                message: format!("cannot find result-{parent_round}.json for {parent_job_id}"),
            });
        }
    };

    let question_text = parent_result
        .question
        .as_ref()
        .map(|q| q.text.as_str())
        .unwrap_or("");

    let files_changed_str = if parent_result.files_changed.is_empty() {
        "none".to_string()
    } else {
        parent_result.files_changed.join(", ")
    };

    let original_task = job_raw
        .get("task")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let cwd = job_raw
        .get("cwd")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| main_root.to_path_buf());

    let agent = job_raw
        .get("agent")
        .and_then(Value::as_str)
        .map(str::to_string);
    let seat = job_raw
        .get("seat")
        .and_then(Value::as_str)
        .map(str::to_string);
    let cell_id = job_raw
        .get("cell_id")
        .and_then(Value::as_str)
        .map(str::to_string);
    let no_pane = job_raw
        .get("no_pane")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let inbox_session = job_raw
        .get("inbox_session")
        .and_then(Value::as_str)
        .map(str::to_string);
    let parent_question_round = job_raw
        .get("question_round")
        .and_then(Value::as_u64)
        .map(|n| n as u32)
        .unwrap_or(0);
    let parent_question_of = job_raw
        .get("question_of")
        .and_then(Value::as_str)
        .map(str::to_string);

    let next_question_round = parent_question_round + 1;
    let max_rounds = read_max_question_rounds(main_root);
    let no_question = next_question_round > (max_rounds as u32);

    let child_task = format!(
        "{original_task}\n\n# Prior Round Question and Answer\nQuestion: {question_text}\nAnswer: {answer_text}\nFiles changed in prior round: {files_changed_str}"
    );

    let is_paseo = job_raw
        .get("transport")
        .and_then(Value::as_str)
        .map(|s| s == "paseo")
        .unwrap_or(false);
    let paseo_agent_id = job_raw
        .get("paseo_agent_id")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty());

    if is_paseo {
        if let Some(agent_id) = paseo_agent_id {
            let inspect_argv = crate::herding::paseo::inspect_argv(agent_id);
            if let Ok(inspect_out) = paseo.call(&inspect_argv) {
                if crate::herding::paseo::parse_inspect(&inspect_out) == Some(crate::herding::paseo::PaseoState::Idle) {
                    let spawn_args = SpawnArgs {
                        job_id: String::new(),
                        task: child_task,
                        cwd,
                        agent,
                        seat,
                        cell_id,
                        no_pane,
                        inbox_session,
                        question_of: None,
                        question_round: next_question_round,
                        no_question,
                        continue_job: Some(parent_job_id.to_string()),
                    };

                    if let Err(e) = spawner.spawn(main_root, &spawn_args) {
                        return Err(BrokerError {
                            code: "spawn_failed",
                            message: format!("failed to spawn child job: {e}"),
                        });
                    }

                    let redispatch_record = json!({
                        "child_job_id": parent_job_id,
                        "round": next_question_round,
                        "dispatched_at": chrono::Utc::now().to_rfc3339(),
                        "mode": "continue",
                    });
                    let redispatch_file = mbox_dir.join(format!("redispatched-{parent_round}.json"));
                    let _ = crate::fsutil::write_json_atomic(&redispatch_file, &redispatch_record);

                    return Ok(parent_job_id.to_string());
                }
            }
        }
    }

    let root_job_id = parent_question_of.as_deref().unwrap_or(parent_job_id);
    let child_job_id = format!("{root_job_id}-q{next_question_round}");

    let spawn_args = SpawnArgs {
        job_id: child_job_id.clone(),
        task: child_task,
        cwd,
        agent,
        seat,
        cell_id,
        no_pane,
        inbox_session,
        question_of: Some(root_job_id.to_string()),
        question_round: next_question_round,
        no_question,
        continue_job: None,
    };

    if let Err(e) = spawner.spawn(main_root, &spawn_args) {
        return Err(BrokerError {
            code: "spawn_failed",
            message: format!("failed to spawn child job: {e}"),
        });
    }

    let redispatch_record = json!({
        "child_job_id": child_job_id,
        "round": next_question_round,
        "dispatched_at": chrono::Utc::now().to_rfc3339()
    });
    let redispatch_file = mbox_dir.join(format!("redispatched-{parent_round}.json"));
    let _ = crate::fsutil::write_json_atomic(&redispatch_file, &redispatch_record);

    Ok(child_job_id)
}


pub(crate) fn tick_with(
    main_root: &Path,
    json: bool,
    advisor_runner: &dyn AdvisorRunner,
    spawner: &dyn JobSpawner,
    paseo: &dyn crate::herding::paseo::PaseoCli,
) -> Result<TickOutcome, BrokerError> {
    if let Err(e) = write_heartbeat(main_root) {
        return Err(BrokerError {
            code: "heartbeat_failed",
            message: e,
        });
    }

    let bee_dir = main_root.join(".bee");
    let mbox_root = bee_dir.join("mailbox");
    let mut notices_sent = 0usize;

    if let Ok(rd) = std::fs::read_dir(&mbox_root) {
        for entry in rd.flatten() {
            let job_dir = entry.path();
            if !job_dir.is_dir() {
                continue;
            }
            let job_id = entry.file_name().to_string_lossy().into_owned();
            let job_json_path = mailbox::job_path(&bee_dir, &job_id);
            let job_spec = match crate::fsutil::read_json(&job_json_path) {
                crate::fsutil::ReadJson::Parsed(Value::Object(m)) => Value::Object(m),
                _ => continue,
            };

            let question_of = job_spec.get("question_of").and_then(Value::as_str);
            let inbox_session = job_spec.get("inbox_session").and_then(Value::as_str).filter(|s| !s.trim().is_empty());
            let notice_marker = job_dir.join("broker-notice-sent.json");

            if question_of.is_some() && inbox_session.is_none() && !notice_marker.exists() {
                if let Some((_round, result)) = find_highest_result(&job_dir) {
                    if result.status != MailboxStatus::Question {
                        let leader_session = job_spec
                            .get("leader_session")
                            .and_then(Value::as_str)
                            .unwrap_or("unattributed");
                        let status_label = status_str(result.status);
                        let point_key = format!("broker-notice-{job_id}");
                        let question_msg = format!("Re-dispatched job {job_id} finished with status {status_label}.");
                        let _ = crate::verbs::supervisor::record_intervention_into(
                            main_root,
                            "herding broker tick",
                            "broker-notice",
                            Some("none"),
                            Some(leader_session),
                            Some(&point_key),
                            Some(&question_msg),
                            None,
                        );
                        let _ = crate::fsutil::write_json_atomic(&notice_marker, &json!({"sent_at": chrono::Utc::now().to_rfc3339()}));
                        notices_sent += 1;
                    }
                }
            }

            let is_paseo = job_spec
                .get("transport")
                .and_then(Value::as_str)
                .map(|s| s == "paseo")
                .unwrap_or(false);
            if is_paseo {
                if let Some(agent_id) = job_spec
                    .get("paseo_agent_id")
                    .and_then(Value::as_str)
                    .filter(|s| !s.trim().is_empty())
                {
                    let inspect_argv = crate::herding::paseo::inspect_argv(agent_id);
                    if let Ok(inspect_out) = paseo.call(&inspect_argv) {
                        if crate::herding::paseo::parse_inspect(&inspect_out)
                            == Some(crate::herding::paseo::PaseoState::Blocked)
                        {
                            let pending = crate::herding::paseo::parse_pending_permissions(&inspect_out);
                            let leader_session = job_spec
                                .get("leader_session")
                                .and_then(Value::as_str)
                                .unwrap_or("unattributed");
                            for (req_id, tool) in pending {
                                let safe_req_id = req_id.replace(['/', '\\', '\0'], "_");
                                let marker_path =
                                    job_dir.join(format!("permission-marker-{safe_req_id}.json"));
                                if marker_path.exists() {
                                    continue;
                                }
                                let raw_key = format!("perm-{job_id}-{req_id}");
                                let point_key = slugify_point_key(&raw_key);
                                let tool_desc = if tool.trim().is_empty() { "tool" } else { &tool };
                                let msg = format!(
                                    "Paseo job {job_id} requires permission for tool {tool_desc}. Answer with `bee herding permit`."
                                );
                                let rec = crate::verbs::supervisor::record_intervention_into(
                                    main_root,
                                    "herding broker tick",
                                    "permission",
                                    Some("big-decision"),
                                    Some(leader_session),
                                    Some(&point_key),
                                    Some(&msg),
                                    None,
                                );
                                if rec.is_ok() {
                                    let _ = crate::fsutil::write_json_atomic(
                                        &marker_path,
                                        &json!({
                                            "request_id": req_id,
                                            "tool": tool,
                                            "sent_at": chrono::Utc::now().to_rfc3339()
                                        }),
                                    );
                                    notices_sent += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Ok(rd) = std::fs::read_dir(&mbox_root) {
        for entry in rd.flatten() {
            let job_dir = entry.path();
            if !job_dir.is_dir() {
                continue;
            }
            let job_id = entry.file_name().to_string_lossy().into_owned();
            let (round, result) = match find_highest_result(&job_dir) {
                Some(r) => r,
                None => continue,
            };

            if result.status != MailboxStatus::Question {
                continue;
            }

            let answer_path = job_dir.join(format!("answer-{round}.json"));
            if answer_path.exists() {
                continue;
            }

            let claim_path = job_dir.join(format!("claim-{round}.json"));
            if claim_path.exists() {
                let point_key = format!("q-{job_id}-{round}");
                let store = crate::verbs::supervisor::read_interventions(main_root);
                let consented_intervention = store.rows.iter().find(|i| {
                    i.point_key == point_key && i.consented_at.is_some()
                });

                if consented_intervention.is_some() {
                    if let Some(leaning) = &result.leaning {
                        let answer_obj = json!({
                            "job_id": job_id,
                            "round": round,
                            "text": leaning,
                            "source": "consent-leaning",
                            "at": chrono::Utc::now().to_rfc3339()
                        });
                        let tmp_ans = job_dir.join(format!("answer-{round}.json.tmp"));
                        let _ = std::fs::write(&tmp_ans, serde_json::to_string_pretty(&answer_obj).unwrap());
                        let _ = std::fs::rename(&tmp_ans, &answer_path);
                        let _ = start_child_job_for_answered_question(main_root, &job_id, round, leaning, spawner, paseo);
                    }
                }
            }
        }
    }


    let mut candidate: Option<(String, u32, MailboxResult, PathBuf)> = None;
    if let Ok(rd) = std::fs::read_dir(&mbox_root) {
        let mut entries: Vec<_> = rd.flatten().collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let job_dir = entry.path();
            if !job_dir.is_dir() {
                continue;
            }
            let job_id = entry.file_name().to_string_lossy().into_owned();
            let (round, result) = match find_highest_result(&job_dir) {
                Some(r) => r,
                None => continue,
            };

            if result.status != MailboxStatus::Question {
                continue;
            }

            let answer_path = job_dir.join(format!("answer-{round}.json"));
            if answer_path.exists() {
                continue;
            }

            let claim_path = job_dir.join(format!("claim-{round}.json"));
            if claim_path.exists() {
                continue;
            }

            candidate = Some((job_id, round, result, job_dir));
            break;
        }
    }

    let claimed_job = if let Some((job_id, round, result, job_dir)) = candidate {
        let claim_path = job_dir.join(format!("claim-{round}.json"));
        let claim_tmp = job_dir.join(format!("claim-{round}.json.tmp"));
        let claim_obj = json!({
            "job_id": job_id,
            "round": round,
            "claimed_at": chrono::Utc::now().to_rfc3339()
        });

        if std::fs::write(&claim_tmp, serde_json::to_string_pretty(&claim_obj).unwrap()).is_err() {
            return Ok(TickOutcome { heartbeat: true, claimed: None, notices_sent });
        }
        if std::fs::rename(&claim_tmp, &claim_path).is_err() {
            let _ = std::fs::remove_file(&claim_tmp);
            return Ok(TickOutcome { heartbeat: true, claimed: None, notices_sent });
        }

        let job_json_path = mailbox::job_path(&bee_dir, &job_id);
        let job_spec = match crate::fsutil::read_json(&job_json_path) {
            crate::fsutil::ReadJson::Parsed(Value::Object(m)) => Value::Object(m),
            _ => Value::Object(Map::new()),
        };

        let leader_session = job_spec
            .get("leader_session")
            .and_then(Value::as_str)
            .unwrap_or("unattributed")
            .to_string();

        let runtime = job_spec
            .get("runtime")
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                if job_spec.get("inbox_session").is_some() {
                    "pi"
                } else {
                    "claude"
                }
            })
            .to_string();

        let question = result.question.unwrap_or(MailboxQuestion {
            text: result.summary.clone(),
            kind: "technical".to_string(),
        });

        let point_key = format!("q-{job_id}-{round}");

        if question.kind == "gate" || question.kind == "product" {
            let clean_q = sanitize_question_for_intervention(&question.text);
            let _ = crate::verbs::supervisor::record_intervention_into(
                main_root,
                "herding broker tick",
                "intervention",
                Some("big-decision"),
                Some(&leader_session),
                Some(&point_key),
                Some(&clean_q),
                None,
            );
        } else {
            let adv_res = advisor_runner.run_advisor(main_root, &runtime, &question.text, &question.kind);
            match adv_res {
                Ok(AdvisorOutcome::Done { summary, report_path }) => {
                    let answer_text = match &report_path {
                        Some(rp) if !rp.trim().is_empty() => format!("{summary}\n\nReport: {rp}"),
                        _ => summary.clone(),
                    };
                    let answer_obj = json!({
                        "job_id": job_id,
                        "round": round,
                        "text": answer_text,
                        "summary": summary,
                        "report_path": report_path,
                        "source": "advisor",
                        "at": chrono::Utc::now().to_rfc3339()
                    });
                    let ans_path = job_dir.join(format!("answer-{round}.json"));
                    let ans_tmp = job_dir.join(format!("answer-{round}.json.tmp"));
                    let _ = std::fs::write(&ans_tmp, serde_json::to_string_pretty(&answer_obj).unwrap());
                    let _ = std::fs::rename(&ans_tmp, &ans_path);
                    let _ = start_child_job_for_answered_question(main_root, &job_id, round, &answer_text, spawner, paseo);
                }
                Ok(AdvisorOutcome::Blocked { .. })
                | Ok(AdvisorOutcome::Failed(_))
                | Ok(AdvisorOutcome::NotHerding)
                | Err(_) => {
                    let clean_q = sanitize_question_for_intervention(&question.text);
                    let _ = crate::verbs::supervisor::record_intervention_into(
                        main_root,
                        "herding broker tick",
                        "intervention",
                        Some("worker-question"),
                        Some(&leader_session),
                        Some(&point_key),
                        Some(&clean_q),
                        None,
                    );
                }
            }
        }

        Some(job_id)
    } else {
        None
    };

    let outcome = TickOutcome {
        heartbeat: true,
        claimed: claimed_job,
        notices_sent,
    };

    if json {
        let out = json!({
            "heartbeat": outcome.heartbeat,
            "claimed": outcome.claimed,
            "notices_sent": outcome.notices_sent,
        });
        println!("{}", serde_json::to_string(&out).unwrap());
    }

    Ok(outcome)
}

pub(crate) fn answer_with(
    main_root: &Path,
    job_id: &str,
    text: &str,
    json: bool,
    spawner: &dyn JobSpawner,
    paseo: &dyn crate::herding::paseo::PaseoCli,
) -> Result<AnswerOutcome, BrokerError> {
    let bee_dir = main_root.join(".bee");
    let mbox_dir = mailbox::mailbox_dir(&bee_dir, job_id);
    if !mbox_dir.is_dir() {
        return Err(BrokerError {
            code: "job_not_found",
            message: format!("herding answer: job \"{job_id}\" not found. FIX: check the job id with bee herding status"),
        });
    }

    let (round, result) = match find_highest_result(&mbox_dir) {
        Some(r) => r,
        None => {
            return Err(BrokerError {
                code: "no_question",
                message: format!("herding answer: job \"{job_id}\" has no result file. FIX: check job with bee herding status"),
            });
        }
    };

    if result.status != MailboxStatus::Question {
        return Err(BrokerError {
            code: "no_question",
            message: format!("herding answer: job \"{job_id}\" round {round} has status \"{}\" (want \"question\"). FIX: check job status with bee herding status", status_str(result.status)),
        });
    }

    let ans_path = mbox_dir.join(format!("answer-{round}.json"));
    let ans_tmp = mbox_dir.join(format!("answer-{round}.json.tmp"));
    let answer_obj = json!({
        "job_id": job_id,
        "round": round,
        "text": text,
        "source": "human",
        "at": chrono::Utc::now().to_rfc3339()
    });

    if let Err(e) = std::fs::write(&ans_tmp, serde_json::to_string_pretty(&answer_obj).unwrap()) {
        return Err(BrokerError {
            code: "write_failed",
            message: format!("herding answer: failed to write {}: {e}", ans_tmp.display()),
        });
    }

    if let Err(e) = std::fs::rename(&ans_tmp, &ans_path) {
        let _ = std::fs::remove_file(&ans_tmp);
        return Err(BrokerError {
            code: "rename_failed",
            message: format!("herding answer: failed to rename {} to {}: {e}", ans_tmp.display(), ans_path.display()),
        });
    }

    let child_job_id = start_child_job_for_answered_question(main_root, job_id, round, text, spawner, paseo)?;


    let outcome = AnswerOutcome {
        job_id: job_id.to_string(),
        round,
        text: text.to_string(),
        child_job_id: child_job_id.clone(),
    };

    if json {
        let out = json!({
            "job_id": outcome.job_id,
            "round": outcome.round,
            "text": outcome.text,
            "child_job_id": outcome.child_job_id,
        });
        println!("{}", serde_json::to_string(&out).unwrap());
    } else {
        println!("herding: answered question for job {job_id} round {round} (started child job {child_job_id})");
    }

    Ok(outcome)
}

struct ParsedArgs<'a> {
    job_id: Option<&'a str>,
    text: Option<&'a str>,
    main_root: Option<&'a str>,
    json: bool,
}

fn parse_args<'a>(args: &[&'a str]) -> ParsedArgs<'a> {
    let mut job_id = None;
    let mut text = None;
    let mut main_root = None;
    let mut json = false;
    let mut i = 0usize;

    while i < args.len() {
        match args[i] {
            "--json" => {
                json = true;
                i += 1;
            }
            "--main-root" => {
                main_root = args.get(i + 1).copied();
                i += 2;
            }
            arg if arg.starts_with("--main-root=") => {
                main_root = Some(&arg["--main-root=".len()..]);
                i += 1;
            }
            "--job" | "--job-id" => {
                job_id = args.get(i + 1).copied();
                i += 2;
            }
            arg if arg.starts_with("--job=") => {
                job_id = Some(&arg["--job=".len()..]);
                i += 1;
            }
            arg if arg.starts_with("--job-id=") => {
                job_id = Some(&arg["--job-id=".len()..]);
                i += 1;
            }
            "--text" => {
                text = args.get(i + 1).copied();
                i += 2;
            }
            arg if arg.starts_with("--text=") => {
                text = Some(&arg["--text=".len()..]);
                i += 1;
            }
            arg if !arg.starts_with('-') && job_id.is_none() => {
                job_id = Some(arg);
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    ParsedArgs { job_id, text, main_root, json }
}

pub(crate) fn route_broker(rest: &[&str]) -> Option<ExitCode> {
    let (subverb, args) = rest.split_first()?;
    match *subverb {
        "tick" => Some(tick(args)),
        _ => None,
    }
}

pub(crate) fn tick(args: &[&str]) -> ExitCode {
    let parsed = parse_args(args);
    let main_root = match resolve_main_root(parsed.main_root) {
        Some(r) => r,
        None => {
            eprintln!("herding broker tick: could not resolve main checkout root. FIX: pass --main-root <path>");
            return ExitCode::FAILURE;
        }
    };

    let cfg = super::run::read_main_config(&main_root);
    let bee_dir = main_root.join(".bee");
    let paseo_cli: Box<dyn crate::herding::paseo::PaseoCli> = {
        if crate::herding::repo_uses_paseo(&cfg, &bee_dir) && crate::herding::paseo::daemon_reachable() {
            let cmd = crate::herding::paseo::paseo_command(&cfg);
            let real = crate::herding::paseo::RealPaseoCli::new(cmd).with_timeout(std::time::Duration::from_secs(5));
            Box::new(crate::herding::paseo::FailFastPaseoCli::new(Box::new(real)))
        } else {
            struct UnavailablePaseoCli;
            impl crate::herding::paseo::PaseoCli for UnavailablePaseoCli {
                fn call(&self, _args: &[String]) -> Result<String, String> {
                    Err("paseo is disabled or daemon unreachable".to_string())
                }
            }
            Box::new(UnavailablePaseoCli)
        }
    };

    match tick_with(&main_root, parsed.json, &RealAdvisorRunner, &RealJobSpawner, paseo_cli.as_ref()) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            if parsed.json {
                let err_obj = json!({
                    "error": e.code,
                    "message": e.message,
                });
                println!("{}", serde_json::to_string(&err_obj).unwrap());
            } else {
                eprintln!("{}", e.message);
            }
            ExitCode::FAILURE
        }
    }
}

pub(crate) fn answer(args: &[&str]) -> ExitCode {
    let parsed = parse_args(args);
    let job_id = match parsed.job_id {
        Some(id) => id,
        None => {
            eprintln!("herding answer: missing required --job argument. FIX: run bee herding answer --job <id> --text <text>");
            return ExitCode::FAILURE;
        }
    };
    let text = match parsed.text {
        Some(t) => t,
        None => {
            eprintln!("herding answer: missing required --text argument. FIX: run bee herding answer --job <id> --text <text>");
            return ExitCode::FAILURE;
        }
    };
    let main_root = match resolve_main_root(parsed.main_root) {
        Some(r) => r,
        None => {
            eprintln!("herding answer: could not resolve main checkout root. FIX: pass --main-root <path>");
            return ExitCode::FAILURE;
        }
    };

    let cfg = super::run::read_main_config(&main_root);
    let paseo_cmd = crate::herding::paseo::paseo_command(&cfg);
    let paseo_cli = crate::herding::paseo::RealPaseoCli::new(paseo_cmd);

    match answer_with(&main_root, job_id, text, parsed.json, &RealJobSpawner, &paseo_cli) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            if parsed.json {
                let err_obj = json!({
                    "error": e.code,
                    "message": e.message,
                });
                println!("{}", serde_json::to_string(&err_obj).unwrap());
            } else {
                eprintln!("{}", e.message);
            }
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct FakeAdvisorRunner {
        outcome: RefCell<Option<AdvisorOutcome>>,
        calls: RefCell<Vec<(String, String, String)>>,
    }

    impl FakeAdvisorRunner {
        fn new(outcome: Option<AdvisorOutcome>) -> Self {
            Self {
                outcome: RefCell::new(outcome),
                calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl AdvisorRunner for FakeAdvisorRunner {
        fn run_advisor(
            &self,
            _main_root: &Path,
            runtime: &str,
            question: &str,
            question_kind: &str,
        ) -> Result<AdvisorOutcome, String> {
            self.calls.borrow_mut().push((runtime.to_string(), question.to_string(), question_kind.to_string()));
            let out = self.outcome.borrow_mut().take().unwrap_or(AdvisorOutcome::NotHerding);
            Ok(out)
        }
    }

    struct FakeJobSpawner {
        spawned: RefCell<Vec<SpawnArgs>>,
    }

    impl FakeJobSpawner {
        fn new() -> Self {
            Self {
                spawned: RefCell::new(Vec::new()),
            }
        }
    }

    impl JobSpawner for FakeJobSpawner {
        fn spawn(&self, _main_root: &Path, args: &SpawnArgs) -> Result<(), String> {
            self.spawned.borrow_mut().push(args.clone());
            Ok(())
        }
    }

    struct FakePaseoCli {
        calls: std::sync::Mutex<Vec<Vec<String>>>,
        inspect: Result<String, String>,
    }

    impl FakePaseoCli {
        fn new(inspect: Result<String, String>) -> Self {
            Self {
                calls: std::sync::Mutex::new(Vec::new()),
                inspect,
            }
        }
    }

    impl crate::herding::paseo::PaseoCli for FakePaseoCli {
        fn call(&self, args: &[String]) -> Result<String, String> {
            self.calls.lock().unwrap().push(args.to_vec());
            if args.first().map(|s| s.as_str()) == Some("inspect") {
                self.inspect.clone()
            } else {
                Ok(String::new())
            }
        }
    }

    fn setup_test_job_full(
        main_root: &Path,
        job_id: &str,
        round: u32,
        status: &str,
        q_text: Option<&str>,
        q_kind: Option<&str>,
        leaning: Option<&str>,
        transport: Option<&str>,
        paseo_agent_id: Option<&str>,
    ) {
        let bee_dir = main_root.join(".bee");
        let mbox_dir = bee_dir.join("mailbox").join(job_id);
        std::fs::create_dir_all(&mbox_dir).unwrap();

        let mut job_json = json!({
            "job_id": job_id,
            "task": "Build the feature",
            "cwd": main_root.display().to_string(),
            "agent": "agent-1",
            "seat": "seat-code",
            "cell_id": "cell-1",
            "no_pane": false,
            "inbox_session": "sess-inbox",
            "leader_session": "sess-leader",
            "question_of": Value::Null,
            "question_round": 0,
        });
        if let Some(t) = transport {
            job_json["transport"] = Value::String(t.to_string());
        }
        if let Some(pid) = paseo_agent_id {
            job_json["paseo_agent_id"] = Value::String(pid.to_string());
        }
        std::fs::write(mbox_dir.join("job.json"), serde_json::to_string_pretty(&job_json).unwrap()).unwrap();

        let mut res_obj = json!({
            "status": status,
            "summary": "Round summary",
            "files_changed": ["src/lib.rs"]
        });
        if status != "question" {
            res_obj["proof"] = Value::String("cargo test - green".to_string());
        }
        if let (Some(text), Some(kind)) = (q_text, q_kind) {
            res_obj["question"] = json!({
                "text": text,
                "kind": kind
            });
        }
        if let Some(l) = leaning {
            res_obj["leaning"] = Value::String(l.to_string());
        }
        std::fs::write(mbox_dir.join(format!("result-{round}.json")), serde_json::to_string_pretty(&res_obj).unwrap()).unwrap();
    }

    fn setup_test_job(
        main_root: &Path,
        job_id: &str,
        round: u32,
        status: &str,
        q_text: Option<&str>,
        q_kind: Option<&str>,
        leaning: Option<&str>,
    ) {
        setup_test_job_full(main_root, job_id, round, status, q_text, q_kind, leaning, None, None);
    }


    #[test]
    fn gate_or_product_question_becomes_intervention_without_advisor_call() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        setup_test_job(main_root, "job-gate-1", 1, "question", Some("Should we allow bypass?"), Some("gate"), None);

        let advisor = FakeAdvisorRunner::new(Some(AdvisorOutcome::Done { summary: "Yes".into(), report_path: None }));
        let spawner = FakeJobSpawner::new();
        let fake_paseo = FakePaseoCli::new(Ok(String::new()));

        let out = tick_with(main_root, false, &advisor, &spawner, &fake_paseo).expect("tick");
        assert_eq!(out.claimed.as_deref(), Some("job-gate-1"));
        assert!(advisor.calls.borrow().is_empty());

        let store = crate::verbs::supervisor::read_interventions(main_root);
        let intervention = store.rows.iter().find(|i| i.point_key == "q-job-gate-1-1").expect("intervention written");
        assert_eq!(intervention.signal, "big-decision");
        assert_eq!(intervention.target_session, "sess-leader");
        assert_eq!(intervention.kind, "intervention");
        assert!(spawner.spawned.borrow().is_empty());
    }

    #[test]
    fn product_question_becomes_intervention_with_big_decision_signal() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        setup_test_job(main_root, "job-prod-1", 1, "question", Some("Which pricing model?"), Some("product"), None);

        let advisor = FakeAdvisorRunner::new(None);
        let spawner = FakeJobSpawner::new();
        let fake_paseo = FakePaseoCli::new(Ok(String::new()));

        let out = tick_with(main_root, false, &advisor, &spawner, &fake_paseo).expect("tick");
        assert_eq!(out.claimed.as_deref(), Some("job-prod-1"));
        assert!(advisor.calls.borrow().is_empty());

        let store = crate::verbs::supervisor::read_interventions(main_root);
        let intervention = store.rows.iter().find(|i| i.point_key == "q-job-prod-1-1").expect("intervention written");
        assert_eq!(intervention.signal, "big-decision");
    }

    #[test]
    fn technical_question_with_herding_advisor_done_writes_answer_and_spawns() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        setup_test_job(main_root, "job-tech-1", 1, "question", Some("Use trait or enum?"), Some("technical"), None);

        let advisor = FakeAdvisorRunner::new(Some(AdvisorOutcome::Done {
            summary: "Use trait".into(),
            report_path: Some("/reports/rep.md".into()),
        }));
        let spawner = FakeJobSpawner::new();
        let fake_paseo = FakePaseoCli::new(Ok(String::new()));

        let out = tick_with(main_root, false, &advisor, &spawner, &fake_paseo).expect("tick");
        assert_eq!(out.claimed.as_deref(), Some("job-tech-1"));
        assert_eq!(advisor.calls.borrow().len(), 1);

        let ans_path = main_root.join(".bee").join("mailbox").join("job-tech-1").join("answer-1.json");
        assert!(ans_path.exists());
        let ans_data: Value = serde_json::from_str(&std::fs::read_to_string(ans_path).unwrap()).unwrap();
        assert_eq!(ans_data.get("summary").and_then(Value::as_str), Some("Use trait"));

        assert_eq!(spawner.spawned.borrow().len(), 1);
        let child = &spawner.spawned.borrow()[0];
        assert_eq!(child.question_of.as_deref(), Some("job-tech-1"));
        assert_eq!(child.question_round, 1);
        assert!(!child.no_question);
        assert!(child.task.contains("Build the feature"));
        assert!(child.task.contains("Use trait or enum?"));
        assert!(child.task.contains("src/lib.rs"));
    }

    #[test]
    fn technical_question_with_not_herding_advisor_becomes_worker_question_intervention() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        setup_test_job(main_root, "job-tech-native", 1, "question", Some("Which DB index?"), Some("technical"), None);

        let advisor = FakeAdvisorRunner::new(Some(AdvisorOutcome::NotHerding));
        let spawner = FakeJobSpawner::new();
        let fake_paseo = FakePaseoCli::new(Ok(String::new()));

        let out = tick_with(main_root, false, &advisor, &spawner, &fake_paseo).expect("tick");
        assert_eq!(out.claimed.as_deref(), Some("job-tech-native"));

        let store = crate::verbs::supervisor::read_interventions(main_root);
        let intervention = store.rows.iter().find(|i| i.point_key == "q-job-tech-native-1").expect("intervention");
        assert_eq!(intervention.signal, "worker-question");
        assert!(spawner.spawned.borrow().is_empty());
    }

    #[test]
    fn technical_question_with_blocked_advisor_becomes_worker_question_intervention() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        setup_test_job(main_root, "job-tech-blocked", 1, "question", Some("Which DB index?"), Some("technical"), None);

        let advisor = FakeAdvisorRunner::new(Some(AdvisorOutcome::Blocked {
            reason: "not sure".into(),
            report_path: Some("/rep.md".into()),
        }));
        let spawner = FakeJobSpawner::new();
        let fake_paseo = FakePaseoCli::new(Ok(String::new()));

        let out = tick_with(main_root, false, &advisor, &spawner, &fake_paseo).expect("tick");
        assert_eq!(out.claimed.as_deref(), Some("job-tech-blocked"));

        let store = crate::verbs::supervisor::read_interventions(main_root);
        let intervention = store.rows.iter().find(|i| i.point_key == "q-job-tech-blocked-1").expect("intervention");
        assert_eq!(intervention.signal, "worker-question");
    }

    #[test]
    fn second_tick_never_routes_the_same_job_and_round_twice() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        setup_test_job(main_root, "job-dedupe-1", 1, "question", Some("Need info"), Some("gate"), None);

        let advisor = FakeAdvisorRunner::new(None);
        let spawner = FakeJobSpawner::new();
        let fake_paseo = FakePaseoCli::new(Ok(String::new()));

        let out1 = tick_with(main_root, false, &advisor, &spawner, &fake_paseo).expect("tick 1");
        assert_eq!(out1.claimed.as_deref(), Some("job-dedupe-1"));

        let out2 = tick_with(main_root, false, &advisor, &spawner, &fake_paseo).expect("tick 2");
        assert_eq!(out2.claimed, None);
    }

    #[test]
    fn every_tick_writes_the_broker_heartbeat() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();

        let advisor = FakeAdvisorRunner::new(None);
        let spawner = FakeJobSpawner::new();
        let fake_paseo = FakePaseoCli::new(Ok(String::new()));

        let out = tick_with(main_root, false, &advisor, &spawner, &fake_paseo).expect("tick");
        assert!(out.heartbeat);

        let hb = main_root.join(".bee").join("supervisor").join("broker-heartbeat.json");
        assert!(hb.exists());
        let content: Value = serde_json::from_str(&std::fs::read_to_string(hb).unwrap()).unwrap();
        assert!(content.get("ts").is_some());
    }

    #[test]
    fn answer_verb_writes_answer_file_and_refuses_unknown_job() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        let spawner = FakeJobSpawner::new();
        let fake_paseo = FakePaseoCli::new(Ok(String::new()));

        let err = answer_with(main_root, "non-existent-job", "answer text", false, &spawner, &fake_paseo).unwrap_err();
        assert_eq!(err.code, "job_not_found");

        setup_test_job(main_root, "job-answer-1", 1, "question", Some("Should we proceed?"), Some("technical"), None);
        let res = answer_with(main_root, "job-answer-1", "Go ahead", false, &spawner, &fake_paseo).expect("answer");
        assert_eq!(res.round, 1);
        assert_eq!(res.text, "Go ahead");

        let ans_file = main_root.join(".bee").join("mailbox").join("job-answer-1").join("answer-1.json");
        assert!(ans_file.exists());

        assert_eq!(spawner.spawned.borrow().len(), 1);
        let child = &spawner.spawned.borrow()[0];
        assert_eq!(child.question_of.as_deref(), Some("job-answer-1"));
        assert!(child.task.contains("Go ahead"));
    }

    #[test]
    fn answer_verb_refuses_job_with_no_question() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        let spawner = FakeJobSpawner::new();
        let fake_paseo = FakePaseoCli::new(Ok(String::new()));

        setup_test_job(main_root, "job-done-1", 1, "done", None, None, None);
        let err = answer_with(main_root, "job-done-1", "some answer", false, &spawner, &fake_paseo).unwrap_err();
        assert_eq!(err.code, "no_question");
    }

    #[test]
    fn child_job_runs_with_no_question_above_max_question_rounds() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        let cfg = json!({
            "broker": {
                "max_question_rounds": 2
            }
        });
        std::fs::create_dir_all(main_root.join(".bee")).unwrap();
        std::fs::write(main_root.join(".bee").join("config.json"), serde_json::to_string(&cfg).unwrap()).unwrap();

        setup_test_job(main_root, "job-r2", 2, "question", Some("Another question?"), Some("technical"), None);
        let job_path = main_root.join(".bee").join("mailbox").join("job-r2").join("job.json");
        let mut job_obj: Value = serde_json::from_str(&std::fs::read_to_string(&job_path).unwrap()).unwrap();
        job_obj["question_round"] = Value::Number(2.into());
        std::fs::write(&job_path, serde_json::to_string_pretty(&job_obj).unwrap()).unwrap();

        let spawner = FakeJobSpawner::new();
        let fake_paseo = FakePaseoCli::new(Ok(String::new()));
        let res = answer_with(main_root, "job-r2", "Final answer", false, &spawner, &fake_paseo).expect("answer");
        assert_eq!(res.round, 2);

        assert_eq!(spawner.spawned.borrow().len(), 1);
        let child = &spawner.spawned.borrow()[0];
        assert_eq!(child.question_round, 3);
        assert!(child.no_question);
    }


    #[test]
    fn finished_child_job_without_inbox_session_gets_broker_notice() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();

        let bee_dir = main_root.join(".bee");
        let mbox_dir = bee_dir.join("mailbox").join("child-1");
        std::fs::create_dir_all(&mbox_dir).unwrap();

        let job_json = json!({
            "job_id": "child-1",
            "task": "Work",
            "cwd": main_root.display().to_string(),
            "inbox_session": Value::Null,
            "leader_session": "sess-leader-1",
            "question_of": "parent-1",
            "question_round": 1,
        });
        std::fs::write(mbox_dir.join("job.json"), serde_json::to_string_pretty(&job_json).unwrap()).unwrap();

        let res_obj = json!({
            "status": "done",
            "summary": "Child finished successfully",
            "files_changed": [],
            "proof": "cargo test - green"
        });
        std::fs::write(mbox_dir.join("result-1.json"), serde_json::to_string_pretty(&res_obj).unwrap()).unwrap();

        let advisor = FakeAdvisorRunner::new(None);
        let spawner = FakeJobSpawner::new();
        let fake_paseo = FakePaseoCli::new(Ok(String::new()));

        let out = tick_with(main_root, false, &advisor, &spawner, &fake_paseo).expect("tick");
        assert_eq!(out.notices_sent, 1);

        let store = crate::verbs::supervisor::read_interventions(main_root);
        let notice = store.rows.iter().find(|i| i.kind == "broker-notice").expect("broker-notice written");
        assert_eq!(notice.target_session, "sess-leader-1");
        assert!(notice.question.contains("child-1"));
        assert!(mbox_dir.join("broker-notice-sent.json").exists());

        let out2 = tick_with(main_root, false, &advisor, &spawner, &fake_paseo).expect("second tick");
        assert_eq!(out2.notices_sent, 0);
    }

    #[test]
    fn consented_technical_question_uses_leaning_and_redispatches() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        setup_test_job(main_root, "job-consent-1", 1, "question", Some("Which option?"), Some("technical"), Some("Pick option B"));

        let bee_dir = main_root.join(".bee");
        let mbox_dir = bee_dir.join("mailbox").join("job-consent-1");
        std::fs::write(mbox_dir.join("claim-1.json"), r#"{"job_id":"job-consent-1","round":1}"#).unwrap();

        let intervention = crate::verbs::supervisor::record_intervention_into(
            main_root,
            "herding broker tick",
            "intervention",
            Some("worker-question"),
            Some("sess-leader"),
            Some("q-job-consent-1-1"),
            Some("Which option?"),
            None,
        ).unwrap();

        let store_path = main_root.join(".bee").join("supervisor").join("interventions.jsonl");
        let consent_event = json!({
            "event": "consented",
            "id": intervention.id,
            "consented_at": "2026-10-03T12:00:00Z"
        });
        crate::fsutil::append_jsonl(&store_path, &consent_event).unwrap();

        let advisor = FakeAdvisorRunner::new(None);
        let spawner = FakeJobSpawner::new();
        let fake_paseo = FakePaseoCli::new(Ok(String::new()));

        let _ = tick_with(main_root, false, &advisor, &spawner, &fake_paseo).expect("tick");

        let ans_path = mbox_dir.join("answer-1.json");
        assert!(ans_path.exists());
        let ans_data: Value = serde_json::from_str(&std::fs::read_to_string(ans_path).unwrap()).unwrap();
        assert_eq!(ans_data.get("text").and_then(Value::as_str), Some("Pick option B"));

        assert_eq!(spawner.spawned.borrow().len(), 1);
        let child = &spawner.spawned.borrow()[0];
        assert!(child.task.contains("Pick option B"));
    }

    #[test]
    fn consented_technical_question_with_no_leaning_stays_waiting() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        setup_test_job(main_root, "job-nolearning-1", 1, "question", Some("No preference?"), Some("technical"), None);

        let bee_dir = main_root.join(".bee");
        let mbox_dir = bee_dir.join("mailbox").join("job-nolearning-1");
        std::fs::write(mbox_dir.join("claim-1.json"), r#"{"job_id":"job-nolearning-1","round":1}"#).unwrap();

        let intervention = crate::verbs::supervisor::record_intervention_into(
            main_root,
            "herding broker tick",
            "intervention",
            Some("worker-question"),
            Some("sess-leader"),
            Some("q-job-nolearning-1-1"),
            Some("No preference?"),
            None,
        ).unwrap();

        let store_path = main_root.join(".bee").join("supervisor").join("interventions.jsonl");
        let consent_event = json!({
            "event": "consented",
            "id": intervention.id,
            "consented_at": "2026-10-03T12:00:00Z"
        });
        crate::fsutil::append_jsonl(&store_path, &consent_event).unwrap();

        let advisor = FakeAdvisorRunner::new(None);
        let spawner = FakeJobSpawner::new();
        let fake_paseo = FakePaseoCli::new(Ok(String::new()));

        let _ = tick_with(main_root, false, &advisor, &spawner, &fake_paseo).expect("tick");

        let ans_path = mbox_dir.join("answer-1.json");
        assert!(!ans_path.exists());
        assert!(spawner.spawned.borrow().is_empty());
    }

    #[test]
    fn idle_paseo_parent_spawns_once_with_continue_job_set_and_no_job_id() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        setup_test_job_full(
            main_root,
            "job-paseo-idle",
            1,
            "question",
            Some("Which model?"),
            Some("technical"),
            None,
            Some("paseo"),
            Some("agent-idle-42"),
        );

        let fake_paseo = FakePaseoCli::new(Ok("{\"Status\":\"idle\"}\n".to_string()));
        let spawner = FakeJobSpawner::new();

        let res = answer_with(main_root, "job-paseo-idle", "Use Claude", false, &spawner, &fake_paseo).expect("answer");
        assert_eq!(res.round, 1);
        assert_eq!(res.job_id, "job-paseo-idle");
        assert_eq!(res.child_job_id, "job-paseo-idle");

        assert_eq!(spawner.spawned.borrow().len(), 1);
        let spawned = &spawner.spawned.borrow()[0];
        assert_eq!(spawned.continue_job.as_deref(), Some("job-paseo-idle"));
        assert!(spawned.job_id.is_empty());
        assert!(spawned.task.contains("Which model?"));
        assert!(spawned.task.contains("Use Claude"));

        let cmd = RealJobSpawner::build_command(main_root, spawned);
        let argv: Vec<String> = cmd.get_args().map(|s| s.to_string_lossy().to_string()).collect();
        assert!(argv.contains(&"--continue".to_string()));
        assert!(!argv.contains(&"--job-id".to_string()));

        let redispatch_path = main_root.join(".bee").join("mailbox").join("job-paseo-idle").join("redispatched-1.json");
        assert!(redispatch_path.exists());
        let redispatch_val: Value = serde_json::from_str(&std::fs::read_to_string(redispatch_path).unwrap()).unwrap();
        assert_eq!(redispatch_val.get("mode").and_then(Value::as_str), Some("continue"));
        assert_eq!(redispatch_val.get("child_job_id").and_then(Value::as_str), Some("job-paseo-idle"));

        let calls = fake_paseo.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], vec!["inspect", "--json", "agent-idle-42"]);
    }

    #[test]
    fn working_paseo_parent_spawns_one_child_job() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        setup_test_job_full(
            main_root,
            "job-paseo-work",
            1,
            "question",
            Some("Which model?"),
            Some("technical"),
            None,
            Some("paseo"),
            Some("agent-busy-99"),
        );

        let fake_paseo = FakePaseoCli::new(Ok("{\"Status\":\"running\"}\n".to_string()));
        let spawner = FakeJobSpawner::new();

        let res = answer_with(main_root, "job-paseo-work", "Use Sonnet", false, &spawner, &fake_paseo).expect("answer");
        assert_eq!(res.round, 1);
        assert_eq!(res.job_id, "job-paseo-work");
        assert_eq!(res.child_job_id, "job-paseo-work-q1");

        assert_eq!(spawner.spawned.borrow().len(), 1);
        let spawned = &spawner.spawned.borrow()[0];
        assert_eq!(spawned.continue_job, None);
        assert_eq!(spawned.job_id, "job-paseo-work-q1");

        let cmd = RealJobSpawner::build_command(main_root, spawned);
        let argv: Vec<String> = cmd.get_args().map(|s| s.to_string_lossy().to_string()).collect();
        assert!(!argv.contains(&"--continue".to_string()));
        assert!(argv.contains(&"--job-id".to_string()));

        let redispatch_path = main_root.join(".bee").join("mailbox").join("job-paseo-work").join("redispatched-1.json");
        assert!(redispatch_path.exists());
        let redispatch_val: Value = serde_json::from_str(&std::fs::read_to_string(redispatch_path).unwrap()).unwrap();
        assert!(redispatch_val.get("mode").is_none());
        assert_eq!(redispatch_val.get("child_job_id").and_then(Value::as_str), Some("job-paseo-work-q1"));

        let calls = fake_paseo.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], vec!["inspect", "--json", "agent-busy-99"]);
    }

    #[test]
    fn non_paseo_parent_is_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        setup_test_job(
            main_root,
            "job-non-paseo",
            1,
            "question",
            Some("Which architecture?"),
            Some("technical"),
            None,
        );

        let fake_paseo = FakePaseoCli::new(Ok("{\"Status\":\"idle\"}\n".to_string()));
        let spawner = FakeJobSpawner::new();

        let res = answer_with(main_root, "job-non-paseo", "Microservices", false, &spawner, &fake_paseo).expect("answer");
        assert_eq!(res.round, 1);
        assert_eq!(res.job_id, "job-non-paseo");
        assert_eq!(res.child_job_id, "job-non-paseo-q1");

        assert_eq!(spawner.spawned.borrow().len(), 1);
        let spawned = &spawner.spawned.borrow()[0];
        assert_eq!(spawned.continue_job, None);
        assert_eq!(spawned.job_id, "job-non-paseo-q1");

        let calls = fake_paseo.calls.lock().unwrap();
        assert!(calls.is_empty());
    }

    #[test]
    fn exactly_one_spawn_per_answered_question_in_each_case() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        setup_test_job_full(
            main_root,
            "job-once-idle",
            1,
            "question",
            Some("Wait here?"),
            Some("technical"),
            None,
            Some("paseo"),
            Some("agent-once-1"),
        );

        let fake_paseo = FakePaseoCli::new(Ok("{\"Status\":\"idle\"}\n".to_string()));
        let spawner = FakeJobSpawner::new();

        let res1 = answer_with(main_root, "job-once-idle", "Proceed", false, &spawner, &fake_paseo);
        assert!(res1.is_ok());
        assert_eq!(spawner.spawned.borrow().len(), 1);

        setup_test_job_full(
            main_root,
            "job-once-working",
            1,
            "question",
            Some("Wait here?"),
            Some("technical"),
            None,
            Some("paseo"),
            Some("agent-once-2"),
        );
        let spawner2 = FakeJobSpawner::new();
        let fake_working = FakePaseoCli::new(Ok("{\"Status\":\"running\"}\n".to_string()));
        let res2 = answer_with(main_root, "job-once-working", "Proceed", false, &spawner2, &fake_working);
        assert!(res2.is_ok());
        assert_eq!(spawner2.spawned.borrow().len(), 1);
    }

    #[test]
    fn inspect_error_on_paseo_parent_falls_back_to_child_job() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        setup_test_job_full(
            main_root,
            "job-paseo-err",
            1,
            "question",
            Some("Which path?"),
            Some("technical"),
            None,
            Some("paseo"),
            Some("agent-err-1"),
        );

        let fake_paseo = FakePaseoCli::new(Err("inspect failed".to_string()));
        let spawner = FakeJobSpawner::new();

        let res = answer_with(main_root, "job-paseo-err", "Take path A", false, &spawner, &fake_paseo).expect("answer");
        assert_eq!(res.child_job_id, "job-paseo-err-q1");
        assert_eq!(spawner.spawned.borrow().len(), 1);
        let spawned = &spawner.spawned.borrow()[0];
        assert_eq!(spawned.continue_job, None);
        assert_eq!(spawned.job_id, "job-paseo-err-q1");
    }

    #[test]
    fn idle_paseo_parent_answered_by_advisor_spawns_with_continue_job() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        setup_test_job_full(
            main_root,
            "job-paseo-advisor",
            1,
            "question",
            Some("Which method?"),
            Some("technical"),
            None,
            Some("paseo"),
            Some("agent-adv-1"),
        );

        let fake_paseo = FakePaseoCli::new(Ok("{\"Status\":\"idle\"}\n".to_string()));
        let advisor = FakeAdvisorRunner::new(Some(AdvisorOutcome::Done {
            summary: "Use method A".into(),
            report_path: None,
        }));
        let spawner = FakeJobSpawner::new();

        let out = tick_with(main_root, false, &advisor, &spawner, &fake_paseo).expect("tick");
        assert_eq!(out.claimed.as_deref(), Some("job-paseo-advisor"));
        assert_eq!(spawner.spawned.borrow().len(), 1);
        let spawned = &spawner.spawned.borrow()[0];
        assert_eq!(spawned.continue_job.as_deref(), Some("job-paseo-advisor"));
    }

    #[test]
    fn one_blocked_request_files_one_intervention_across_two_ticks() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        let bee_dir = main_root.join(".bee");
        let mbox_dir = bee_dir.join("mailbox").join("job-blocked-1");
        std::fs::create_dir_all(&mbox_dir).unwrap();

        let job_json = json!({
            "job_id": "job-blocked-1",
            "transport": "paseo",
            "paseo_agent_id": "agent-blocked-1",
            "leader_session": "sess-leader-blocked",
        });
        std::fs::write(mbox_dir.join("job.json"), serde_json::to_string(&job_json).unwrap()).unwrap();

        let inspect_output = json!({
            "Status": "running",
            "PendingPermissions": [
                {
                    "id": "req-99",
                    "tool": "bash"
                }
            ]
        })
        .to_string();

        let fake_paseo = FakePaseoCli::new(Ok(inspect_output));
        let advisor = FakeAdvisorRunner::new(None);
        let spawner = FakeJobSpawner::new();

        let out1 = tick_with(main_root, false, &advisor, &spawner, &fake_paseo).expect("tick 1");
        assert_eq!(out1.notices_sent, 1);

        let marker = mbox_dir.join("permission-marker-req-99.json");
        assert!(marker.exists());

        let store1 = crate::verbs::supervisor::read_interventions(main_root);
        let perms1: Vec<_> = store1.rows.iter().filter(|i| i.kind == "permission").collect();
        assert_eq!(perms1.len(), 1);
        assert_eq!(perms1[0].target_session, "sess-leader-blocked");
        assert!(perms1[0].question.contains("job-blocked-1"));
        assert!(perms1[0].question.contains("bash"));
        assert!(perms1[0].question.contains("`bee herding permit`"));

        let out2 = tick_with(main_root, false, &advisor, &spawner, &fake_paseo).expect("tick 2");
        assert_eq!(out2.notices_sent, 0);

        let store2 = crate::verbs::supervisor::read_interventions(main_root);
        let perms2: Vec<_> = store2.rows.iter().filter(|i| i.kind == "permission").collect();
        assert_eq!(perms2.len(), 1);
    }
}


