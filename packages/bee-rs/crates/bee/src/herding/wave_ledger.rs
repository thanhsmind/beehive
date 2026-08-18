// herding::wave_ledger — the bee-side append-only wave ledger
// (herding-orchestration D10, docs/history/herding-orchestration/CONTEXT.md).
//
// D5 keeps the coordination core memoryless: it drives workers, waits, and
// aggregates, but writes nothing to disk. D10 puts exactly one record of a
// wave on the bee side instead — one append-only row per wave, in the shape
// `.bee/` already uses for `backlog.jsonl`, `decisions.jsonl` and
// `capture-queue.jsonl` (`crate::fsutil::append_jsonl`: compact JSON + "\n",
// opened for append, never rewritten in place). This module never imports
// from the fleet crate and the fleet crate never imports this module — the
// core records nothing, full stop.
//
// THE POINT OF THIS FILE IS OCCUPANCY, NOT THE RECORD. The cockpit's
// four-slot cap is enforced today by the control model counting herdr panes,
// and a working agent that never names its own pane leaves a slot looking
// free — a recorded Open Gap
// (docs/knowledge/areas/bee-herding/overview.md). `live_worker_count` below
// is the read side that lets `role-dispatch.md` §4 ask the ledger instead of
// counting panes.
//
// ─── Write shape ────────────────────────────────────────────────────────────
// One JSON object per line:
//   {"wave_id": "...", "started_at": "<RFC3339>",
//    "workers": [{"name", "pane_id", "worktree", "task",
//                 "outcome": <string|null>, "evidence": <string|null>}, ...]}
// `append_wave` writes exactly one such line per call — a wave never appends
// twice and never rewrites a line it already wrote (must_have: "A wave
// appends exactly one row, and the file is only ever appended to.").
// `outcome`/`evidence` are nullable so a row MAY be appended before every
// worker has reported: a caller either appends once at wave completion (every
// outcome already known — the ordinary case, since D9's wave wait is a
// blocking call) or, when it wants occupancy visible while the wave is still
// running, appends once at dispatch time with `outcome: None` per worker.
// Either way the ledger only ever sees one physical append for that wave.
//
// ─── The staleness rule (occupancy's hard half) ─────────────────────────────
// A row is CLOSED once every worker inside it carries an outcome — closed
// rows are never live, at any age (`is_closed`). A row that is NOT closed is
// LIVE only while it is younger than `DEFAULT_STALE_AFTER_MS`; past that
// bound an outcome-less worker is STALE, not live (`is_live`) — the row is
// most likely orphaned by a crashed dispatcher that never got to append its
// worker's outcome, not a worker still genuinely running. Counting it forever
// would reproduce the exact over-spawn defect D10 exists to remove, just
// moved from "a pane nobody named" to "a ledger row nobody closed".
//
// A stale row is PROVABLY IGNORED, never swept: the ledger is append-only by
// contract, so "sweeping" cannot mean deleting or rewriting its bytes.
// `live_worker_count` excludes it from the occupancy count, but `read_waves`
// still returns it — the row stays on disk, unedited, forever; only the
// read side's judgement of it changes, and that judgement is re-derived from
// `now - started_at` on every call rather than cached or written back
// anywhere.

use crate::fsutil::append_jsonl;
use crate::verbs::feedback::read_jsonl;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub(crate) fn wave_ledger_path(root: &Path) -> PathBuf {
    root.join(".bee").join("wave-ledger.jsonl")
}

/// One worker slot inside a wave row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkerRow {
    pub(crate) name: String,
    pub(crate) pane_id: String,
    pub(crate) worktree: String,
    pub(crate) task: String,
    /// `None` until the worker reports. Never rewritten once the row is on
    /// disk — see the module doc above.
    pub(crate) outcome: Option<String>,
    /// A pointer to the worker's evidence (a log path, a report path, a
    /// commit — whatever the caller's proof for this worker actually is),
    /// never the evidence itself.
    pub(crate) evidence: Option<String>,
}

/// One wave: the row this module appends exactly once per wave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WaveRow {
    pub(crate) wave_id: String,
    /// RFC3339, e.g. `chrono::Utc::now()` formatted with `to_rfc3339()`.
    pub(crate) started_at: String,
    pub(crate) workers: Vec<WorkerRow>,
}

