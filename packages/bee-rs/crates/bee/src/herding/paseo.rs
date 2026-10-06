use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaseoSpec {
    pub provider: String,
    pub model: Option<String>,
    pub thinking: Option<String>,
    pub mode: Option<String>,
}

impl PaseoSpec {
    pub fn from_config(cfg: &Value, agent: &str) -> Option<Result<Self, String>> {
        let agents = cfg
            .get("herding")
            .and_then(|h| h.get("agents"))
            .or_else(|| cfg.get("agents"))?;
        let entry = agents.get(agent)?;
        let entry_obj = entry.as_object()?;
        let paseo_val = entry_obj.get("paseo")?;
        if paseo_val.is_null() {
            return None;
        }
        let paseo_obj = match paseo_val.as_object() {
            Some(obj) => obj,
            None => {
                return Some(Err(format!(
                    "herding agent {agent:?} paseo configuration must be an object"
                )));
            }
        };

        let provider = match paseo_obj.get("provider") {
            Some(Value::String(s)) if !s.trim().is_empty() => s.clone(),
            _ => {
                return Some(Err(format!(
                    "herding agent {agent:?} paseo block requires a non-empty string for 'provider'"
                )));
            }
        };

        let model = paseo_obj
            .get("model")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(String::from);
        let thinking = paseo_obj
            .get("thinking")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(String::from);
        let mode = paseo_obj
            .get("mode")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(String::from);

        Some(Ok(Self {
            provider,
            model,
            thinking,
            mode,
        }))
    }
}

pub fn paseo_command(cfg: &Value) -> String {
    cfg.get("herding")
        .and_then(|h| h.get("paseo"))
        .or_else(|| cfg.get("paseo"))
        .and_then(|p| p.get("command"))
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("paseo")
        .to_string()
}

pub fn run_argv(
    spec: &PaseoSpec,
    job_id: &str,
    cwd: &str,
    env: &[(String, String)],
    prompt: &str,
) -> Vec<String> {
    let mut argv = vec![
        "run".to_string(),
        "-d".to_string(),
        "--json".to_string(),
        "--provider".to_string(),
    ];

    if let Some(ref m) = spec.model {
        argv.push(format!("{}/{}", spec.provider, m));
    } else {
        argv.push(spec.provider.clone());
    }

    if let Some(ref t) = spec.thinking {
        argv.push("--thinking".to_string());
        argv.push(t.clone());
    }

    if let Some(ref m) = spec.mode {
        argv.push("--mode".to_string());
        argv.push(m.clone());
    }

    argv.push("--cwd".to_string());
    argv.push(cwd.to_string());

    argv.push("--title".to_string());
    argv.push(job_id.to_string());

    argv.push("--label".to_string());
    argv.push(format!("bee_job={job_id}"));

    for (k, v) in env {
        argv.push("--env".to_string());
        argv.push(format!("{k}={v}"));
    }

    argv.push(prompt.to_string());

    argv
}

