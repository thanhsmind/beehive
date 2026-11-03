use crate::hooks::session_close::{aggregate_usage, claude_projects_root, event_ms, js_date_parse, ms_to_iso, read_jsonl};
use crate::jsjson;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const CMD: &str = "report usage";
const SESSION_ENV: &str = "CLAUDE_CODE_SESSION_ID";
const ACCOUNTING_SOURCE: &str = "provider_telemetry";
const VALUE_FLAGS: [&str; 6] = ["session", "transcript", "corr", "since", "until", "task"];

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
    if args.first()?.to_str()? != "report" || args.get(1)?.to_str()? != "usage" {
        return None;
    }
    let opts = parse(&args[2..])?;
    Some(run(&opts, t0))
}

fn run(opts: &Opts, t0: Instant) -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(c) => c,
        Err(e) => return fail(Path::new("."), opts.json, "bad_cwd", format!("bee {CMD}: {e}"), t0),
    };
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => {
            return emit_unsupported_root(&cwd, CMD, opts.json, t0, &why)
        }
        Roots::None => return emit_no_root_error(&cwd, CMD, opts.json, t0),
    };
    let now_ms = chrono::Utc::now().timestamp_millis() as f64;
    match resolve_transcript(opts).and_then(|path| build(&path, opts, now_ms)) {
        Ok(report) => {
            if opts.json {
                println!("{}", jsjson::stringify_pretty(&render_json(&report)));
            } else {
                println!("{}", render_text(&report));
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

fn fail(root: &Path, use_json: bool, kind: &'static str, msg: String, t0: Instant) -> ExitCode {
    emit(root, use_json, &Fail { kind, msg }, t0)
}

#[derive(Debug)]
struct Fail {
    kind: &'static str,
    msg: String,
}

fn resolve_transcript(opts: &Opts) -> Result<PathBuf, Fail> {
    if let Some(raw) = opts.get("transcript") {
        let path = PathBuf::from(raw);
        if !path.is_file() {
            return Err(Fail {
                kind: "transcript_not_found",
                msg: format!(
                    "bee {CMD}: refused — --transcript {} is not a readable file. FIX: name the session transcript JSONL that holds the task's events.",
                    path.display()
                ),
            });
        }
        return Ok(path);
    }
    let sid = opts
        .get("session")
        .map(str::to_string)
        .or_else(|| std::env::var(SESSION_ENV).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let Some(sid) = sid else {
        return Err(Fail {
            kind: "session_missing",
            msg: format!(
                "bee {CMD}: refused — no transcript to read: neither --transcript nor --session was given and {SESSION_ENV} is unset. FIX: pass --transcript <path> or --session <id>."
            ),
        });
    };
    find_session_transcript(&claude_projects_root(), &sid).ok_or_else(|| Fail {
        kind: "session_transcript_not_found",
        msg: format!(
            "bee {CMD}: refused — no {sid}.jsonl under {}. FIX: pass --transcript <path> or start the session through Claude Code, which writes that file.",
            claude_projects_root().display()
        ),
    })
}

fn find_session_transcript(projects_root: &Path, sid: &str) -> Option<PathBuf> {
    let wanted = format!("{sid}.jsonl");
    let mut dirs = vec![projects_root.to_path_buf()];
    let mut found: Vec<(PathBuf, f64)> = Vec::new();
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else { continue };
            let path = entry.path();
            if ft.is_dir() {
                dirs.push(path);
            } else if ft.is_file() && entry.file_name().to_string_lossy() == wanted {
                let mtime = entry
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs_f64())
                    .unwrap_or(0.0);
                found.push((path, mtime));
            }
        }
    }
    found.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    found.into_iter().next().map(|(p, _)| p)
}

struct Row {
    model: String,
    input: f64,
    cache: f64,
    output: f64,
    total: f64,
}

struct UsageReport {
    transcript: PathBuf,
    corr: Option<String>,
    task: Option<String>,
    start_ms: f64,
    end_ms: f64,
    rows: Vec<Row>,
    subagent_files: usize,
}

impl UsageReport {
    fn totals(&self) -> Row {
        let mut t = Row { model: String::new(), input: 0.0, cache: 0.0, output: 0.0, total: 0.0 };
        for r in &self.rows {
            t.input += r.input;
            t.cache += r.cache;
            t.output += r.output;
            t.total += r.total;
        }
        t
    }
}

fn bad_time(flag: &str, raw: &str) -> Fail {
    Fail {
        kind: "bad_time",
        msg: format!(
            "bee {CMD}: refused — --{flag} {raw:?} is not an ISO-8601 timestamp. FIX: pass a value like 2026-10-23T01:00:00Z."
        ),
    }
}

fn build(transcript: &Path, opts: &Opts, now_ms: f64) -> Result<UsageReport, Fail> {
    let main_events = read_jsonl(transcript);
    let corr = opts.get("corr").map(str::to_string);
    let end_ms = match opts.get("until") {
        Some(raw) => js_date_parse(raw).ok_or_else(|| bad_time("until", raw))?,
        None => now_ms,
    };
    let start_ms = match opts.get("since") {
        Some(raw) => js_date_parse(raw).ok_or_else(|| bad_time("since", raw))?,
        None => corr
            .as_deref()
            .and_then(|c| corr_start_ms(&main_events, c))
            .or_else(|| main_events.iter().find_map(event_ms))
            .ok_or_else(|| Fail {
                kind: "empty_transcript",
                msg: format!(
                    "bee {CMD}: refused — {} holds no dated event, so no task window can be measured.",
                    transcript.display()
                ),
            })?,
    };
    if end_ms < start_ms {
        return Err(Fail {
            kind: "empty_window",
            msg: format!("bee {CMD}: refused — the window ends before it starts ({start_ms} > {end_ms})."),
        });
    }

    let mut rows: Vec<(String, Row)> = Vec::new();
    merge_into(&mut rows, &windowed(&main_events, start_ms, end_ms));
    let mut subagent_files = 0usize;
    for file in subagent_paths(transcript) {
        let events = read_jsonl(&file);
        if !events.is_empty() {
            subagent_files += 1;
        }
        merge_into(&mut rows, &windowed(&events, start_ms, end_ms));
    }

    Ok(UsageReport {
        transcript: transcript.to_path_buf(),
        corr,
        task: opts.get("task").map(str::to_string),
        start_ms,
        end_ms,
        rows: rows.into_iter().map(|(_, r)| r).collect(),
        subagent_files,
    })
}

fn windowed(events: &[Value], start_ms: f64, end_ms: f64) -> Vec<Value> {
    events
        .iter()
        .filter(|e| event_ms(e).is_some_and(|t| t >= start_ms && t <= end_ms))
        .cloned()
        .collect()
}

fn merge_into(rows: &mut Vec<(String, Row)>, events: &[Value]) {
    for (model, acc) in aggregate_usage(events).models.0 {
        let row = match rows.iter().position(|(m, _)| *m == model) {
            Some(pos) => &mut rows[pos].1,
            None => {
                rows.push((
                    model.clone(),
                    Row { model: model.clone(), input: 0.0, cache: 0.0, output: 0.0, total: 0.0 },
                ));
                let last = rows.len() - 1;
                &mut rows[last].1
            }
        };
        let cache = acc.cache_write + acc.cache_read;
        let input = acc.input + cache;
        row.input += input;
        row.cache += cache;
        row.output += acc.output;
        row.total += input + acc.output;
    }
}

fn subagent_paths(transcript: &Path) -> Vec<PathBuf> {
    let dir = transcript.with_extension("").join("subagents");
    let Ok(entries) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut names: Vec<String> =
        entries.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    names
        .into_iter()
        .filter(|n| n.ends_with(".jsonl"))
        .map(|n| dir.join(n))
        .collect()
}

fn corr_start_ms(events: &[Value], corr: &str) -> Option<f64> {
    events
        .iter()
        .filter(|e| e.get("type").and_then(Value::as_str) == Some("user"))
        .find(|e| event_text(e).is_some_and(|t| t.contains(corr)))
        .and_then(event_ms)
}

fn event_text(ev: &Value) -> Option<String> {
    let content = ev
        .get("message")
        .and_then(|m| m.get("content"))
        .or_else(|| ev.get("content"))?;
    match content {
        Value::String(s) => Some(s.clone()),
        Value::Array(blocks) => {
            let parts: Vec<String> = blocks
                .iter()
                .filter_map(|b| b.get("text").and_then(Value::as_str).map(str::to_string))
                .collect();
            (!parts.is_empty()).then(|| parts.join("\n"))
        }
        Value::Object(o) => o.get("text").and_then(Value::as_str).map(str::to_string),
        _ => None,
    }
}

fn n(v: f64) -> u64 {
    if v.is_finite() && v > 0.0 {
        v.round() as u64
    } else {
        0
    }
}

fn hhmmss(ms: f64) -> String {
    let total = if ms.is_finite() && ms > 0.0 { (ms / 1000.0).round() as u64 } else { 0 };
    format!("{:02}:{:02}:{:02}", total / 3600, (total % 3600) / 60, total % 60)
}

fn iso(ms: f64) -> String {
    ms_to_iso(ms).unwrap_or_else(|_| "not recorded".to_string())
}

fn render_text(r: &UsageReport) -> String {
    let t = r.totals();
    let mut out = String::new();
    out.push_str(&format!(
        "### [COMPLETION-REPORT] Task: {} / {}\n",
        r.task.as_deref().unwrap_or("(no task name)"),
        r.corr.as_deref().unwrap_or("(no correlation id)")
    ));
    out.push_str(&format!(
        "- **Correlation ID:** {}\n",
        r.corr.as_deref().unwrap_or("not recorded")
    ));
    out.push_str("- **Status:** COMPLETED\n");
    out.push_str(&format!(
        "- **Wall-Clock Duration:** {} (Start: {} | End: {})\n",
        hhmmss(r.end_ms - r.start_ms),
        iso(r.start_ms),
        iso(r.end_ms)
    ));
    out.push_str(&format!(
        "- **Total Tokens:** {} (Input: {} | Output: {} | Cache: {})\n\n",
        n(t.total),
        n(t.input),
        n(t.output),
        n(t.cache)
    ));
    out.push_str("#### Token Breakdown by Model:\n");
    out.push_str("| Model Name | Input Tokens | Output Tokens | Total Tokens | Accounting Source |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
    if r.rows.is_empty() {
        out.push_str(&format!("| no usage in the window | 0 | 0 | 0 | {ACCOUNTING_SOURCE} |"));
    } else {
        for row in &r.rows {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {ACCOUNTING_SOURCE} |\n",
                row.model,
                n(row.input),
                n(row.output),
                n(row.total)
            ));
        }
        out.pop();
    }
    out
}

