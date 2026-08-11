// bee timings report — read-only rank of the self-timing log by slowest
// command.
//
// Feature: PBI p-10caed3f, deferred at
// docs/knowledge/areas/performance-log/cli-self-timing.md:37-39 ("R2 — The
// log is analysis material, not ceremony... The aggregation verb (`timings
// report`, slowest-command ranking) is deferred... until it lands, the
// JSONL is read directly.").
//
// Source: every native verb appends one line to `.bee/logs/timings.jsonl`
// on direct invocation (`record_timing`, verbs/mod.rs) — `{ts, cmd, ms,
// ok}`. Nothing aggregated those lines before this verb existed.
//
// This verb groups rows by `cmd` and reports, per command: `count`,
// `total_ms`, `median_ms`, `p95_ms`, `max_ms` — ranked slowest-median
// first, `name` ascending as the tiebreak. `--limit N` caps the ranked rows
// (default 15); it only ever narrows the OUTPUT, never the stats math,
// which is computed over every row of that command before truncation.
//
// Malformed lines (not valid JSON, or valid JSON missing a non-empty
// string `cmd` or a finite non-negative number `ms`) are skipped from
// every group's stats and counted separately in `malformed_count` — never
// fatal, and never a delegate. A missing or empty log is a clean empty
// report (`commands: []`, `malformed_count: 0`), exit 0 — `read_jsonl`
// (verbs::feedback) already fails open on an absent file.
//
// Read-only: no `std::fs::write`/`create_dir_all` anywhere in this file;
// only `read_jsonl`'s `std::fs::read` touches the log. This verb writes
// nothing but its own `record_timing` self-line, exactly like every other
// verb on the WIDE door (roots.rs) — it is added there as data-plane-only
// (`.bee/logs/timings.jsonl`), no control-plane path.

use crate::jsjson;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::verbs::feedback::read_jsonl;
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{Map, Value};
use std::cmp::Ordering;
use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

const DEFAULT_LIMIT: usize = 15;

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "timings" {
        return None;
    }
    if args.get(1)?.to_str()? != "report" {
        return None;
    }
    let mut use_json = false;
    let mut limit = DEFAULT_LIMIT;
    let mut i = 2usize;
    while i < args.len() {
        let tok = args[i].to_str()?;
        if tok == "--json" {
            use_json = true;
        } else if tok == "--limit" {
            let v = args.get(i + 1)?.to_str()?;
            limit = parse_limit(v)?;
            i += 1;
        } else if let Some(v) = tok.strip_prefix("--limit=") {
            limit = parse_limit(v)?;
        } else {
            return None; // unknown flag/positional — refuse out loud below
        }
        i += 1;
    }
    run(use_json, limit, t0)
}

fn parse_limit(raw: &str) -> Option<usize> {
    raw.parse::<usize>().ok().filter(|n| *n >= 1)
}

fn run(use_json: bool, limit: usize, t0: Instant) -> Option<ExitCode> {
    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, "timings report", use_json, t0, &why))
        }
        Roots::None => return Some(emit_no_root_error(&cwd, "timings report", use_json, t0)),
    };

    let (rows, malformed) = build_report(&root, limit);
    let text = render_text(&rows, malformed);

    let mut result = Map::new();
    result.insert(
        "commands".into(),
        Value::Array(rows.iter().map(row_to_json).collect()),
    );
    result.insert("malformed_count".into(), Value::from(malformed));

    if use_json {
        println!("{}", jsjson::stringify_pretty(&Value::Object(result)));
    } else {
        println!("{text}");
    }
    record_timing(&root, "timings report", t0, true);
    Some(ExitCode::SUCCESS)
}

// ─── stats (pure — root + limit in, ranked rows + malformed count out) ────

pub(crate) struct CommandStats {
    pub(crate) name: String,
    pub(crate) count: usize,
    pub(crate) total_ms: u64,
    pub(crate) median_ms: f64,
    pub(crate) p95_ms: f64,
    pub(crate) max_ms: u64,
}

fn row_to_json(s: &CommandStats) -> Value {
    let mut row = Map::new();
    row.insert("command".into(), Value::String(s.name.clone()));
    row.insert("count".into(), Value::from(s.count));
    row.insert("total_ms".into(), Value::from(s.total_ms));
    row.insert("median_ms".into(), Value::from(s.median_ms));
    row.insert("p95_ms".into(), Value::from(s.p95_ms));
    row.insert("max_ms".into(), Value::from(s.max_ms));
    Value::Object(row)
}

