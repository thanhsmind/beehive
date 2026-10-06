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

pub trait PaseoCli: Send + Sync {
    fn call(&self, args: &[String]) -> Result<String, String>;
}

#[derive(Debug, Clone)]
pub struct RealPaseoCli {
    pub command: String,
}

impl RealPaseoCli {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
        }
    }
}

impl PaseoCli for RealPaseoCli {
    fn call(&self, args: &[String]) -> Result<String, String> {
        let output = std::process::Command::new(&self.command)
            .args(args)
            .stdin(std::process::Stdio::null())
            .output()
            .map_err(|e| format!("failed to spawn {}: {e}", self.command))?;

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
}
