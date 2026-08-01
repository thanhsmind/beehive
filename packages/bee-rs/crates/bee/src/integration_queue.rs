// integration_queue — native port of packages/bee/lib/integration-queue.mjs
// (multisession-native-22: the durable merge queue + the processor lease that
// `bee worktree merge` drains through).
//
// LIBRARY module (no `try_native`, no probe line in verbs/mod.rs) — the same
// shape lease_store.rs and verbs/workflow_store.rs have. Its ONE consumer is
// verbs/worktree.rs's `worktree merge` (advisor condition A: no other CLI verb
// is queue-aware), which is why this file exists at all.
//
// Provenance, function by function (integration-queue.mjs → this file):
//   integrationQueueDir          → queue_dir
//   queueRecordPath              → queue_record_path
//   listQueueRecords             → list_queue_records
//   nextQueueSeq                 → next_queue_seq
//   enqueueMergeRequest          → enqueue_merge_request
//   writeQueueStatus             → write_queue_status
//   queueStatusFor / queuePosition → queue_status_for / queue_position
//   frontOfLine                  → front_of_line
//   readProcessorLeaseRaw        → read_processor_lease_raw
//   tryBecomeProcessor           → try_become_processor
//   renewProcessorLease          → renew_processor_lease
//   releaseProcessorLease        → release_processor_lease
//   checkProcessorLeaseEpoch     → check_processor_lease_epoch
//   runThroughQueue              → run_through_queue
//   processAsOwner               → process_as_owner
//
// Locking: identical lock-name strings to Node — `integration-queue` on
// `controlRoot` for every enqueue/status write (never held across a merge),
// and lease_store.rs's own `lease:<sha256(file)>` for the processor record. So
// the two runtimes serialize against each other mid-campaign (contract C1).
//
// DELEGATION POSTURE. `run_through_queue` is reached only AFTER
// verbs/worktree.rs has taken the `integration-queue` lock, so nothing in here
// may return "delegate" for an ordinary shape — campaign rule 2 (a delegation
// after a lock acquire doubles lock.rs's contention.jsonl row, which is real
// `.bee/` state, not just telemetry). Everything that COULD have delegated is
// therefore pre-checked by the caller before the first acquire:
//   * a corrupt queue record — `readJson(file, null)` warns with a V8 message
//     in Node, so `preflight_queue_readable` classifies the whole directory up
//     front (verbs/worktree.rs calls it before any lock).
//   * a corrupt/exotic processor-lease record — lease-store's `readLeaseSafe`
//     never warns (it returns null), so a corrupt record is genuinely silent
//     in both runtimes and needs no probe.
// The one accepted residual, identical in kind to verbs/workspace_store.rs's:
// an fs WRITE that fails inside the hold still delegates late. Every step
// taken by then is idempotent (a failed enqueue wrote no record, so the Node
// re-run picks the same seq back up), so the re-run reproduces the same store
// — the cost is one extra contention.jsonl "acquired" row, the same cost that
// module already documents.

#![allow(dead_code)]