fn worker_row_to_json(w: &WorkerRow) -> Value {
    json!({
        "name": w.name,
        "pane_id": w.pane_id,
        "worktree": w.worktree,
        "task": w.task,
        "outcome": w.outcome,
        "evidence": w.evidence,
    })
}

fn wave_row_to_json(row: &WaveRow) -> Value {
    json!({
        "wave_id": row.wave_id,
        "started_at": row.started_at,
        "workers": row.workers.iter().map(worker_row_to_json).collect::<Vec<_>>(),
    })
}

/// Append exactly one wave row. Creates `.bee/wave-ledger.jsonl` (and its
/// parents) on first use, exactly like every other `.bee/*.jsonl` ledger
/// (`crate::fsutil::append_jsonl`). Never call this twice for the same
/// `wave_id` — the ledger has no update path, by contract (D10).
pub(crate) fn append_wave(root: &Path, row: &WaveRow) -> std::io::Result<()> {
    append_jsonl(&wave_ledger_path(root), &wave_row_to_json(row))
}

fn worker_row_from_json(v: &Value) -> Option<WorkerRow> {
    let obj = v.as_object()?;
    Some(WorkerRow {
        name: obj.get("name")?.as_str()?.to_string(),
        pane_id: obj.get("pane_id")?.as_str()?.to_string(),
        worktree: obj.get("worktree")?.as_str()?.to_string(),
        task: obj.get("task")?.as_str()?.to_string(),
        outcome: obj.get("outcome").and_then(Value::as_str).map(str::to_string),
        evidence: obj.get("evidence").and_then(Value::as_str).map(str::to_string),
    })
}

fn wave_row_from_json(v: &Value) -> Option<WaveRow> {
    let obj = v.as_object()?;
    let wave_id = obj.get("wave_id")?.as_str()?.to_string();
    let started_at = obj.get("started_at")?.as_str()?.to_string();
    let workers = obj
        .get("workers")?
        .as_array()?
        .iter()
        .filter_map(worker_row_from_json)
        .collect();
    Some(WaveRow { wave_id, started_at, workers })
}

/// Every wave row currently in the ledger. A corrupt LINE is skipped, never a
/// reason to refuse the rest of the file — the same fail-open shape every
/// other `.bee/*.jsonl` reader in this crate uses
/// (`crate::verbs::feedback::read_jsonl`, reused here rather than
/// reimplemented). A row whose JSON parses but is missing a required field
/// (no `wave_id`, no `started_at`, an unshaped worker) is skipped the same
/// way — it is exactly as unusable as a line that failed to parse at all.
pub(crate) fn read_waves(root: &Path) -> Vec<WaveRow> {
    read_jsonl(&wave_ledger_path(root))
        .rows
        .iter()
        .filter_map(wave_row_from_json)
        .collect()
}

/// The staleness bound: how long a worker with no outcome yet is still
/// trusted as live, measured from its wave's `started_at`. One hour errs
/// toward false "live" over false "stale" — undercounting occupancy would
/// let a new wave over-spawn on top of workers that are, in fact, still
/// running, which is the exact failure D10 removes; overcounting only ever
/// costs a wave slot sitting idle a little longer than it has to.
pub(crate) const DEFAULT_STALE_AFTER_MS: i64 = 60 * 60 * 1000;

fn parse_rfc3339_ms(s: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.timestamp_millis())
}

/// A wave whose workers have ALL reported an outcome is closed — closed rows
/// are never live, regardless of age. An empty worker list is vacuously
/// closed (there is nothing left unresolved to go stale).
fn is_closed(row: &WaveRow) -> bool {
    row.workers.iter().all(|w| w.outcome.is_some())
}

/// A row that is not closed is live only while it is younger than
/// `stale_after_ms`. A `started_at` this module cannot parse fails CLOSED —
/// never live — the same fail-closed posture the choreography's own status
/// model uses for `unverifiable` (CONTEXT.md's Ordering Invariants).
fn is_live(row: &WaveRow, now_ms: i64, stale_after_ms: i64) -> bool {
    if is_closed(row) {
        return false;
    }
    match parse_rfc3339_ms(&row.started_at) {
        Some(started_ms) => now_ms.saturating_sub(started_ms) <= stale_after_ms,
        None => false,
    }
}

