use crate::fsutil::{read_json, ReadJson};
use crate::hooks::session_close::{claude_projects_root, encode_project_dir, event_ms, js_date_parse, read_jsonl};
use crate::jsjson;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const CMD: &str = "report context";
const VALUE_FLAGS: [&str; 2] = ["since", "projects-dir"];
const DEFAULT_HANDOFF_MARK: u64 = 120_000;
const REWORK_WINDOW_CALLS: usize = 40;
const REWORK_NEEDLES: [&str; 2] = ["cells reopen", "mailbox reflect"];
const HANDOFF_PROMPT_NEEDLES: [&str; 2] = ["handoff", "follow-up"];

struct Opts {
    values: Vec<(&'static str, String)>,
    json: bool,
}

impl Opts {
    fn get(&self, name: &str) -> Option<&str> {
        self.values.iter().find(|(k, _)| *k == name).map(|(_, v)| v.as_str())
    }
}

fn parse(tokens: &[OsString]) -> Option<Opts> {
    let toks: Vec<&str> = tokens.iter().map(|t| t.to_str()).collect::<Option<_>>()?;
    let mut values: Vec<(&'static str, String)> = Vec::new();
    let mut json = false;
    let mut i = 0;
    while i < toks.len() {
        let tok = toks[i];
        if tok == "--json" {
            json = true;
            i += 1;
            continue;
        }
        let body = tok.strip_prefix("--")?;
        let (name, inline) = match body.split_once('=') {
            Some((n, v)) => (n, Some(v)),
            None => (body, None),
        };
        let key = *VALUE_FLAGS.iter().find(|f| **f == name)?;
        let value = match inline {
            Some(v) => v.to_string(),
            None => {
                i += 1;
                toks.get(i)?.to_string()
            }
        };
        values.retain(|(k, _)| *k != key);
        values.push((key, value));
        i += 1;
    }
    Some(Opts { values, json })
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "report" || args.get(1)?.to_str()? != "context" {
        return None;
    }
    let opts = parse(&args[2..])?;
    Some(run(&opts, t0))
}

#[derive(Debug)]
struct Fail {
    kind: &'static str,
    msg: String,
}

fn run(opts: &Opts, t0: Instant) -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(c) => c,
        Err(e) => {
            return emit(Path::new("."), opts.json, &Fail { kind: "bad_cwd", msg: format!("bee {CMD}: {e}") }, t0)
        }
    };
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => return emit_unsupported_root(&cwd, CMD, opts.json, t0, &why),
        Roots::None => return emit_no_root_error(&cwd, CMD, opts.json, t0),
    };
    let control = crate::hooks::session_init::control_root_for(&root);
    match measure(&control, opts) {
        Ok(stats) => {
            if opts.json {
                println!("{}", jsjson::stringify_pretty(&render_json(&stats)));
            } else {
                println!("{}", render_text(&stats));
            }
            record_timing(&root, CMD, t0, true);
            ExitCode::SUCCESS
        }
        Err(f) => emit(&root, opts.json, &f, t0),
    }
}

fn emit(root: &Path, use_json: bool, f: &Fail, t0: Instant) -> ExitCode {
    if use_json {
        println!("{}", jsjson::stringify(&json!({"error": f.msg, "kind": f.kind})));
    } else {
        eprintln!("{}", f.msg);
    }
    record_timing(root, CMD, t0, false);
    ExitCode::FAILURE
}

fn measure(control: &Path, opts: &Opts) -> Result<Stats, Fail> {
    let since_ms = match opts.get("since") {
        Some(raw) => Some(js_date_parse(raw).ok_or_else(|| Fail {
            kind: "bad_time",
            msg: format!(
                "bee {CMD}: refused — --since {raw:?} is not an ISO-8601 timestamp. FIX: pass a value like 2026-10-23T01:00:00Z."
            ),
        })?),
        None => None,
    };
    let dir = match opts.get("projects-dir") {
        Some(raw) => PathBuf::from(raw),
        None => claude_projects_root().join(encode_project_dir(&control.to_string_lossy())),
    };
    if !dir.is_dir() {
        return Err(Fail {
            kind: "projects_dir_not_found",
            msg: format!(
                "bee {CMD}: refused — {} is not a directory. FIX: pass --projects-dir <path> naming the folder that holds the session transcripts.",
                dir.display()
            ),
        });
    }
    let claimed = claimed_session_ids(control);
    let mark = handoff_mark(control);
    let sessions: Vec<Session> = leader_transcripts(&dir, &claimed)
        .iter()
        .map(|path| parse_session(&read_jsonl(path), since_ms))
        .collect();
    Ok(aggregate(&sessions, mark))
}

