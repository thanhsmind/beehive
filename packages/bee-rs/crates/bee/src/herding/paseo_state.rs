use serde_json::Value;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct AgentRecord {
    pub id: String,
    pub provider: Option<String>,
    pub cwd: Option<String>,
    pub labels: BTreeMap<String, String>,
    pub last_status: Option<String>,
    pub archived_at: Option<String>,
    pub model: Option<String>,
    pub thinking_option_id: Option<String>,
    pub session_id: Option<String>,
    pub native_handle: Option<String>,
}

#[cfg(test)]
thread_local! {
    pub(crate) static TEST_PASEO_HOME: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn env_homes() -> (Option<OsString>, Option<PathBuf>) {
    (TEST_PASEO_HOME.with(|h| h.borrow().clone()).map(OsString::from), None)
}

#[cfg(not(test))]
fn env_homes() -> (Option<OsString>, Option<PathBuf>) {
    let user = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .filter(|h| !h.is_empty())
        .map(PathBuf::from);
    (std::env::var_os("PASEO_HOME"), user)
}

pub(crate) fn home_from(command: &str, paseo_home: Option<OsString>, user_home: Option<PathBuf>) -> Option<PathBuf> {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    let flagged = tokens.iter().enumerate().find_map(|(i, t)| match t.strip_prefix("--home=") {
        Some(v) => Some(v),
        None if *t == "--home" => tokens.get(i + 1).copied(),
        None => None,
    });
    if let Some(path) = flagged.filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(path));
    }
    if let Some(env) = paseo_home.filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(env));
    }
    user_home.map(|h| h.join(".paseo"))
}

pub(crate) fn resolve_home(command: &str) -> Option<PathBuf> {
    let (paseo_home, user_home) = env_homes();
    home_from(command, paseo_home, user_home).filter(|home| home.join("agents").is_dir())
}

fn text(v: Option<&Value>) -> Option<String> {
    v.and_then(Value::as_str).map(str::to_string)
}