use crate::fsutil::{ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::lease_store::{self, LeaseErr, LR};
use crate::lock;
use crate::verbs::reservations::{jget, js_numberify, js_strict_eq, now_iso, now_ms, truthy};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

const QUEUE_LOCK_NAME: &str = "integration-queue";
const PROCESSOR_RESOURCE_ID: &str = "integration-processor";
/// descriptive only — see the .mjs module header, C4.
const PROCESSOR_WORKFLOW_ID: &str = "integration-queue";
const SEQ_WIDTH: usize = 6;

pub(crate) const DEFAULT_PROCESSOR_TTL_SECONDS: f64 = 120.0;
pub(crate) const DEFAULT_RENEW_INTERVAL_MS: f64 = 30_000.0;
/// "a few minutes" (advisor condition B).
pub(crate) const DEFAULT_WAIT_BOUND_MS: f64 = 180_000.0;
pub(crate) const DEFAULT_POLL_INTERVAL_MS: f64 = 500.0;

/// The failure channel this module hands back to verbs/worktree.rs. `Ex` is
/// the "Node would print V8/libuv bytes here" case; see the module header for
/// why it can only ever arise from a write failure once the caller has locked.
#[derive(Debug)]
pub(crate) enum QErr {
    /// lock.mjs LockBusyError — deterministic bytes, reproduced natively.
    LockBusy(String),
    /// A typed IntegrationQueueError / LeaseStoreError message.
    Msg(String),
    /// Delegate.
    Ex,
}

impl From<LeaseErr> for QErr {
    fn from(e: LeaseErr) -> Self {
        match e {
            LeaseErr::Refused(r) => QErr::Msg(r.message),
            LeaseErr::LockBusy(b) => QErr::LockBusy(b.message()),
            LeaseErr::Exotic => QErr::Ex,
        }
    }
}

pub(crate) type QR<T> = Result<T, QErr>;

// ─── queue directory / records ─────────────────────────────────────────────

pub(crate) fn queue_dir(control_root: &Path) -> PathBuf {
    control_root
        .join(".bee")
        .join("runtime")
        .join("integration")
        .join("queue")
}

fn queue_record_path(control_root: &Path, seq: f64) -> PathBuf {
    // `String(seq).padStart(6, '0')` — seq is always a small positive integer
    // here (nextQueueSeq derives it from parsed filenames).
    let n = seq as i64;
    queue_dir(control_root).join(format!("{:0width$}.json", n, width = SEQ_WIDTH))
}

/// A queue record plus the `seq` spread in over whatever the file carried.
#[derive(Clone, Debug)]
pub(crate) struct QueueRecord {
    pub seq: f64,
    pub value: Map<String, Value>,
}

impl QueueRecord {
    fn status(&self) -> Option<&str> {
        match self.value.get("status") {
            Some(Value::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }
}

/// `Number.parseInt(name.slice(0, -'.json'.length), 10)` — leading digits win,
/// trailing junk is ignored, a non-numeric prefix is NaN (dropped by the
/// `Number.isFinite` guard).
fn js_parse_int(s: &str) -> Option<f64> {
    let t = s.trim_start_matches([' ', '\t', '\n', '\r', '\u{0b}', '\u{0c}', '\u{a0}']);
    let (neg, t) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let digits: String = t.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    let v: f64 = digits.parse().ok()?;
    Some(if neg { -v } else { v })
}

/// integration-queue.mjs listQueueRecords. A record file that does not parse
/// makes `readJson` warn with a V8 message in Node — `preflight_queue_readable`
/// (below) is what keeps that shape from ever reaching here.
fn list_queue_records(control_root: &Path) -> QR<Vec<QueueRecord>> {
    let Ok(entries) = std::fs::read_dir(queue_dir(control_root)) else {
        return Ok(Vec::new()); // directory not created yet
    };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        match entry.file_name().to_str() {
            Some(n) => names.push(n.to_string()),
            None => return Err(QErr::Ex), // a non-UTF-8 name Node would still read
        }
    }
    let mut records: Vec<QueueRecord> = Vec::new();
    for name in names {
        if !name.ends_with(".json") {
            continue;
        }
        let Some(seq) = js_parse_int(&name[..name.len() - ".json".len()]) else {
            continue; // Number.isFinite(NaN) === false
        };
        if !seq.is_finite() {
            continue;
        }
        let parsed = match read_json(&queue_dir(control_root).join(&name)) {
            ReadJson::Missing => continue,              // readJson -> null -> skipped
            ReadJson::Corrupt => return Err(QErr::Ex),  // V8-worded warn in Node
            ReadJson::Parsed(v) => js_numberify(&v).map_err(|_| QErr::Ex)?,
        };
        // `if (record)` — a falsy parse (null/false/0/"") is skipped.
        if !truthy(&parsed) {
            continue;
        }
        // `{ ...record, seq }` — a non-object spreads to nothing but `seq`.
        let mut value = match parsed {
            Value::Object(m) => m,
            _ => Map::new(),
        };
        value.insert("seq".into(), json!(seq));
        records.push(QueueRecord { seq, value });
    }
    // `records.sort((a, b) => a.seq - b.seq)` — a numeric comparator; the
    // seqs are all finite here, so a total order exists.
    records.sort_by(|a, b| a.seq.partial_cmp(&b.seq).unwrap_or(std::cmp::Ordering::Equal));
    Ok(records)
}

/// The caller-side probe (see the module header): classify every queue record
/// BEFORE the first lock, so a corrupt one delegates with zero bytes emitted
/// and zero locks taken.
pub(crate) fn preflight_queue_readable(control_root: &Path) -> bool {
    list_queue_records(control_root).is_ok()
}

/// nextQueueSeq — highest existing seq + 1 (1 when empty). Only ever called
/// from inside the `integration-queue` hold.
fn next_queue_seq(control_root: &Path) -> QR<f64> {
    let records = list_queue_records(control_root)?;
    Ok(match records.last() {
        None => 1.0,
        Some(last) => last.seq + 1.0,
    })
}

/// enqueueMergeRequest — one durable record, appended under
/// `withStoreLock(controlRoot, 'integration-queue')`.
pub(crate) fn enqueue_merge_request(
    control_root: &Path,
    worktree_id: &str,
    requested_by_session: &str,
    now: &str,
) -> QR<QueueRecord> {
    let id = worktree_id.trim();
    if id.is_empty() {
        return Err(QErr::Msg(
            "enqueueMergeRequest: worktreeId is required.".to_string(),
        ));
    }
    let session = requested_by_session.trim();
    if session.is_empty() {
        return Err(QErr::Msg(
            "enqueueMergeRequest: requestedBySession is required.".to_string(),
        ));
    }
    let mut guard = lock::acquire_store_lock(control_root, QUEUE_LOCK_NAME, lock::MAX_ATTEMPTS)
        .map_err(|b| QErr::LockBusy(b.message()))?;
    let out = (|| -> QR<QueueRecord> {
        let seq = next_queue_seq(control_root)?;
        let mut record = Map::new();
        record.insert("seq".into(), json!(seq));
        record.insert("worktree_id".into(), Value::String(id.to_string()));
        record.insert("feature".into(), Value::Null); // bee.mjs always passes null
        record.insert(
            "requested_by_session".into(),
            Value::String(session.to_string()),
        );
        record.insert("requested_at".into(), Value::String(now.to_string()));
        record.insert("status".into(), json!("queued"));
        ensure_dir(&queue_dir(control_root)).map_err(|_| QErr::Ex)?;
        write_json_atomic(
            &queue_record_path(control_root, seq),
            &Value::Object(record.clone()),
        )
        .map_err(|_| QErr::Ex)?;
        Ok(QueueRecord { seq, value: record })
    })();
    guard.release();
    out
}

/// writeQueueStatus — `{...current, status, ...extra}` under the queue lock.
/// A pruned record is a no-op (Node returns null).
fn write_queue_status(
    control_root: &Path,
    seq: f64,
    status: &str,
    extra: &[(&str, Value)],
) -> QR<()> {
    let mut guard = lock::acquire_store_lock(control_root, QUEUE_LOCK_NAME, lock::MAX_ATTEMPTS)
        .map_err(|b| QErr::LockBusy(b.message()))?;
    let out = (|| -> QR<()> {
        let file = queue_record_path(control_root, seq);
        let current = match read_json(&file) {
            ReadJson::Missing => return Ok(()), // record pruned/gone
            ReadJson::Corrupt => return Err(QErr::Ex),
            ReadJson::Parsed(v) => js_numberify(&v).map_err(|_| QErr::Ex)?,
        };
        let mut next = match current {
            Value::Object(m) => m,
            Value::Null | Value::Bool(false) => return Ok(()), // `if (!current) return null`
            _ => Map::new(),
        };
        // JS spread: an existing key keeps its ORIGINAL position.
        next.insert("status".into(), Value::String(status.to_string()));
        for (k, v) in extra {
            next.insert((*k).to_string(), v.clone());
        }
        write_json_atomic(&file, &Value::Object(next)).map_err(|_| QErr::Ex)?;
        Ok(())
    })();
    guard.release();
    out
}

/// queueStatusFor(controlRoot, seq)?.status ?? 'queued'.
fn queue_status_for(control_root: &Path, seq: f64) -> QR<Option<String>> {
    match read_json(&queue_record_path(control_root, seq)) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => Err(QErr::Ex),
        ReadJson::Parsed(v) => Ok(match jget(&v, "status") {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        }),
    }
}

/// queuePosition — 1-based position among every still-OPEN record.
struct Position {
    position: usize,
    ahead: usize,
    total_open: usize,
}

fn queue_position(control_root: &Path, seq: f64) -> QR<Option<Position>> {
    let open: Vec<QueueRecord> = list_queue_records(control_root)?
        .into_iter()
        .filter(|r| matches!(r.status(), Some("queued") | Some("processing")))
        .collect();
    let Some(idx) = open.iter().position(|r| r.seq == seq) else {
        return Ok(None);
    };
    Ok(Some(Position {
        position: idx + 1,
        ahead: idx,
        total_open: open.len(),
    }))
}

/// frontOfLine — the oldest still-'queued' record.
fn front_of_line(control_root: &Path) -> QR<Option<QueueRecord>> {
    Ok(list_queue_records(control_root)?
        .into_iter()
        .find(|r| r.status() == Some("queued")))
}

// ─── processor lease (advisor condition C) ─────────────────────────────────

fn processor_resource_key() -> String {
    format!("path:{PROCESSOR_RESOURCE_ID}")
}

fn read_processor_lease_raw(control_root: &Path) -> Option<Value> {
    let (leases, _skipped) = lease_store::list_leases(control_root);
    let key = processor_resource_key();
    leases
        .into_iter()
        .find(|l| matches!(jget(l, "resource"), Some(Value::String(s)) if *s == key))
}

fn processor_request() -> Value {
    json!({ "type": "path", "id": PROCESSOR_RESOURCE_ID })
}

/// The `{ok:true, lease}` / `{ok:false, holder}` answer tryBecomeProcessor
/// gives. `tookOver` is unused by every current caller (runThroughQueue drops
/// it), so it is not modelled.
enum Processor {
    Acquired(Value),
    Held,
}

/// tryBecomeProcessor. `ttlSeconds` is always the module default here (no
/// caller overrides it), so the QUEUE_INVALID_TTL refusal is structurally
/// unreachable and is not modelled — the .mjs guard exists for library callers
/// this port has none of.
fn try_become_processor(
    control_root: &Path,
    session_id: &str,
    workspace_id: &str,
    ttl_seconds: f64,
    now: f64,
) -> QR<Processor> {
    if session_id.trim().is_empty() {
        return Err(QErr::Msg(
            "tryBecomeProcessor: sessionId is required.".to_string(),
        ));
    }
    let before = read_processor_lease_raw(control_root);
    lease_store::sweep_expired_leases(control_root, now);
    // `(before && Number.isFinite(before.epoch) ? before.epoch : 0) + 1`
    let prev_epoch = before
        .as_ref()
        .and_then(|b| jget(b, "epoch"))
        .and_then(|v| match v {
            Value::Number(n) => n.as_f64().filter(|f| f.is_finite()),
            _ => None,
        })
        .unwrap_or(0.0);
    let next_epoch = prev_epoch + 1.0;
    let request = json!({
        "type": "path",
        "id": PROCESSOR_RESOURCE_ID,
        "mode": "processor",
        "workflow_id": PROCESSOR_WORKFLOW_ID,
        "session_id": session_id.trim(),
        "workspace_id": workspace_id,
        "epoch": next_epoch,
        "ttl": ttl_seconds,
    });
    match lease_store::acquire_leases(control_root, &[request], now) {
        Ok(mut leases) if !leases.is_empty() => Ok(Processor::Acquired(leases.remove(0))),
        Ok(_) => Err(QErr::Ex), // structurally impossible (one request in, one out)
        Err(LeaseErr::Refused(r)) if r.code == "LEASE_HELD" => Ok(Processor::Held),
        Err(e) => Err(e.into()),
    }
}

/// renewProcessorLease — best-effort heartbeat, fenced on the acquired epoch.
fn renew_processor_lease(control_root: &Path, ttl_seconds: f64, presented_epoch: &Value) -> LR<Value> {
    lease_store::renew_lease(
        control_root,
        &processor_request(),
        ttl_seconds,
        now_ms(),
        Some(presented_epoch),
        lock::MAX_ATTEMPTS,
    )
}

/// releaseProcessorLease — fenced release, so a zombie cannot delete a
/// takeover's fresh lease. Always `.catch(() => {})` at the call site.
fn release_processor_lease(control_root: &Path, presented_epoch: &Value) {
    let _ = lease_store::release_lease(
        control_root,
        &processor_request(),
        Some(presented_epoch),
        lock::MAX_ATTEMPTS,
    );
}

/// checkProcessorLeaseEpoch — P3's primary fence. `String | null`, exactly the
/// contract worktree-store.mjs's checkMergeFence shares so the two compose.
pub(crate) fn check_processor_lease_epoch(control_root: &Path, expected_epoch: &Value) -> Option<String> {
    let expected = crate::jsjson::js_to_string(expected_epoch);
    let Some(current) = read_processor_lease_raw(control_root) else {
        return Some(format!(
            "the integration-queue processor lease disappeared (this processor acquired epoch {expected}) while its verify was running — released or swept out from under it"
        ));
    };
    let epoch_now = jget(&current, "epoch").cloned().unwrap_or(Value::Null);
    if js_strict_eq(&epoch_now, expected_epoch) {
        return None;
    }
    let session = jget(&current, "session_id")
        .map(crate::jsjson::js_to_string)
        .unwrap_or_else(|| "undefined".to_string());
    Some(format!(
        "the integration-queue processor lease was taken over while this processor's verify was running (epoch {expected} -> {}, now held by session \"{session}\") — this processor is a zombie",
        crate::jsjson::js_to_string(&epoch_now)
    ))
}

// ─── the drainer (advisor condition B) ─────────────────────────────────────

/// The hooks `runMerge` receives. `checkProcessorLease` is P3's first fence;
/// `on_verify_tick` is the renewal heartbeat P2 fires while the verify child
/// runs; `verify_tick_interval_ms` is how often.
pub(crate) struct Hooks<'a> {
    pub control_root: &'a Path,
    pub lease_epoch: Value,
    pub ttl_seconds: f64,
    pub verify_tick_interval_ms: f64,
}

