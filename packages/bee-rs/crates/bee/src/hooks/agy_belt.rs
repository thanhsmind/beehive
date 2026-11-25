use super::Outcome;
use super::adapter;
use crate::devtools::hook_manifests::{Runtime, Target, render_projection};
use crate::jsjson;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

const NO_WRITE_TOOLS: &[&str] = &[
    "search_web",
    "send_message",
    "call_mcp_tool",
    "read_url_content",
    "list_dir",
    "view_file",
    "list_directory",
    "update_steps",
];

const NO_WRITE_PREFIXES: &[&str] = &["browser_", "capture_browser", "read_browser"];

const PATH_ARG_KEYS: &[&str] = &["Path", "AbsolutePath", "TargetFile", "File"];

const SUBAGENT_DENY_REASON: &str = "Direct invoke_subagent is disabled on agy. Run `bee dispatch prepare --runtime agy --kind cell|gather|reviewer|advisor --json` and dispatch through herding instead.";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AgyToolCall {
    pub id: String,
    pub name: String,
    pub args: Value,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AgyPayload {
    pub hook_event_name: String,
    pub conversation_id: String,
    pub cwd: String,
    pub workspace_paths: Vec<String>,
    pub transcript_path: String,
    pub last_user_input: String,
    pub invocation_num: Option<u64>,
    pub tool_call: Option<AgyToolCall>,
    pub error: Option<Value>,
    pub continuation_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedTool {
    Check {
        tool_name: String,
        tool_input: Map<String, Value>,
    },
    NoWrite,
    Subagent,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookOutput {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn parse_agy_payload(raw: &str) -> Result<AgyPayload, String> {
    let value: Value = serde_json::from_str(raw).map_err(|e| e.to_string())?;
    if !value.is_object() {
        return Err("payload must be a JSON object".to_string());
    }
    serde_json::from_value(value).map_err(|e| e.to_string())
}

fn str_arg(args: &Map<String, Value>, key: &str) -> String {
    args.get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

pub fn map_tool(name: &str, args: &Value) -> MappedTool {
    let empty = Map::new();
    let obj = args.as_object().unwrap_or(&empty);
    match name {
        "run_command" => {
            let mut input = Map::new();
            input.insert("command".into(), Value::String(str_arg(obj, "CommandLine")));
            MappedTool::Check {
                tool_name: "Bash".to_string(),
                tool_input: input,
            }
        }
        "write_file" => {
            let mut input = Map::new();
            input.insert("file_path".into(), Value::String(str_arg(obj, "Path")));
            input.insert("content".into(), Value::String(str_arg(obj, "Content")));
            MappedTool::Check {
                tool_name: "Write".to_string(),
                tool_input: input,
            }
        }
        "edit_file" => {
            let mut input = Map::new();
            input.insert("file_path".into(), Value::String(str_arg(obj, "Path")));
            input.insert("old_string".into(), Value::String(str_arg(obj, "Target")));
            input.insert("new_string".into(), Value::String(str_arg(obj, "Replacement")));
            MappedTool::Check {
                tool_name: "Edit".to_string(),
                tool_input: input,
            }
        }
        "read_file" => {
            let mut input = Map::new();
            input.insert("file_path".into(), Value::String(str_arg(obj, "Path")));
            MappedTool::Check {
                tool_name: "Read".to_string(),
                tool_input: input,
            }
        }
        "invoke_subagent" => MappedTool::Subagent,
        _ if NO_WRITE_TOOLS.contains(&name)
            || NO_WRITE_PREFIXES.iter().any(|p| name.starts_with(p)) =>
        {
            MappedTool::NoWrite
        }
        _ => {
            if let Some(path) = PATH_ARG_KEYS
                .iter()
                .find_map(|k| obj.get(*k).and_then(Value::as_str).filter(|s| !s.is_empty()))
            {
                let mut input = Map::new();
                input.insert("file_path".into(), Value::String(path.to_string()));
                input.insert("content".into(), Value::String(String::new()));
                MappedTool::Check {
                    tool_name: "Write".to_string(),
                    tool_input: input,
                }
            } else {
                MappedTool::Unknown
            }
        }
    }
}

pub fn effective_cwd(payload: &AgyPayload) -> String {
    if !payload.cwd.trim().is_empty() {
        return payload.cwd.clone();
    }
    payload
        .workspace_paths
        .iter()
        .find(|p| !p.trim().is_empty())
        .cloned()
        .unwrap_or_default()
}

pub fn build_claude_payload(
    agy: &AgyPayload,
    claude_event: &str,
    tool: Option<(&str, &Map<String, Value>)>,
) -> Value {
    let mut out = Map::new();
    out.insert("hook_event_name".into(), Value::String(claude_event.to_string()));
    out.insert("session_id".into(), Value::String(agy.conversation_id.clone()));
    out.insert("cwd".into(), Value::String(effective_cwd(agy)));
    out.insert("bee_runtime".into(), Value::String("agy".to_string()));
    out.insert("transcript_path".into(), Value::String(agy.transcript_path.clone()));
    match claude_event {
        "UserPromptSubmit" => {
            out.insert("prompt".into(), Value::String(agy.last_user_input.clone()));
        }
        "PreToolUse" | "PostToolUse" => {
            if let Some((name, input)) = tool {
                out.insert("tool_name".into(), Value::String(name.to_string()));
                out.insert("tool_input".into(), Value::Object(input.clone()));
            }
        }
        "Stop" => {
            out.insert("stop_hook_active".into(), Value::Bool(agy.continuation_active));
        }
        _ => {}
    }
    Value::Object(out)
}

fn matcher_matches(matcher: Option<&str>, tool_name: &str) -> bool {
    let Some(m) = matcher else {
        return true;
    };
    if m.is_empty() || m == "*" {
        return true;
    }
    m.split('|').any(|pat| {
        if let Some(prefix) = pat.strip_suffix(".*") {
            tool_name.starts_with(prefix)
        } else {
            tool_name == pat
        }
    })
}

fn extract_hook_name(command: &str) -> Option<String> {
    let marker = "exec \"$b\" hook ";
    let rest = command.split_once(marker)?.1;
    let name = rest.split(';').next()?.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

pub fn claude_hooks_for(claude_event: &str, tool_name: Option<&str>) -> Vec<String> {
    let projection = render_projection(Runtime::Claude, Target::Plugin);
    let Some(groups) = projection
        .get("hooks")
        .and_then(|h| h.get(claude_event))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for group in groups {
        let matched = if claude_event == "SessionStart" {
            true
        } else if let Some(tn) = tool_name {
            matcher_matches(group.get("matcher").and_then(Value::as_str), tn)
        } else {
            group.get("matcher").is_none()
        };
        if !matched {
            continue;
        }
        if let Some(entries) = group.get("hooks").and_then(Value::as_array) {
            for entry in entries {
                if let Some(cmd) = entry.get("command").and_then(Value::as_str)
                    && let Some(name) = extract_hook_name(cmd)
                    && !names.contains(&name)
                {
                    names.push(name);
                }
            }
        }
    }
    names
}

pub fn collect_inject_steps(outputs: &[HookOutput]) -> Value {
    let mut steps = Vec::new();
    for out in outputs {
        if out.code != 0 {
            continue;
        }
        let trimmed = out.stdout.trim();
        if trimmed.is_empty() {
            continue;
        }
        let text = match serde_json::from_str::<Value>(trimmed) {
            Ok(Value::Object(obj)) => obj
                .get("systemMessage")
                .and_then(Value::as_str)
                .or_else(|| {
                    obj.get("hookSpecificOutput")
                        .and_then(|h| h.get("additionalContext"))
                        .and_then(Value::as_str)
                })
                .map(str::to_string),
            _ => Some(trimmed.to_string()),
        };
        if let Some(t) = text
            && !t.trim().is_empty()
        {
            steps.push(json!({ "systemMessage": { "systemMessage": t } }));
        }
    }
    if steps.is_empty() {
        json!({})
    } else {
        json!({ "injectSteps": steps })
    }
}

fn map_updated_input_to_agy(updated: &Map<String, Value>) -> Map<String, Value> {
    let mut mapped = Map::new();
    for (k, v) in updated {
        let agy_key = match k.as_str() {
            "command" => "CommandLine",
            "file_path" => "Path",
            "content" => "Content",
            "old_string" => "Target",
            "new_string" => "Replacement",
            other => other,
        };
        mapped.insert(agy_key.to_string(), v.clone());
    }
    mapped
}

pub fn map_pre_tool_use_answers(results: &[Result<HookOutput, String>]) -> Value {
    let mut first_repair: Option<Map<String, Value>> = None;
    for res in results {
        let out = match res {
            Ok(o) => o,
            Err(err) => {
                return json!({
                    "decision": "deny",
                    "reason": format!("agy-belt: hook execution failed: {err}")
                });
            }
        };
        if out.code == 2 {
            let reason = if out.stderr.trim().is_empty() {
                "blocked by bee hook".to_string()
            } else {
                out.stderr.trim().to_string()
            };
            return json!({ "decision": "deny", "reason": reason });
        }
        if out.code != 0 {
            let reason = if out.stderr.trim().is_empty() {
                format!("agy-belt: hook exited with status {}", out.code)
            } else {
                out.stderr.trim().to_string()
            };
            return json!({ "decision": "deny", "reason": reason });
        }
        let trimmed = out.stdout.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(Value::Object(obj)) = serde_json::from_str::<Value>(trimmed) {
            if obj.get("decision").and_then(Value::as_str) == Some("block") {
                let reason = obj
                    .get("reason")
                    .and_then(Value::as_str)
                    .unwrap_or("blocked by bee hook")
                    .to_string();
                return json!({ "decision": "deny", "reason": reason });
            }
            if let Some(hso) = obj.get("hookSpecificOutput").and_then(Value::as_object) {
                if hso.get("permissionDecision").and_then(Value::as_str) == Some("deny") {
                    let reason = hso
                        .get("permissionDecisionReason")
                        .and_then(Value::as_str)
                        .or_else(|| obj.get("systemMessage").and_then(Value::as_str))
                        .unwrap_or("denied by bee hook")
                        .to_string();
                    return json!({ "decision": "deny", "reason": reason });
                }
                if first_repair.is_none()
                    && let Some(updated) = hso.get("updatedInput").and_then(Value::as_object)
                {
                    first_repair = Some(map_updated_input_to_agy(updated));
                }
            }
        }
    }
    if let Some(overwrite) = first_repair {
        json!({ "decision": "ask", "overwrite": overwrite })
    } else {
        json!({ "decision": "ask" })
    }
}

pub fn map_stop_answers(results: &[Result<HookOutput, String>]) -> Value {
    for res in results {
        let Ok(out) = res else { continue };
        if out.code != 0 {
            continue;
        }
        let trimmed = out.stdout.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(Value::Object(obj)) = serde_json::from_str::<Value>(trimmed)
            && obj.get("decision").and_then(Value::as_str) == Some("block")
        {
            let reason = obj
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("blocked by session-close")
                .to_string();
            return json!({ "decision": "continue", "reason": reason });
        }
    }
    json!({})
}

fn spawn_hook(hook_name: &str, stdin_json: &str) -> Result<HookOutput, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut child = Command::new(exe)
        .args(["hook", hook_name])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(stdin_json.as_bytes())
            .map_err(|e| e.to_string())?;
    }
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    Ok(HookOutput {
        code: output.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn run_hooks_for_event(
    claude_event: &str,
    tool_name: Option<&str>,
    claude_payload: &Value,
) -> Vec<Result<HookOutput, String>> {
    let stdin_json = jsjson::stringify(claude_payload);
    claude_hooks_for(claude_event, tool_name)
        .into_iter()
        .map(|name| spawn_hook(&name, &stdin_json))
        .collect()
}

fn log_unknown_tool(cwd_str: &str, tool_name: &str) {
    let cwd = if cwd_str.is_empty() {
        std::env::current_dir().unwrap_or_default()
    } else {
        Path::new(cwd_str).to_path_buf()
    };
    if let Some(root) = adapter::resolve_roots(&cwd).work_root {
        adapter::log_coverage_gap(&root, "agy-belt", "unknown-agy-tool", tool_name, None);
    }
}

pub fn handle_payload(raw: &str) -> Value {
    let payload = match parse_agy_payload(raw) {
        Ok(p) => p,
        Err(err) => {
            return json!({
                "decision": "deny",
                "reason": format!("agy-belt: unreadable hook payload: {err}")
            });
        }
    };
    match payload.hook_event_name.as_str() {
        "SessionStart" => {
            let claude = build_claude_payload(&payload, "SessionStart", None);
            let outputs: Vec<HookOutput> = run_hooks_for_event("SessionStart", None, &claude)
                .into_iter()
                .filter_map(Result::ok)
                .collect();
            collect_inject_steps(&outputs)
        }
        "PreInvocation" => {
            if payload.invocation_num.unwrap_or(0) > 0 {
                return json!({});
            }
            let claude = build_claude_payload(&payload, "UserPromptSubmit", None);
            let outputs: Vec<HookOutput> = run_hooks_for_event("UserPromptSubmit", None, &claude)
                .into_iter()
                .filter_map(Result::ok)
                .collect();
            collect_inject_steps(&outputs)
        }
        "PreToolUse" => {
            let Some(tc) = payload.tool_call.as_ref() else {
                return json!({
                    "decision": "deny",
                    "reason": "agy-belt: missing toolCall in PreToolUse payload"
                });
            };
            if tc.name.trim().is_empty() {
                return json!({
                    "decision": "deny",
                    "reason": "agy-belt: empty toolCall.name in PreToolUse payload"
                });
            }
            match map_tool(&tc.name, &tc.args) {
                MappedTool::Check {
                    tool_name,
                    tool_input,
                } => {
                    let claude = build_claude_payload(
                        &payload,
                        "PreToolUse",
                        Some((&tool_name, &tool_input)),
                    );
                    let results = run_hooks_for_event("PreToolUse", Some(&tool_name), &claude);
                    map_pre_tool_use_answers(&results)
                }
                MappedTool::Subagent => json!({
                    "decision": "deny",
                    "reason": SUBAGENT_DENY_REASON
                }),
                MappedTool::NoWrite => json!({ "decision": "ask" }),
                MappedTool::Unknown => {
                    log_unknown_tool(&effective_cwd(&payload), &tc.name);
                    json!({ "decision": "ask" })
                }
            }
        }
        "PostToolUse" => {
            if let Some(tc) = payload.tool_call.as_ref() {
                let (tool_name, tool_input) = match map_tool(&tc.name, &tc.args) {
                    MappedTool::Check {
                        tool_name,
                        tool_input,
                    } => (tool_name, tool_input),
                    _ => (
                        tc.name.clone(),
                        tc.args.as_object().cloned().unwrap_or_default(),
                    ),
                };
                let claude = build_claude_payload(
                    &payload,
                    "PostToolUse",
                    Some((&tool_name, &tool_input)),
                );
                let _ = run_hooks_for_event("PostToolUse", Some(&tool_name), &claude);
            }
            json!({})
        }
        "Stop" => {
            let claude = build_claude_payload(&payload, "Stop", None);
            let results = run_hooks_for_event("Stop", None, &claude);
            map_stop_answers(&results)
        }
        _ => json!({}),
    }
}

pub fn run(_argv: &[String], stdin: &str) -> Outcome {
    let answer = handle_payload(stdin);
    println!("{}", jsjson::stringify(&answer));
    Outcome::Done(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_tool_maps_all_named_tools_and_unknown_with_path() {
        assert_eq!(
            map_tool("run_command", &json!({ "CommandLine": "git status" })),
            MappedTool::Check {
                tool_name: "Bash".into(),
                tool_input: [("command".into(), json!("git status"))].into_iter().collect(),
            }
        );
        assert_eq!(
            map_tool("write_file", &json!({ "Path": "/tmp/a.rs", "Content": "fn main() {}" })),
            MappedTool::Check {
                tool_name: "Write".into(),
                tool_input: [
                    ("file_path".into(), json!("/tmp/a.rs")),
                    ("content".into(), json!("fn main() {}")),
                ]
                .into_iter()
                .collect(),
            }
        );
        assert_eq!(
            map_tool(
                "edit_file",
                &json!({ "Path": "/tmp/a.rs", "Target": "old", "Replacement": "new" })
            ),
            MappedTool::Check {
                tool_name: "Edit".into(),
                tool_input: [
                    ("file_path".into(), json!("/tmp/a.rs")),
                    ("old_string".into(), json!("old")),
                    ("new_string".into(), json!("new")),
                ]
                .into_iter()
                .collect(),
            }
        );
        assert_eq!(
            map_tool("read_file", &json!({ "Path": "/tmp/a.rs" })),
            MappedTool::Check {
                tool_name: "Read".into(),
                tool_input: [("file_path".into(), json!("/tmp/a.rs"))].into_iter().collect(),
            }
        );
        assert_eq!(
            map_tool("invoke_subagent", &json!({ "Prompt": "hi" })),
            MappedTool::Subagent
        );
        for nw in [
            "search_web",
            "send_message",
            "call_mcp_tool",
            "read_url_content",
            "list_dir",
            "view_file",
            "list_directory",
            "update_steps",
            "browser_click",
            "capture_browser_screenshot",
            "read_browser_dom",
        ] {
            assert_eq!(map_tool(nw, &json!({})), MappedTool::NoWrite, "{nw}");
        }
        assert_eq!(
            map_tool("custom_tool", &json!({ "TargetFile": "/tmp/x.txt" })),
            MappedTool::Check {
                tool_name: "Write".into(),
                tool_input: [
                    ("file_path".into(), json!("/tmp/x.txt")),
                    ("content".into(), json!("")),
                ]
                .into_iter()
                .collect(),
            }
        );
        assert_eq!(
            map_tool("brand_new_tool", &json!({ "foo": "bar" })),
            MappedTool::Unknown
        );
    }

    #[test]
    fn claude_hooks_for_reads_catalog_without_hardcoding() {
        let pre_write = claude_hooks_for("PreToolUse", Some("Write"));
        assert!(pre_write.contains(&"write-guard".to_string()));
        assert!(pre_write.contains(&"worker-guard".to_string()));
        assert!(pre_write.contains(&"activity".to_string()));
        assert!(!pre_write.contains(&"model-guard".to_string()));

        let stop = claude_hooks_for("Stop", None);
        assert!(stop.contains(&"session-close".to_string()));
        assert!(stop.contains(&"state-sync".to_string()));
        assert!(stop.contains(&"activity".to_string()));
    }

    #[test]
    fn answer_mapping_handles_deny_repair_pass_stop_and_invocation_num() {
        let deny_exit2 = map_pre_tool_use_answers(&[Ok(HookOutput {
            code: 2,
            stdout: String::new(),
            stderr: "gate not approved".to_string(),
        })]);
        assert_eq!(
            deny_exit2,
            json!({ "decision": "deny", "reason": "gate not approved" })
        );

        let deny_json = map_pre_tool_use_answers(&[Ok(HookOutput {
            code: 0,
            stdout: json!({
                "hookSpecificOutput": {
                    "permissionDecision": "deny",
                    "permissionDecisionReason": "secret path"
                }
            })
            .to_string(),
            stderr: String::new(),
        })]);
        assert_eq!(
            deny_json,
            json!({ "decision": "deny", "reason": "secret path" })
        );

        let repair = map_pre_tool_use_answers(&[Ok(HookOutput {
            code: 0,
            stdout: json!({
                "hookSpecificOutput": {
                    "permissionDecision": "ask",
                    "updatedInput": {
                        "file_path": "/tmp/fixed.rs",
                        "old_string": "a",
                        "new_string": "b"
                    }
                }
            })
            .to_string(),
            stderr: String::new(),
        })]);
        assert_eq!(
            repair,
            json!({
                "decision": "ask",
                "overwrite": {
                    "Path": "/tmp/fixed.rs",
                    "Target": "a",
                    "Replacement": "b"
                }
            })
        );

        let pass = map_pre_tool_use_answers(&[Ok(HookOutput {
            code: 0,
            stdout: String::new(),
            stderr: String::new(),
        })]);
        assert_eq!(pass, json!({ "decision": "ask" }));

        let spawn_err = map_pre_tool_use_answers(&[Err("no binary".to_string())]);
        assert_eq!(spawn_err["decision"], json!("deny"));

        let stop_block = map_stop_answers(&[Ok(HookOutput {
            code: 0,
            stdout: json!({ "decision": "block", "reason": "open cells" }).to_string(),
            stderr: String::new(),
        })]);
        assert_eq!(
            stop_block,
            json!({ "decision": "continue", "reason": "open cells" })
        );

        let stop_pass = map_stop_answers(&[Ok(HookOutput {
            code: 0,
            stdout: String::new(),
            stderr: String::new(),
        })]);
        assert_eq!(stop_pass, json!({}));

        let later_invocation = handle_payload(
            &json!({
                "hookEventName": "PreInvocation",
                "conversationId": "c1",
                "invocationNum": 2,
                "lastUserInput": "hello"
            })
            .to_string(),
        );
        assert_eq!(later_invocation, json!({}));

        let subagent = handle_payload(
            &json!({
                "hookEventName": "PreToolUse",
                "conversationId": "c1",
                "toolCall": { "id": "1", "name": "invoke_subagent", "args": {} }
            })
            .to_string(),
        );
        assert_eq!(subagent["decision"], json!("deny"));
        assert!(
            subagent["reason"]
                .as_str()
                .unwrap()
                .contains("bee dispatch prepare --runtime agy")
        );

        let inject = collect_inject_steps(&[
            HookOutput {
                code: 0,
                stdout: "hello context".to_string(),
                stderr: String::new(),
            },
            HookOutput {
                code: 0,
                stdout: json!({ "systemMessage": "from json" }).to_string(),
                stderr: String::new(),
            },
        ]);
        assert_eq!(
            inject,
            json!({
                "injectSteps": [
                    { "systemMessage": { "systemMessage": "hello context" } },
                    { "systemMessage": { "systemMessage": "from json" } }
                ]
            })
        );
    }
}
