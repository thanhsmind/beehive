// the maybePerfRefresh pipeline
//
// Split out of the single 2.8k-line hooks/session_close.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, ReadJson};
use crate::hooks::adapter::{emit_hook_output, encode_block, log_crash, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson::{self, js_to_string};
use crate::state::{bypass_level, read_config_raw};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// ─── perf.mjs — the maybePerfRefresh pipeline ──────────────────────────────

pub(crate) fn env_nonempty(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

pub(crate) fn node_homedir() -> String {
    if cfg!(windows) {
        env_nonempty("USERPROFILE").unwrap_or_default()
    } else {
        env_nonempty("HOME").unwrap_or_default()
    }
}

pub(crate) fn claude_projects_root() -> PathBuf {
    let base = env_nonempty("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(node_homedir()).join(".claude"));
    base.join("projects")
}

pub(crate) fn global_perf_dir() -> PathBuf {
    if let Some(dir) = env_nonempty("BEEHIVE_PERF_DIR") {
        return PathBuf::from(dir);
    }
    if let Some(xdg) = env_nonempty("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("beehive");
    }
    PathBuf::from(node_homedir()).join(".config").join("beehive")
}

pub(crate) fn global_perf_log_path() -> PathBuf {
    global_perf_dir().join("performance.jsonl")
}

/// encodeProjectDir: replace [\\/.:] with '-'.
///
/// Divergence from perf.mjs, taken deliberately AT CUTOVER (plans/rust-port.md,
/// "the two filed win32 defects"): Node's `/[\\/.]/g` kept the Windows drive
/// colon, spelling a transcript directory (`D:-projects-…`) that cannot exist
/// on NTFS. Mapping ':' as well gives `D--projects-…`, the spelling Claude Code
/// itself writes, so the perf rollup can finally find a transcript on win32.
pub(crate) fn encode_project_dir(project_path: &str) -> String {
    project_path
        .chars()
        .map(|c| if matches!(c, '\\' | '/' | '.' | ':') { '-' } else { c })
        .collect()
}

pub(crate) fn strip_jsonl_suffix(file: &Path) -> PathBuf {
    let s = file.to_string_lossy();
    PathBuf::from(s.strip_suffix(".jsonl").map(String::from).unwrap_or_else(|| s.into_owned()))
}

pub(crate) fn codex_home() -> PathBuf {
    if let Some(dir) = env_nonempty("CODEX_HOME") {
        return PathBuf::from(dir);
    }
    PathBuf::from(node_homedir()).join(".codex")
}

pub(crate) fn codex_sessions_root() -> PathBuf {
    codex_home().join("sessions")
}

pub(crate) fn verify_transcript_session_identity(path: &Path, sid: &str) -> bool {
    if sid.trim().is_empty() || !path.is_file() {
        return false;
    }
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let name_matches = file_name == format!("{sid}.jsonl") || file_name.ends_with(&format!("-{sid}.jsonl"));

    let Ok(file) = std::fs::File::open(path) else {
        return false;
    };
    use std::io::BufRead;
    let reader = std::io::BufReader::new(file);
    let mut saw_contradicting_sid = false;
    let mut saw_matching_sid = false;

    for line in reader.lines().take(25).flatten() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(val) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };

        if val.get("type").and_then(Value::as_str) == Some("session_meta") {
            if let Some(payload) = val.get("payload") {
                let id_found = payload
                    .get("id")
                    .or_else(|| payload.get("session_id"))
                    .and_then(Value::as_str);
                if let Some(found) = id_found {
                    if found == sid {
                        saw_matching_sid = true;
                    } else {
                        saw_contradicting_sid = true;
                    }
                }
            }
        }

        let top_sid = val
            .get("session_id")
            .or_else(|| val.get("sessionId"))
            .or_else(|| val.get("payload").and_then(|p| p.get("session_id")))
            .and_then(Value::as_str);
        if let Some(found) = top_sid {
            if found == sid {
                saw_matching_sid = true;
            } else {
                saw_contradicting_sid = true;
            }
        }
    }

    if saw_contradicting_sid {
        return false;
    }
    if saw_matching_sid {
        return true;
    }

    name_matches
}

pub(crate) fn find_codex_transcript(sessions_root: &Path, sid: &str) -> Option<PathBuf> {
    if !sessions_root.is_dir() {
        return None;
    }
    let mut matches: Vec<(PathBuf, f64)> = Vec::new();
    let mut dirs_to_visit = vec![(sessions_root.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = dirs_to_visit.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else { continue };
            let path = entry.path();
            if ft.is_dir() {
                if depth < 4 {
                    dirs_to_visit.push((path, depth + 1));
                }
            } else if ft.is_file() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if (name == format!("{sid}.jsonl") || name.ends_with(&format!("-{sid}.jsonl")))
                    && verify_transcript_session_identity(&path, sid)
                {
                    let mtime = entry
                        .metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs_f64() * 1000.0)
                        .unwrap_or(0.0);
                    matches.push((path, mtime));
                }
            }
        }
    }
    matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    matches.into_iter().next().map(|(p, _)| p)
}

