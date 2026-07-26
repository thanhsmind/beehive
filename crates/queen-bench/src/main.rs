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
//! - `--check [--budget-ms N] [--status-budget-ms N] [--bin-path PATH]
//!   [--runs N]` — the spawn-inclusive p95 gates, over >= 50 runs each.
//!   Budgets are PER COMMAND (D5 supersession e119fc8b), never one shared
//!   number:
//!     * `queen-bee ping` — the spawn floor — keeps D5's original 5 ms dev
//!       budget (`--budget-ms`; CI perf smoke passes `--budget-ms 15`).
//!     * `queen-bee status --json` carries its own 70 ms dev budget
//!       (`--status-budget-ms`; CI perf smoke 210 ms, keeping D5's 3x
//!       runner-variance ratio). This is an INTERIM REGRESSION GUARD, not
//!       the goal: the mandatory follow-up (per-invocation store-read
//!       memoization inside bee-core) targets <= 20 ms, at which point the
//!       budget tightens to 25 ms. Status never returns to 5 ms — the
//!       reasons are measured and recorded in e119fc8b.
//!   Prints a JSON report naming which budget each gate was measured
//!   against, and exits non-zero if any gate's p95 >= its own budget.

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
            eprintln!("  --check [--budget-ms N] [--status-budget-ms N] [--bin-path PATH] [--runs N]");
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

/// D5's original spawn-floor budget, still gating `queen-bee ping` — the
/// one command the 5 ms number still describes honestly.
const PING_BUDGET_MS: f64 = 5.0;

/// The `status` command's own budget (supersession e119fc8b). Deliberately
/// a SEPARATE constant from [`PING_BUDGET_MS`]: folding both gates onto one
/// `--budget-ms` would mean either loosening the spawn floor to 70 ms or
/// leaving `status` permanently red, and neither is a gate.
const STATUS_BUDGET_MS: f64 = 70.0;

