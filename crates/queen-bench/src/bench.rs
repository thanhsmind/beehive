//! Spawn-inclusive p95 runner (CONTEXT.md D5). Times a full process
//! lifetime — spawn through exit — for a given command, over >= 50 runs,
//! and reports p50/p95/p99/max in milliseconds. Used by `--check` to gate
//! `queen-bee ping` against a budget, and to record the `node -e ""`
//! baseline alongside it in the same JSON report (D5 acceptance instrument:
//! not `timings.jsonl`, a dedicated harness).

use std::path::Path;
use std::process::Command;
use std::time::Instant;

/// D5: "≥50 runs per command" is a floor on the harness itself, not just a
/// default — a caller asking for fewer runs gets corrected up, never a
/// silently thinner measurement.
pub const MIN_RUNS: usize = 50;

#[derive(Debug, Clone, Copy)]
pub struct RunStats {
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub max_ms: f64,
    pub samples: usize,
}

impl RunStats {
    pub fn to_json_fields(&self) -> String {
        format!(
            "\"p50_ms\":{:.3},\"p95_ms\":{:.3},\"p99_ms\":{:.3},\"max_ms\":{:.3},\"samples\":{}",
            self.p50_ms, self.p95_ms, self.p99_ms, self.max_ms, self.samples
        )
    }
}

/// Nearest-rank percentile over an ascending-sorted slice.
fn percentile(sorted_ms: &[f64], p: f64) -> f64 {
    if sorted_ms.is_empty() {
        return 0.0;
    }
    let n = sorted_ms.len();
    let rank = (p / 100.0 * n as f64).ceil() as usize;
    let idx = rank.saturating_sub(1).min(n - 1);
    sorted_ms[idx]
}

pub fn stats_from_samples(mut samples_ms: Vec<f64>) -> RunStats {
    samples_ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let max_ms = samples_ms.last().copied().unwrap_or(0.0);
    RunStats {
        p50_ms: percentile(&samples_ms, 50.0),
        p95_ms: percentile(&samples_ms, 95.0),
        p99_ms: percentile(&samples_ms, 99.0),
        max_ms,
        samples: samples_ms.len(),
    }
}

/// Run `runs` spawn-inclusive timings of the command `cmd_builder` produces
/// fresh each iteration (a `Command` cannot be reused after `.output()`).
/// Every run must exit successfully — a failing subprocess mid-measurement
/// invalidates the whole series rather than silently dropping a sample.
pub fn time_spawns(runs: usize, mut cmd_builder: impl FnMut() -> Command) -> Result<Vec<f64>, String> {
    let runs = runs.max(MIN_RUNS);
    let mut samples = Vec::with_capacity(runs);
    for n in 0..runs {
        let mut cmd = cmd_builder();
        let start = Instant::now();
        let output = cmd
            .output()
            .map_err(|e| format!("spawn failed on run {n}: {e}"))?;
        let elapsed = start.elapsed();
        if !output.status.success() {
            return Err(format!(
                "run {n} exited non-zero ({:?}); stderr: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        samples.push(elapsed.as_secs_f64() * 1000.0);
    }
    Ok(samples)
}

/// Time `runs` spawn-inclusive invocations of `bin_path ping`.
pub fn time_queen_bee_ping(bin_path: &Path, runs: usize) -> Result<Vec<f64>, String> {
    time_spawns(runs, || {
        let mut c = Command::new(bin_path);
        c.arg("ping");
        c
    })
}

/// Time `runs` spawn-inclusive invocations of `node -e ""` — the D5 cold-
/// start baseline color (CONTEXT.md: "node -e \"\" ≈ 20 ms").
pub fn time_node_baseline(runs: usize) -> Result<Vec<f64>, String> {
    time_spawns(runs, || {
        let mut c = Command::new("node");
        c.arg("-e").arg("");
        c
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_matches_known_values() {
        let sorted: Vec<f64> = (1..=100).map(|n| n as f64).collect();
        assert_eq!(percentile(&sorted, 50.0), 50.0);
        assert_eq!(percentile(&sorted, 95.0), 95.0);
        assert_eq!(percentile(&sorted, 99.0), 99.0);
        assert_eq!(percentile(&sorted, 100.0), 100.0);
    }

    #[test]
    fn stats_from_samples_reports_max() {
        let stats = stats_from_samples(vec![3.0, 1.0, 2.0]);
        assert_eq!(stats.max_ms, 3.0);
        assert_eq!(stats.samples, 3);
    }
}