/// The read side D10 exists for: how many worker slots does the ledger
/// believe are occupied right now? Sums the outcome-less workers of every
/// row that `is_live` still counts as live; a closed row contributes 0
/// regardless of how many workers it lists, and a stale row's still-pending
/// workers are excluded too (provably ignored — see the module doc's
/// staleness section; the row itself is untouched on disk).
pub(crate) fn live_worker_count(root: &Path, now_ms: i64, stale_after_ms: i64) -> usize {
    read_waves(root)
        .iter()
        .filter(|row| is_live(row, now_ms, stale_after_ms))
        .map(|row| row.workers.iter().filter(|w| w.outcome.is_none()).count())
        .sum()
}

/// `live_worker_count` at the current wall-clock time, under
/// `DEFAULT_STALE_AFTER_MS` — the call role-dispatch.md's §4 anomaly check
/// makes once it reads this ledger instead of counting panes.
#[allow(dead_code)]
pub(crate) fn live_worker_count_now(root: &Path) -> usize {
    live_worker_count(root, chrono::Utc::now().timestamp_millis(), DEFAULT_STALE_AFTER_MS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_root() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn worker(name: &str, outcome: Option<&str>) -> WorkerRow {
        WorkerRow {
            name: name.to_string(),
            pane_id: format!("pane-{name}"),
            worktree: format!("/tmp/wt-{name}"),
            task: "do the thing".to_string(),
            outcome: outcome.map(str::to_string),
            evidence: outcome.map(|_| format!(".bee/logs/wave/{name}.log")),
        }
    }

    fn iso(ms_ago: i64) -> String {
        let now = chrono::Utc::now();
        (now - chrono::Duration::milliseconds(ms_ago)).to_rfc3339()
    }

    #[test]
    fn one_wave_leaves_exactly_one_row() {
        let tmp = tmp_root();
        let root = tmp.path();
        let row = WaveRow {
            wave_id: "w-1".to_string(),
            started_at: iso(0),
            workers: vec![worker("a", Some("success")), worker("b", Some("failure"))],
        };
        append_wave(root, &row).unwrap();

        let raw = std::fs::read_to_string(wave_ledger_path(root)).unwrap();
        let lines: Vec<&str> = raw.lines().collect();
        assert_eq!(lines.len(), 1, "one append must leave exactly one line");

        let waves = read_waves(root);
        assert_eq!(waves.len(), 1);
        assert_eq!(waves[0].wave_id, "w-1");
    }

    #[test]
    fn a_second_wave_appends_without_disturbing_the_first_rows_bytes() {
        let tmp = tmp_root();
        let root = tmp.path();
        let row1 = WaveRow { wave_id: "w-1".to_string(), started_at: iso(0), workers: vec![worker("a", Some("success"))] };
        append_wave(root, &row1).unwrap();
        let raw_after_first = std::fs::read_to_string(wave_ledger_path(root)).unwrap();

        let row2 = WaveRow { wave_id: "w-2".to_string(), started_at: iso(0), workers: vec![worker("b", Some("success"))] };
        append_wave(root, &row2).unwrap();
        let raw_after_second = std::fs::read_to_string(wave_ledger_path(root)).unwrap();

        assert!(
            raw_after_second.starts_with(&raw_after_first),
            "the first row's exact bytes must survive a later append untouched"
        );
        let lines: Vec<&str> = raw_after_second.lines().collect();
        assert_eq!(lines.len(), 2);
        let waves = read_waves(root);
        assert_eq!(waves.iter().map(|w| w.wave_id.as_str()).collect::<Vec<_>>(), vec!["w-1", "w-2"]);
    }

    #[test]
    fn a_rows_worker_outcomes_round_trip() {
        let tmp = tmp_root();
        let root = tmp.path();
        let row = WaveRow {
            wave_id: "w-rt".to_string(),
            started_at: iso(0),
            workers: vec![
                worker("done", Some("success")),
                worker("failed", Some("failure")),
                worker("pending", None),
            ],
        };
        append_wave(root, &row).unwrap();

        let waves = read_waves(root);
        assert_eq!(waves.len(), 1);
        assert_eq!(waves[0].workers, row.workers, "every field, including a None outcome/evidence, must round-trip exactly");
    }

    #[test]
    fn occupancy_counts_what_is_live_not_what_is_merely_recorded() {
        let tmp = tmp_root();
        let root = tmp.path();
        // A fully closed wave: two workers, both resolved. Recorded, not live.
        append_wave(
            root,
            &WaveRow {
                wave_id: "closed".to_string(),
                started_at: iso(0),
                workers: vec![worker("a", Some("success")), worker("b", Some("success"))],
            },
        )
        .unwrap();
        // A fresh wave with one worker still pending. Live.
        append_wave(
            root,
            &WaveRow {
                wave_id: "fresh".to_string(),
                started_at: iso(0),
                workers: vec![worker("c", Some("success")), worker("d", None)],
            },
        )
        .unwrap();

        let now = chrono::Utc::now().timestamp_millis();
        let live = live_worker_count(root, now, DEFAULT_STALE_AFTER_MS);
        assert_eq!(live, 1, "only worker d (fresh, no outcome) is live; the closed wave's 2 recorded workers must not count");
    }

    #[test]
    fn a_stale_row_is_provably_ignored_not_swept() {
        let tmp = tmp_root();
        let root = tmp.path();
        let two_hours_ago = iso(2 * 60 * 60 * 1000);
        append_wave(
            root,
            &WaveRow {
                wave_id: "orphaned".to_string(),
                started_at: two_hours_ago,
                workers: vec![worker("e", None)],
            },
        )
        .unwrap();

        let now = chrono::Utc::now().timestamp_millis();
        let live = live_worker_count(root, now, DEFAULT_STALE_AFTER_MS);
        assert_eq!(live, 0, "a worker unresolved past the staleness bound must not be counted as live");

        // Ignored, not swept: the row is still readable, byte-for-byte on disk.
        let waves = read_waves(root);
        assert_eq!(waves.len(), 1, "a stale row is never deleted or rewritten — the ledger stays append-only");
        assert_eq!(waves[0].wave_id, "orphaned");
        assert!(waves[0].workers[0].outcome.is_none());
    }

    #[test]
    fn a_row_just_inside_the_bound_still_counts_live() {
        let tmp = tmp_root();
        let root = tmp.path();
        append_wave(
            root,
            &WaveRow {
                wave_id: "borderline".to_string(),
                started_at: iso(DEFAULT_STALE_AFTER_MS - 1000),
                workers: vec![worker("f", None)],
            },
        )
        .unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        assert_eq!(live_worker_count(root, now, DEFAULT_STALE_AFTER_MS), 1);
    }

    #[test]
    fn an_unparseable_started_at_fails_closed_never_live() {
        let tmp = tmp_root();
        let root = tmp.path();
        append_wave(
            root,
            &WaveRow {
                wave_id: "bad-ts".to_string(),
                started_at: "not-a-timestamp".to_string(),
                workers: vec![worker("g", None)],
            },
        )
        .unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        assert_eq!(live_worker_count(root, now, DEFAULT_STALE_AFTER_MS), 0);
    }

    #[test]
    fn an_absent_ledger_reads_as_empty_and_zero_occupancy() {
        let tmp = tmp_root();
        let root = tmp.path();
        assert!(read_waves(root).is_empty());
        assert_eq!(live_worker_count_now(root), 0);
    }

    #[test]
    fn a_corrupt_line_is_skipped_and_the_rest_of_the_file_still_reads() {
        let tmp = tmp_root();
        let root = tmp.path();
        let good = WaveRow { wave_id: "ok".to_string(), started_at: iso(0), workers: vec![worker("h", Some("success"))] };
        append_wave(root, &good).unwrap();
        // Hand-append a corrupt line, exactly the way a torn write would.
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new().append(true).open(wave_ledger_path(root)).unwrap();
        writeln!(f, "{{not json").unwrap();

        let waves = read_waves(root);
        assert_eq!(waves.len(), 1, "the corrupt line must be skipped, not fail the whole read");
        assert_eq!(waves[0].wave_id, "ok");
    }
}