fn cmd_check(args: &[String]) -> ExitCode {
    let mut budget_ms: f64 = PING_BUDGET_MS;
    let mut status_budget_ms: f64 = STATUS_BUDGET_MS;
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
            "--status-budget-ms" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<f64>().ok()) {
                    status_budget_ms = v;
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

    match run_check(&bin_path, budget_ms, status_budget_ms, runs) {
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

/// A temp fixture root this run created, always removed on the way out
/// (success and failure alike) — the bench never leaves a 1.5 MB store
/// behind, and never touches the repo's own live `.bee/`.
struct TempFixture {
    root: PathBuf,
}

impl Drop for TempFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn fresh_temp_dir(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    env::temp_dir().join(format!("queen-bench-{label}-{}-{nanos}", std::process::id()))
}

fn run_check(bin_path: &Path, budget_ms: f64, status_budget_ms: f64, runs: usize) -> Result<(String, bool), String> {
    let queen_samples = bench::time_queen_bee_ping(bin_path, runs)
        .map_err(|e| format!("timing queen-bee ping: {e}"))?;
    let node_samples =
        bench::time_node_baseline(runs).map_err(|e| format!("timing node -e \"\" baseline: {e}"))?;

    let queen_stats = bench::stats_from_samples(queen_samples);
    let node_stats = bench::stats_from_samples(node_samples);
    let ping_pass = queen_stats.p95_ms < budget_ms;

    let status = run_status_check(bin_path, status_budget_ms, runs)?;
    let pass = ping_pass && status.pass;

    // Every gate names its OWN budget inline (supersession e119fc8b): a
    // reader of this report must never have to guess which number a given
    // pass/fail was measured against. `budgets` restates the pair up front
    // so the split is visible without reading both gate objects.
    let report = format!(
        "{{\"budgets\":{{\"ping_ms\":{budget_ms},\"status_ms\":{status_budget_ms}}},\"runs\":{runs},\"bin_path\":\"{}\",\"pass\":{pass},\"queen_bee_ping\":{{\"command\":\"queen-bee ping\",\"budget_ms\":{budget_ms},\"gated_on\":\"p95\",\"pass\":{ping_pass},{}}},\"node_baseline\":{{\"command\":\"node -e \\\"\\\"\",{}}},{}}}",
        json_escape(&bin_path.display().to_string()),
        queen_stats.to_json_fields(),
        node_stats.to_json_fields(),
        status.to_json_fields(),
    );
    Ok((report, pass))
}

struct StatusCheckReport {
    pass: bool,
    budget_ms: f64,
    fixture_root: PathBuf,
    cold: bench::RunStats,
    warm: bench::RunStats,
    node: bench::RunStats,
}

impl StatusCheckReport {
    /// The gate is the WARM p95 against `status`'s OWN budget (supersession
    /// e119fc8b), never the ping floor's. The cold percentiles are emitted
    /// beside it unconditionally (decision a7d7b3d5 / truth 7), so a reader
    /// can never mistake the favourable number for the whole story — even
    /// though, on this fixture, the two are indistinguishable and store I/O
    /// rather than the review-git cache is what dominates.
    fn to_json_fields(&self) -> String {
        format!(
            "\"queen_bee_status\":{{\"command\":\"queen-bee status --json\",\"budget_ms\":{},\"budget_source\":\"D5 supersession e119fc8b (interim regression guard; dedup follow-up targets <=20ms, then 25ms)\",\"fixture_root\":\"{}\",\"gated_on\":\"{}\",\"cache_artifact\":\".bee/runtime/review-git-cache.json\",\"pass\":{},\"{}_cache\":{{{}}},\"{}_cache\":{{{}}},\"node_status_baseline\":{{\"command\":\"node .bee/bin/bee.mjs status --json\",{}}}}}",
            self.budget_ms,
            json_escape(&self.fixture_root.display().to_string()),
            bench::CacheState::Warm.label(),
            self.pass,
            bench::CacheState::Cold.label(),
            self.cold.to_json_fields(),
            bench::CacheState::Warm.label(),
            self.warm.to_json_fields(),
            self.node.to_json_fields(),
        )
    }
}

/// The status gate: `queen-bee status --json` on a freshly generated,
/// PINNED host-real fixture (never a shrunk one — `fixture::generate`
/// refuses below its floors), measured spawn-inclusive over `runs` >= 50
/// runs, in BOTH cache states, with the same command's node cost recorded
/// alongside.
///
/// `budget_ms` here is the STATUS budget (supersession e119fc8b), which is
/// not D5's 5 ms spawn-floor number — that one still gates `ping` and is
/// passed separately by [`run_check`].
fn run_status_check(bin_path: &Path, budget_ms: f64, runs: usize) -> Result<StatusCheckReport, String> {
    let repo_root = repo_root();
    let bee_mjs = repo_root.join(".bee").join("bin").join("bee.mjs");
    if !bee_mjs.exists() {
        return Err(format!("bee.mjs not found at {} — the node status baseline needs it", bee_mjs.display()));
    }

    let fixture = TempFixture { root: fresh_temp_dir("status-fixture") };
    // Never inside the repo tree: the bench must not read or write the
    // repo's own live `.bee/` store (D5 / validation decision B5).
    if let (Ok(parent), Ok(repo)) = (
        fixture.root.parent().map(Path::canonicalize).unwrap_or_else(|| Ok(PathBuf::new())),
        repo_root.canonicalize(),
    ) {
        if !parent.as_os_str().is_empty() && parent.starts_with(&repo) {
            return Err(format!(
                "refusing to run: temp fixture parent {} resolves inside the repo tree {}",
                parent.display(),
                repo.display()
            ));
        }
    }

    let req = fixture::FixtureRequest::at_floors(fixture.root.clone());
    fixture::generate(&req).map_err(|e| format!("generating the pinned status fixture: {e}"))?;

    // COLD: the review-git cache is deleted before every single run, so no
    // iteration ever benefits from a previous one's warm-up.
    let cold_samples = bench::time_queen_bee_status(bin_path, &fixture.root, runs, bench::CacheState::Cold)
        .map_err(|e| format!("timing queen-bee status --json (cold cache): {e}"))?;

    // WARM: one unmeasured warm-up populates the cache, then every timed
    // run reads it.
    bench::time_queen_bee_status(bin_path, &fixture.root, 1, bench::CacheState::Warm)
        .map_err(|e| format!("warming the review-git cache: {e}"))?;
    if !bench::review_git_cache_path(&fixture.root).exists() {
        return Err(format!(
            "the warm-up run did not produce {} — a 'warm cache' series with no cache on disk would be a cold series mislabelled",
            bench::review_git_cache_path(&fixture.root).display()
        ));
    }
    let warm_samples = bench::time_queen_bee_status(bin_path, &fixture.root, runs, bench::CacheState::Warm)
        .map_err(|e| format!("timing queen-bee status --json (warm cache): {e}"))?;

    let node_samples = bench::time_node_status(&bee_mjs, &fixture.root, runs)
        .map_err(|e| format!("timing node bee.mjs status --json baseline: {e}"))?;

    let cold = bench::stats_from_samples(cold_samples);
    let warm = bench::stats_from_samples(warm_samples);
    let node = bench::stats_from_samples(node_samples);
    let pass = warm.p95_ms < budget_ms;

    // D5's escape is "report the measured p95 PLUS a profile", never a
    // shrunk fixture — so a red gate emits the profile itself rather than
    // leaving whoever reads it to reproduce the fixture by hand.
    if !pass {
        eprintln!(
            "queen-bench --check: status gate RED — warm p95 {:.3} ms >= budget {budget_ms} ms (cold p95 {:.3} ms). Per-block profile of one warm run:",
            warm.p95_ms, cold.p95_ms
        );
        match std::process::Command::new(bin_path)
            .arg("status")
            .arg("--json")
            .arg("--profile")
            .current_dir(&fixture.root)
            .output()
        {
            Ok(out) => eprint!("{}", String::from_utf8_lossy(&out.stderr)),
            Err(err) => eprintln!("  (could not collect a profile: {err})"),
        }
    }

    Ok(StatusCheckReport { pass, budget_ms, fixture_root: fixture.root.clone(), cold, warm, node })
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
