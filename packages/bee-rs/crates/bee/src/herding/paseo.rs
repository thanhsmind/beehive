use serde_json::Value;
use std::path::{Path, PathBuf};

#[cfg(test)]
thread_local! {
    static TEST_DISCOVERY: std::cell::RefCell<Option<(Option<String>, Option<PathBuf>, PathBuf)>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn discovery_inputs() -> (Option<String>, Option<PathBuf>, PathBuf) {
    TEST_DISCOVERY
        .with(|d| d.borrow().clone())
        .unwrap_or((None, None, PathBuf::from("/proc")))
}

#[cfg(not(test))]
fn discovery_inputs() -> (Option<String>, Option<PathBuf>, PathBuf) {
    let user_home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .filter(|h| !h.is_empty())
        .map(PathBuf::from);
    let home =
        crate::herding::paseo_state::home_from("", std::env::var_os("PASEO_HOME"), user_home);
    (std::env::var("PASEO_CLI").ok(), home, PathBuf::from("/proc"))
}

pub const AGENT_SCOPE_VARS: [&str; 2] = ["PASEO_AGENT_ID", "PASEO_AGENT_CWD"];

pub fn scrubbed_command(program: impl AsRef<std::ffi::OsStr>) -> std::process::Command {
    let mut command = std::process::Command::new(program);
    for var in AGENT_SCOPE_VARS {
        command.env_remove(var);
    }
    command
}

#[cfg(test)]
thread_local! {
    static TEST_INSIDE_PASEO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub fn set_inside_paseo_for_test(value: bool) {
    TEST_INSIDE_PASEO.with(|c| c.set(value));
}

#[cfg(test)]
pub fn inside_paseo() -> bool {
    TEST_INSIDE_PASEO.with(|c| c.get())
}

#[cfg(not(test))]
pub fn inside_paseo() -> bool {
    ["PASEO_AGENT_ID", "PASEO_CLI"]
        .iter()
        .any(|k| std::env::var(k).is_ok_and(|v| !v.trim().is_empty()))
}

fn existing_file(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    Path::new(value).is_file().then(|| value.to_string())
}

fn daemon_pid(home: &Path) -> Option<u64> {
    let raw = std::fs::read_to_string(home.join("paseo.pid")).ok()?;
    serde_json::from_str::<Value>(&raw).ok()?.get("pid")?.as_u64()
}

fn daemon_env_cli(proc_root: &Path, pid: u64) -> Option<String> {
    let raw = std::fs::read(proc_root.join(pid.to_string()).join("environ")).ok()?;
    let entry = raw
        .split(|b| *b == 0)
        .find_map(|chunk| chunk.strip_prefix(b"PASEO_CLI="))?;
    existing_file(&String::from_utf8_lossy(entry))
}

fn daemon_exe_cli(proc_root: &Path, pid: u64) -> Option<String> {
    let exe = std::fs::read_link(proc_root.join(pid.to_string()).join("exe")).ok()?;
    let candidate = exe.parent()?.join("resources").join("bin").join("paseo");
    candidate.is_file().then(|| candidate.to_string_lossy().into_owned())
}

fn discover(command_env: Option<&str>, paseo_home: Option<&Path>, proc_root: &Path) -> Option<String> {
    if let Some(cli) = command_env.and_then(existing_file) {
        return Some(cli);
    }
    let home = paseo_home?;
    let pid = daemon_pid(home)?;
    daemon_env_cli(proc_root, pid).or_else(|| daemon_exe_cli(proc_root, pid))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaseoSpec {
    pub provider: String,
    pub model: Option<String>,
    pub thinking: Option<String>,
    pub mode: Option<String>,
    pub isolated_config: bool,
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
        let isolated_config = if provider.trim().eq_ignore_ascii_case("pi") {
            paseo_obj
                .get("isolated_config")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        } else {
            false
        };

        Some(Ok(Self {
            provider,
            model,
            thinking,
            mode,
            isolated_config,
        }))
    }
}

fn configured_command(cfg: &Value) -> Option<String> {
    cfg.get("herding")
        .and_then(|h| h.get("paseo"))
        .or_else(|| cfg.get("paseo"))
        .and_then(|p| p.get("command"))
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
}

pub fn paseo_command(cfg: &Value) -> String {
    if let Some(command) = configured_command(cfg) {
        return command;
    }
    let (command_env, paseo_home, proc_root) = discovery_inputs();
    discover(command_env.as_deref(), paseo_home.as_deref(), &proc_root)
        .unwrap_or_else(|| "paseo".to_string())
}

pub fn run_argv(
    spec: &PaseoSpec,
    job_id: &str,
    cwd: &str,
    env: &[(String, String)],
    prompt: &str,
    agent: &str,
    cell_id: Option<&str>,
    workspace: Option<&str>,
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

    if let Some(w) = workspace {
        argv.push("--workspace".to_string());
        argv.push(w.to_string());
    }

    argv.push("--cwd".to_string());
    argv.push(cwd.to_string());

    let title = match cell_id.filter(|c| !c.trim().is_empty()) {
        Some(c) => format!("{c} {agent}"),
        None => format!("{agent} {job_id}"),
    };
    argv.push("--title".to_string());
    argv.push(title);

    argv.push("--label".to_string());
    argv.push(format!("bee_job={job_id}"));

    argv.push("--label".to_string());
    argv.push(format!("bee_agent={agent}"));

    if let Some(c) = cell_id.filter(|c| !c.trim().is_empty()) {
        argv.push("--label".to_string());
        argv.push(format!("bee_cell={c}"));
    }

    for (k, v) in env {
        argv.push("--env".to_string());
        argv.push(format!("{k}={v}"));
    }

    argv.push(prompt.to_string());

    argv
}

fn first_json_from(stdout: &str, starts: &[char]) -> Option<Value> {
    for (i, c) in stdout.char_indices() {
        if starts.contains(&c) {
            let mut de = serde_json::Deserializer::from_str(&stdout[i..]).into_iter::<Value>();
            if let Some(Ok(value)) = de.next() {
                return Some(value);
            }
        }
    }
    None
}

pub fn first_json_document(stdout: &str) -> Option<Value> {
    first_json_from(stdout, &['{', '['])
}

pub fn parse_run_agent_id(stdout: &str) -> Result<String, String> {
    match first_json_from(stdout, &['{']) {
        Some(Value::Object(map)) => match map.get("agentId").and_then(Value::as_str) {
            Some(id) if !id.trim().is_empty() => Ok(id.to_string()),
            _ => Err("first JSON object in output does not contain non-empty 'agentId'".to_string()),
        },
        _ => Err("no JSON object found in output".to_string()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaseoState {
    Working,
    Idle,
    Blocked,
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitStatus {
    Idle,
    Permission,
    Error,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PaseoInspect {
    pub state: Option<PaseoState>,
    pub updated_at: Option<String>,
    pub model: Option<String>,
    pub thinking: Option<String>,
    pub mode: Option<String>,
    pub last_usage: Option<Value>,
}

pub fn parse_inspect_full(stdout: &str) -> Option<PaseoInspect> {
    for (i, c) in stdout.char_indices() {
        if c == '{' {
            let mut de = serde_json::Deserializer::from_str(&stdout[i..]).into_iter::<Value>();
            if let Some(Ok(Value::Object(map))) = de.next() {
                let status = map
                    .get("Status")
                    .or_else(|| map.get("status"))
                    .and_then(Value::as_str);
                let state = match status {
                    Some("running") => {
                        let has_pending = map
                            .get("PendingPermissions")
                            .or_else(|| map.get("pendingPermissions"))
                            .or_else(|| map.get("pending_permissions"))
                            .and_then(Value::as_array)
                            .map(|a| !a.is_empty())
                            .unwrap_or(false);
                        if has_pending {
                            Some(PaseoState::Blocked)
                        } else {
                            Some(PaseoState::Working)
                        }
                    }
                    Some("idle") => Some(PaseoState::Idle),
                    Some("error") | Some("closed") => Some(PaseoState::Dead),
                    _ => None,
                };

                let updated_at = map
                    .get("UpdatedAt")
                    .or_else(|| map.get("updatedAt"))
                    .or_else(|| map.get("updated_at"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(String::from);

                let model = map
                    .get("Model")
                    .or_else(|| map.get("model"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty() && *s != "-")
                    .map(String::from);

                let thinking = map
                    .get("Thinking")
                    .or_else(|| map.get("thinking"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("auto"))
                    .map(String::from);

                let mode = map
                    .get("Mode")
                    .or_else(|| map.get("mode"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty() && *s != "-")
                    .map(String::from);

                let last_usage = map
                    .get("LastUsage")
                    .or_else(|| map.get("lastUsage"))
                    .filter(|v| v.is_object())
                    .cloned();

                return Some(PaseoInspect {
                    state,
                    updated_at,
                    model,
                    thinking,
                    mode,
                    last_usage,
                });
            }
        }
    }
    None
}

pub fn parse_inspect(stdout: &str) -> Option<PaseoState> {
    parse_inspect_full(stdout).and_then(|i| i.state)
}

pub fn parse_wait(stdout: &str) -> Option<WaitStatus> {
    for (i, c) in stdout.char_indices() {
        if c == '{' {
            let mut de = serde_json::Deserializer::from_str(&stdout[i..]).into_iter::<Value>();
            if let Some(Ok(Value::Object(map))) = de.next() {
                let status_str = map
                    .get("status")
                    .or_else(|| map.get("Status"))
                    .and_then(Value::as_str);
                if let Some(status) = status_str {
                    match status.to_ascii_lowercase().as_str() {
                        "idle" => return Some(WaitStatus::Idle),
                        "permission" => return Some(WaitStatus::Permission),
                        "error" => return Some(WaitStatus::Error),
                        "timeout" => return Some(WaitStatus::Timeout),
                        _ => return None,
                    }
                }
                return None;
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

pub fn wait_argv(id: &str, timeout_secs: u64) -> Vec<String> {
    vec![
        "agent".to_string(),
        "wait".to_string(),
        id.to_string(),
        "--timeout".to_string(),
        timeout_secs.to_string(),
        "--json".to_string(),
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

pub fn workspace_ls_argv() -> Vec<String> {
    vec!["workspace".to_string(), "ls".to_string(), "--json".to_string()]
}

pub fn project_ls_argv() -> Vec<String> {
    vec!["project".to_string(), "ls".to_string(), "--json".to_string()]
}

pub fn workspace_create_argv(path: &str, project: Option<&str>) -> Vec<String> {
    let mut argv = vec![
        "workspace".to_string(),
        "create".to_string(),
        "--path".to_string(),
        path.to_string(),
        "--isolation".to_string(),
        "local".to_string(),
    ];
    if let Some(p) = project {
        argv.push("--project".to_string());
        argv.push(p.to_string());
    }
    argv.push("--json".to_string());
    argv
}

pub fn workspace_archive_argv(id: &str) -> Vec<String> {
    vec![
        "workspace".to_string(),
        "archive".to_string(),
        id.to_string(),
        "--json".to_string(),
    ]
}

fn paseo_error_text(raw: &str) -> String {
    first_json_document(raw)
        .and_then(|v| {
            v.get("error")
                .and_then(|e| e.get("message").or(Some(e)))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| raw.trim().to_string())
}

fn rows_with_path<'a>(doc: &'a Option<Value>, path_key: &str, id_key: &str, target: &Path) -> Option<&'a str> {
    doc.as_ref()?.as_array()?.iter().find_map(|row| {
        let path = row.get(path_key)?.as_str()?;
        let id = row.get(id_key)?.as_str().filter(|s| !s.trim().is_empty())?;
        crate::path_identity::canonical_paths_equal(Path::new(path), target).then_some(id)
    })
}

pub fn ensure_workspace(cli: &dyn PaseoCli, main_root: &Path, dir: &Path) -> Result<String, String> {
    let listed = cli.call(&workspace_ls_argv()).map_err(|e| paseo_error_text(&e))?;
    let listed = first_json_document(&listed);
    if let Some(id) = rows_with_path(&listed, "cwd", "workspaceId", dir) {
        return Ok(id.to_string());
    }
    let projects = cli.call(&project_ls_argv()).map_err(|e| paseo_error_text(&e))?;
    let projects = first_json_document(&projects);
    let project = rows_with_path(&projects, "path", "projectId", main_root);
    let created = cli
        .call(&workspace_create_argv(&dir.to_string_lossy(), project))
        .map_err(|e| paseo_error_text(&e))?;
    match first_json_document(&created) {
        Some(doc) => match doc.get("workspaceId").and_then(Value::as_str) {
            Some(id) if !id.trim().is_empty() => Ok(id.to_string()),
            _ => Err(paseo_error_text(&created)),
        },
        None => Err(format!("no JSON in paseo workspace create output: {}", created.trim())),
    }
}

pub fn archive_workspace(cli: &dyn PaseoCli, id: &str) -> Result<(), String> {
    cli.call(&workspace_archive_argv(id))
        .map(|_| ())
        .map_err(|e| paseo_error_text(&e))
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
    let mut child = scrubbed_command(command)
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
            None => scrubbed_command(&self.command)
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
                isolated_config: false,
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

    fn with_discovery<R>(
        env: Option<&str>,
        home: Option<&Path>,
        proc_root: &Path,
        f: impl FnOnce() -> R,
    ) -> R {
        TEST_DISCOVERY.with(|d| {
            *d.borrow_mut() = Some((
                env.map(str::to_string),
                home.map(Path::to_path_buf),
                proc_root.to_path_buf(),
            ))
        });
        let out = f();
        TEST_DISCOVERY.with(|d| *d.borrow_mut() = None);
        out
    }

    fn touch(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }

    fn fake_proc(root: &Path, pid: u64, environ: &[u8]) {
        let dir = root.join(pid.to_string());
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("environ"), environ).unwrap();
    }

    #[test]
    fn paseo_command_config_wins_over_env_and_daemon() {
        let tmp = tempfile::tempdir().unwrap();
        let env_cli = tmp.path().join("env-paseo");
        touch(&env_cli);
        let cfg = json!({"paseo": {"command": "configured-paseo"}});
        let got = with_discovery(Some(env_cli.to_str().unwrap()), None, tmp.path(), || {
            paseo_command(&cfg)
        });
        assert_eq!(got, "configured-paseo");
    }

    #[test]
    fn paseo_command_env_file_wins_over_daemon() {
        let tmp = tempfile::tempdir().unwrap();
        let env_cli = tmp.path().join("env-paseo");
        touch(&env_cli);
        let home = tmp.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let daemon_cli = tmp.path().join("daemon-paseo");
        touch(&daemon_cli);
        std::fs::write(home.join("paseo.pid"), json!({"pid": 4242}).to_string()).unwrap();
        let proc_root = tmp.path().join("proc");
        fake_proc(
            &proc_root,
            4242,
            format!("PASEO_CLI={}\0", daemon_cli.display()).as_bytes(),
        );
        let got = with_discovery(Some(env_cli.to_str().unwrap()), Some(&home), &proc_root, || {
            paseo_command(&json!({}))
        });
        assert_eq!(got, env_cli.to_str().unwrap());
    }

    #[test]
    fn paseo_command_env_naming_a_missing_file_is_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let daemon_cli = tmp.path().join("daemon-paseo");
        touch(&daemon_cli);
        std::fs::write(home.join("paseo.pid"), json!({"pid": 7}).to_string()).unwrap();
        let proc_root = tmp.path().join("proc");
        fake_proc(
            &proc_root,
            7,
            format!("PASEO_CLI={}\0", daemon_cli.display()).as_bytes(),
        );
        let missing = tmp.path().join("not-there-paseo");
        let got = with_discovery(Some(missing.to_str().unwrap()), Some(&home), &proc_root, || {
            paseo_command(&json!({}))
        });
        assert_eq!(got, daemon_cli.to_str().unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn paseo_command_falls_back_to_the_daemon_exe_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(home.join("paseo.pid"), json!({"pid": 9}).to_string()).unwrap();
        let app = tmp.path().join("app");
        let bundled = app.join("resources").join("bin").join("paseo");
        touch(&bundled);
        let proc_root = tmp.path().join("proc");
        fake_proc(&proc_root, 9, b"OTHER=1\0");
        std::os::unix::fs::symlink(app.join("paseo-desktop"), proc_root.join("9").join("exe")).unwrap();
        let got = with_discovery(None, Some(&home), &proc_root, || paseo_command(&json!({})));
        assert_eq!(got, bundled.to_str().unwrap());
    }

    #[test]
    fn paseo_command_missing_or_garbage_pid_file_gives_plain_paseo() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let proc_root = tmp.path().join("proc");
        std::fs::create_dir_all(&proc_root).unwrap();
        let missing = with_discovery(None, Some(&home), &proc_root, || paseo_command(&json!({})));
        assert_eq!(missing, "paseo");
        std::fs::write(home.join("paseo.pid"), "{not json").unwrap();
        let garbage = with_discovery(None, Some(&home), &proc_root, || paseo_command(&json!({})));
        assert_eq!(garbage, "paseo");
        std::fs::write(home.join("paseo.pid"), json!({"pid": "x"}).to_string()).unwrap();
        let bad_pid = with_discovery(None, Some(&home), &proc_root, || paseo_command(&json!({})));
        assert_eq!(bad_pid, "paseo");
        let no_home = with_discovery(None, None, &proc_root, || paseo_command(&json!({})));
        assert_eq!(no_home, "paseo");
    }

    #[test]
    fn run_argv_joins_provider_and_model_and_builds_expected_argv() {
        let spec = PaseoSpec {
            provider: "pi".to_string(),
            model: Some("deepseek-flash".to_string()),
            thinking: Some("high".to_string()),
            mode: Some("arch".to_string()),
            isolated_config: false,
        };
        let env = vec![
            ("BEE_HERDING_WORKER".to_string(), "1".to_string()),
            ("JOB_VAR".to_string(), "xyz".to_string()),
        ];
        let argv = run_argv(&spec, "job-42", "/path/to/cwd", &env, "do task", "w-worker", None, None);
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
                "w-worker job-42",
                "--label",
                "bee_job=job-42",
                "--label",
                "bee_agent=w-worker",
                "--env",
                "BEE_HERDING_WORKER=1",
                "--env",
                "JOB_VAR=xyz",
                "do task"
            ]
        );

        let cell_argv = run_argv(&spec, "job-42", "/path/to/cwd", &env, "do task", "w-worker", Some("hlt-4"), None);
        assert_eq!(
            cell_argv,
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
                "hlt-4 w-worker",
                "--label",
                "bee_job=job-42",
                "--label",
                "bee_agent=w-worker",
                "--label",
                "bee_cell=hlt-4",
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
            isolated_config: false,
        };
        let minimal_argv = run_argv(&minimal_spec, "job-1", "/cwd", &[], "task", "plain-agent", None, None);
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
                "plain-agent job-1",
                "--label",
                "bee_job=job-1",
                "--label",
                "bee_agent=plain-agent",
                "task"
            ]
        );
    }

    #[test]
    fn wait_argv_builds_expected_tokens() {
        assert_eq!(
            wait_argv("agent-123", 45),
            vec!["agent", "wait", "agent-123", "--timeout", "45", "--json"]
        );
    }

    #[test]
    fn parse_wait_reads_status_skipping_leading_lines() {
        let out_idle = "Paseo starting...\n{\"agentId\":\"a1\",\"status\":\"idle\"}\n";
        assert_eq!(parse_wait(out_idle), Some(WaitStatus::Idle));

        let out_perm = "log\n{\"status\":\"permission\"}";
        assert_eq!(parse_wait(out_perm), Some(WaitStatus::Permission));

        let out_err = "{\"Status\":\"Error\"}";
        assert_eq!(parse_wait(out_err), Some(WaitStatus::Error));

        let out_timeout = "{\"status\":\"timeout\"}";
        assert_eq!(parse_wait(out_timeout), Some(WaitStatus::Timeout));

        assert_eq!(parse_wait("not json"), None);
        assert_eq!(parse_wait("{\"status\":\"other\"}"), None);
        assert_eq!(parse_wait("{\"no_status\":true}"), None);
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
    fn parse_inspect_full_reads_all_fields_and_sentinels() {
        let full_json = json!({
            "Status": "running",
            "UpdatedAt": "2026-10-06T12:00:00Z",
            "Model": "claude-3-5-sonnet",
            "Thinking": "high",
            "Mode": "bypassPermissions",
            "LastUsage": {"inputTokens": 120, "outputTokens": 45, "totalCostUsd": 0.02}
        })
        .to_string();
        assert_eq!(
            parse_inspect_full(&full_json),
            Some(PaseoInspect {
                state: Some(PaseoState::Working),
                updated_at: Some("2026-10-06T12:00:00Z".to_string()),
                model: Some("claude-3-5-sonnet".to_string()),
                thinking: Some("high".to_string()),
                mode: Some("bypassPermissions".to_string()),
                last_usage: Some(json!({"inputTokens": 120, "outputTokens": 45, "totalCostUsd": 0.02})),
            })
        );

        let sentinels_json = json!({
            "Status": "idle",
            "UpdatedAt": "2026-10-06T12:00:00Z",
            "Model": "-",
            "Thinking": "auto",
            "Mode": "-",
            "LastUsage": null
        })
        .to_string();
        assert_eq!(
            parse_inspect_full(&sentinels_json),
            Some(PaseoInspect {
                state: Some(PaseoState::Idle),
                updated_at: Some("2026-10-06T12:00:00Z".to_string()),
                model: None,
                thinking: None,
                mode: None,
                last_usage: None,
            })
        );

        let tolerance_json = json!({
            "status": "running",
            "updated_at": "2026-10-06T12:00:00Z",
            "model": "gpt-4o"
        })
        .to_string();
        assert_eq!(
            parse_inspect_full(&tolerance_json),
            Some(PaseoInspect {
                state: Some(PaseoState::Working),
                updated_at: Some("2026-10-06T12:00:00Z".to_string()),
                model: Some("gpt-4o".to_string()),
                thinking: None,
                mode: None,
                last_usage: None,
            })
        );

        let minimal_json = json!({"Model": "claude-3"}).to_string();
        assert_eq!(
            parse_inspect_full(&minimal_json),
            Some(PaseoInspect {
                state: None,
                updated_at: None,
                model: Some("claude-3".to_string()),
                thinking: None,
                mode: None,
                last_usage: None,
            })
        );

        assert_eq!(parse_inspect_full("not json"), None);
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

    #[test]
    fn paseo_spec_isolated_config_parsed_only_for_pi_provider() {
        let pi_cfg = json!({
            "herding": {
                "agents": {
                    "w-pi": {
                        "paseo": {
                            "provider": "pi",
                            "isolated_config": true
                        }
                    }
                }
            }
        });
        let spec = PaseoSpec::from_config(&pi_cfg, "w-pi").unwrap().unwrap();
        assert!(spec.isolated_config);

        let pi_default_cfg = json!({
            "herding": {
                "agents": {
                    "w-pi": {
                        "paseo": {
                            "provider": "pi"
                        }
                    }
                }
            }
        });
        let default_spec = PaseoSpec::from_config(&pi_default_cfg, "w-pi").unwrap().unwrap();
        assert!(!default_spec.isolated_config);

        let claude_cfg = json!({
            "herding": {
                "agents": {
                    "w-claude": {
                        "paseo": {
                            "provider": "claude",
                            "isolated_config": true
                        }
                    }
                }
            }
        });
        let claude_spec = PaseoSpec::from_config(&claude_cfg, "w-claude").unwrap().unwrap();
        assert!(!claude_spec.isolated_config);
    }

    struct ScriptedCli {
        calls: std::sync::Mutex<Vec<Vec<String>>>,
        reply: Box<dyn Fn(&[String]) -> Result<String, String> + Send + Sync>,
    }

    impl ScriptedCli {
        fn new(reply: impl Fn(&[String]) -> Result<String, String> + Send + Sync + 'static) -> Self {
            Self { calls: std::sync::Mutex::new(Vec::new()), reply: Box::new(reply) }
        }

        fn calls(&self) -> Vec<Vec<String>> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl PaseoCli for ScriptedCli {
        fn call(&self, args: &[String]) -> Result<String, String> {
            self.calls.lock().unwrap().push(args.to_vec());
            (self.reply)(args)
        }
    }

    #[test]
    fn first_json_document_skips_wrapper_log_lines() {
        let noisy = "12:00:01.123 › starting\n[login-shell-env] resolved {\n  \"PATH\": \"/x\"\n\n[{\"workspaceId\":\"wks_a\"}]\n";
        let doc = first_json_document(noisy).expect("document");
        assert_eq!(doc[0]["workspaceId"], "wks_a");
        let arr = first_json_document("Using workspace wks_1\n[{\"workspaceId\":\"wks_a\"}]").unwrap();
        assert_eq!(arr[0]["workspaceId"], "wks_a");
        let obj = first_json_document("log line\n{\"workspaceId\":\"wks_b\"}").unwrap();
        assert_eq!(obj["workspaceId"], "wks_b");
        assert!(first_json_document("only logs\n").is_none());
    }

    #[test]
    fn parse_run_agent_id_results_are_unchanged() {
        assert_eq!(parse_run_agent_id("Using workspace wks_1\n{\"agentId\":\"a-1\"}").unwrap(), "a-1");
        assert_eq!(
            parse_run_agent_id("{\"status\":\"ok\"}").unwrap_err(),
            "first JSON object in output does not contain non-empty 'agentId'"
        );
        assert_eq!(parse_run_agent_id("[1,2]").unwrap_err(), "no JSON object found in output");
        assert_eq!(parse_run_agent_id("nothing").unwrap_err(), "no JSON object found in output");
    }

    #[test]
    fn workspace_argv_shapes() {
        assert_eq!(workspace_ls_argv(), vec!["workspace", "ls", "--json"]);
        assert_eq!(project_ls_argv(), vec!["project", "ls", "--json"]);
        assert_eq!(
            workspace_create_argv("/w", Some("prj_1")),
            vec!["workspace", "create", "--path", "/w", "--isolation", "local", "--project", "prj_1", "--json"]
        );
        assert_eq!(
            workspace_create_argv("/w", None),
            vec!["workspace", "create", "--path", "/w", "--isolation", "local", "--json"]
        );
        assert_eq!(workspace_archive_argv("wks_1"), vec!["workspace", "archive", "wks_1", "--json"]);
    }

    #[cfg(unix)]
    #[test]
    fn ensure_workspace_reuses_a_workspace_at_a_symlinked_path() {
        let tmp = tempfile::tempdir().unwrap();
        let real = tmp.path().join("wt");
        std::fs::create_dir_all(&real).unwrap();
        let link = tmp.path().join("link");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let link_s = link.to_string_lossy().into_owned();
        let cli = ScriptedCli::new(move |args: &[String]| {
            if args == workspace_ls_argv().as_slice() {
                Ok(format!("[{{\"workspaceId\":\"wks_link\",\"cwd\":\"{link_s}\"}}]"))
            } else {
                Err("unexpected".to_string())
            }
        });
        assert_eq!(ensure_workspace(&cli, tmp.path(), &real).unwrap(), "wks_link");
        assert_eq!(cli.calls().len(), 1);
    }

    #[test]
    fn ensure_workspace_creates_with_the_main_checkout_project() {
        let tmp = tempfile::tempdir().unwrap();
        let main = tmp.path().join("main");
        let wt = tmp.path().join("wt");
        std::fs::create_dir_all(&main).unwrap();
        std::fs::create_dir_all(&wt).unwrap();
        let main_s = main.to_string_lossy().into_owned();
        let other_s = tmp.path().join("other").to_string_lossy().into_owned();
        let cli = ScriptedCli::new(move |args: &[String]| {
            if args == workspace_ls_argv().as_slice() {
                Ok("[]".to_string())
            } else if args == project_ls_argv().as_slice() {
                Ok(format!(
                    "[{{\"projectId\":\"prj_other\",\"path\":\"{other_s}\"}},{{\"projectId\":\"prj_main\",\"path\":\"{main_s}\"}}]"
                ))
            } else {
                Ok("noise line\n{\"workspaceId\":\"wks_new\"}".to_string())
            }
        });
        assert_eq!(ensure_workspace(&cli, &main, &wt).unwrap(), "wks_new");
        assert_eq!(cli.calls()[2], workspace_create_argv(&wt.to_string_lossy(), Some("prj_main")));
    }

    #[test]
    fn ensure_workspace_omits_project_when_none_matches() {
        let tmp = tempfile::tempdir().unwrap();
        let main = tmp.path().join("main");
        let wt = tmp.path().join("wt");
        std::fs::create_dir_all(&main).unwrap();
        std::fs::create_dir_all(&wt).unwrap();
        let cli = ScriptedCli::new(|args: &[String]| {
            if args.get(1).map(String::as_str) == Some("create") {
                Ok("{\"workspaceId\":\"wks_plain\"}".to_string())
            } else {
                Ok("[{\"projectId\":\"prj_x\",\"path\":\"/nowhere/at/all\"}]".to_string())
            }
        });
        assert_eq!(ensure_workspace(&cli, &main, &wt).unwrap(), "wks_plain");
        assert_eq!(cli.calls()[2], workspace_create_argv(&wt.to_string_lossy(), None));
    }

    #[test]
    fn ensure_workspace_reports_paseos_own_error_message() {
        let tmp = tempfile::tempdir().unwrap();
        let failing = ScriptedCli::new(|args: &[String]| {
            if args.get(1).map(String::as_str) == Some("create") {
                Err("{\n  \"error\": {\n    \"code\": \"WORKSPACE_CREATE_FAILED\",\n    \"message\": \"Unknown project: beehive\"\n  }\n}".to_string())
            } else {
                Ok("[]".to_string())
            }
        });
        assert_eq!(ensure_workspace(&failing, tmp.path(), tmp.path()).unwrap_err(), "Unknown project: beehive");
        let soft = ScriptedCli::new(|args: &[String]| {
            if args.get(1).map(String::as_str) == Some("create") {
                Ok("{\"error\":{\"message\":\"boom\"}}".to_string())
            } else {
                Ok("[]".to_string())
            }
        });
        assert_eq!(ensure_workspace(&soft, tmp.path(), tmp.path()).unwrap_err(), "boom");
    }

    #[test]
    fn archive_workspace_calls_workspace_archive() {
        let cli = ScriptedCli::new(|_args: &[String]| Ok("{}".to_string()));
        archive_workspace(&cli, "wks_9").unwrap();
        assert_eq!(cli.calls(), vec![workspace_archive_argv("wks_9")]);
    }

    #[test]
    fn inside_paseo_defaults_to_false_in_tests() {
        assert!(!inside_paseo());
        set_inside_paseo_for_test(true);
        assert!(inside_paseo());
        set_inside_paseo_for_test(false);
        assert!(!inside_paseo());
    }

    #[cfg(unix)]
    #[test]
    fn paseo_children_do_not_see_agent_scope_vars() {
        let _guard = crate::hooks::herding_env_lock();
        let prior: Vec<_> = AGENT_SCOPE_VARS.iter().map(|k| (*k, std::env::var_os(k))).collect();
        for k in AGENT_SCOPE_VARS {
            unsafe { std::env::set_var(k, "leak") };
        }
        let script = "echo ${PASEO_AGENT_ID:-absent} ${PASEO_AGENT_CWD:-absent}".to_string();
        let plain = RealPaseoCli::new("sh").call(&["-c".to_string(), script.clone()]);
        let timed = RealPaseoCli::new("sh")
            .with_timeout(std::time::Duration::from_secs(10))
            .call(&["-c".to_string(), script]);
        for (k, v) in prior {
            match v {
                Some(v) => unsafe { std::env::set_var(k, v) },
                None => unsafe { std::env::remove_var(k) },
            }
        }
        assert_eq!(plain.unwrap().trim(), "absent absent");
        assert_eq!(timed.unwrap().trim(), "absent absent");
    }
}