pub(crate) fn resolve_transcript_for(root: &Path, session_id: Option<&str>) -> Option<PathBuf> {
    let sid = session_id?;
    if sid.trim().is_empty() {
        return None;
    }

    // 1. Check session record's stored transcript_path
    if let Some(session) = read_session_record(root, sid) {
        if let Some(tp) = session.get("transcript_path").and_then(Value::as_str) {
            let p = PathBuf::from(tp);
            if p.is_file() && verify_transcript_session_identity(&p, sid) {
                return Some(p);
            }
        }
    }

    // 2. Fallback A: Claude projects directory
    let projects_root = claude_projects_root();
    let dir = projects_root.join(encode_project_dir(&root.to_string_lossy()));
    let claude_file = dir.join(format!("{sid}.jsonl"));
    if claude_file.is_file() && verify_transcript_session_identity(&claude_file, sid) {
        return Some(claude_file);
    }

    // 3. Fallback B: Codex sessions directory
    let codex_root = codex_sessions_root();
    if let Some(codex_file) = find_codex_transcript(&codex_root, sid) {
        return Some(codex_file);
    }

    None
}

#[derive(Clone, Default)]
pub(crate) struct ModelAcc {
    pub(crate) input: f64,
    pub(crate) output: f64,
    pub(crate) cache_write: f64,
    pub(crate) cache_read: f64,
    pub(crate) new_t: f64,
    pub(crate) cached: f64,
    pub(crate) total: f64,
}

impl ModelAcc {
    pub(crate) fn finalize(&mut self) {
        self.new_t = self.input + self.output + self.cache_write;
        self.cached = self.cache_read;
        self.total = self.new_t + self.cached;
    }
    pub(crate) fn to_value(&self) -> Value {
        let mut m = Map::new();
        let n = |v: f64| serde_json::Number::from_f64(v).map(Value::Number).unwrap_or(Value::Null);
        m.insert("input".into(), n(self.input));
        m.insert("output".into(), n(self.output));
        m.insert("cache_write".into(), n(self.cache_write));
        m.insert("cache_read".into(), n(self.cache_read));
        m.insert("new".into(), n(self.new_t));
        m.insert("cached".into(), n(self.cached));
        m.insert("total".into(), n(self.total));
        Value::Object(m)
    }
}

/// Insertion-ordered model accumulator map (JS object semantics).
#[derive(Default)]
pub(crate) struct ModelMap(pub(crate) Vec<(String, ModelAcc)>);

impl ModelMap {
    pub(crate) fn entry(&mut self, key: &str) -> &mut ModelAcc {
        if let Some(pos) = self.0.iter().position(|(k, _)| k == key) {
            return &mut self.0[pos].1;
        }
        self.0.push((key.to_string(), ModelAcc::default()));
        let last = self.0.len() - 1;
        &mut self.0[last].1
    }
    pub(crate) fn finalize(&mut self) {
        for (_, acc) in &mut self.0 {
            acc.finalize();
        }
    }
    pub(crate) fn to_value(&self) -> Value {
        let mut m = Map::new();
        for (k, acc) in &self.0 {
            m.insert(k.clone(), acc.to_value());
        }
        Value::Object(m)
    }
}