fn handoff_mark(control: &Path) -> u64 {
    match read_json(&control.join(".bee").join("config.json")) {
        ReadJson::Parsed(cfg) => match cfg.get("handoff_context_tokens") {
            Some(Value::Number(n)) => n.as_u64().unwrap_or(DEFAULT_HANDOFF_MARK),
            _ => DEFAULT_HANDOFF_MARK,
        },
        _ => DEFAULT_HANDOFF_MARK,
    }
}

fn stem(path: &Path) -> Option<String> {
    path.file_stem().map(|s| s.to_string_lossy().into_owned())
}

fn claimed_session_ids(control: &Path) -> HashSet<String> {
    let Ok(entries) = std::fs::read_dir(control.join(".bee").join("mailbox")) else { return HashSet::new() };
    entries
        .flatten()
        .filter_map(|job| match read_json(&job.path().join("activity.json")) {
            ReadJson::Parsed(activity) => activity
                .get("transcript_path")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .and_then(|p| stem(Path::new(p))),
            _ => None,
        })
        .collect()
}

fn leader_transcripts(dir: &Path, claimed: &HashSet<String>) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
        .filter(|p| stem(p).is_some_and(|s| !claimed.contains(&s)))
        .collect();
    paths.sort();
    paths
}

#[derive(Debug, Default)]
struct Call {
    context: u64,
    bash: Vec<String>,
}

#[derive(Debug, Default)]
struct Session {
    first_prompt: Option<String>,
    calls: Vec<Call>,
}

#[derive(Debug, PartialEq)]
struct Stats {
    sessions: usize,
    calls: usize,
    mean_context: u64,
    p90_context: u64,
    handoff_mark: u64,
    above_mark_calls: usize,
    above_mark_share: Option<f64>,
    handoff_started_sessions: usize,
    rework_commands: usize,
}

fn token(usage: &Value, key: &str) -> u64 {
    usage.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn message_text(ev: &Value) -> Option<String> {
    match ev.get("message")?.get("content")? {
        Value::String(s) => Some(s.clone()),
        Value::Array(blocks) => {
            let parts: Vec<&str> = blocks
                .iter()
                .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
                .filter_map(|b| b.get("text").and_then(Value::as_str))
                .collect();
            (!parts.is_empty()).then(|| parts.join("\n"))
        }
        _ => None,
    }
}

fn bash_commands(ev: &Value) -> Vec<String> {
    let Some(blocks) = ev.get("message").and_then(|m| m.get("content")).and_then(Value::as_array) else {
        return Vec::new();
    };
    blocks
        .iter()
        .filter(|b| b.get("type").and_then(Value::as_str) == Some("tool_use"))
        .filter(|b| b.get("name").and_then(Value::as_str) == Some("Bash"))
        .filter_map(|b| b.get("input")?.get("command")?.as_str().map(str::to_string))
        .collect()
}

fn parse_session(events: &[Value], since_ms: Option<f64>) -> Session {
    let mut session = Session::default();
    let mut index: Vec<String> = Vec::new();
    for ev in events {
        if since_ms.is_some_and(|s| !event_ms(ev).is_some_and(|t| t >= s)) {
            continue;
        }
        match ev.get("type").and_then(Value::as_str) {
            Some("user") => {
                if session.first_prompt.is_none() && ev.get("isMeta").and_then(Value::as_bool) != Some(true) {
                    session.first_prompt = message_text(ev).filter(|t| !t.trim().is_empty());
                }
            }
            Some("assistant") => {
                let Some(message) = ev.get("message") else { continue };
                if message.get("model").and_then(Value::as_str) == Some("<synthetic>") {
                    continue;
                }
                let Some(usage) = message.get("usage").filter(|u| u.is_object()) else { continue };
                let context = token(usage, "input_tokens")
                    + token(usage, "cache_creation_input_tokens")
                    + token(usage, "cache_read_input_tokens");
                let id = message.get("id").and_then(Value::as_str).map(str::to_string);
                let pos = match id.as_ref().and_then(|i| index.iter().position(|k| k == i)) {
                    Some(pos) => pos,
                    None => {
                        index.push(id.unwrap_or_else(|| format!("#{}", index.len())));
                        session.calls.push(Call { context, bash: Vec::new() });
                        session.calls.len() - 1
                    }
                };
                session.calls[pos].bash.extend(bash_commands(ev));
            }
            _ => {}
        }
    }
    session.calls.retain(|c| c.context > 0);
    session
}

fn p90(sorted: &[u64]) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let rank = (sorted.len() * 9).div_ceil(10);
    sorted[rank.saturating_sub(1)]
}