pub(crate) fn parse_record(raw: &str) -> Option<AgentRecord> {
    let v: Value = serde_json::from_str(raw).ok()?;
    let id = text(v.get("id")).filter(|id| !id.trim().is_empty())?;
    let runtime = v.get("runtimeInfo");
    let labels = v
        .get("labels")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .filter_map(|(k, val)| val.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();
    Some(AgentRecord {
        id,
        provider: text(v.get("provider")),
        cwd: text(v.get("cwd")),
        labels,
        last_status: text(v.get("lastStatus")),
        archived_at: text(v.get("archivedAt")),
        model: text(runtime.and_then(|r| r.get("model"))),
        thinking_option_id: text(runtime.and_then(|r| r.get("thinkingOptionId"))),
        session_id: text(runtime.and_then(|r| r.get("sessionId"))),
        native_handle: text(v.get("persistence").and_then(|p| p.get("nativeHandle"))),
    })
}

fn project_dirs(home: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(home.join("agents"))
        .map(|entries| entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect())
        .unwrap_or_default()
}

fn read_record(path: &Path) -> Option<AgentRecord> {
    parse_record(&std::fs::read_to_string(path).ok()?)
}

pub(crate) fn find_agent(home: &Path, id: &str) -> Option<AgentRecord> {
    if id.is_empty() || id.contains(['/', '\\']) || id.contains("..") {
        return None;
    }
    project_dirs(home)
        .into_iter()
        .find_map(|dir| read_record(&dir.join(format!("{id}.json"))).filter(|r| r.id == id))
}

pub(crate) fn list_labeled(home: &Path, key: &str) -> Vec<AgentRecord> {
    let mut found: Vec<AgentRecord> = project_dirs(home)
        .into_iter()
        .filter_map(|dir| std::fs::read_dir(dir).ok())
        .flat_map(|entries| entries.flatten().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .filter_map(|p| read_record(&p))
        .filter(|r| r.labels.contains_key(key))
        .collect();
    found.sort_by(|a, b| a.id.cmp(&b.id));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_agent(home: &Path, project: &str, id: &str, body: &str) {
        let dir = home.join("agents").join(project);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{id}.json")), body).unwrap();
    }

    fn full_agent(id: &str, job: &str) -> String {
        serde_json::json!({
            "id": id,
            "provider": "pi",
            "cwd": "/work/repo",
            "labels": {"bee_job": job},
            "lastStatus": "idle",
            "config": {"provider": "pi"},
            "runtimeInfo": {"model": "flash", "thinkingOptionId": "low", "sessionId": "sess-1"},
            "persistence": {"nativeHandle": "/home/u/.pi/agent/sessions/x/s.jsonl"},
            "archivedAt": null
        })
        .to_string()
    }

    #[test]
    fn paseo_state_finds_agent_by_id_under_paseo_home_and_returns_native_handle() {
        let tmp = tempfile::tempdir().unwrap();
        write_agent(tmp.path(), "proj-a", "agent-1", &full_agent("agent-1", "job-1"));
        write_agent(tmp.path(), "proj-b", "agent-2", &full_agent("agent-2", "job-2"));
        let home = home_from("paseo", Some(tmp.path().as_os_str().to_owned()), None).unwrap();
        let rec = find_agent(&home, "agent-2").unwrap();
        assert_eq!(rec.native_handle.as_deref(), Some("/home/u/.pi/agent/sessions/x/s.jsonl"));
        assert_eq!(rec.provider.as_deref(), Some("pi"));
        assert_eq!(rec.cwd.as_deref(), Some("/work/repo"));
        assert_eq!(rec.last_status.as_deref(), Some("idle"));
        assert_eq!(rec.model.as_deref(), Some("flash"));
        assert_eq!(rec.thinking_option_id.as_deref(), Some("low"));
        assert_eq!(rec.session_id.as_deref(), Some("sess-1"));
        assert_eq!(rec.archived_at, None);
        assert_eq!(rec.labels.get("bee_job").map(String::as_str), Some("job-2"));
    }

    #[test]
    fn paseo_state_lists_agents_labeled_bee_job_from_disk() {
        let tmp = tempfile::tempdir().unwrap();
        write_agent(tmp.path(), "proj-a", "agent-b", &full_agent("agent-b", "job-b"));
        write_agent(tmp.path(), "proj-b", "agent-a", &full_agent("agent-a", "job-a"));
        write_agent(tmp.path(), "proj-b", "agent-user", r#"{"id":"agent-user","labels":{"other":"x"}}"#);
        write_agent(tmp.path(), "proj-b", "agent-bad", "{not json");
        let ids: Vec<String> = list_labeled(tmp.path(), "bee_job").into_iter().map(|r| r.id).collect();
        assert_eq!(ids, vec!["agent-a", "agent-b"]);
    }

    #[test]
    fn paseo_state_missing_or_corrupt_gives_none_or_empty() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(find_agent(tmp.path(), "agent-1"), None);
        assert!(list_labeled(tmp.path(), "bee_job").is_empty());
        write_agent(tmp.path(), "proj", "agent-1", "{corrupt");
        write_agent(tmp.path(), "proj", "agent-2", r#"{"labels":{}}"#);
        assert_eq!(find_agent(tmp.path(), "agent-1"), None);
        assert_eq!(find_agent(tmp.path(), "agent-2"), None);
        assert_eq!(find_agent(tmp.path(), "../proj/agent-1"), None);
        let minimal = parse_record(r#"{"id":"only-id"}"#).unwrap();
        assert_eq!(minimal.id, "only-id");
        assert_eq!(minimal.native_handle, None);
        assert!(parse_record("[]").is_none());
    }

    #[test]
    fn paseo_state_home_token_in_command_wins_over_paseo_home() {
        let env = Some(OsString::from("/env/paseo"));
        let user = Some(PathBuf::from("/home/u"));
        assert_eq!(home_from("paseo --home /cfg/paseo", env.clone(), user.clone()), Some(PathBuf::from("/cfg/paseo")));
        assert_eq!(home_from("paseo --home=/cfg/eq", env.clone(), user.clone()), Some(PathBuf::from("/cfg/eq")));
        assert_eq!(home_from("paseo", env, user.clone()), Some(PathBuf::from("/env/paseo")));
        assert_eq!(home_from("paseo", Some(OsString::new()), user), Some(PathBuf::from("/home/u/.paseo")));
        assert_eq!(home_from("paseo", None, None), None);
    }

    #[test]
    fn paseo_state_resolve_home_needs_an_agents_dir() {
        let tmp = tempfile::tempdir().unwrap();
        TEST_PASEO_HOME.with(|h| *h.borrow_mut() = Some(tmp.path().to_path_buf()));
        assert_eq!(resolve_home("paseo"), None);
        std::fs::create_dir_all(tmp.path().join("agents")).unwrap();
        assert_eq!(resolve_home("paseo"), Some(tmp.path().to_path_buf()));
        TEST_PASEO_HOME.with(|h| *h.borrow_mut() = None);
    }
}