pub fn parse_run_agent_id(stdout: &str) -> Result<String, String> {
    for (i, c) in stdout.char_indices() {
        if c == '{' {
            let mut de = serde_json::Deserializer::from_str(&stdout[i..]).into_iter::<Value>();
            if let Some(Ok(Value::Object(map))) = de.next() {
                if let Some(id) = map.get("agentId").and_then(Value::as_str) {
                    if !id.trim().is_empty() {
                        return Ok(id.to_string());
                    }
                }
                return Err("first JSON object in output does not contain non-empty 'agentId'".to_string());
            }
        }
    }
    Err("no JSON object found in output".to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaseoState {
    Working,
    Idle,
    Blocked,
    Dead,
}

pub fn parse_inspect(stdout: &str) -> Option<PaseoState> {
    for (i, c) in stdout.char_indices() {
        if c == '{' {
            let mut de = serde_json::Deserializer::from_str(&stdout[i..]).into_iter::<Value>();
            if let Some(Ok(Value::Object(map))) = de.next() {
                let status = map.get("Status").and_then(Value::as_str)?;
                return match status {
                    "running" => {
                        let has_pending = map
                            .get("PendingPermissions")
                            .and_then(Value::as_array)
                            .map(|a| !a.is_empty())
                            .unwrap_or(false);
                        if has_pending {
                            Some(PaseoState::Blocked)
                        } else {
                            Some(PaseoState::Working)
                        }
                    }
                    "idle" => Some(PaseoState::Idle),
                    "error" | "closed" => Some(PaseoState::Dead),
                    _ => None,
                };
            }
        }
    }
    None
}

pub fn version_at_least(output: &str, required: (u64, u64, u64)) -> bool {
    let mut last_semver = None;
    for token in output.split_whitespace() {
        let clean = token.trim_matches(|c: char| !c.is_ascii_alphanumeric());
        let clean = clean
            .strip_prefix('v')
            .or_else(|| clean.strip_prefix('V'))
            .unwrap_or(clean);
        let core = clean.split(['-', '+']).next().unwrap_or(clean);
        let parts: Vec<&str> = core.split('.').collect();
        if parts.len() == 3 {
            if let (Ok(maj), Ok(min), Ok(pat)) = (
                parts[0].parse::<u64>(),
                parts[1].parse::<u64>(),
                parts[2].parse::<u64>(),
            ) {
                last_semver = Some((maj, min, pat));
            }
        }
    }

    match last_semver {
        Some(v) => v >= required,
        None => false,
    }
}

pub fn inspect_argv(id: &str) -> Vec<String> {
    vec![
        "inspect".to_string(),
        "--json".to_string(),
        id.to_string(),
    ]
}

pub fn archive_argv(id: &str) -> Vec<String> {
    vec![
        "archive".to_string(),
        "--force".to_string(),
        id.to_string(),
    ]
}

pub fn logs_argv(id: &str) -> Vec<String> {
    vec![
        "logs".to_string(),
        id.to_string(),
    ]
}

pub fn send_argv(id: &str, text: &str) -> Vec<String> {
    vec![
        "send".to_string(),
        id.to_string(),
        "--no-wait".to_string(),
        text.to_string(),
    ]
}

pub fn stop_argv(id: &str) -> Vec<String> {
    vec!["stop".to_string(), id.to_string()]
}

pub fn ls_label_argv(key: &str) -> Vec<String> {
    vec![
        "ls".to_string(),
        "--global".to_string(),
        "--label".to_string(),
        key.to_string(),
        "--json".to_string(),
    ]
}

pub fn logs_tail_argv(id: &str, n: usize, filter: Option<&str>) -> Vec<String> {
    let mut argv = vec![
        "logs".to_string(),
        id.to_string(),
        "--tail".to_string(),
        n.to_string(),
    ];
    if let Some(f) = filter {
        argv.push("--filter".to_string());
        argv.push(f.to_string());
    }
    argv
}

pub fn permit_argv(agent: &str, allow: bool, request: Option<&str>, all: bool) -> Vec<String> {
    let action = if allow { "allow" } else { "deny" };
    let mut argv = vec!["permit".to_string(), action.to_string(), agent.to_string()];
    if let Some(req) = request {
        argv.push(req.to_string());
    }
    if all {
        argv.push("--all".to_string());
    }
    argv
}

pub fn parse_ls_agents_checked(stdout: &str) -> Option<Vec<(String, String, bool)>> {
    for (i, c) in stdout.char_indices() {
        if c == '[' || c == '{' {
            let mut de = serde_json::Deserializer::from_str(&stdout[i..]).into_iter::<Value>();
            if let Some(Ok(val)) = de.next() {
                let readable = match &val {
                    Value::Array(_) => true,
                    Value::Object(m) => m.values().any(Value::is_array),
                    _ => false,
                };
                return readable.then(|| parse_ls_agents(stdout));
            }
        }
    }
    None
}

pub fn parse_ls_agents(stdout: &str) -> Vec<(String, String, bool)> {
    for (i, c) in stdout.char_indices() {
        if c == '[' || c == '{' {
            let mut de = serde_json::Deserializer::from_str(&stdout[i..]).into_iter::<Value>();
            if let Some(Ok(val)) = de.next() {
                let array_opt = match val {
                    Value::Array(a) => Some(a),
                    Value::Object(m) => m
                        .get("agents")
                        .or_else(|| m.get("Agents"))
                        .and_then(Value::as_array)
                        .cloned()
                        .or_else(|| m.values().find_map(|v| v.as_array().cloned())),
                    _ => None,
                };
                let array = match array_opt {
                    Some(a) => a,
                    None => return Vec::new(),
                };
                let mut out = Vec::new();
                for item in array {
                    let map = match item {
                        Value::Object(m) => m,
                        _ => continue,
                    };
                    let id = map
                        .get("id")
                        .or_else(|| map.get("Id"))
                        .or_else(|| map.get("agentId"))
                        .or_else(|| map.get("AgentId"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    if id.trim().is_empty() {
                        continue;
                    }
                    let mut bee_job = String::new();
                    let labels_val = map
                        .get("labels")
                        .or_else(|| map.get("Labels"))
                        .or_else(|| map.get("label"))
                        .or_else(|| map.get("Label"));
                    if let Some(lv) = labels_val {
                        match lv {
                            Value::Object(lm) => {
                                if let Some(j) = lm
                                    .get("bee_job")
                                    .or_else(|| lm.get("Bee_job"))
                                    .and_then(Value::as_str)
                                {
                                    bee_job = j.to_string();
                                }
                            }
                            Value::Array(la) => {
                                for elem in la {
                                    if let Some(s) = elem.as_str() {
                                        if let Some(stripped) = s.strip_prefix("bee_job=") {
                                            bee_job = stripped.to_string();
                                            break;
                                        } else if let Some(stripped) = s.strip_prefix("bee_job:") {
                                            bee_job = stripped.to_string();
                                            break;
                                        }
                                    } else if let Value::Object(em) = elem {
                                        let k = em
                                            .get("key")
                                            .or_else(|| em.get("name"))
                                            .and_then(Value::as_str);
                                        let v = em.get("value").and_then(Value::as_str);
                                        if k == Some("bee_job") {
                                            if let Some(v_str) = v {
                                                bee_job = v_str.to_string();
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if bee_job.is_empty() {
                        if let Some(j) = map
                            .get("bee_job")
                            .or_else(|| map.get("Bee_job"))
                            .and_then(Value::as_str)
                        {
                            bee_job = j.to_string();
                        }
                    }
                    let archived = map
                        .get("archived")
                        .or_else(|| map.get("Archived"))
                        .and_then(|v| {
                            v.as_bool().or_else(|| {
                                v.as_str().map(|s| s == "true" || s == "archived")
                            })
                        })
                        .or_else(|| {
                            map.get("archivedAt")
                                .or_else(|| map.get("ArchivedAt"))
                                .and_then(Value::as_str)
                                .map(|s| !s.trim().is_empty())
                        })
                        .or_else(|| {
                            map.get("status")
                                .or_else(|| map.get("Status"))
                                .and_then(Value::as_str)
                                .map(|s| s == "archived")
                        })
                        .unwrap_or(false);
                    out.push((id.to_string(), bee_job, archived));
                }
                return out;
            }
        }
    }
    Vec::new()
}

pub fn parse_pending_permissions(stdout: &str) -> Vec<(String, String)> {
    for (i, c) in stdout.char_indices() {
        if c == '{' {
            let mut de = serde_json::Deserializer::from_str(&stdout[i..]).into_iter::<Value>();
            if let Some(Ok(Value::Object(map))) = de.next() {
                let pending = map
                    .get("PendingPermissions")
                    .or_else(|| map.get("pendingPermissions"))
                    .or_else(|| map.get("pending_permissions"))
                    .and_then(Value::as_array);
                let array = match pending {
                    Some(a) => a,
                    None => return Vec::new(),
                };
                let mut out = Vec::new();
                for item in array {
                    match item {
                        Value::Object(m) => {
                            let id = m
                                .get("id")
                                .or_else(|| m.get("Id"))
                                .or_else(|| m.get("requestId"))
                                .or_else(|| m.get("RequestId"))
                                .or_else(|| m.get("request_id"))
                                .and_then(Value::as_str);
                            let tool = m
                                .get("tool")
                                .or_else(|| m.get("name"))
                                .or_else(|| m.get("toolName"))
                                .or_else(|| m.get("Tool"))
                                .or_else(|| m.get("Name"))
                                .or_else(|| m.get("ToolName"))
                                .and_then(Value::as_str);
                            if let Some(id_str) = id {
                                out.push((id_str.to_string(), tool.unwrap_or("").to_string()));
                            } else if let Some(tool_str) = tool {
                                out.push((tool_str.to_string(), String::new()));
                            } else {
                                out.push((item.to_string(), String::new()));
                            }
                        }
                        Value::String(s) => {
                            out.push((s.clone(), String::new()));
                        }
                        other => {
                            out.push((other.to_string(), String::new()));
                        }
                    }
                }
                return out;
            }
        }
    }
    Vec::new()
}

pub trait PaseoCli: Send + Sync {
    fn call(&self, args: &[String]) -> Result<String, String>;
}

#[derive(Debug, Clone)]
pub struct RealPaseoCli {
    pub command: String,
    pub timeout: Option<std::time::Duration>,
}

impl RealPaseoCli {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            timeout: None,
        }
    }

    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

pub fn daemon_reachable() -> bool {
    if std::env::var("PASEO_HOST").is_ok_and(|h| !h.trim().is_empty()) {
        return true;
    }
    let addr: std::net::SocketAddr = ([127, 0, 0, 1], 6767).into();
    std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(300)).is_ok()
}

pub struct FailFastPaseoCli {
    inner: Box<dyn PaseoCli>,
    failed: std::sync::atomic::AtomicBool,
}

impl FailFastPaseoCli {
    pub fn new(inner: Box<dyn PaseoCli>) -> Self {
        Self { inner, failed: std::sync::atomic::AtomicBool::new(false) }
    }
}

impl PaseoCli for FailFastPaseoCli {
    fn call(&self, args: &[String]) -> Result<String, String> {
        if self.failed.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("paseo unreachable earlier in this run".to_string());
        }
        let result = self.inner.call(args);
        if result.is_err() {
            self.failed.store(true, std::sync::atomic::Ordering::SeqCst);
        }
        result
    }
}

fn run_with_deadline(
    command: &str,
    args: &[String],
    timeout: std::time::Duration,
) -> Result<std::process::Output, String> {
    use std::io::Read;
    let mut child = std::process::Command::new(command)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn {command}: {e}"))?;
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
    let deadline = std::time::Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if std::time::Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("{command} timed out after {}s", timeout.as_secs()));
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(50)),
            Err(e) => return Err(format!("failed to wait for {command}: {e}")),
        }
    };
    let stdout = out_reader.join().unwrap_or_default();
    let stderr = err_reader.join().unwrap_or_default();
    Ok(std::process::Output { status, stdout, stderr })
}

impl PaseoCli for RealPaseoCli {
    fn call(&self, args: &[String]) -> Result<String, String> {
        let output = match self.timeout {
            Some(t) => run_with_deadline(&self.command, args, t)?,
            None => std::process::Command::new(&self.command)
                .args(args)
                .stdin(std::process::Stdio::null())
                .output()
                .map_err(|e| format!("failed to spawn {}: {e}", self.command))?,
        };

        if output.status.success() {
            String::from_utf8(output.stdout)
                .map_err(|e| format!("invalid utf-8 in stdout from {}: {e}", self.command))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let err_msg = if !stderr.trim().is_empty() {
                stderr
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                if !stdout.trim().is_empty() {
                    stdout
                } else {
                    format!("{} exited with status {}", self.command, output.status)
                }
            };
            Err(err_msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn real_cli_with_timeout_kills_a_slow_command() {
        let cli = RealPaseoCli::new("sleep").with_timeout(std::time::Duration::from_secs(1));
        let started = std::time::Instant::now();
        let err = cli.call(&["5".to_string()]).unwrap_err();
        assert!(err.contains("timed out"), "{err}");
        assert!(started.elapsed() < std::time::Duration::from_secs(4));
    }

    #[test]
    fn fail_fast_cli_stops_calling_after_the_first_error() {
        struct Counting(std::sync::Arc<std::sync::atomic::AtomicUsize>);
        impl PaseoCli for Counting {
            fn call(&self, _args: &[String]) -> Result<String, String> {
                self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err("down".to_string())
            }
        }
        let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let cli = FailFastPaseoCli::new(Box::new(Counting(count.clone())));
        assert!(cli.call(&[]).is_err());
        assert!(cli.call(&[]).is_err());
        assert!(cli.call(&[]).is_err());
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn agent_entry_with_paseo_block_parses_into_spec() {
        let cfg = json!({
            "herding": {
                "agents": {
                    "w-pi": {
                        "paseo": {
                            "provider": "pi",
                            "model": "deepseek-flash",
                            "thinking": "high",
                            "mode": "agent"
                        }
                    }
                }
            }
        });
        let res = PaseoSpec::from_config(&cfg, "w-pi");
        let spec = res.expect("some").expect("ok");
        assert_eq!(
            spec,
            PaseoSpec {
                provider: "pi".to_string(),
                model: Some("deepseek-flash".to_string()),
                thinking: Some("high".to_string()),
                mode: Some("agent".to_string()),
            }
        );
    }

    #[test]
    fn agent_entry_with_no_paseo_block_returns_none() {
        let cfg = json!({
            "herding": {
                "agents": {
                    "plain-agent": ["pi", "--flag"],
                    "obj-agent": {"env": {"FOO": "BAR"}}
                }
            }
        });
        assert!(PaseoSpec::from_config(&cfg, "plain-agent").is_none());
        assert!(PaseoSpec::from_config(&cfg, "obj-agent").is_none());
        assert!(PaseoSpec::from_config(&cfg, "missing-agent").is_none());
    }

    #[test]
    fn paseo_block_without_provider_returns_err_naming_agent() {
        let cfg = json!({
            "herding": {
                "agents": {
                    "bad-worker": {
                        "paseo": {
                            "model": "deepseek-flash"
                        }
                    }
                }
            }
        });
        let res = PaseoSpec::from_config(&cfg, "bad-worker").expect("some");
        let err = res.expect_err("should fail without provider");
        assert!(err.contains("bad-worker"));

        let empty_cfg = json!({
            "herding": {
                "agents": {
                    "bad-worker-2": {
                        "paseo": {
                            "provider": "   "
                        }
                    }
                }
            }
        });
        let res2 = PaseoSpec::from_config(&empty_cfg, "bad-worker-2").expect("some");
        let err2 = res2.expect_err("should fail with empty provider");
        assert!(err2.contains("bad-worker-2"));
    }

    #[test]
    fn paseo_command_reads_config_and_defaults_to_paseo() {
        let default_cfg = json!({});
        assert_eq!(paseo_command(&default_cfg), "paseo");

        let custom_cfg = json!({
            "herding": {
                "paseo": {
                    "command": "custom-paseo"
                }
            }
        });
        assert_eq!(paseo_command(&custom_cfg), "custom-paseo");
    }

    #[test]
    fn run_argv_joins_provider_and_model_and_builds_expected_argv() {
        let spec = PaseoSpec {
            provider: "pi".to_string(),
            model: Some("deepseek-flash".to_string()),
            thinking: Some("high".to_string()),
            mode: Some("arch".to_string()),
        };
        let env = vec![
            ("BEE_HERDING_WORKER".to_string(), "1".to_string()),
            ("JOB_VAR".to_string(), "xyz".to_string()),
        ];
        let argv = run_argv(&spec, "job-42", "/path/to/cwd", &env, "do task");
        assert_eq!(
            argv,
            vec![
                "run",
                "-d",
                "--json",
                "--provider",
                "pi/deepseek-flash",
                "--thinking",
                "high",
                "--mode",
                "arch",
                "--cwd",
                "/path/to/cwd",
                "--title",
                "job-42",
                "--label",
                "bee_job=job-42",
                "--env",
                "BEE_HERDING_WORKER=1",
                "--env",
                "JOB_VAR=xyz",
                "do task"
            ]
        );

        let minimal_spec = PaseoSpec {
            provider: "claude".to_string(),
            model: None,
            thinking: None,
            mode: None,
        };
        let minimal_argv = run_argv(&minimal_spec, "job-1", "/cwd", &[], "task");
        assert_eq!(
            minimal_argv,
            vec![
                "run",
                "-d",
                "--json",
                "--provider",
                "claude",
                "--cwd",
                "/cwd",
                "--title",
                "job-1",
                "--label",
                "bee_job=job-1",
                "task"
            ]
        );
    }

    #[test]
    fn parse_run_agent_id_reads_agent_id_after_leading_lines() {
        let stdout = "Starting Paseo daemon...\n[desktop] app startup { version: '0.10.3' }\n{\"agentId\": \"f236f8a4-43c2-4411-b2a4-3a6b2607c32e\"}\n";
        assert_eq!(
            parse_run_agent_id(stdout),
            Ok("f236f8a4-43c2-4411-b2a4-3a6b2607c32e".to_string())
        );

        let no_json = "plain log without json";
        assert!(parse_run_agent_id(no_json).is_err());

        let missing_id = "{\"status\": \"ok\"}";
        assert!(parse_run_agent_id(missing_id).is_err());
    }

    #[test]
    fn parse_inspect_maps_statuses_correctly() {
        let running_json = json!({"Status": "running"}).to_string();
        assert_eq!(parse_inspect(&running_json), Some(PaseoState::Working));

        let blocked_json = json!({
            "Status": "running",
            "PendingPermissions": ["terminal.exec"]
        })
        .to_string();
        assert_eq!(parse_inspect(&blocked_json), Some(PaseoState::Blocked));

        let empty_perms_json = json!({
            "Status": "running",
            "PendingPermissions": []
        })
        .to_string();
        assert_eq!(parse_inspect(&empty_perms_json), Some(PaseoState::Working));

        let idle_json = json!({"Status": "idle"}).to_string();
        assert_eq!(parse_inspect(&idle_json), Some(PaseoState::Idle));

        let closed_json = json!({"Status": "closed"}).to_string();
        assert_eq!(parse_inspect(&closed_json), Some(PaseoState::Dead));

        let error_json = json!({"Status": "error"}).to_string();
        assert_eq!(parse_inspect(&error_json), Some(PaseoState::Dead));

        let unknown_json = json!({"Status": "other"}).to_string();
        assert_eq!(parse_inspect(&unknown_json), None);

        assert_eq!(parse_inspect("not json"), None);
    }

    #[test]
    fn version_at_least_checks_semver() {
        let current_out = "[desktop] app startup { version: '0.10.3' }\n0.10.3\n";
        assert!(version_at_least(current_out, (0, 10, 3)));
        assert!(version_at_least(current_out, (0, 10, 2)));
        assert!(version_at_least(current_out, (0, 6, 1)));
        assert!(!version_at_least(current_out, (0, 10, 4)));

        let old_out = "0.6.1\n";
        assert!(!version_at_least(old_out, (0, 10, 3)));
        assert!(version_at_least(old_out, (0, 6, 1)));

        let bad_out = "no version here";
        assert!(!version_at_least(bad_out, (0, 10, 3)));
    }

    #[test]
    fn argv_builders_produce_expected_tokens() {
        assert_eq!(inspect_argv("agent-1"), vec!["inspect", "--json", "agent-1"]);
        assert_eq!(archive_argv("agent-1"), vec!["archive", "--force", "agent-1"]);
        assert_eq!(logs_argv("agent-1"), vec!["logs", "agent-1"]);
        assert_eq!(send_argv("agent-1", "do next"), vec!["send", "agent-1", "--no-wait", "do next"]);
    }

    #[test]
    fn real_paseo_cli_executes_command() {
        let cli = RealPaseoCli::new("echo");
        let res = cli.call(&["hello".to_string()]);
        assert_eq!(res.expect("echo ok").trim(), "hello");

        let bad_cli = RealPaseoCli::new("non_existent_command_12345");
        assert!(bad_cli.call(&[]).is_err());
    }

    #[test]
    fn new_argv_builders_produce_expected_tokens() {
        assert_eq!(stop_argv("agent-1"), vec!["stop", "agent-1"]);
        assert_eq!(
            ls_label_argv("bee_job"),
            vec!["ls", "--global", "--label", "bee_job", "--json"]
        );
        assert_eq!(
            logs_tail_argv("agent-1", 40, None),
            vec!["logs", "agent-1", "--tail", "40"]
        );
        assert_eq!(
            logs_tail_argv("agent-1", 10, Some("tools")),
            vec!["logs", "agent-1", "--tail", "10", "--filter", "tools"]
        );
        assert_eq!(
            permit_argv("agent-1", true, Some("req-123"), false),
            vec!["permit", "allow", "agent-1", "req-123"]
        );
        assert_eq!(
            permit_argv("agent-1", false, None, true),
            vec!["permit", "deny", "agent-1", "--all"]
        );
    }

    #[test]
    fn parse_ls_agents_parses_array_and_object_with_capitalized_keys() {
        let raw_array = json!([
            {
                "id": "agent-1",
                "labels": { "bee_job": "job-101" },
                "archived": false
            },
            {
                "Id": "agent-2",
                "Labels": { "Bee_job": "job-102" },
                "Archived": true
            },
            {
                "agentId": "agent-3",
                "labels": ["bee_job=job-103"],
                "archivedAt": "2026-10-06T00:00:00Z"
            },
            {
                "id": "agent-4",
                "bee_job": "job-104",
                "status": "archived"
            }
        ])
        .to_string();
        let stdout_with_prefix = format!("[desktop] startup line\n{raw_array}\n");
        let parsed = parse_ls_agents(&stdout_with_prefix);
        assert_eq!(
            parsed,
            vec![
                ("agent-1".to_string(), "job-101".to_string(), false),
                ("agent-2".to_string(), "job-102".to_string(), true),
                ("agent-3".to_string(), "job-103".to_string(), true),
                ("agent-4".to_string(), "job-104".to_string(), true),
            ]
        );

        let wrapped_obj = json!({
            "Agents": [
                {
                    "Id": "agent-5",
                    "Labels": { "bee_job": "job-105" },
                    "Archived": false
                }
            ]
        })
        .to_string();
        let parsed_obj = parse_ls_agents(&wrapped_obj);
        assert_eq!(
            parsed_obj,
            vec![("agent-5".to_string(), "job-105".to_string(), false)]
        );
    }

    #[test]
    fn parse_pending_permissions_extracts_id_and_tool_shapes() {
        let inspect_json = json!({
            "Id": "agent-1",
            "Status": "running",
            "PendingPermissions": [
                { "id": "req-1", "tool": "terminal.exec" },
                { "Id": "req-2", "name": "bash" },
                { "id": "req-3", "toolName": "file_write" },
                { "id": "req-4" },
                "perm-string-only",
                { "unknown_key": "val" }
            ]
        })
        .to_string();
        let stdout = format!("log line before json\n{inspect_json}\n");
        let perms = parse_pending_permissions(&stdout);
        assert_eq!(
            perms,
            vec![
                ("req-1".to_string(), "terminal.exec".to_string()),
                ("req-2".to_string(), "bash".to_string()),
                ("req-3".to_string(), "file_write".to_string()),
                ("req-4".to_string(), "".to_string()),
                ("perm-string-only".to_string(), "".to_string()),
                ("{\"unknown_key\":\"val\"}".to_string(), "".to_string()),
            ]
        );
    }
}