fn handoff_started(session: &Session) -> bool {
    session.first_prompt.as_ref().is_some_and(|p| {
        let lower = p.to_lowercase();
        HANDOFF_PROMPT_NEEDLES.iter().any(|n| lower.contains(n))
    })
}

fn rework_in_first_calls(session: &Session) -> usize {
    session
        .calls
        .iter()
        .take(REWORK_WINDOW_CALLS)
        .flat_map(|c| c.bash.iter())
        .filter(|cmd| REWORK_NEEDLES.iter().any(|n| cmd.contains(n)))
        .count()
}

fn aggregate(sessions: &[Session], mark: u64) -> Stats {
    let live: Vec<&Session> = sessions.iter().filter(|s| !s.calls.is_empty()).collect();
    let mut contexts: Vec<u64> = live.iter().flat_map(|s| s.calls.iter().map(|c| c.context)).collect();
    contexts.sort_unstable();
    let total: u64 = contexts.iter().sum();
    let above: Vec<u64> = contexts.iter().copied().filter(|c| mark > 0 && *c >= mark).collect();
    let above_sum: u64 = above.iter().sum();
    let started: Vec<&&Session> = live.iter().filter(|s| handoff_started(s)).collect();
    Stats {
        sessions: live.len(),
        calls: contexts.len(),
        mean_context: if contexts.is_empty() { 0 } else { (total as f64 / contexts.len() as f64).round() as u64 },
        p90_context: p90(&contexts),
        handoff_mark: mark,
        above_mark_calls: above.len(),
        above_mark_share: (mark > 0 && total > 0).then(|| above_sum as f64 / total as f64),
        handoff_started_sessions: started.len(),
        rework_commands: started.iter().map(|s| rework_in_first_calls(s)).sum(),
    }
}

fn share_text(share: Option<f64>) -> String {
    share.map_or_else(|| "n/a (mark is 0)".to_string(), |s| format!("{:.1}%", s * 100.0))
}

fn render_text(s: &Stats) -> String {
    [
        format!("Leader sessions: {}", s.sessions),
        format!("Calls: {}", s.calls),
        format!("Context per call: mean {} | p90 {}", s.mean_context, s.p90_context),
        format!(
            "Above the handoff mark ({}): {} of summed context, {} calls",
            s.handoff_mark,
            share_text(s.above_mark_share),
            s.above_mark_calls
        ),
        format!(
            "Handoff-started sessions: {} | rework commands in their first {REWORK_WINDOW_CALLS} calls: {}",
            s.handoff_started_sessions, s.rework_commands
        ),
    ]
    .join("\n")
}

fn render_json(s: &Stats) -> Value {
    let mut m = Map::new();
    m.insert("sessions".into(), Value::from(s.sessions));
    m.insert("calls".into(), Value::from(s.calls));
    m.insert("mean_context".into(), Value::from(s.mean_context));
    m.insert("p90_context".into(), Value::from(s.p90_context));
    m.insert("handoff_mark".into(), Value::from(s.handoff_mark));
    m.insert("above_mark_calls".into(), Value::from(s.above_mark_calls));
    m.insert("above_mark_share".into(), s.above_mark_share.map_or(Value::Null, Value::from));
    m.insert("handoff_started_sessions".into(), Value::from(s.handoff_started_sessions));
    m.insert("rework_commands".into(), Value::from(s.rework_commands));
    Value::Object(m)
}