impl Hooks<'_> {
    pub(crate) fn check_processor_lease(&self) -> Option<String> {
        check_processor_lease_epoch(self.control_root, &self.lease_epoch)
    }
    /// processAsOwner's onVerifyTick: a best-effort renew whose every failure
    /// is swallowed (the .mjs's own `catch {}` — a missed renewal is a
    /// liveness concern, never a correctness one).
    pub(crate) fn on_verify_tick(&self) {
        let _ = renew_processor_lease(self.control_root, self.ttl_seconds, &self.lease_epoch);
    }
}

/// The queue's own answer when the wait bound elapsed before this request
/// reached the front — advisor condition B's "NEVER reads as success".
pub(crate) struct QueueTimeout {
    pub result: Value,
    pub message: String,
}

pub(crate) enum Drain<T> {
    Ran(T),
    TimedOut(Box<QueueTimeout>),
}

/// runThroughQueue — the ONE entrypoint `bee worktree merge` calls.
///
/// `run_merge` performs the real merge and returns `(result, ok)`; `ok` drives
/// the terminal queue status ('done' vs 'failed'), matching the .mjs's
/// `result && result.ok === false ? 'failed' : 'done'`. A `run_merge` that
/// fails outright (Err) marks the record 'failed' with the thrown message in
/// `error`, exactly as processAsOwner's catch does, and re-raises.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_through_queue<T>(
    control_root: &Path,
    worktree_id: &str,
    session_id: &str,
    workspace_id: &str,
    wait_bound_ms: f64,
    poll_interval_ms: f64,
    ttl_seconds: f64,
    renew_interval_ms: f64,
    run_merge: impl FnOnce(&Hooks<'_>) -> Result<(T, bool), String>,
) -> QR<Drain<T>> {
    let record = enqueue_merge_request(control_root, worktree_id, session_id, &now_iso())?;

    let deadline = now_ms() + wait_bound_ms;
    let mut run_merge = Some(run_merge);
    loop {
        let front = front_of_line(control_root)?;
        if front.map(|f| f.seq) == Some(record.seq) {
            match try_become_processor(control_root, session_id, workspace_id, ttl_seconds, now_ms())?
            {
                Processor::Acquired(lease) => {
                    let f = run_merge.take().expect("run_merge is consumed exactly once");
                    return process_as_owner(
                        control_root,
                        &record,
                        &lease,
                        ttl_seconds,
                        renew_interval_ms,
                        f,
                    )
                    .map(Drain::Ran);
                }
                // Lease still held by a live processor from a PRIOR era — keep
                // waiting; a lost race alone never falls through to the
                // timeout check, only a genuinely elapsed waitBoundMs does.
                Processor::Held => {}
            }
        }
        if now_ms() >= deadline {
            let position = queue_position(control_root, record.seq)?;
            let status = queue_status_for(control_root, record.seq)?
                .unwrap_or_else(|| "queued".to_string());
            let mut queue = Map::new();
            queue.insert("seq".into(), json!(record.seq));
            queue.insert("status".into(), Value::String(status.clone()));
            if let Some(p) = &position {
                queue.insert("position".into(), json!(p.position as f64));
                queue.insert("ahead".into(), json!(p.ahead as f64));
                queue.insert("total_open".into(), json!(p.total_open as f64));
            }
            let pos_disp = position
                .as_ref()
                .map(|p| p.position.to_string())
                .unwrap_or_else(|| "?".to_string());
            let ahead_disp = position
                .as_ref()
                .map(|p| p.ahead.to_string())
                .unwrap_or_else(|| "?".to_string());
            let message = format!(
                "the integration queue's wait bound ({}ms) elapsed before this merge request reached the front of the line — the merge did NOT run and nothing was committed. This request is still {status} at position {pos_disp} ({ahead_disp} ahead of it) — retry \"bee worktree merge\" to wait again, or check \"bee worktree list\" once the queue drains.",
                crate::jsjson::js_f64_to_string(wait_bound_ms)
            );
            let mut result = Map::new();
            result.insert("ok".into(), Value::Bool(false));
            result.insert("code".into(), json!("INTEGRATION_QUEUE_TIMEOUT"));
            result.insert("merged".into(), Value::Bool(false));
            result.insert("queue".into(), Value::Object(queue));
            result.insert("message".into(), Value::String(message.clone()));
            return Ok(Drain::TimedOut(Box::new(QueueTimeout {
                result: Value::Object(result),
                message,
            })));
        }
        std::thread::sleep(std::time::Duration::from_millis(poll_interval_ms.max(0.0) as u64));
    }
}