fn render_json(r: &UsageReport) -> Value {
    let t = r.totals();
    let mut m = Map::new();
    m.insert("correlation_id".into(), opt_str(&r.corr));
    m.insert("task".into(), opt_str(&r.task));
    m.insert("status".into(), Value::String("COMPLETED".into()));
    m.insert(
        "window".into(),
        json!({
            "start": iso(r.start_ms),
            "end": iso(r.end_ms),
            "duration_ms": (r.end_ms - r.start_ms).max(0.0),
            "duration": hhmmss(r.end_ms - r.start_ms),
        }),
    );
    m.insert(
        "models".into(),
        Value::Array(
            r.rows
                .iter()
                .map(|row| {
                    json!({
                        "model": row.model,
                        "input": n(row.input),
                        "cache": n(row.cache),
                        "output": n(row.output),
                        "total": n(row.total),
                        "accounting_source": ACCOUNTING_SOURCE,
                    })
                })
                .collect(),
        ),
    );
    m.insert(
        "total".into(),
        json!({
            "input": n(t.input),
            "cache": n(t.cache),
            "output": n(t.output),
            "total": n(t.total),
            "accounting_source": ACCOUNTING_SOURCE,
        }),
    );
    m.insert("transcript".into(), Value::String(r.transcript.display().to_string()));
    m.insert("subagent_files".into(), Value::from(r.subagent_files));
    Value::Object(m)
}

