use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

const INSPECT_KEYS: &[&str] = &[
    "Id",
    "Status",
    "UpdatedAt",
    "PendingPermissions",
    "ParentAgentId",
    "Model",
    "Thinking",
    "Mode",
    "LastUsage",
];

const FILE_KEYS: &[&str] = &["id", "labels", "lastStatus", "runtimeInfo", "persistence.nativeHandle"];

fn paseo(args: &[&str]) -> Option<String> {
    let out = Command::new("paseo").args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

fn json(args: &[&str], version: &str) -> Value {
    let raw = paseo(args).unwrap_or_else(|| panic!("paseo {} failed on Paseo {version}", args.join(" ")));
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("paseo {} is not JSON on Paseo {version}: {e}", args.join(" ")))
}

fn live() -> Option<String> {
    let version = paseo(&["--version"]).map(|v| v.trim().to_string());
    let answers = paseo(&["ls", "--json"]).is_some_and(|raw| serde_json::from_str::<Value>(&raw).is_ok());
    match version {
        Some(v) if answers => Some(v),
        _ => {
            eprintln!("skip: paseo is not installed or its daemon does not answer `paseo ls --json`");
            None
        }
    }
}

fn first_agent(version: &str) -> Option<String> {
    let id = json(&["ls", "-a", "-g", "--json"], version)
        .as_array()
        .and_then(|a| a.first())
        .and_then(|a| a.get("id").or_else(|| a.get("Id")))
        .and_then(Value::as_str)
        .map(str::to_string);
    if id.is_none() {
        eprintln!("skip: paseo lists no agent to read on Paseo {version}");
    }
    id
}

fn require(v: &Value, keys: &[&str], what: &str, version: &str) {
    for key in keys {
        let found = key.split('.').try_fold(v, |node, part| node.get(part));
        assert!(found.is_some(), "{what} has no key `{key}` on Paseo {version}");
    }
}

fn agent_file(id: &str) -> Option<PathBuf> {
    let home = std::env::var_os("PASEO_HOME")
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".paseo")))?;
    std::fs::read_dir(home.join("agents"))
        .ok()?
        .flatten()
        .map(|dir| dir.path().join(format!("{id}.json")))
        .find(|p| p.is_file())
}

#[test]
#[ignore]
fn paseo_live_ls_answers_a_json_array() {
    let Some(version) = live() else { return };
    let listed = json(&["ls", "-a", "--json"], &version);
    assert!(listed.is_array(), "paseo ls -a --json is not a JSON array on Paseo {version}");
}

#[test]
#[ignore]
fn paseo_live_inspect_carries_the_keys_bee_reads() {
    let Some(version) = live() else { return };
    let Some(id) = first_agent(&version) else { return };
    let inspected = json(&["inspect", &id, "--json"], &version);
    require(&inspected, INSPECT_KEYS, "paseo inspect --json", &version);
}

#[test]
#[ignore]
fn paseo_live_agent_file_carries_the_keys_bee_reads() {
    let Some(version) = live() else { return };
    let Some(id) = first_agent(&version) else { return };
    let path = agent_file(&id)
        .unwrap_or_else(|| panic!("no agents/*/{id}.json under PASEO_HOME or ~/.paseo on Paseo {version}"));
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let record: Value = serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    require(&record, FILE_KEYS, "the on-disk agent file", &version);
}