/// processAsOwner — mark 'processing', run the merge with the lease hooks
/// live, then ALWAYS write a terminal status and release the lease (the .mjs's
/// `finally`), so a failing merge never leaves the queue or the lease stuck.
fn process_as_owner<T>(
    control_root: &Path,
    record: &QueueRecord,
    lease: &Value,
    ttl_seconds: f64,
    renew_interval_ms: f64,
    run_merge: impl FnOnce(&Hooks<'_>) -> Result<(T, bool), String>,
) -> QR<T> {
    write_queue_status(
        control_root,
        record.seq,
        "processing",
        &[("processing_started_at", Value::String(now_iso()))],
    )?;
    let lease_epoch = jget(lease, "epoch").cloned().unwrap_or(Value::Null);
    let hooks = Hooks {
        control_root,
        lease_epoch: lease_epoch.clone(),
        ttl_seconds,
        verify_tick_interval_ms: renew_interval_ms,
    };
    let outcome = run_merge(&hooks);
    let terminal = match &outcome {
        Ok((_, ok)) => write_queue_status(
            control_root,
            record.seq,
            if *ok { "done" } else { "failed" },
            &[("finished_at", Value::String(now_iso()))],
        ),
        Err(message) => write_queue_status(
            control_root,
            record.seq,
            "failed",
            &[
                ("finished_at", Value::String(now_iso())),
                ("error", Value::String(message.clone())),
            ],
        ),
    };
    // `finally { await releaseProcessorLease(...).catch(() => {}) }`
    release_processor_lease(control_root, &lease_epoch);
    terminal?;
    match outcome {
        Ok((value, _)) => Ok(value),
        // The .mjs rethrows; bee.mjs's dispatcher then emitErrors `.message`.
        Err(message) => Err(QErr::Msg(message)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        tmp
    }

    /// The record bytes Node writes: writeJsonAtomic's 2-space JSON + newline,
    /// key order exactly enqueueMergeRequest's object literal.
    #[test]
    fn enqueue_writes_nodes_record_bytes_and_order() {
        let tmp = fixture();
        let rec = enqueue_merge_request(tmp.path(), " wt-a ", " sess-1 ", "2026-01-01T00:00:00.000Z")
            .unwrap();
        assert_eq!(rec.seq, 1.0);
        let text = std::fs::read_to_string(queue_record_path(tmp.path(), 1.0)).unwrap();
        assert_eq!(
            text,
            "{\n  \"seq\": 1,\n  \"worktree_id\": \"wt-a\",\n  \"feature\": null,\n  \"requested_by_session\": \"sess-1\",\n  \"requested_at\": \"2026-01-01T00:00:00.000Z\",\n  \"status\": \"queued\"\n}\n"
        );
        // The filename is zero-padded to six.
        assert!(queue_dir(tmp.path()).join("000001.json").exists());
    }

    /// nextQueueSeq is "highest + 1", and listQueueRecords sorts numerically
    /// (never lexically — 000010 must follow 000009).
    #[test]
    fn seq_allocation_is_numeric_not_lexical() {
        let tmp = fixture();
        for _ in 0..11 {
            enqueue_merge_request(tmp.path(), "wt-a", "s", &now_iso()).unwrap();
        }
        let records = list_queue_records(tmp.path()).unwrap();
        assert_eq!(records.len(), 11);
        assert_eq!(records.last().unwrap().seq, 11.0);
        assert!(queue_dir(tmp.path()).join("000011.json").exists());
    }

    /// writeQueueStatus keeps `status`'s ORIGINAL key position (JS spread) and
    /// appends the extras — the byte property the .bee diff pins.
    #[test]
    fn status_write_preserves_key_position() {
        let tmp = fixture();
        let rec = enqueue_merge_request(tmp.path(), "wt-a", "s", "T0").unwrap();
        write_queue_status(
            tmp.path(),
            rec.seq,
            "processing",
            &[("processing_started_at", json!("T1"))],
        )
        .unwrap();
        let text = std::fs::read_to_string(queue_record_path(tmp.path(), rec.seq)).unwrap();
        let parsed: Map<String, Value> = serde_json::from_str(&text).unwrap();
        let keys: Vec<&str> = parsed.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            vec![
                "seq",
                "worktree_id",
                "feature",
                "requested_by_session",
                "requested_at",
                "status",
                "processing_started_at"
            ]
        );
        assert!(text.contains("\"status\": \"processing\""));
    }

    /// A corrupt record is classified BEFORE any lock — the probe the caller
    /// runs so a V8-worded readJson warn never has to be reproduced.
    #[test]
    fn corrupt_record_fails_the_preflight() {
        let tmp = fixture();
        enqueue_merge_request(tmp.path(), "wt-a", "s", "T0").unwrap();
        assert!(preflight_queue_readable(tmp.path()));
        std::fs::write(queue_dir(tmp.path()).join("000002.json"), "{oops").unwrap();
        assert!(!preflight_queue_readable(tmp.path()));
    }

    /// The solo case: an empty queue resolves on the FIRST iteration with no
    /// sleep at all, and the record is driven straight to 'done'.
    #[test]
    fn solo_drain_runs_immediately_and_marks_done() {
        let tmp = fixture();
        let started = std::time::Instant::now();
        let out = run_through_queue(
            tmp.path(),
            "wt-a",
            "sess-1",
            "main",
            DEFAULT_WAIT_BOUND_MS,
            DEFAULT_POLL_INTERVAL_MS,
            DEFAULT_PROCESSOR_TTL_SECONDS,
            DEFAULT_RENEW_INTERVAL_MS,
            |hooks| {
                // The lease is live while the merge runs, so the fence is clean.
                assert_eq!(hooks.check_processor_lease(), None);
                hooks.on_verify_tick();
                Ok(("merged", true))
            },
        )
        .unwrap();
        assert!(started.elapsed().as_millis() < 400, "no poll sleep on a solo drain");
        match out {
            Drain::Ran(v) => assert_eq!(v, "merged"),
            Drain::TimedOut(_) => panic!("a solo drain never times out"),
        }
        let text = std::fs::read_to_string(queue_record_path(tmp.path(), 1.0)).unwrap();
        assert!(text.contains("\"status\": \"done\""), "{text}");
        assert!(text.contains("\"processing_started_at\""));
        assert!(text.contains("\"finished_at\""));
        // The lease is always released in the `finally`.
        assert!(read_processor_lease_raw(tmp.path()).is_none());
    }

    /// A merge that returns `ok: false` marks the record 'failed' — never
    /// 'done' (the queue must never claim a red merge succeeded).
    #[test]
    fn not_ok_result_marks_the_record_failed() {
        let tmp = fixture();
        let out = run_through_queue(
            tmp.path(),
            "wt-a",
            "s",
            "main",
            DEFAULT_WAIT_BOUND_MS,
            DEFAULT_POLL_INTERVAL_MS,
            DEFAULT_PROCESSOR_TTL_SECONDS,
            DEFAULT_RENEW_INTERVAL_MS,
            |_| Ok(("red", false)),
        )
        .unwrap();
        assert!(matches!(out, Drain::Ran("red")));
        let text = std::fs::read_to_string(queue_record_path(tmp.path(), 1.0)).unwrap();
        assert!(text.contains("\"status\": \"failed\""), "{text}");
    }

    /// A THROWING merge marks 'failed' AND persists the thrown message in
    /// `error` — the queue-record field the .mjs's catch writes.
    #[test]
    fn throwing_merge_persists_its_message_into_the_record() {
        let tmp = fixture();
        let outcome = run_through_queue(
            tmp.path(),
            "wt-a",
            "s",
            "main",
            DEFAULT_WAIT_BOUND_MS,
            DEFAULT_POLL_INTERVAL_MS,
            DEFAULT_PROCESSOR_TTL_SECONDS,
            DEFAULT_RENEW_INTERVAL_MS,
            |_| Err::<((), bool), String>("[WORKTREE_MERGE_MAIN_DIRTY] boom".to_string()),
        );
        let Err(err) = outcome else { panic!("a throwing merge must propagate") };
        assert!(matches!(&err, QErr::Msg(m) if m == "[WORKTREE_MERGE_MAIN_DIRTY] boom"));
        let text = std::fs::read_to_string(queue_record_path(tmp.path(), 1.0)).unwrap();
        assert!(text.contains("\"status\": \"failed\""), "{text}");
        assert!(text.contains("\"error\": \"[WORKTREE_MERGE_MAIN_DIRTY] boom\""), "{text}");
        assert!(read_processor_lease_raw(tmp.path()).is_none(), "lease released in finally");
    }

    /// A request stuck behind an older QUEUED record times out with advisor
    /// condition B's exact bytes and never runs the merge.
    #[test]
    fn timeout_never_runs_the_merge_and_says_so() {
        let tmp = fixture();
        // An older record sitting at the front of the line, never drained.
        enqueue_merge_request(tmp.path(), "wt-older", "other", "T0").unwrap();
        let out = run_through_queue(
            tmp.path(),
            "wt-a",
            "s",
            "main",
            0.0, // an already-elapsed wait bound
            DEFAULT_POLL_INTERVAL_MS,
            DEFAULT_PROCESSOR_TTL_SECONDS,
            DEFAULT_RENEW_INTERVAL_MS,
            |_| -> Result<((), bool), String> { panic!("the merge must never run on a timeout") },
        )
        .unwrap();
        let Drain::TimedOut(t) = out else { panic!("expected a timeout") };
        assert!(t.message.starts_with(
            "the integration queue's wait bound (0ms) elapsed before this merge request reached the front of the line — the merge did NOT run and nothing was committed."
        ), "{}", t.message);
        assert!(t.message.contains("still queued at position 2 (1 ahead of it)"), "{}", t.message);
        assert_eq!(t.result["ok"], Value::Bool(false));
        assert_eq!(t.result["merged"], Value::Bool(false));
        assert_eq!(t.result["code"], json!("INTEGRATION_QUEUE_TIMEOUT"));
        assert_eq!(t.result["queue"]["position"], json!(2.0));
    }

    /// The epoch fence: a takeover bumps the epoch and the zombie's fence goes
    /// non-null with Node's exact drift wording.
    #[test]
    fn epoch_fence_detects_takeover_and_disappearance() {
        let tmp = fixture();
        let Processor::Acquired(first) =
            try_become_processor(tmp.path(), "sess-1", "main", 120.0, now_ms()).unwrap()
        else {
            panic!("a free lease must be acquirable")
        };
        let epoch1 = jget(&first, "epoch").cloned().unwrap();
        assert_eq!(epoch1, json!(1.0));
        assert_eq!(check_processor_lease_epoch(tmp.path(), &epoch1), None);

        // A live lease refuses a second acquire (the ordinary contention case).
        assert!(matches!(
            try_become_processor(tmp.path(), "sess-2", "main", 120.0, now_ms()).unwrap(),
            Processor::Held
        ));

        // Expire it, then take over: epoch moves 1 -> 2 and the zombie sees it.
        let Processor::Acquired(second) =
            try_become_processor(tmp.path(), "sess-2", "main", 120.0, now_ms() + 200_000.0).unwrap()
        else {
            panic!("an expired lease must be takeable")
        };
        assert_eq!(jget(&second, "epoch").cloned().unwrap(), json!(2.0));
        let drift = check_processor_lease_epoch(tmp.path(), &epoch1).unwrap();
        assert_eq!(
            drift,
            "the integration-queue processor lease was taken over while this processor's verify was running (epoch 1 -> 2, now held by session \"sess-2\") — this processor is a zombie"
        );

        // And a vanished record is its own drift string.
        release_processor_lease(tmp.path(), &json!(2.0));
        assert_eq!(
            check_processor_lease_epoch(tmp.path(), &epoch1).unwrap(),
            "the integration-queue processor lease disappeared (this processor acquired epoch 1) while its verify was running — released or swept out from under it"
        );
    }

    /// The queue lock this module contends on is Node's, by name.
    #[test]
    fn queue_writes_use_nodes_lock_name() {
        let tmp = fixture();
        let Ok(mut guard) = lock::acquire_store_lock(tmp.path(), QUEUE_LOCK_NAME, lock::MAX_ATTEMPTS)
        else {
            panic!("a fresh root's integration-queue lock must be free")
        };
        assert!(lock::lock_file_path(tmp.path(), "integration-queue").exists());
        guard.release();
    }
}
