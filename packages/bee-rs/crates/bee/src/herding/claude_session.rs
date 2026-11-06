use super::paseo;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, Instant};

pub(crate) const EFFORTS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];

pub(crate) fn claude_command(cfg: &Value) -> String {
    cfg.get("herding")
        .and_then(|h| h.get("claude_command"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .unwrap_or("claude")
        .to_string()
}

pub(crate) trait ClaudeCli: Send + Sync {
    fn spawn(&self, args: &[String], cwd: &Path, env: &BTreeMap<String, String>) -> Result<String, String>;
    fn call(&self, args: &[String]) -> Result<String, String>;
}

pub(crate) struct RealClaudeCli {
    command: String,
    timeout: Duration,
}

impl RealClaudeCli {
    pub(crate) fn new(command: impl Into<String>) -> Self {
        Self { command: command.into(), timeout: Duration::from_secs(30) }
    }

    fn run(&self, mut command: std::process::Command) -> Result<String, String> {
        use std::io::Read;
        let mut child = command
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to spawn {}: {e}", self.command))?;
        let mut out_pipe = child.stdout.take();
        let mut err_pipe = child.stderr.take();
        let out_reader = std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(p) = out_pipe.as_mut() {
                let _ = p.read_to_end(&mut buf);
            }
            buf
        });
        let err_reader = std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(p) = err_pipe.as_mut() {
                let _ = p.read_to_end(&mut buf);
            }
            buf
        });
        let deadline = Instant::now() + self.timeout;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("{} timed out after {}s", self.command, self.timeout.as_secs()));
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(50)),
                Err(e) => return Err(format!("failed to wait for {}: {e}", self.command)),
            }
        };
        let stdout = String::from_utf8_lossy(&out_reader.join().unwrap_or_default()).into_owned();
        let stderr = String::from_utf8_lossy(&err_reader.join().unwrap_or_default()).into_owned();
        if status.success() {
            Ok(stdout)
        } else if !stderr.trim().is_empty() {
            Err(stderr)
        } else if !stdout.trim().is_empty() {
            Err(stdout)
        } else {
            Err(format!("{} exited with status {status}", self.command))
        }
    }
}

impl ClaudeCli for RealClaudeCli {
    fn spawn(&self, args: &[String], cwd: &Path, env: &BTreeMap<String, String>) -> Result<String, String> {
        let mut command = std::process::Command::new(&self.command);
        command.args(args).current_dir(cwd).env_clear().envs(env);
        self.run(command)
    }

    fn call(&self, args: &[String]) -> Result<String, String> {
        let mut command = paseo::scrubbed_command(&self.command);
        command.args(args);
        self.run(command)
    }
}

pub(crate) fn session_name(cell_id: Option<&str>, seat: Option<&str>, job_id: &str) -> String {
    let label = cell_id.or(seat).map(str::trim).filter(|l| !l.is_empty()).unwrap_or("run");
    let alnum: String = job_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    let tail: String = alnum.chars().rev().take(12).collect::<Vec<_>>().into_iter().rev().collect();
    format!("bee-{label}-{}", tail.trim_start_matches('-'))
}

pub(crate) fn settings_json(env: &BTreeMap<String, String>) -> String {
    let mut settings = json!({ "worktree": { "bgIsolation": "none" } });
    if !env.is_empty() {
        let pairs: Map<String, Value> = env.iter().map(|(k, v)| (k.clone(), Value::String(v.clone()))).collect();
        settings["env"] = Value::Object(pairs);
    }
    settings.to_string()
}

pub(crate) fn spawn_argv(name: &str, model: &str, effort: Option<&str>, settings: &str, prompt: &str) -> Vec<String> {
    let mut argv: Vec<String> = ["--bg", "--name", name, "--model", model].iter().map(|s| s.to_string()).collect();
    if let Some(effort) = effort {
        argv.push("--effort".to_string());
        argv.push(effort.to_string());
    }
    for part in ["--permission-mode", "bypassPermissions", "--settings", settings, prompt] {
        argv.push(part.to_string());
    }
    argv
}

pub(crate) fn agents_argv() -> Vec<String> {
    ["agents", "--json", "--all"].iter().map(|s| s.to_string()).collect()
}

pub(crate) fn stop_argv(session_id: &str) -> Vec<String> {
    vec!["stop".to_string(), session_id.to_string()]
}

