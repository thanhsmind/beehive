//! queen-bench: the D5 performance-benchmark harness (CONTEXT.md D5,
//! rust-port-2) — host-real fixture generator + spawn-inclusive p95 runner
//! over queen-bee hot paths.
//!
//! Subcommands:
//! - `--generate --out <dir> [--decisions-bytes N] [--reservations-bytes N]
//!   [--backlog-bytes N] [--cells N] [--review-candidates N]
//!   [--git-commits N] [--transcript-tail-bytes N]` — the D5 fixture
//!   generator. Refuses (non-zero exit, no writes) if any size is under its
//!   pinned floor (rust-port-19 added the last three: git ancestry, review
//!   candidates, crash-candidate transcript — the two `status` blocks the
//!   original fixture measured as zero).
//! - `--self-test` — proves the generator both directions: all four
//!   sub-pinned-size refusal cases refuse, and a happy-path generation
//!   measurably meets every pin and passes a real `bee.mjs status --json`
//!   sanity read.
//! - `--check [--budget-ms N] [--bin-path PATH] [--runs N]` — the p95 floor
//!   gate on `queen-bee ping`: spawn-inclusive wall time over >= 50 runs,
//!   default budget 5 ms (dev gate per D5; CI perf smoke passes
//!   `--budget-ms 15`). Prints a JSON report (queen-bee percentiles + the
//!   `node -e ""` baseline) and exits non-zero if p95 >= budget.

mod bench;
mod fixture;
mod selftest;

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--self-test") => cmd_self_test(),
        Some("--check") => cmd_check(&args[1..]),
        Some("--generate") => cmd_generate(&args[1..]),
        _ => {
            eprintln!("usage: queen-bench <--generate|--self-test|--check> [flags]");
            eprintln!("  --generate --out <dir> [--decisions-bytes N] [--reservations-bytes N] [--backlog-bytes N] [--cells N] [--review-candidates N] [--git-commits N] [--transcript-tail-bytes N]");
            eprintln!("  --self-test");
            eprintln!("  --check [--budget-ms N] [--bin-path PATH] [--runs N]");
            ExitCode::FAILURE
        }
    }
}

/// `crates/queen-bench` at compile time -> repo root two levels up. Robust
/// to the process's actual cwd at run time (the verify command invokes this
/// via `cargo run --manifest-path crates/Cargo.toml`, but nothing here
/// depends on that particular invocation shape).
fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or(manifest_dir)
}

fn cmd_self_test() -> ExitCode {
    match selftest::run(&repo_root()) {
        Ok(summary) => {
            println!("queen-bench --self-test: PASS — {summary}");
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("queen-bench --self-test: FAIL — {msg}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_generate(args: &[String]) -> ExitCode {
    let mut out: Option<PathBuf> = None;
    let mut decisions_bytes = fixture::DECISIONS_FLOOR_BYTES;
    let mut reservations_bytes = fixture::RESERVATIONS_FLOOR_BYTES;
    let mut backlog_bytes = fixture::BACKLOG_FLOOR_BYTES;
    let mut cells_count = fixture::CELLS_FLOOR_COUNT;
    let mut review_candidates_count = fixture::REVIEW_CANDIDATES_FLOOR_COUNT;
    let mut git_commits_count = fixture::GIT_COMMITS_FLOOR_COUNT;
    let mut transcript_tail_bytes = fixture::TRANSCRIPT_TAIL_FLOOR_BYTES;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                i += 1;
                out = args.get(i).map(PathBuf::from);
            }
            "--decisions-bytes" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<u64>().ok()) {
                    decisions_bytes = v;
                }
            }
            "--reservations-bytes" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<u64>().ok()) {
                    reservations_bytes = v;
                }
            }
            "--backlog-bytes" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<u64>().ok()) {
                    backlog_bytes = v;
                }
            }
            "--cells" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<usize>().ok()) {
                    cells_count = v;
                }
            }
            "--review-candidates" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<usize>().ok()) {
                    review_candidates_count = v;
                }
            }
            "--git-commits" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<usize>().ok()) {
                    git_commits_count = v;
                }
            }
            "--transcript-tail-bytes" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<u64>().ok()) {
                    transcript_tail_bytes = v;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let Some(out_dir) = out else {
        eprintln!("--generate requires --out <dir>");
        return ExitCode::FAILURE;
    };

    let req = fixture::FixtureRequest {
        out_dir,
        decisions_bytes,
        reservations_bytes,
        backlog_bytes,
        cells_count,
        review_candidates_count,
        git_commits_count,
        transcript_tail_bytes,
    };
    match fixture::generate(&req) {
        Ok(report) => {
            println!("{}", report.to_json());
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("queen-bench --generate: REFUSED — {msg}");
            ExitCode::FAILURE
        }
    }
}

fn default_bin_path() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    let dir = exe.parent()?;
    let name = if cfg!(windows) { "queen-bee.exe" } else { "queen-bee" };
    Some(dir.join(name))
}

fn cmd_check(args: &[String]) -> ExitCode {
    let mut budget_ms: f64 = 5.0;
    let mut bin_path: Option<PathBuf> = None;
    let mut runs: usize = bench::MIN_RUNS;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--budget-ms" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<f64>().ok()) {
                    budget_ms = v;
                }
            }
            "--bin-path" => {
                i += 1;
                bin_path = args.get(i).map(PathBuf::from);
            }
            "--runs" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<usize>().ok()) {
                    runs = v;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let bin_path = match bin_path.or_else(default_bin_path) {
        Some(p) => p,
        None => {
            eprintln!("queen-bench --check: could not resolve the queen-bee binary path (pass --bin-path)");
            return ExitCode::FAILURE;
        }
    };
    if !bin_path.exists() {
        eprintln!(
            "queen-bench --check: queen-bee binary not found at {} (build the workspace first, or pass --bin-path)",
            bin_path.display()
        );
        return ExitCode::FAILURE;
    }

    match run_check(&bin_path, budget_ms, runs) {
        Ok((report_json, pass)) => {
            println!("{report_json}");
            if pass {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(msg) => {
            eprintln!("queen-bench --check: FAIL — {msg}");
            ExitCode::FAILURE
        }
    }
}

fn run_check(bin_path: &Path, budget_ms: f64, runs: usize) -> Result<(String, bool), String> {
    let queen_samples = bench::time_queen_bee_ping(bin_path, runs)
        .map_err(|e| format!("timing queen-bee ping: {e}"))?;
    let node_samples =
        bench::time_node_baseline(runs).map_err(|e| format!("timing node -e \"\" baseline: {e}"))?;

    let queen_stats = bench::stats_from_samples(queen_samples);
    let node_stats = bench::stats_from_samples(node_samples);
    let pass = queen_stats.p95_ms < budget_ms;

    let report = format!(
        "{{\"budget_ms\":{budget_ms},\"runs\":{runs},\"bin_path\":\"{}\",\"pass\":{pass},\"queen_bee_ping\":{{{}}},\"node_baseline\":{{\"command\":\"node -e \\\"\\\"\",{}}}}}",
        json_escape(&bin_path.display().to_string()),
        queen_stats.to_json_fields(),
        node_stats.to_json_fields(),
    );
    Ok((report, pass))
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}