fn opt_str(v: &Option<String>) -> Value {
    v.clone().map(Value::String).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn ev(ty: &str, ts: &str, model: &str, req: &str, inp: u32, out: u32, cw: u32, cr: u32) -> Value {
        json!({
            "type": ty,
            "requestId": req,
            "timestamp": ts,
            "message": {
                "model": model,
                "usage": {
                    "input_tokens": inp,
                    "output_tokens": out,
                    "cache_creation_input_tokens": cw,
                    "cache_read_input_tokens": cr
                }
            }
        })
    }

    fn user(ts: &str, text: &str) -> Value {
        json!({"type": "user", "timestamp": ts, "message": {"role": "user", "content": text}})
    }

    fn opts(pairs: &[(&'static str, &str)]) -> Opts {
        Opts { values: pairs.iter().map(|(k, v)| (*k, v.to_string())).collect(), json: false }
    }

    fn fixture(main: &[Value], subs: &[&[Value]]) -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("sess.jsonl");
        write_jsonl(&path, main);
        let dir = tmp.path().join("sess").join("subagents");
        std::fs::create_dir_all(&dir).unwrap();
        for (i, sub) in subs.iter().enumerate() {
            write_jsonl(&dir.join(format!("agent-{i}.jsonl")), sub);
        }
        (tmp, path)
    }

    fn write_jsonl(path: &Path, events: &[Value]) {
        let text: Vec<String> = events.iter().map(|e| e.to_string()).collect();
        std::fs::write(path, text.join("\n") + "\n").unwrap();
    }

    const T0: &str = "2026-10-23T01:00:00.000Z";
    const T1: &str = "2026-10-23T01:10:00.000Z";
    const T2: &str = "2026-10-23T01:20:00.000Z";
    const NOW: f64 = 1_792_717_800_000.0;

    #[test]
    fn window_starts_at_the_first_user_event_naming_the_corr() {
        let main = vec![
            ev("assistant", T0, "opus", "r0", 1000, 100, 0, 0),
            user(T1, "task t-abc123 please"),
            ev("assistant", T1, "opus", "r1", 10, 1, 2, 3),
            ev("assistant", T2, "opus", "r2", 20, 2, 0, 0),
        ];
        let (_tmp, path) = fixture(&main, &[]);
        let r = build(&path, &opts(&[("corr", "t-abc123"), ("until", "2026-10-23T01:30:00Z")]), NOW).unwrap();
        assert_eq!(mi(r.start_ms), 1_792_717_800_000);
        assert_eq!(r.rows.len(), 1);
        let row = &r.rows[0];
        assert_eq!(n(row.input), 10 + 2 + 3 + 20);
        assert_eq!(n(row.cache), 2 + 3);
        assert_eq!(n(row.output), 1 + 2);
        assert_eq!(n(row.total), 35 + 3);
    }

    fn mi(ms: f64) -> i64 {
        ms.round() as i64
    }

    #[test]
    fn since_overrides_the_corr_start_and_until_bounds_the_end() {
        let main = vec![
            ev("assistant", T0, "opus", "r0", 1, 1, 0, 0),
            ev("assistant", T1, "opus", "r1", 2, 2, 0, 0),
            ev("assistant", T2, "opus", "r2", 4, 4, 0, 0),
        ];
        let (_tmp, path) = fixture(&main, &[]);
        let r = build(
            &path,
            &opts(&[("since", "2026-10-23T01:05:00Z"), ("until", "2026-10-23T01:15:00Z")]),
            NOW,
        )
        .unwrap();
        assert_eq!(r.rows.len(), 1);
        assert_eq!(n(r.rows[0].input), 2);
        assert_eq!(n(r.rows[0].output), 2);
    }

    #[test]
    fn subagent_tokens_merge_into_the_same_model_row() {
        let main = vec![
            user(T0, "t-def456 go"),
            ev("assistant", T0, "opus", "r1", 100, 10, 5, 1000),
        ];
        let sub = vec![ev("assistant", T0, "opus", "s1", 50, 4, 1, 200)];
        let (_tmp, path) = fixture(&main, &[&sub]);
        let r = build(&path, &opts(&[("corr", "t-def456")]), NOW).unwrap();
        assert_eq!(r.subagent_files, 1);
        assert_eq!(r.rows.len(), 1);
        let row = &r.rows[0];
        assert_eq!(row.model, "opus");
        assert_eq!(n(row.input), 100 + 5 + 1000 + 50 + 1 + 200);
        assert_eq!(n(row.cache), 5 + 1000 + 1 + 200);
        assert_eq!(n(row.output), 14);
        assert_eq!(n(row.total), n(row.input) + 14);
    }

    #[test]
    fn a_repeated_request_id_is_counted_once() {
        let main = vec![
            user(T0, "t-abc"),
            ev("assistant", T0, "opus", "r1", 100, 10, 0, 0),
            ev("assistant", T0, "opus", "r1", 100, 10, 0, 0),
        ];
        let (_tmp, path) = fixture(&main, &[]);
        let r = build(&path, &opts(&[("corr", "t-abc")]), NOW).unwrap();
        assert_eq!(n(r.rows[0].input), 100);
        assert_eq!(n(r.rows[0].output), 10);
    }

    #[test]
    fn events_outside_the_window_are_excluded_from_main_and_subagents() {
        let main = vec![
            ev("assistant", T0, "opus", "r0", 900, 90, 0, 0),
            user(T1, "t-abc"),
            ev("assistant", T1, "opus", "r1", 10, 1, 0, 0),
            ev("assistant", T2, "opus", "r2", 800, 80, 0, 0),
        ];
        let sub = vec![
            ev("assistant", T0, "sonnet", "s0", 700, 70, 0, 0),
            ev("assistant", T1, "sonnet", "s1", 7, 1, 0, 0),
        ];
        let (_tmp, path) = fixture(&main, &[&sub]);
        let r = build(
            &path,
            &opts(&[("corr", "t-abc"), ("until", "2026-10-23T01:15:00Z")]),
            NOW,
        )
        .unwrap();
        let opus = r.rows.iter().find(|row| row.model == "opus").unwrap();
        let sonnet = r.rows.iter().find(|row| row.model == "sonnet").unwrap();
        assert_eq!(n(opus.input), 10);
        assert_eq!(n(sonnet.input), 7);
    }

    #[test]
    fn the_text_report_matches_the_agents_completion_report_header() {
        let main = vec![
            user(T0, "t-abc123 start"),
            ev("assistant", T0, "opus", "r1", 100, 10, 5, 1000),
        ];
        let (_tmp, path) = fixture(&main, &[]);
        let r = build(&path, &opts(&[("corr", "t-abc123"), ("task", "build the thing")]), NOW).unwrap();
        let text = render_text(&r);
        assert!(text.starts_with("### [COMPLETION-REPORT] Task: build the thing / t-abc123\n"), "{text}");
        assert!(text.contains("- **Correlation ID:** t-abc123\n"), "{text}");
        assert!(text.contains("- **Status:** COMPLETED\n"), "{text}");
        assert!(text.contains("- **Wall-Clock Duration:** 00:10:00 (Start: 2026-10-23T01:00:00.000Z | End: 2026-10-23T01:10:00.000Z)\n"), "{text}");
        assert!(text.contains("- **Total Tokens:** 1115 (Input: 1105 | Output: 10 | Cache: 1005)\n"), "{text}");
        assert!(text.contains("| opus | 1105 | 10 | 1115 | provider_telemetry |"), "{text}");
    }

    #[test]
    fn the_json_report_carries_the_same_numbers() {
        let main = vec![user(T0, "t-abc"), ev("assistant", T0, "opus", "r1", 100, 10, 5, 1000)];
        let (_tmp, path) = fixture(&main, &[]);
        let r = build(&path, &opts(&[("corr", "t-abc")]), NOW).unwrap();
        let v = render_json(&r);
        assert_eq!(v["correlation_id"], "t-abc");
        assert_eq!(v["total"]["input"], 1105);
        assert_eq!(v["total"]["cache"], 1005);
        assert_eq!(v["total"]["output"], 10);
        assert_eq!(v["total"]["total"], 1115);
        assert_eq!(v["models"][0]["model"], "opus");
        assert_eq!(v["models"][0]["total"], 1115);
        assert_eq!(v["total"]["accounting_source"], ACCOUNTING_SOURCE);
    }

    #[test]
    fn a_missing_transcript_is_a_typed_error_never_zeros() {
        let err = resolve_transcript(&opts(&[("transcript", "/nonexistent/sess.jsonl")])).err().unwrap();
        assert_eq!(err.kind, "transcript_not_found");
        assert!(err.msg.starts_with("bee report usage: refused"), "{}", err.msg);
        let err = resolve_transcript(&opts(&[("session", "no-such-session-6f1d")])).err().unwrap();
        assert_eq!(err.kind, "session_transcript_not_found");
        if std::env::var(SESSION_ENV).map_or(true, |v| v.trim().is_empty()) {
            let err = resolve_transcript(&opts(&[])).err().unwrap();
            assert_eq!(err.kind, "session_missing");
        }
    }

    #[test]
    fn a_bad_time_flag_is_refused() {
        let main = vec![user(T0, "t-abc"), ev("assistant", T0, "opus", "r1", 1, 1, 0, 0)];
        let (_tmp, path) = fixture(&main, &[]);
        assert_eq!(build(&path, &opts(&[("since", "yesterday")]), NOW).err().unwrap().kind, "bad_time");
        assert_eq!(build(&path, &opts(&[("until", "soon")]), NOW).err().unwrap().kind, "bad_time");
    }

    #[test]
    fn try_native_only_claims_report_usage() {
        let now = Instant::now();
        assert!(try_native(&[OsString::from("report")], now).is_none());
        assert!(try_native(&[OsString::from("report"), OsString::from("issue")], now).is_none());
        assert!(try_native(&[OsString::from("status")], now).is_none());
        assert!(try_native(&[OsString::from("timings"), OsString::from("report")], now).is_none());
    }
}