pub(crate) fn num_field(v: &Value, key: &str) -> f64 {
    match v.get(key) {
        Some(Value::Number(n)) => {
            let f = n.as_f64().unwrap_or(0.0);
            if f.is_finite() {
                f
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

pub(crate) struct UsageAgg {
    pub(crate) models: ModelMap,
    /// Kept for shape parity with aggregateUsage; only computeMetrics (not
    /// ported — the hook never calls it) reads the totals.
    #[allow(dead_code)]
    pub(crate) totals: ModelAcc,
}

pub(crate) fn aggregate_usage(events: &[Value]) -> UsageAgg {
    struct Rec {
        model: String,
        input: f64,
        output: f64,
        cache_write: f64,
        cache_read: f64,
    }
    let mut by_req: Vec<(String, Rec)> = Vec::new();
    let mut no_req: Vec<Rec> = Vec::new();
    let mut obj_counter = 0usize;

    let mut active_model: Option<String> = None;
    let mut last_seen_codex_totals: Option<(f64, f64, f64, f64, f64)> = None;

    for ev in events {
        let ev_type = ev.get("type").and_then(Value::as_str).unwrap_or("");
        match ev_type {
            "turn_context" => {
                if let Some(m) = ev.get("payload").and_then(|p| p.get("model")).and_then(Value::as_str) {
                    if !m.is_empty() {
                        active_model = Some(m.to_string());
                    }
                }
            }
            "session_meta" => {
                if let Some(m) = ev.get("payload").and_then(|p| p.get("model")).and_then(Value::as_str) {
                    if !m.is_empty() {
                        active_model = Some(m.to_string());
                    }
                }
            }
            "assistant" => {
                let msg = ev.get("message").filter(|m| js_truthy(m)).cloned().unwrap_or(Value::Object(Map::new()));
                let Some(model_v) = msg.get("model").filter(|m| js_truthy(m)) else { continue };
                let model = js_to_string(model_v);
                if model == "<synthetic>" {
                    continue;
                }
                let usage = msg.get("usage").filter(|u| js_truthy(u)).cloned().unwrap_or(Value::Object(Map::new()));
                let rec = Rec {
                    model,
                    input: num_field(&usage, "input_tokens"),
                    output: num_field(&usage, "output_tokens"),
                    cache_write: num_field(&usage, "cache_creation_input_tokens"),
                    cache_read: num_field(&usage, "cache_read_input_tokens"),
                };
                let rid = ev.get("requestId").filter(|r| js_truthy(r));
                if let Some(rid) = rid {
                    let key = primitive_key(rid).unwrap_or_else(|| {
                        obj_counter += 1;
                        format!("o:{obj_counter}")
                    });
                    if let Some(pos) = by_req.iter().position(|(k, _)| *k == key) {
                        if rec.output > by_req[pos].1.output {
                            by_req[pos].1 = rec;
                        }
                    } else {
                        by_req.push((key, rec));
                    }
                } else {
                    no_req.push(rec);
                }
            }
            "event_msg" => {
                let Some(payload) = ev.get("payload").and_then(Value::as_object) else { continue };
                let p_type = payload.get("type").and_then(Value::as_str).unwrap_or("");
                if p_type == "thread_settings_applied" {
                    let m = payload
                        .get("thread_settings")
                        .and_then(|ts| ts.get("model"))
                        .or_else(|| payload.get("model"))
                        .and_then(Value::as_str);
                    if let Some(m) = m {
                        if !m.is_empty() {
                            active_model = Some(m.to_string());
                        }
                    }
                } else if p_type == "token_count" {
                    let info = payload.get("info").and_then(Value::as_object);
                    let tot_opt = info.and_then(|i| i.get("total_token_usage")).and_then(Value::as_object);
                    let last_opt = info.and_then(|i| i.get("last_token_usage")).and_then(Value::as_object);

                    if let Some(tot_obj) = tot_opt {
                        let tot_val = Value::Object(tot_obj.clone());
                        let curr_total_tokens = num_field(&tot_val, "total_tokens");
                        let curr_input = num_field(&tot_val, "input_tokens");
                        let curr_cached = num_field(&tot_val, "cached_input_tokens");
                        let curr_cache_write = num_field(&tot_val, "cache_write_input_tokens");
                        let curr_output = num_field(&tot_val, "output_tokens");

                        let curr_totals = (curr_input, curr_cached, curr_cache_write, curr_output, curr_total_tokens);

                        // Dedup repeated token_count records:
                        // In live Codex, identical token_count events are repeated multiple times.
                        // If total_token_usage has not advanced, skip.
                        if let Some(prev) = last_seen_codex_totals {
                            if prev == curr_totals {
                                continue;
                            }
                        }

                        // Derive incremental usage for this request:
                        // If last_token_usage is present, use it.
                        // Otherwise derive increment from total_token_usage difference.
                        let (raw_input, cached, cache_write, output) = if let Some(last_obj) = last_opt {
                            let last_val = Value::Object(last_obj.clone());
                            (
                                num_field(&last_val, "input_tokens"),
                                num_field(&last_val, "cached_input_tokens"),
                                num_field(&last_val, "cache_write_input_tokens"),
                                num_field(&last_val, "output_tokens"),
                            )
                        } else {
                            let (p_in, p_ca, p_cw, p_out, _) = last_seen_codex_totals.unwrap_or_default();
                            (
                                (curr_input - p_in).max(0.0),
                                (curr_cached - p_ca).max(0.0),
                                (curr_cache_write - p_cw).max(0.0),
                                (curr_output - p_out).max(0.0),
                            )
                        };

                        last_seen_codex_totals = Some(curr_totals);

                        // Normalize Codex input tokens:
                        // In Codex, raw prompt `input_tokens` INCLUDES `cached_input_tokens` (and `cache_write_input_tokens`).
                        // Normalize to bee's uncached-input contract so that
                        // finalize() (new = input + output + cache_write; total = new + cached)
                        // yields total = input + output from the raw shape without double-counting cached tokens.
                        let uncached_input = (raw_input - cached - cache_write).max(0.0);

                        let model = payload
                            .get("model")
                            .and_then(Value::as_str)
                            .map(String::from)
                            .or_else(|| active_model.clone())
                            .unwrap_or_else(|| "codex".to_string());

                        let rec = Rec {
                            model,
                            input: uncached_input,
                            output,
                            cache_write,
                            cache_read: cached,
                        };

                        no_req.push(rec);
                    } else if let Some(last_obj) = last_opt {
                        // Fallback if total_token_usage is absent but last_token_usage is present
                        let last_val = Value::Object(last_obj.clone());
                        let raw_input = num_field(&last_val, "input_tokens");
                        let cached = num_field(&last_val, "cached_input_tokens");
                        let cache_write = num_field(&last_val, "cache_write_input_tokens");
                        let output = num_field(&last_val, "output_tokens");
                        let uncached_input = (raw_input - cached - cache_write).max(0.0);

                        let model = payload
                            .get("model")
                            .and_then(Value::as_str)
                            .map(String::from)
                            .or_else(|| active_model.clone())
                            .unwrap_or_else(|| "codex".to_string());

                        let rec = Rec {
                            model,
                            input: uncached_input,
                            output,
                            cache_write,
                            cache_read: cached,
                        };

                        no_req.push(rec);
                    }
                }
            }
            _ => {}
        }
    }
    let mut models = ModelMap::default();
    let mut totals = ModelAcc::default();
    for r in by_req.into_iter().map(|(_, r)| r).chain(no_req) {
        let m = models.entry(&r.model);
        m.input += r.input;
        m.output += r.output;
        m.cache_write += r.cache_write;
        m.cache_read += r.cache_read;
        totals.input += r.input;
        totals.output += r.output;
        totals.cache_write += r.cache_write;
        totals.cache_read += r.cache_read;
    }
    models.finalize();
    totals.finalize();
    UsageAgg { models, totals }
}

pub(crate) fn event_ms(ev: &Value) -> Option<f64> {
    ev.get("timestamp")
        .or_else(|| ev.get("created_at"))
        .or_else(|| ev.get("payload").and_then(|p| p.get("timestamp")))
        .or_else(|| ev.get("payload").and_then(|p| p.get("created_at")))
        .and_then(Value::as_str)
        .and_then(js_date_parse)
}

pub(crate) fn running_time_ms(events: &[Value]) -> f64 {
    let turns: Vec<f64> = events
        .iter()
        .filter(|e| {
            e.get("type").and_then(Value::as_str) == Some("system")
                && e.get("subtype").and_then(Value::as_str) == Some("turn_duration")
                && matches!(e.get("durationMs"), Some(Value::Number(n)) if n.as_f64().is_some_and(f64::is_finite))
        })
        .map(|e| e.get("durationMs").and_then(Value::as_f64).unwrap_or(0.0))
        .collect();
    if !turns.is_empty() {
        return turns.iter().sum();
    }
    let mut stamps: Vec<f64> = events.iter().filter_map(event_ms).collect();
    stamps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut sum = 0.0;
    for pair in stamps.windows(2) {
        let gap = pair[1] - pair[0];
        if gap > 0.0 && gap < 300_000.0 {
            sum += gap;
        }
    }
    sum
}

pub(crate) struct AgentSpan {
    pub(crate) start_ms: f64,
    pub(crate) end_ms: f64,
}

pub(crate) fn detect_parallel(agents: &[AgentSpan], parent_events: &[Value]) -> bool {
    let mut spans: Vec<(f64, f64)> = agents
        .iter()
        .filter(|a| a.start_ms.is_finite() && a.end_ms.is_finite())
        .map(|a| (a.start_ms, a.end_ms))
        .collect();
    spans.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for pair in spans.windows(2) {
        if pair[1].0 <= pair[0].1 {
            return true;
        }
    }
    for ev in parent_events {
        let ev_type = ev.get("type").and_then(Value::as_str).unwrap_or("");
        if ev_type == "assistant" {
            let Some(Value::Array(content)) = ev.get("message").and_then(|m| m.get("content")) else {
                continue;
            };
            let agent_calls = content
                .iter()
                .filter(|b| {
                    js_truthy(b)
                        && (b.get("type").and_then(Value::as_str) == Some("tool_use")
                            || b.get("type").and_then(Value::as_str) == Some("function_call"))
                        && matches!(b.get("name").and_then(Value::as_str), Some("Agent" | "spawn_agent"))
                })
                .count();
            if agent_calls >= 2 {
                return true;
            }
        } else if ev_type == "response_item" {
            if let Some(payload) = ev.get("payload") {
                if let Some(Value::Array(content)) = payload.get("content") {
                    let agent_calls = content
                        .iter()
                        .filter(|b| {
                            js_truthy(b)
                                && (b.get("type").and_then(Value::as_str) == Some("tool_use")
                                    || b.get("type").and_then(Value::as_str) == Some("function_call")
                                    || b.get("type").and_then(Value::as_str) == Some("custom_tool_call"))
                                && matches!(
                                    b.get("name").or_else(|| b.get("tool_name")).and_then(Value::as_str),
                                    Some("Agent" | "spawn_agent")
                                )
                        })
                        .count();
                    if agent_calls >= 2 {
                        return true;
                    }
                }
                if let Some(Value::Array(calls)) = payload.get("tool_calls") {
                    let agent_calls = calls
                        .iter()
                        .filter(|b| {
                            matches!(
                                b.get("function")
                                    .and_then(|f| f.get("name"))
                                    .or_else(|| b.get("name"))
                                    .and_then(Value::as_str),
                                Some("Agent" | "spawn_agent")
                            )
                        })
                        .count();
                    if agent_calls >= 2 {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub(crate) struct SubWalk {
    pub(crate) models: ModelMap,
    pub(crate) agents: Vec<AgentSpan>,
}

pub(crate) fn walk_subagents(session_dir: &Path) -> SubWalk {
    let mut out = SubWalk { models: ModelMap::default(), agents: Vec::new() };
    let sub_dir = session_dir.join("subagents");
    let Ok(entries) = std::fs::read_dir(&sub_dir) else { return out };
    let mut names: Vec<String> =
        entries.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    // fs.readdirSync returns OS order; sort for cross-run determinism — the
    // aggregation below is order-insensitive for every serialized field.
    names.sort();
    for name in names {
        if !name.ends_with(".jsonl") {
            continue;
        }
        let events = read_jsonl(&sub_dir.join(&name));
        let stamps: Vec<f64> = events.iter().filter_map(event_ms).collect();
        if stamps.is_empty() {
            continue;
        }
        // perf.mjs reads `<name>.meta.json` here for agentType, which nothing
        // downstream of this port consumes — but readJson still WARNS on a
        // corrupt sidecar, so the read is kept for that one observable effect.
        // (Was covered by the deleted corrupt-JSON pre-flight.)
        let meta = sub_dir.join(format!("{}.meta.json", &name[..name.len() - ".jsonl".len()]));
        let _ = read_json_failopen(&meta);
        let a_start = stamps.iter().cloned().fold(f64::INFINITY, f64::min);
        let a_end = stamps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if a_end < 0.0 {
            continue; // outside the [0, MAX] window
        }
        let agg = aggregate_usage(&events);
        for (model, m) in &agg.models.0 {
            let acc = out.models.entry(model);
            acc.input += m.input;
            acc.output += m.output;
            acc.cache_write += m.cache_write;
            acc.cache_read += m.cache_read;
        }
        out.agents.push(AgentSpan { start_ms: a_start, end_ms: a_end });
    }
    out.models.finalize();
    out
}

pub(crate) struct Rollup {
    pub(crate) session_id: String,
    pub(crate) cwd: Option<Value>,
    pub(crate) models: Value,
    pub(crate) subagent_models: Value,
    pub(crate) subagent_count: usize,
    pub(crate) parallel: bool,
    pub(crate) running_time_ms: f64,
    pub(crate) event_count: usize,
    pub(crate) started_ms: Option<f64>,
    pub(crate) ended_ms: Option<f64>,
}

/// auto-wait-mark rework: `perf_refresh` no longer calls this — it reads the
/// transcript itself and calls `rollup_from_events` directly so the events
/// can be reused for the turn-end mark. The read-then-roll wrapper stays for
/// callers that hold only a PATH: close's token-usage section
/// (`verbs/drivers/close.rs`, decision 2d3abd12) walks each feature-bound
/// session record's stored `transcript_path` and has no pre-read events, so
/// the `#[allow(dead_code)]` this carried while it was test-only is gone.
pub(crate) fn rollup_transcript(file: &Path) -> Option<Rollup> {
    let events = read_jsonl(file);
    rollup_from_events(file, &events)
}

fn is_uuid(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 36 {
        return false;
    }
    for (i, &byte) in b.iter().enumerate() {
        if i == 8 || i == 13 || i == 18 || i == 23 {
            if byte != b'-' {
                return false;
            }
        } else if !byte.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

fn is_valid_session_id(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub(crate) fn extract_validated_session_id_from_stem(stem: &str) -> Option<String> {
    let s = stem.trim();
    if s.is_empty() {
        return None;
    }
    // Check if stem ends with a 36-character UUID (e.g. rollout-2026-09-12T14-28-28-01a095e4-8306-70d0-bf1f-7a48da530e71)
    if let Some(pos) = s.len().checked_sub(36) {
        if let Some(suffix) = s.get(pos..) {
            if is_uuid(suffix) {
                if pos == 0 || s.as_bytes().get(pos - 1) == Some(&b'-') {
                    return Some(suffix.to_string());
                }
            }
        }
    }
    if let Some(after_rollout) = s.strip_prefix("rollout-") {
        if let Some((ts, candidate)) = after_rollout.split_at_checked(20) {
            let b = ts.as_bytes();
            let is_digit = |idx: usize| b.get(idx).map_or(false, u8::is_ascii_digit);
            if is_digit(0) && is_digit(1) && is_digit(2) && is_digit(3)
                && b.get(4) == Some(&b'-')
                && is_digit(5) && is_digit(6)
                && b.get(7) == Some(&b'-')
                && is_digit(8) && is_digit(9)
                && b.get(10) == Some(&b'T')
                && is_digit(11) && is_digit(12)
                && b.get(13) == Some(&b'-')
                && is_digit(14) && is_digit(15)
                && b.get(16) == Some(&b'-')
                && is_digit(17) && is_digit(18)
                && b.get(19) == Some(&b'-')
            {
                let candidate = candidate.trim();
                if is_valid_session_id(candidate) {
                    return Some(candidate.to_string());
                }
            }
        }
        if is_uuid(after_rollout) {
            return Some(after_rollout.to_string());
        }
        return None;
    }

    if is_valid_session_id(s) {
        Some(s.to_string())
    } else {
        None
    }
}

/// auto-wait-mark rework: the rollup half of `rollup_transcript`, split out
/// so a caller that already holds the parsed events (the Stop hook's own
/// `perf_refresh`, which then hands the same events to `turn_end_subject`)
/// never has to `read_jsonl` the transcript a second time. `rollup_transcript`
/// above is kept as a thin read-then-roll wrapper for callers (tests, and
/// any future direct caller) that only have a path.
pub(crate) fn rollup_from_events(file: &Path, events: &[Value]) -> Option<Rollup> {
    if events.is_empty() {
        return None;
    }
    let usage = aggregate_usage(events);
    let session_dir = strip_jsonl_suffix(file);
    let sub = walk_subagents(&session_dir);
    let stamps: Vec<f64> = events.iter().filter_map(event_ms).collect();
    let cwd = events.iter().find_map(|e| {
        e.get("cwd")
            .or_else(|| e.get("payload").and_then(|p| p.get("cwd")))
            .filter(|c| matches!(c, Value::String(s) if !s.is_empty()))
            .cloned()
    });

    let session_id_from_meta = events.iter().find_map(|e| {
        if e.get("type").and_then(Value::as_str) == Some("session_meta") {
            e.get("payload")
                .and_then(|p| p.get("id").or_else(|| p.get("session_id")))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
        } else {
            None
        }
    });

    let session_id_from_events = events.iter().find_map(|e| {
        e.get("session_id")
            .or_else(|| e.get("sessionId"))
            .or_else(|| e.get("payload").and_then(|p| p.get("session_id")))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
    });

    let name = file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let stem = name.strip_suffix(".jsonl").unwrap_or(&name);
    let session_id = if let Some(sid) = session_id_from_meta {
        sid
    } else if let Some(sid) = session_id_from_events {
        sid
    } else {
        extract_validated_session_id_from_stem(stem)?
    };

    Some(Rollup {
        session_id,
        cwd,
        models: usage.models.to_value(),
        subagent_models: sub.models.to_value(),
        subagent_count: sub.agents.len(),
        parallel: detect_parallel(&sub.agents, events),
        running_time_ms: running_time_ms(events),
        event_count: events.len(),
        started_ms: stamps.iter().cloned().fold(None, |acc: Option<f64>, x| Some(acc.map_or(x, |a| a.min(x)))),
        ended_ms: stamps.iter().cloned().fold(None, |acc: Option<f64>, x| Some(acc.map_or(x, |a| a.max(x)))),
    })
}

pub(crate) fn project_name(p: &Value) -> String {
    if !js_truthy(p) {
        return "(unknown)".to_string();
    }
    let s = js_to_string(p);
    let trimmed = s.trim_end_matches(['\\', '/']);
    let last = trimmed.rsplit(['\\', '/']).next().unwrap_or("");
    if last.is_empty() {
        s
    } else {
        last.to_string()
    }
}

pub(crate) fn session_record(rollup: &Rollup) -> Result<Value, String> {
    let mut m = Map::new();
    let n = |v: f64| serde_json::Number::from_f64(v).map(Value::Number).unwrap_or(Value::Null);
    let project = rollup.cwd.clone().unwrap_or(Value::Null);
    m.insert("schema".into(), Value::String("bee-perf/v1".into()));
    m.insert("kind".into(), Value::String("session".into()));
    m.insert("session_id".into(), Value::String(rollup.session_id.clone()));
    m.insert("project".into(), project.clone());
    m.insert("project_name".into(), Value::String(project_name(&project)));
    m.insert("branch".into(), Value::Null);
    let iso_or_null = |ms: Option<f64>| -> Result<Value, String> {
        match ms {
            None => Ok(Value::Null),
            Some(ms) => ms_to_iso(ms).map(Value::String).map_err(|_| "RangeError: Invalid time value".to_string()),
        }
    };
    m.insert("started_at".into(), iso_or_null(rollup.started_ms)?);
    m.insert("ended_at".into(), iso_or_null(rollup.ended_ms)?);
    m.insert("running_time_ms".into(), n(rollup.running_time_ms));
    m.insert("parallel".into(), Value::Bool(rollup.parallel));
    m.insert("subagent_count".into(), n(rollup.subagent_count as f64));
    m.insert("models".into(), rollup.models.clone());
    m.insert("subagent_models".into(), rollup.subagent_models.clone());
    m.insert("event_count".into(), n(rollup.event_count as f64));
    m.insert("started_ms".into(), rollup.started_ms.map(n).unwrap_or(Value::Null));
    m.insert("ended_ms".into(), rollup.ended_ms.map(n).unwrap_or(Value::Null));
    m.insert("logged_at".into(), Value::String(now_iso()));
    Ok(Value::Object(m))
}

pub(crate) fn upsert_session_records(records: &[Value]) -> Result<(), String> {
    let file = global_perf_log_path();
    let ids: HashSet<String> =
        records.iter().filter_map(|r| r.get("session_id").and_then(primitive_key)).collect();
    let kept: Vec<Value> = read_jsonl(&file)
        .into_iter()
        .filter(|r| {
            !(js_truthy(r)
                && r.get("kind").and_then(Value::as_str) == Some("session")
                && r.get("session_id").and_then(primitive_key).map(|k| ids.contains(&k)).unwrap_or(false))
        })
        .collect();
    let merged: Vec<&Value> = kept.iter().chain(records.iter()).collect();
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("Error: {e}"))?;
    }
    let content = if merged.is_empty() {
        String::new()
    } else {
        format!("{}\n", merged.iter().map(|r| jsjson::stringify(r)).collect::<Vec<_>>().join("\n"))
    };
    std::fs::write(&file, content).map_err(|e| format!("Error: {e}"))
}

pub(crate) fn read_session_records() -> Vec<Value> {
    let mut by_session: Vec<(String, Value)> = Vec::new();
    for r in read_jsonl(&global_perf_log_path()) {
        if !js_truthy(&r)
            || r.get("kind").and_then(Value::as_str) != Some("session")
            || !r.get("session_id").map(js_truthy).unwrap_or(false)
        {
            continue;
        }
        let Some(key) = r.get("session_id").and_then(primitive_key) else { continue };
        let logged = js_to_string(r.get("logged_at").filter(|v| js_truthy(v)).unwrap_or(&Value::String(String::new())));
        if let Some(pos) = by_session.iter().position(|(k, _)| *k == key) {
            let prev_logged = js_to_string(
                by_session[pos].1.get("logged_at").filter(|v| js_truthy(v)).unwrap_or(&Value::String(String::new())),
            );
            if logged >= prev_logged {
                by_session[pos].1 = r;
            }
        } else {
            by_session.push((key, r));
        }
    }
    by_session.into_iter().map(|(_, v)| v).collect()
}

pub(crate) struct ProjectAgg {
    pub(crate) project: String,
    pub(crate) paths: Vec<String>,
    pub(crate) sessions: f64,
    pub(crate) parallel_sessions: f64,
    pub(crate) subagent_count: f64,
    pub(crate) event_count: f64,
    pub(crate) running_time_ms: f64,
    pub(crate) models: ModelMap,
    pub(crate) first_ms: Option<f64>,
    pub(crate) last_ms: Option<f64>,
    pub(crate) total_tokens: f64,
    pub(crate) new_tokens: f64,
    pub(crate) cached_tokens: f64,
}

pub(crate) fn add_raw_models(dst: &mut ModelMap, src: Option<&Value>) {
    let Some(Value::Object(src)) = src else { return };
    for (model, v) in src {
        let acc = dst.entry(model);
        acc.input += num_field(v, "input");
        acc.output += num_field(v, "output");
        acc.cache_write += num_field(v, "cache_write");
        acc.cache_read += num_field(v, "cache_read");
    }
}

pub(crate) fn build_matrix_from_log() -> Vec<ProjectAgg> {
    let mut by_name: Vec<ProjectAgg> = Vec::new();
    for r in read_session_records() {
        let name = match r.get("project_name").filter(|v| js_truthy(v)) {
            Some(v) => js_to_string(v),
            None => project_name(r.get("project").unwrap_or(&Value::Null)),
        };
        let pos = by_name.iter().position(|p| p.project == name).unwrap_or_else(|| {
            by_name.push(ProjectAgg {
                project: name.clone(),
                paths: Vec::new(),
                sessions: 0.0,
                parallel_sessions: 0.0,
                subagent_count: 0.0,
                event_count: 0.0,
                running_time_ms: 0.0,
                models: ModelMap::default(),
                first_ms: None,
                last_ms: None,
                total_tokens: 0.0,
                new_tokens: 0.0,
                cached_tokens: 0.0,
            });
            by_name.len() - 1
        });
        let p = &mut by_name[pos];
        if let Some(project) = r.get("project").filter(|v| js_truthy(v)) {
            let project = js_to_string(project);
            if !p.paths.contains(&project) {
                p.paths.push(project);
            }
        }
        // mergeSessionIntoProject
        p.sessions += 1.0;
        add_raw_models(&mut p.models, r.get("models"));
        p.running_time_ms += r.get("running_time_ms").map(|v| num_finite(v)).unwrap_or(0.0);
        if r.get("parallel").map(js_truthy).unwrap_or(false) {
            p.parallel_sessions += 1.0;
        }
        p.subagent_count += r.get("subagent_count").map(|v| num_finite(v)).unwrap_or(0.0);
        p.event_count += r.get("event_count").map(|v| num_finite(v)).unwrap_or(0.0);
        for (field, is_first) in [("started_ms", true), ("ended_ms", false)] {
            match r.get(field) {
                None | Some(Value::Null) => {}
                Some(v) => {
                    let x = js_number(v);
                    let slot = if is_first { &mut p.first_ms } else { &mut p.last_ms };
                    *slot = Some(match *slot {
                        None => x,
                        Some(prev) => {
                            if prev.is_nan() || x.is_nan() {
                                f64::NAN
                            } else if is_first {
                                prev.min(x)
                            } else {
                                prev.max(x)
                            }
                        }
                    });
                }
            }
        }
    }
    for p in &mut by_name {
        p.models.finalize();
        let mut total = 0.0;
        let mut fresh = 0.0;
        let mut cached = 0.0;
        for (_, m) in &p.models.0 {
            total += m.total;
            fresh += m.new_t;
            cached += m.cached;
        }
        p.total_tokens = total;
        p.new_tokens = fresh;
        p.cached_tokens = cached;
    }
    by_name.sort_by(|a, b| {
        b.total_tokens.partial_cmp(&a.total_tokens).unwrap_or(std::cmp::Ordering::Equal)
    });
    by_name
}

pub(crate) fn num_finite(v: &Value) -> f64 {
    match v {
        Value::Number(n) => {
            let f = n.as_f64().unwrap_or(0.0);
            if f.is_finite() {
                f
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}