/// A line's `(cmd, ms)` pair, or None when the line is malformed: not an
/// object, `cmd` absent/empty/non-string, or `ms` absent/negative/
/// non-finite. `ms` is rounded to the nearest whole millisecond — the log
/// only ever writes whole-`u64` milliseconds, so a fractional value here
/// would itself be a sign of a hand-edited or corrupt line, not real data.
fn extract_entry(row: &Value) -> Option<(String, u64)> {
    let obj = row.as_object()?;
    let cmd = obj.get("cmd").and_then(Value::as_str)?.trim();
    if cmd.is_empty() {
        return None;
    }
    let ms = obj.get("ms").and_then(Value::as_f64)?;
    if !ms.is_finite() || ms < 0.0 {
        return None;
    }
    Some((cmd.to_string(), ms.round() as u64))
}

/// Nearest-rank percentile over an ascending-sorted, non-empty slice:
/// `rank = ceil(p * n)`, 1-indexed, clamped to `[1, n]`. The 50th
/// percentile (median) of an EVEN-length slice averages the two middle
/// values, the conventional split; every other percentile — including p95
/// — uses the plain nearest-rank value, so a single-sample group answers
/// with that one sample for both stats rather than leaving a gap.
fn percentile(sorted: &[u64], p: f64) -> f64 {
    let n = sorted.len();
    if p == 0.5 && n % 2 == 0 {
        let lo = sorted[n / 2 - 1] as f64;
        let hi = sorted[n / 2] as f64;
        return (lo + hi) / 2.0;
    }
    let rank = ((p * n as f64).ceil() as usize).clamp(1, n);
    sorted[rank - 1] as f64
}

/// Reads `.bee/logs/timings.jsonl`, groups by `cmd`, computes stats over
/// EVERY row of each command, ranks slowest-median first (name ascending
/// tiebreak), then truncates to `limit`. Returns the ranked rows plus the
/// malformed-line count (JSON-parse failures from `read_jsonl` plus rows
/// that parsed but failed `extract_entry`). A missing/empty log is `(vec![],
/// 0)` — `read_jsonl` already fails open, so this is never a delegate.
pub(crate) fn build_report(root: &Path, limit: usize) -> (Vec<CommandStats>, usize) {
    let path = root.join(".bee").join("logs").join("timings.jsonl");
    build_report_from(&path, limit)
}

fn build_report_from(path: &Path, limit: usize) -> (Vec<CommandStats>, usize) {
    let read = read_jsonl(path);
    let mut malformed = read.bad_lines;
    let mut by_cmd: std::collections::BTreeMap<String, Vec<u64>> = std::collections::BTreeMap::new();
    for row in &read.rows {
        match extract_entry(row) {
            Some((cmd, ms)) => by_cmd.entry(cmd).or_default().push(ms),
            None => malformed += 1,
        }
    }

    let mut stats: Vec<CommandStats> = by_cmd
        .into_iter()
        .map(|(name, mut values)| {
            values.sort_unstable();
            let count = values.len();
            let total_ms: u64 = values.iter().sum();
            let median_ms = percentile(&values, 0.5);
            let p95_ms = percentile(&values, 0.95);
            let max_ms = *values.last().expect("group always has >= 1 sample");
            CommandStats { name, count, total_ms, median_ms, p95_ms, max_ms }
        })
        .collect();

    stats.sort_by(|a, b| {
        b.median_ms
            .partial_cmp(&a.median_ms)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.name.cmp(&b.name))
    });
    stats.truncate(limit);
    (stats, malformed)
}

fn render_text(rows: &[CommandStats], malformed: usize) -> String {
    let mut lines = Vec::new();
    if rows.is_empty() {
        lines.push("timings report: no timing data.".to_string());
    } else {
        for s in rows {
            lines.push(format!(
                "{} — count={} total={}ms median={:.1}ms p95={:.1}ms max={}ms",
                s.name, s.count, s.total_ms, s.median_ms, s.p95_ms, s.max_ms
            ));
        }
    }
    if malformed > 0 {
        lines.push(format!("{malformed} malformed line(s) skipped."));
    }
    lines.join("\n")
}