pub(crate) fn parse_backgrounded(output: &str) -> Result<(String, String), String> {
    for line in output.lines() {
        let parts: Vec<&str> = line.split('·').map(str::trim).collect();
        if parts.len() >= 3 && parts[0].ends_with("backgrounded") && !parts[1].is_empty() {
            return Ok((parts[1].to_string(), parts[2].to_string()));
        }
    }
    Err(format!("no `backgrounded · <id> · <name>` line in claude output: {}", output.trim()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentRow {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) cwd: String,
    pub(crate) state: String,
    pub(crate) status: String,
    pub(crate) started_at_ms: Option<i64>,
}

fn text(item: &Map<String, Value>, key: &str) -> String {
    item.get(key).and_then(Value::as_str).unwrap_or("").trim().to_string()
}

fn instant_ms(value: &Value) -> Option<i64> {
    match value {
        Value::String(s) => chrono::DateTime::parse_from_rfc3339(s.trim()).ok().map(|dt| dt.timestamp_millis()),
        Value::Number(n) => n.as_i64().map(|n| if n > 100_000_000_000 { n } else { n.saturating_mul(1000) }),
        _ => None,
    }
}

pub(crate) fn parse_agents(listed: &str) -> Option<Vec<AgentRow>> {
    let doc = paseo::first_json_document(listed)?;
    let items = match &doc {
        Value::Array(items) => items,
        other => other.get("agents")?.as_array()?,
    };
    Some(
        items
            .iter()
            .filter_map(Value::as_object)
            .map(|item| AgentRow {
                id: text(item, "id"),
                name: text(item, "name"),
                cwd: text(item, "cwd"),
                state: text(item, "state"),
                status: text(item, "status"),
                started_at_ms: item.get("startedAt").and_then(instant_ms),
            })
            .filter(|row| !row.id.is_empty())
            .collect(),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Seen {
    Working,
    Idle,
    Gone,
    Unknown,
}

pub(crate) fn seen_in(listed: Option<&str>, session_id: &str) -> Seen {
    let Some(rows) = listed.and_then(parse_agents) else {
        return Seen::Unknown;
    };
    let Some(row) = rows.iter().find(|r| r.id == session_id) else {
        return Seen::Gone;
    };
    let state = row.state.to_ascii_lowercase();
    if state == "failed" || state == "stopped" {
        Seen::Gone
    } else if row.status.eq_ignore_ascii_case("idle") || state == "idle" {
        Seen::Idle
    } else {
        Seen::Working
    }
}

#[derive(Debug, Default)]
pub(crate) struct Watch {
    idle_streak: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Signals {
    pub(crate) working: bool,
    pub(crate) gone: bool,
    pub(crate) unknown: bool,
    pub(crate) silent_idle: bool,
}

impl Watch {
    pub(crate) fn observe(&mut self, seen: Seen) -> Signals {
        self.idle_streak = if seen == Seen::Idle { self.idle_streak + 1 } else { 0 };
        Signals {
            working: seen == Seen::Working,
            gone: seen == Seen::Gone,
            unknown: seen == Seen::Unknown,
            silent_idle: self.idle_streak >= 2,
        }
    }
}

fn same_dir(a: &str, b: &str) -> bool {
    a.trim_end_matches('/') == b.trim_end_matches('/')
}

pub(crate) fn sole_session_since(rows: &[AgentRow], cwd: &Path, spawned_at_ms: i64) -> Option<String> {
    let cwd = cwd.to_string_lossy();
    let mut matching = rows
        .iter()
        .filter(|r| same_dir(&r.cwd, &cwd) && r.started_at_ms.is_some_and(|at| at > spawned_at_ms));
    let first = matching.next()?;
    if matching.next().is_some() || first.name.is_empty() {
        return None;
    }
    Some(first.name.clone())
}

pub(crate) fn steer_name_after_spawn(cli: &dyn ClaudeCli, cwd: &Path, spawned_at_ms: i64) -> Option<String> {
    let listed = cli.call(&agents_argv()).ok()?;
    sole_session_since(&parse_agents(&listed)?, cwd, spawned_at_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LISTING: &str = r#"[
      {"id":"4e1731f9","name":"bee-cssd-1-abc","cwd":"/w/feature","state":"working","status":"busy","startedAt":"2026-11-06T13:00:05Z"},
      {"id":"aa11","name":"other","cwd":"/w/other","state":"working","status":"busy","startedAt":"2026-11-06T13:00:06Z"}
    ]"#;

    fn ms(rfc: &str) -> i64 {
        chrono::DateTime::parse_from_rfc3339(rfc).unwrap().timestamp_millis()
    }

    #[test]
    fn claude_session_command_defaults_and_reads_config() {
        assert_eq!(claude_command(&json!({})), "claude");
        assert_eq!(claude_command(&json!({"herding":{"claude_command":"  /opt/claude "}})), "/opt/claude");
        assert_eq!(claude_command(&json!({"herding":{"claude_command":""}})), "claude");
    }

    #[test]
    fn claude_session_name_carries_the_cell_or_seat_and_a_short_job_id() {
        assert_eq!(session_name(Some("cssd-1"), None, "job-1793970405275-1617378-1"), "bee-cssd-1-75-1617378-1");
        assert_eq!(session_name(None, Some("hat-risks"), "job-7"), "bee-hat-risks-job-7");
        assert_eq!(session_name(None, None, "job-7"), "bee-run-job-7");
    }

    #[test]
    fn claude_session_spawn_argv_pins_model_mode_and_isolation() {
        let settings = settings_json(&BTreeMap::new());
        assert_eq!(settings, r#"{"worktree":{"bgIsolation":"none"}}"#);
        let argv = spawn_argv("bee-x-1", "haiku", None, &settings, "go");
        assert_eq!(
            argv,
            vec![
                "--bg", "--name", "bee-x-1", "--model", "haiku", "--permission-mode", "bypassPermissions", "--settings",
                r#"{"worktree":{"bgIsolation":"none"}}"#, "go"
            ]
        );
        let with_effort = spawn_argv("bee-x-1", "fable", Some("high"), &settings, "go");
        let at = with_effort.iter().position(|a| a == "--effort").unwrap();
        assert_eq!(with_effort[at + 1], "high");
        assert_eq!(with_effort.last().unwrap(), "go");
    }

    #[test]
    fn claude_session_settings_carry_the_worker_env() {
        let env: BTreeMap<String, String> =
            [("BEE_HERDING_READ_ONLY".to_string(), "1".to_string())].into_iter().collect();
        let parsed: Value = serde_json::from_str(&settings_json(&env)).unwrap();
        assert_eq!(parsed["worktree"]["bgIsolation"], "none");
        assert_eq!(parsed["env"]["BEE_HERDING_READ_ONLY"], "1");
    }

    #[test]
    fn claude_session_parse_backgrounded_reads_id_and_name() {
        let out = "starting\nbackgrounded · 4e1731f9 · cssd-probe\n";
        assert_eq!(parse_backgrounded(out), Ok(("4e1731f9".to_string(), "cssd-probe".to_string())));
        let err = parse_backgrounded("error: no such model\n").unwrap_err();
        assert!(err.contains("error: no such model"), "{err}");
    }

    #[test]
    fn claude_session_parse_agents_reads_both_document_shapes() {
        let rows = parse_agents(LISTING).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, "4e1731f9");
        assert_eq!(rows[0].started_at_ms, Some(ms("2026-11-06T13:00:05Z")));
        let wrapped = parse_agents(r#"{"agents":[{"id":"x","name":"n","startedAt":1762434005000}]}"#).unwrap();
        assert_eq!(wrapped[0].started_at_ms, Some(1762434005000));
        assert!(parse_agents("not json").is_none());
    }

    #[test]
    fn claude_session_seen_in_maps_states() {
        let one = |state: &str, status: &str| {
            format!(r#"[{{"id":"s1","name":"n","state":"{state}","status":"{status}"}}]"#)
        };
        assert_eq!(seen_in(Some(&one("working", "busy")), "s1"), Seen::Working);
        assert_eq!(seen_in(Some(&one("idle", "idle")), "s1"), Seen::Idle);
        assert_eq!(seen_in(Some(&one("working", "idle")), "s1"), Seen::Idle);
        assert_eq!(seen_in(Some(&one("failed", "idle")), "s1"), Seen::Gone);
        assert_eq!(seen_in(Some(&one("stopped", "idle")), "s1"), Seen::Gone);
        assert_eq!(seen_in(Some("[]"), "s1"), Seen::Gone);
        assert_eq!(seen_in(None, "s1"), Seen::Unknown);
        assert_eq!(seen_in(Some("garbage"), "s1"), Seen::Unknown);
    }

    #[test]
    fn claude_session_watch_reports_silent_idle_on_the_second_idle_in_a_row() {
        let mut watch = Watch::default();
        assert!(!watch.observe(Seen::Idle).silent_idle);
        assert!(watch.observe(Seen::Idle).silent_idle);
        let mut watch = Watch::default();
        assert!(!watch.observe(Seen::Idle).silent_idle);
        assert!(watch.observe(Seen::Working).working);
        assert!(!watch.observe(Seen::Idle).silent_idle);
    }

    #[test]
    fn claude_session_steer_name_needs_exactly_one_new_session_in_the_cwd() {
        let rows = parse_agents(LISTING).unwrap();
        let before = ms("2026-11-06T13:00:00Z");
        assert_eq!(sole_session_since(&rows, Path::new("/w/feature"), before), Some("bee-cssd-1-abc".to_string()));
        assert_eq!(sole_session_since(&rows, Path::new("/w/feature/"), before), Some("bee-cssd-1-abc".to_string()));
        assert_eq!(sole_session_since(&rows, Path::new("/w/feature"), ms("2026-11-06T13:00:10Z")), None);
        assert_eq!(sole_session_since(&rows, Path::new("/w/nowhere"), before), None);
        let mut two = rows.clone();
        two.push(AgentRow { id: "b".into(), name: "second".into(), ..rows[0].clone() });
        assert_eq!(sole_session_since(&two, Path::new("/w/feature"), before), None);
    }
}