// ─── tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(lines: &[&str]) -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join(".bee").join("logs");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("timings.jsonl");
        std::fs::write(&path, lines.join("\n") + "\n").unwrap();
        (tmp, path)
    }

    fn line(cmd: &str, ms: u64) -> String {
        format!(r#"{{"ts":"2026-08-11T00:00:00.000Z","cmd":"{cmd}","ms":{ms},"ok":true}}"#)
    }

    #[test]
    fn ranks_slowest_median_first_with_known_stats() {
        // cells.claim: single sample [1000] -> median=1000, p95=1000, max=1000.
        // backlog.plan: five samples [100,200,300,400,500] -> median=300
        //   (middle element), p95 nearest-rank ceil(0.95*5)=5th value=500.
        // status: two samples [50,150] -> median=(50+150)/2=100, p95
        //   nearest-rank ceil(0.95*2)=2nd (highest) value=150.
        let mut lines: Vec<String> = vec![
            line("cells.claim", 1000),
            line("backlog.plan", 100),
            line("backlog.plan", 500),
            line("backlog.plan", 300),
            line("backlog.plan", 200),
            line("backlog.plan", 400),
            line("status", 150),
            line("status", 50),
        ];
        lines.push("not valid json at all".to_string()); // malformed: bad JSON
        let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let (_tmp, path) = fixture(&refs);

        let (rows, malformed) = build_report_from(&path, 15);
        assert_eq!(malformed, 1);

        let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["cells.claim", "backlog.plan", "status"]);

        let claim = &rows[0];
        assert_eq!(claim.count, 1);
        assert_eq!(claim.total_ms, 1000);
        assert_eq!(claim.median_ms, 1000.0);
        assert_eq!(claim.p95_ms, 1000.0);
        assert_eq!(claim.max_ms, 1000);

        let plan = &rows[1];
        assert_eq!(plan.count, 5);
        assert_eq!(plan.total_ms, 1500);
        assert_eq!(plan.median_ms, 300.0);
        assert_eq!(plan.p95_ms, 500.0);
        assert_eq!(plan.max_ms, 500);

        let status = &rows[2];
        assert_eq!(status.count, 2);
        assert_eq!(status.total_ms, 200);
        assert_eq!(status.median_ms, 100.0);
        assert_eq!(status.p95_ms, 150.0);
        assert_eq!(status.max_ms, 150);
    }

    #[test]
    fn malformed_lines_are_skipped_and_counted_never_fatal() {
        let lines = vec![
            line("status", 10),
            "{\"cmd\":\"status\"}".to_string(),           // malformed: no ms
            "{\"ms\":10}".to_string(),                    // malformed: no cmd
            "{\"cmd\":\"\",\"ms\":5}".to_string(),          // malformed: empty cmd
            "{\"cmd\":\"status\",\"ms\":-5}".to_string(),   // malformed: negative ms
            "not json".to_string(),                        // malformed: bad JSON
        ];
        let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let (_tmp, path) = fixture(&refs);

        let (rows, malformed) = build_report_from(&path, 15);
        assert_eq!(malformed, 5);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "status");
        assert_eq!(rows[0].count, 1);
    }

    #[test]
    fn limit_caps_the_ranked_output_without_touching_the_stats_math() {
        let lines = vec![
            line("cells.claim", 1000),
            line("backlog.plan", 300),
            line("status", 100),
        ];
        let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let (_tmp, path) = fixture(&refs);

        let (rows, malformed) = build_report_from(&path, 2);
        assert_eq!(malformed, 0);
        assert_eq!(rows.len(), 2);
        let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["cells.claim", "backlog.plan"]);
    }

    #[test]
    fn missing_log_is_a_clean_empty_report() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".bee").join("logs").join("timings.jsonl"); // never created
        let (rows, malformed) = build_report_from(&path, DEFAULT_LIMIT);
        assert!(rows.is_empty());
        assert_eq!(malformed, 0);
        assert_eq!(render_text(&rows, malformed), "timings report: no timing data.");
    }

    #[test]
    fn empty_log_is_a_clean_empty_report() {
        let (_tmp, path) = fixture(&[]);
        let (rows, malformed) = build_report_from(&path, DEFAULT_LIMIT);
        assert!(rows.is_empty());
        assert_eq!(malformed, 0);
    }

    #[test]
    fn text_render_is_pinned() {
        let lines = vec![line("status", 100), line("status", 200), "garbage".to_string()];
        let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let (_tmp, path) = fixture(&refs);
        let (rows, malformed) = build_report_from(&path, 15);
        assert_eq!(
            render_text(&rows, malformed),
            "status — count=2 total=300ms median=150.0ms p95=200.0ms max=200ms\n1 malformed line(s) skipped."
        );
    }

    #[test]
    fn json_row_shape_is_pinned() {
        let lines = vec![line("status", 100)];
        let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let (_tmp, path) = fixture(&refs);
        let (rows, _malformed) = build_report_from(&path, 15);
        let json = row_to_json(&rows[0]);
        assert_eq!(
            json,
            serde_json::json!({
                "command": "status",
                "count": 1,
                "total_ms": 100,
                "median_ms": 100.0,
                "p95_ms": 100.0,
                "max_ms": 100
            })
        );
    }

    #[test]
    fn try_native_only_claims_timings_report() {
        assert!(try_native(&[OsString::from("timings")], Instant::now()).is_none());
        assert!(try_native(&[OsString::from("timings"), OsString::from("log")], Instant::now())
            .is_none());
        assert!(try_native(&[OsString::from("status")], Instant::now()).is_none());
    }

    #[test]
    fn parse_limit_rejects_zero_and_non_numeric() {
        assert_eq!(parse_limit("5"), Some(5));
        assert_eq!(parse_limit("0"), None);
        assert_eq!(parse_limit("-1"), None);
        assert_eq!(parse_limit("abc"), None);
    }
}
