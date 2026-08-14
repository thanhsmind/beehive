// bee deferred-queue — the ONE claimable queue D5 asks for: deferred
// capture, scribing, review, and promote-proposal work as REAL records an
// agent absent when they were queued can pick up and execute.
//
// This is NEW functionality, not a Node port — there is no delegate to fall
// back to, so every accepted argv shape is served to a typed conclusion
// natively (a validation failure is a native `emit_error`, never a silent
// `None`; `None` is reserved for an argv this group does not claim at all).
//
// Verbs:
//   deferred-queue add     --kind <capture|scribe|review|promote>
//                           --feature <v> --reason <v>
//                           [--cells <c1,c2>] [--areas <a1,a2>] [--files <f1,f2>]
//                           [--json]
//   deferred-queue list    [--kind <v>] [--claimed <claimed|unclaimed>] [--json]
//   deferred-queue claim   --id <v> [--owner <v>] [--lease <seconds>] [--json]
//   deferred-queue release --id <v> [--owner <v>] [--json]
//   deferred-queue complete --id <v> [--owner <v>] [--json]
//
// Store: `.bee/deferred-queue.jsonl`, event-sourced (add/claim/release/
// complete), folded last-event-wins per id — the same shape backlog.rs's
// PBI fold already proves (fold_pbis, `verbs/backlog.rs`).
//
// Exclusivity (the point of this cell): `claim` follows backlog.rs's
// append-then-fold critical section — a pre-lock fold as a cheap
// deterministic-refusal probe, `lock::acquire_store_lock_once` (the same
// O_EXCL-backed store lock every other mutating store in this crate
// contends on — see `lock.rs`'s `try_acquire`, `OpenOptions::create_new`),
// a RE-fold under the lock as the actual race check, the append, then
// release. Only one process can hold the lock at a time, so only one
// process's re-fold can ever see the item as claimable and win the append.
//
// Reclaim rule (dual condition, cells/claims.rs's pattern re-derived here —
// PLAN.md marks this reusable as a pattern, not as importable code): an
// already-claimed item is reclaimable only when its lease has expired AND
// the claiming owner's own session heartbeat is stale. Lease expiry alone
// is never enough — a live owner mid-lease-renewal must not be raced out
// from under itself.
//
// Owner identity: `crate::verbs::reservations::resolve_session_id` (flag ->
// BEE_SESSION_ID -> CLAUDE_CODE_SESSION_ID -> single-live-session adoption
// -> None), the exact precedence chain `cells claim` already uses — reused
// directly, not re-derived, per PLAN.md ("reusable with zero new code").
//
// Migration: `.bee/capture-queue.jsonl` and `.bee/review-candidates.jsonl`
// are UNTOUCHED by this cell. This module builds the queue; a later cell
// decides what feeds it.

use super::feedback::{emit_error, emit_success, js_trim, now_iso, parse_shape, ParsedArgs};
use crate::fsutil::append_jsonl;
use crate::lock::{acquire_store_lock_once, AcquireOnce};
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::verbs::reservations as rsv;
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use serde_json::{Map, Number, Value};
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const KINDS: [&str; 4] = ["capture", "scribe", "review", "promote"];
const DEFAULT_LEASE_SECONDS: f64 = 3600.0;
const QUEUE_LOCK_NAME: &str = "deferred-queue";
const LOCK_RETRY_ATTEMPTS: u32 = 15;
const LOCK_RETRY_DELAY_MS: u64 = 20;

fn queue_path(root: &Path) -> PathBuf {
    root.join(".bee").join("deferred-queue.jsonl")
}

// ─── event-sourced fold ─────────────────────────────────────────────────────

#[derive(Clone)]
struct Claim {
    owner: Option<String>,
    claimed_at: String,
    ttl_seconds: f64,
}

#[derive(Clone)]
struct Item {
    id: String,
    kind: String,
    feature: String,
    cells: Vec<String>,
    areas: Vec<String>,
    files: Vec<String>,
    reason: String,
    queued_at: String,
    claim: Option<Claim>,
    completed: bool,
    completed_at: Option<String>,
}

struct Fold {
    order: Vec<String>,
    items: HashMap<String, Item>,
}

fn str_list(m: &Map<String, Value>, name: &str) -> Vec<String> {
    match m.get(name) {
        Some(Value::Array(a)) => {
            a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()
        }
        _ => Vec::new(),
    }
}

/// Last-event-wins fold over `.bee/deferred-queue.jsonl`, mirroring
/// backlog.rs's `fold_pbis`: an unparseable line is skipped (never delegated
/// — there is nothing left to delegate to), and a first `add` for a given id
/// wins; a later duplicate `add` is ignored.
fn fold(root: &Path) -> Fold {
    let read = super::feedback::read_jsonl(&queue_path(root));
    let mut f = Fold { order: Vec::new(), items: HashMap::new() };
    for row in &read.rows {
        let Value::Object(m) = row else { continue };
        let Some(id) = m.get("id").and_then(Value::as_str).filter(|s| !s.is_empty()) else {
            continue;
        };
        let ts = m.get("ts").and_then(Value::as_str).unwrap_or("").to_string();
        match m.get("event").and_then(Value::as_str) {
            Some("add") => {
                if f.items.contains_key(id) {
                    continue; // first add wins
                }
                let kind = m.get("kind").and_then(Value::as_str).unwrap_or("").to_string();
                let feature = m.get("feature").and_then(Value::as_str).unwrap_or("").to_string();
                let reason = m.get("reason").and_then(Value::as_str).unwrap_or("").to_string();
                f.items.insert(
                    id.to_string(),
                    Item {
                        id: id.to_string(),
                        kind,
                        feature,
                        cells: str_list(m, "cells"),
                        areas: str_list(m, "areas"),
                        files: str_list(m, "files"),
                        reason,
                        queued_at: ts,
                        claim: None,
                        completed: false,
                        completed_at: None,
                    },
                );
                f.order.push(id.to_string());
            }
            Some("claim") => {
                if let Some(item) = f.items.get_mut(id) {
                    let owner = m.get("owner").and_then(Value::as_str).map(str::to_string);
                    let claimed_at =
                        m.get("claimed_at").and_then(Value::as_str).unwrap_or(&ts).to_string();
                    let ttl_seconds = m
                        .get("ttl_seconds")
                        .and_then(Value::as_f64)
                        .unwrap_or(DEFAULT_LEASE_SECONDS);
                    item.claim = Some(Claim { owner, claimed_at, ttl_seconds });
                }
            }
            Some("release") => {
                if let Some(item) = f.items.get_mut(id) {
                    item.claim = None;
                }
            }
            Some("complete") => {
                if let Some(item) = f.items.get_mut(id) {
                    item.completed = true;
                    item.completed_at = Some(ts);
                    item.claim = None;
                }
            }
            _ => {}
        }
    }
    f
}

// ─── claim exclusivity: the dual-condition stale rule ──────────────────────
//
// Pattern re-derived from cells/claims.rs (claim_expired + heartbeat_stale +
// handlers_select.rs's dual-condition sweep), not imported — those items are
// pub(crate) to the `cells` module only. `resolve_session_id` and the
// session-record scan (`list_session_records`, `heartbeat_stale`) ARE
// imported directly from `reservations`, per PLAN.md's "reusable with zero
// new code" half of the same paragraph.

fn parse_iso_ms(iso: &str) -> Option<f64> {
    rsv::date_parse_val(Some(&Value::String(iso.to_string()))).ok().flatten()
}

fn lease_expired(claim: &Claim, now_ms: f64) -> bool {
    if !claim.ttl_seconds.is_finite() || claim.ttl_seconds <= 0.0 {
        return false; // never expires, matching claims.rs's isClaimExpired
    }
    match parse_iso_ms(&claim.claimed_at) {
        Some(ms) => ms + claim.ttl_seconds * 1000.0 <= now_ms,
        None => false,
    }
}

fn owner_heartbeat_stale(root: &Path, owner: Option<&str>, now_ms: f64) -> bool {
    let Some(owner) = owner else { return true }; // sessionless claim: always stale
    let control_root = control_root_string(root);
    let records = rsv::list_session_records(&control_root).unwrap_or_default();
    match records.iter().find(|r| r.get("id").and_then(Value::as_str) == Some(owner)) {
        Some(record) => rsv::heartbeat_stale(record, now_ms).unwrap_or(true),
        None => true, // no live session record for this owner: treat as dead
    }
}

/// The dual condition itself: reclaimable only when BOTH the lease has
/// expired AND the owning session's heartbeat is stale — never on the lease
/// alone (must_have, D5).
fn claim_reclaimable(root: &Path, claim: &Claim, now_ms: f64) -> bool {
    lease_expired(claim, now_ms) && owner_heartbeat_stale(root, claim.owner.as_deref(), now_ms)
}

fn claim_expires_at(claim: &Claim) -> Option<String> {
    if !claim.ttl_seconds.is_finite() || claim.ttl_seconds <= 0.0 {
        return None;
    }
    let ms = parse_iso_ms(&claim.claimed_at)?;
    rsv::iso_from_ms(ms + claim.ttl_seconds * 1000.0).ok()
}

fn claim_expiry_text(claim: &Claim) -> String {
    claim_expires_at(claim).map(|s| format!("expires {s}")).unwrap_or_else(|| "no expiry".to_string())
}

fn control_root_string(root: &Path) -> String {
    let root_s = root.to_string_lossy().into_owned();
    rsv::control_root_for(&root_s).unwrap_or(root_s)
}

fn resolve_owner(root: &Path, flag: Option<&str>) -> Option<String> {
    let control_root = control_root_string(root);
    rsv::resolve_session_id(flag, &control_root).ok().flatten()
}

// ─── JSON rendering ─────────────────────────────────────────────────────────

fn claim_value(claim: &Claim) -> Value {
    let mut m = Map::new();
    m.insert("owner".into(), claim.owner.clone().map(Value::String).unwrap_or(Value::Null));
    m.insert("claimed_at".into(), Value::String(claim.claimed_at.clone()));
    m.insert(
        "ttl_seconds".into(),
        Number::from_f64(claim.ttl_seconds).map(Value::Number).unwrap_or(Value::Null),
    );
    m.insert(
        "expires_at".into(),
        claim_expires_at(claim).map(Value::String).unwrap_or(Value::Null),
    );
    Value::Object(m)
}

fn item_value(item: &Item) -> Value {
    let mut m = Map::new();
    m.insert("id".into(), Value::String(item.id.clone()));
    m.insert("kind".into(), Value::String(item.kind.clone()));
    m.insert("feature".into(), Value::String(item.feature.clone()));
    m.insert("cells".into(), Value::Array(item.cells.iter().cloned().map(Value::String).collect()));
    m.insert("areas".into(), Value::Array(item.areas.iter().cloned().map(Value::String).collect()));
    m.insert("files".into(), Value::Array(item.files.iter().cloned().map(Value::String).collect()));
    m.insert("reason".into(), Value::String(item.reason.clone()));
    m.insert("queued_at".into(), Value::String(item.queued_at.clone()));
    m.insert("claim".into(), item.claim.as_ref().map(claim_value).unwrap_or(Value::Null));
    m.insert("completed".into(), Value::Bool(item.completed));
    m.insert(
        "completed_at".into(),
        item.completed_at.clone().map(Value::String).unwrap_or(Value::Null),
    );
    Value::Object(m)
}

// ─── argv plumbing ──────────────────────────────────────────────────────────

fn split_list(raw: Option<&str>) -> Vec<String> {
    match raw {
        Some(s) if !js_trim(s).is_empty() => {
            s.split(',').map(js_trim).filter(|p| !p.is_empty()).map(str::to_string).collect()
        }
        _ => Vec::new(),
    }
}

struct Ctx {
    root: PathBuf,
    drift: crate::registry::Drift,
}

fn preamble(cmd: &str, pre_json: bool, t0: Instant) -> Result<Option<Ctx>, ExitCode> {
    let Ok(cwd) = std::env::current_dir() else { return Ok(None) };
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => return Err(emit_unsupported_root(&cwd, cmd, pre_json, t0, &why)),
        Roots::None => return Err(emit_no_root_error(&cwd, cmd, pre_json, t0)),
    };
    let drift = check_manifest_drift(&root);
    Ok(Some(Ctx { root, drift }))
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "deferred-queue" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let rest = &args[2..];
    match verb {
        "add" => run_add(parse_shape(rest, &["kind", "feature", "reason", "cells", "areas", "files"])?, t0),
        "list" => run_list(parse_shape(rest, &["kind", "claimed"])?, t0),
        "claim" => run_claim(parse_shape(rest, &["id", "owner", "lease"])?, t0),
        "release" => run_release(parse_shape(rest, &["id", "owner"])?, t0),
        "complete" => run_complete(parse_shape(rest, &["id", "owner"])?, t0),
        _ => None,
    }
}

fn flag<'a>(parsed: &'a ParsedArgs, name: &str) -> Option<&'a str> {
    parsed.flags.get(name).map(|s| js_trim(s)).filter(|s| !s.is_empty())
}

// ─── add ─────────────────────────────────────────────────────────────────

fn run_add(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let cmd = "deferred-queue add";
    let ctx = match preamble(cmd, parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let Some(kind) = flag(&parsed, "kind").filter(|k| KINDS.contains(k)) else {
        let msg = format!(
            "bee {cmd}: --kind is required and must be one of capture, scribe, review, promote."
        );
        return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0));
    };
    let Some(feature) = flag(&parsed, "feature") else {
        return Some(emit_error(&ctx.root, cmd, parsed.json, &format!("bee {cmd}: --feature is required."), t0));
    };
    let Some(reason) = flag(&parsed, "reason") else {
        return Some(emit_error(&ctx.root, cmd, parsed.json, &format!("bee {cmd}: --reason is required."), t0));
    };
    let cells = split_list(parsed.flags.get("cells").map(String::as_str));
    let areas = split_list(parsed.flags.get("areas").map(String::as_str));
    let files = split_list(parsed.flags.get("files").map(String::as_str));

    let id = super::feedback::random_uuid_v4();
    let ts = now_iso();
    let mut event = Map::new();
    event.insert("ts".into(), Value::String(ts.clone()));
    event.insert("event".into(), Value::String("add".into()));
    event.insert("id".into(), Value::String(id.clone()));
    event.insert("kind".into(), Value::String(kind.to_string()));
    event.insert("feature".into(), Value::String(feature.to_string()));
    event.insert("cells".into(), Value::Array(cells.iter().cloned().map(Value::String).collect()));
    event.insert("areas".into(), Value::Array(areas.iter().cloned().map(Value::String).collect()));
    event.insert("files".into(), Value::Array(files.iter().cloned().map(Value::String).collect()));
    event.insert("reason".into(), Value::String(reason.to_string()));

    if append_jsonl(&queue_path(&ctx.root), &Value::Object(event)).is_err() {
        let msg = format!("bee {cmd}: could not append to the deferred queue.");
        return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0));
    }

    let item = Item {
        id: id.clone(),
        kind: kind.to_string(),
        feature: feature.to_string(),
        cells,
        areas,
        files,
        reason: reason.to_string(),
        queued_at: ts,
        claim: None,
        completed: false,
        completed_at: None,
    };
    let text = format!("Queued deferred {kind} item {id} for feature \"{feature}\".");
    Some(emit_success(&ctx.root, cmd, parsed.json, &ctx.drift, &item_value(&item), &text, t0))
}

// ─── list ────────────────────────────────────────────────────────────────

fn run_list(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let cmd = "deferred-queue list";
    let ctx = match preamble(cmd, parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let kind_filter = match flag(&parsed, "kind") {
        Some(k) if KINDS.contains(&k) => Some(k.to_string()),
        Some(_) => {
            let msg = format!("bee {cmd}: --kind must be one of capture, scribe, review, promote.");
            return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0));
        }
        None => None,
    };
    let claimed_filter = match flag(&parsed, "claimed") {
        Some("claimed") => Some(true),
        Some("unclaimed") => Some(false),
        Some(_) => {
            let msg = format!("bee {cmd}: --claimed must be \"claimed\" or \"unclaimed\".");
            return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0));
        }
        None => None,
    };

    let f = fold(&ctx.root);
    let items: Vec<&Item> = f
        .order
        .iter()
        .filter_map(|id| f.items.get(id))
        .filter(|item| kind_filter.as_deref().map(|k| k == item.kind).unwrap_or(true))
        .filter(|item| claimed_filter.map(|c| item.claim.is_some() == c).unwrap_or(true))
        .collect();

    let text = if items.is_empty() {
        "Deferred queue is empty.".to_string()
    } else {
        items
            .iter()
            .map(|item| {
                let status = if item.completed {
                    "completed".to_string()
                } else if let Some(claim) = &item.claim {
                    format!("claimed by {} ({})", claim.owner.as_deref().unwrap_or("no owner"), claim_expiry_text(claim))
                } else {
                    "open".to_string()
                };
                format!("[{}] {} — {} ({})", item.kind, item.id, item.reason, status)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let mut result = Map::new();
    result.insert("count".into(), Value::from(items.len()));
    result.insert("items".into(), Value::Array(items.iter().map(|i| item_value(i)).collect()));
    Some(emit_success(&ctx.root, cmd, parsed.json, &ctx.drift, &Value::Object(result), &text, t0))
}

// ─── claim: the point of the cell ──────────────────────────────────────────

enum ClaimDecision {
    /// The item is unclaimed (or its claim is reclaimable) as of this fold —
    /// a claim MAY be appended. Never a write itself: read-only, so it is
    /// safe to call outside any lock.
    Claimable(Item),
    NotFound,
    Completed,
    Claimed(Claim),
}

/// Read-only: fold, decide. Never appends — safe to call BEFORE the lock as
/// a cheap deterministic-refusal probe (backlog.rs's `add_pbi` runs the same
/// shape of pre-lock check) and again UNDER the lock as the real race
/// arbiter's decision half.
fn decide_claim(root: &Path, id: &str, now_ms: f64) -> ClaimDecision {
    let f = fold(root);
    let Some(item) = f.items.get(id) else { return ClaimDecision::NotFound };
    if item.completed {
        return ClaimDecision::Completed;
    }
    if let Some(claim) = &item.claim {
        if !claim_reclaimable(root, claim, now_ms) {
            return ClaimDecision::Claimed(claim.clone());
        }
    }
    ClaimDecision::Claimable(item.clone())
}

/// The write half: appends the `claim` event. Callers MUST hold the store
/// lock — this function does not acquire it — and must have just re-derived
/// `ClaimDecision::Claimable` under that same lock, or the append is not the
/// race-safe one this cell exists to prove.
fn apply_claim(root: &Path, item: &Item, id: &str, owner: Option<&str>, lease: f64) -> Item {
    let ts = now_iso();
    let mut event = Map::new();
    event.insert("ts".into(), Value::String(ts.clone()));
    event.insert("event".into(), Value::String("claim".into()));
    event.insert("id".into(), Value::String(id.to_string()));
    event.insert("owner".into(), owner.map(|o| Value::String(o.to_string())).unwrap_or(Value::Null));
    event.insert("claimed_at".into(), Value::String(ts.clone()));
    event.insert(
        "ttl_seconds".into(),
        Number::from_f64(lease).map(Value::Number).unwrap_or(Value::Null),
    );
    let _ = append_jsonl(&queue_path(root), &Value::Object(event));
    let mut claimed = item.clone();
    claimed.claim = Some(Claim { owner: owner.map(str::to_string), claimed_at: ts, ttl_seconds: lease });
    claimed
}

fn run_claim(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let cmd = "deferred-queue claim";
    let ctx = match preamble(cmd, parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let Some(id) = flag(&parsed, "id") else {
        return Some(emit_error(&ctx.root, cmd, parsed.json, &format!("bee {cmd}: --id is required."), t0));
    };
    let lease = match flag(&parsed, "lease") {
        Some(raw) => match raw.parse::<f64>() {
            Ok(v) if v.is_finite() => v,
            _ => {
                let msg = format!("bee {cmd}: --lease must be a finite number of seconds.");
                return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0));
            }
        },
        None => DEFAULT_LEASE_SECONDS,
    };
    let owner_flag = flag(&parsed, "owner");
    let owner = resolve_owner(&ctx.root, owner_flag).or_else(|| owner_flag.map(str::to_string));

    // Pre-lock probe: READ-ONLY. A deterministic refusal (not found /
    // completed / an ACTIVE claim that is not yet reclaimable) never needs
    // the lock at all — and critically, this probe never writes, or every
    // racer's own probe would race the lock itself (the bug this comment
    // now documents having caught: an earlier draft folded-decide-AND-wrote
    // in one function reused for both phases, so the fastest racer's own
    // unlocked probe won the item out from under its own locked re-check).
    let now_ms = rsv::now_ms();
    match decide_claim(&ctx.root, id, now_ms) {
        ClaimDecision::NotFound => {
            return Some(emit_error(&ctx.root, cmd, parsed.json, &format!("bee {cmd}: no queued item with id \"{id}\"."), t0));
        }
        ClaimDecision::Completed => {
            return Some(emit_error(&ctx.root, cmd, parsed.json, &format!("bee {cmd}: item \"{id}\" is already completed."), t0));
        }
        ClaimDecision::Claimed(claim) => {
            let who = claim.owner.as_deref().unwrap_or("no owner (sessionless claim)");
            let msg = format!(
                "bee {cmd}: item \"{id}\" is already claimed by \"{who}\" ({}).",
                claim_expiry_text(&claim)
            );
            return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0));
        }
        ClaimDecision::Claimable(_) => {} // fall through to the locked re-check below
    }

    // Real arbiter: acquire the store lock, RE-fold under it, decide again,
    // and ONLY THEN append. Only one process can hold this lock at a time —
    // the O_EXCL create underneath `lock::acquire_store_lock_once` (lock.rs's
    // `try_acquire`) is what makes exactly one racer's locked re-fold see the
    // item as still claimable; every other racer's locked re-fold observes
    // that racer's own just-appended claim event and refuses.
    let mut attempt = 0u32;
    let mut guard = loop {
        match acquire_store_lock_once(&ctx.root, QUEUE_LOCK_NAME) {
            AcquireOnce::Acquired(g) => break g,
            AcquireOnce::Busy { holder } => {
                if attempt >= LOCK_RETRY_ATTEMPTS {
                    let who = holder
                        .as_ref()
                        .and_then(|h| h.get("session"))
                        .map(crate::jsjson::js_to_string)
                        .unwrap_or_else(|| "unknown".to_string());
                    let msg = format!("bee {cmd}: deferred-queue store lock busy: held by {who}.");
                    return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0));
                }
                attempt += 1;
                std::thread::sleep(std::time::Duration::from_millis(LOCK_RETRY_DELAY_MS));
            }
        }
    };
    let now_ms = rsv::now_ms();
    let decision = decide_claim(&ctx.root, id, now_ms);
    let result = match decision {
        ClaimDecision::Claimable(item) => Ok(apply_claim(&ctx.root, &item, id, owner.as_deref(), lease)),
        ClaimDecision::NotFound => Err(format!("bee {cmd}: no queued item with id \"{id}\".")),
        ClaimDecision::Completed => Err(format!("bee {cmd}: item \"{id}\" is already completed.")),
        ClaimDecision::Claimed(claim) => {
            let who = claim.owner.as_deref().unwrap_or("no owner (sessionless claim)");
            Err(format!(
                "bee {cmd}: item \"{id}\" is already claimed by \"{who}\" ({}).",
                claim_expiry_text(&claim)
            ))
        }
    };
    guard.release();

    match result {
        Ok(item) => {
            let text = format!(
                "Claimed deferred {} item {id} as \"{}\".",
                item.kind,
                owner.as_deref().unwrap_or("no owner (sessionless claim)")
            );
            Some(emit_success(&ctx.root, cmd, parsed.json, &ctx.drift, &item_value(&item), &text, t0))
        }
        Err(msg) => Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0)),
    }
}

// ─── release ─────────────────────────────────────────────────────────────

fn run_release(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let cmd = "deferred-queue release";
    let ctx = match preamble(cmd, parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let Some(id) = flag(&parsed, "id") else {
        return Some(emit_error(&ctx.root, cmd, parsed.json, &format!("bee {cmd}: --id is required."), t0));
    };
    let owner_flag = flag(&parsed, "owner");
    let owner = resolve_owner(&ctx.root, owner_flag).or_else(|| owner_flag.map(str::to_string));

    let mut attempt = 0u32;
    let mut guard = loop {
        match acquire_store_lock_once(&ctx.root, QUEUE_LOCK_NAME) {
            AcquireOnce::Acquired(g) => break g,
            AcquireOnce::Busy { .. } => {
                if attempt >= LOCK_RETRY_ATTEMPTS {
                    let msg = format!("bee {cmd}: deferred-queue store lock busy.");
                    return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0));
                }
                attempt += 1;
                std::thread::sleep(std::time::Duration::from_millis(LOCK_RETRY_DELAY_MS));
            }
        }
    };
    let f = fold(&ctx.root);
    let outcome: Result<Item, String> = match f.items.get(id) {
        None => Err(format!("bee {cmd}: no queued item with id \"{id}\".")),
        Some(item) => match &item.claim {
            None => Err(format!("bee {cmd}: item \"{id}\" is not currently claimed.")),
            Some(claim) if claim.owner.as_deref() != owner.as_deref() => Err(format!(
                "bee {cmd}: item \"{id}\" is claimed by \"{}\", not \"{}\".",
                claim.owner.as_deref().unwrap_or("no owner"),
                owner.as_deref().unwrap_or("no owner")
            )),
            Some(_) => {
                let mut event = Map::new();
                event.insert("ts".into(), Value::String(now_iso()));
                event.insert("event".into(), Value::String("release".into()));
                event.insert("id".into(), Value::String(id.to_string()));
                if append_jsonl(&queue_path(&ctx.root), &Value::Object(event)).is_err() {
                    Err(format!("bee {cmd}: could not append to the deferred queue."))
                } else {
                    let mut released = item.clone();
                    released.claim = None;
                    Ok(released)
                }
            }
        },
    };
    guard.release();

    match outcome {
        Err(msg) => Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0)),
        Ok(item) => {
            let text = format!("Released deferred {} item {id}.", item.kind);
            Some(emit_success(&ctx.root, cmd, parsed.json, &ctx.drift, &item_value(&item), &text, t0))
        }
    }
}

// ─── complete ────────────────────────────────────────────────────────────

fn run_complete(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let cmd = "deferred-queue complete";
    let ctx = match preamble(cmd, parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let Some(id) = flag(&parsed, "id") else {
        return Some(emit_error(&ctx.root, cmd, parsed.json, &format!("bee {cmd}: --id is required."), t0));
    };
    let owner_flag = flag(&parsed, "owner");
    let owner = resolve_owner(&ctx.root, owner_flag).or_else(|| owner_flag.map(str::to_string));

    let mut attempt = 0u32;
    let mut guard = loop {
        match acquire_store_lock_once(&ctx.root, QUEUE_LOCK_NAME) {
            AcquireOnce::Acquired(g) => break g,
            AcquireOnce::Busy { .. } => {
                if attempt >= LOCK_RETRY_ATTEMPTS {
                    let msg = format!("bee {cmd}: deferred-queue store lock busy.");
                    return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0));
                }
                attempt += 1;
                std::thread::sleep(std::time::Duration::from_millis(LOCK_RETRY_DELAY_MS));
            }
        }
    };
    let f = fold(&ctx.root);
    let outcome: Result<Item, String> = match f.items.get(id) {
        None => Err(format!("bee {cmd}: no queued item with id \"{id}\".")),
        Some(item) if item.completed => Err(format!("bee {cmd}: item \"{id}\" is already completed.")),
        Some(item) => match &item.claim {
            None => Err(format!(
                "bee {cmd}: item \"{id}\" must be claimed before it can be completed."
            )),
            Some(claim) if claim.owner.as_deref() != owner.as_deref() => Err(format!(
                "bee {cmd}: item \"{id}\" is claimed by \"{}\", not \"{}\".",
                claim.owner.as_deref().unwrap_or("no owner"),
                owner.as_deref().unwrap_or("no owner")
            )),
            Some(_) => {
                let ts = now_iso();
                let mut event = Map::new();
                event.insert("ts".into(), Value::String(ts.clone()));
                event.insert("event".into(), Value::String("complete".into()));
                event.insert("id".into(), Value::String(id.to_string()));
                if append_jsonl(&queue_path(&ctx.root), &Value::Object(event)).is_err() {
                    Err(format!("bee {cmd}: could not append to the deferred queue."))
                } else {
                    let mut completed = item.clone();
                    completed.completed = true;
                    completed.completed_at = Some(ts);
                    completed.claim = None;
                    Ok(completed)
                }
            }
        },
    };
    guard.release();

    match outcome {
        Err(msg) => Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0)),
        Ok(item) => {
            let text = format!("Completed deferred {} item {id}.", item.kind);
            Some(emit_success(&ctx.root, cmd, parsed.json, &ctx.drift, &item_value(&item), &text, t0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write_queue(root: &Path, lines: &str) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(queue_path(root), lines).unwrap();
    }

    #[test]
    fn fold_applies_add_claim_release_complete_last_event_wins() {
        let tmp = tempfile::tempdir().unwrap();
        write_queue(
            tmp.path(),
            concat!(
                "{\"ts\":\"2026-01-01T00:00:00.000Z\",\"event\":\"add\",\"id\":\"a\",\"kind\":\"capture\",\"feature\":\"f\",\"cells\":[],\"areas\":[],\"files\":[],\"reason\":\"r\"}\n",
                "{\"ts\":\"2026-01-01T00:00:01.000Z\",\"event\":\"claim\",\"id\":\"a\",\"owner\":\"sess-1\",\"claimed_at\":\"2026-01-01T00:00:01.000Z\",\"ttl_seconds\":3600}\n",
            ),
        );
        let f = fold(tmp.path());
        assert_eq!(f.order, vec!["a".to_string()]);
        let item = f.items.get("a").unwrap();
        assert_eq!(item.kind, "capture");
        assert!(item.claim.is_some());
        assert_eq!(item.claim.as_ref().unwrap().owner.as_deref(), Some("sess-1"));

        // Release clears the claim.
        write_queue(
            tmp.path(),
            concat!(
                "{\"ts\":\"2026-01-01T00:00:00.000Z\",\"event\":\"add\",\"id\":\"a\",\"kind\":\"capture\",\"feature\":\"f\",\"cells\":[],\"areas\":[],\"files\":[],\"reason\":\"r\"}\n",
                "{\"ts\":\"2026-01-01T00:00:01.000Z\",\"event\":\"claim\",\"id\":\"a\",\"owner\":\"sess-1\",\"claimed_at\":\"2026-01-01T00:00:01.000Z\",\"ttl_seconds\":3600}\n",
                "{\"ts\":\"2026-01-01T00:00:02.000Z\",\"event\":\"release\",\"id\":\"a\"}\n",
            ),
        );
        let f = fold(tmp.path());
        assert!(f.items.get("a").unwrap().claim.is_none());

        // Complete marks completed and clears any claim.
        write_queue(
            tmp.path(),
            concat!(
                "{\"ts\":\"2026-01-01T00:00:00.000Z\",\"event\":\"add\",\"id\":\"a\",\"kind\":\"capture\",\"feature\":\"f\",\"cells\":[],\"areas\":[],\"files\":[],\"reason\":\"r\"}\n",
                "{\"ts\":\"2026-01-01T00:00:01.000Z\",\"event\":\"claim\",\"id\":\"a\",\"owner\":\"sess-1\",\"claimed_at\":\"2026-01-01T00:00:01.000Z\",\"ttl_seconds\":3600}\n",
                "{\"ts\":\"2026-01-01T00:00:03.000Z\",\"event\":\"complete\",\"id\":\"a\"}\n",
            ),
        );
        let f = fold(tmp.path());
        let item = f.items.get("a").unwrap();
        assert!(item.completed);
        assert!(item.claim.is_none());
    }

    #[test]
    fn fold_ignores_a_duplicate_add_first_add_wins() {
        let tmp = tempfile::tempdir().unwrap();
        write_queue(
            tmp.path(),
            concat!(
                "{\"ts\":\"t\",\"event\":\"add\",\"id\":\"a\",\"kind\":\"capture\",\"feature\":\"f1\",\"cells\":[],\"areas\":[],\"files\":[],\"reason\":\"first\"}\n",
                "{\"ts\":\"t\",\"event\":\"add\",\"id\":\"a\",\"kind\":\"scribe\",\"feature\":\"f2\",\"cells\":[],\"areas\":[],\"files\":[],\"reason\":\"second\"}\n",
            ),
        );
        let f = fold(tmp.path());
        assert_eq!(f.order.len(), 1);
        assert_eq!(f.items.get("a").unwrap().reason, "first");
    }

    #[test]
    fn fold_skips_corrupt_lines() {
        let tmp = tempfile::tempdir().unwrap();
        write_queue(
            tmp.path(),
            concat!(
                "not json at all\n",
                "{\"ts\":\"t\",\"event\":\"add\",\"id\":\"a\",\"kind\":\"capture\",\"feature\":\"f\",\"cells\":[],\"areas\":[],\"files\":[],\"reason\":\"r\"}\n",
            ),
        );
        let f = fold(tmp.path());
        assert_eq!(f.order, vec!["a".to_string()]);
    }

    #[test]
    fn reclaim_requires_both_lease_expiry_and_stale_heartbeat() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".bee").join("sessions")).unwrap();
        std::fs::write(
            root.join(".bee").join("sessions").join("sess-1.json"),
            json!({"id": "sess-1", "last_heartbeat": "2026-01-01T00:00:00.000Z"}).to_string(),
        )
        .unwrap();

        // Lease not yet expired, heartbeat stale: NOT reclaimable — lease
        // expiry is a necessary condition even with a dead owner.
        let claimed_at = rsv::iso_from_ms(rsv::now_ms() - 10_000.0).ok().unwrap();
        let live_lease = Claim { owner: Some("sess-1".into()), claimed_at, ttl_seconds: 3600.0 };
        assert!(!claim_reclaimable(root, &live_lease, rsv::now_ms()));

        // Lease expired, but the owner's heartbeat is FRESH: NOT reclaimable.
        std::fs::write(
            root.join(".bee").join("sessions").join("sess-1.json"),
            json!({"id": "sess-1", "last_heartbeat": now_iso()}).to_string(),
        )
        .unwrap();
        let expired_claimed_at = rsv::iso_from_ms(rsv::now_ms() - 7200_000.0).ok().unwrap();
        let fresh_owner = Claim {
            owner: Some("sess-1".into()),
            claimed_at: expired_claimed_at.clone(),
            ttl_seconds: 60.0,
        };
        assert!(!claim_reclaimable(root, &fresh_owner, rsv::now_ms()));

        // BOTH lease expired AND heartbeat stale: reclaimable.
        std::fs::write(
            root.join(".bee").join("sessions").join("sess-1.json"),
            json!({"id": "sess-1", "last_heartbeat": "2026-01-01T00:00:00.000Z"}).to_string(),
        )
        .unwrap();
        let dead_owner =
            Claim { owner: Some("sess-1".into()), claimed_at: expired_claimed_at, ttl_seconds: 60.0 };
        assert!(claim_reclaimable(root, &dead_owner, rsv::now_ms()));

        // A sessionless claim (no owner) is always heartbeat-stale, so lease
        // expiry alone decides it.
        let sessionless_live = Claim {
            owner: None,
            claimed_at: rsv::iso_from_ms(rsv::now_ms() - 10_000.0).ok().unwrap(),
            ttl_seconds: 3600.0,
        };
        assert!(!claim_reclaimable(root, &sessionless_live, rsv::now_ms()));
        let sessionless_expired = Claim {
            owner: None,
            claimed_at: rsv::iso_from_ms(rsv::now_ms() - 7200_000.0).ok().unwrap(),
            ttl_seconds: 60.0,
        };
        assert!(claim_reclaimable(root, &sessionless_expired, rsv::now_ms()));
    }

    #[test]
    fn decide_and_apply_claim_lifecycle_open_claimed_claimed_again() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".bee")).unwrap();

        let mut add = Map::new();
        add.insert("ts".into(), Value::String(now_iso()));
        add.insert("event".into(), Value::String("add".into()));
        add.insert("id".into(), Value::String("item-1".into()));
        add.insert("kind".into(), Value::String("review".into()));
        add.insert("feature".into(), Value::String("feat".into()));
        add.insert("cells".into(), json!([]));
        add.insert("areas".into(), json!([]));
        add.insert("files".into(), json!([]));
        add.insert("reason".into(), Value::String("needs a second pass".into()));
        append_jsonl(&queue_path(root), &Value::Object(add)).unwrap();

        // Read-only decision: the fresh item is Claimable, and deciding never
        // writes (calling it twice in a row must not itself claim anything).
        let now_ms = rsv::now_ms();
        let claimable_item = match decide_claim(root, "item-1", now_ms) {
            ClaimDecision::Claimable(item) => item,
            _ => panic!("expected Claimable"),
        };
        match decide_claim(root, "item-1", now_ms) {
            ClaimDecision::Claimable(_) => {}
            _ => panic!("a read-only decide_claim must not mutate state between calls"),
        }

        // Applying the claim (the write half) succeeds.
        let claimed = apply_claim(root, &claimable_item, "item-1", Some("owner-a"), 3600.0);
        assert_eq!(claimed.claim.unwrap().owner.as_deref(), Some("owner-a"));

        // A second claimant's decision now sees CLAIMED (lease not expired,
        // owner has no session record so its heartbeat reads stale — but the
        // lease is still live, and the dual condition requires BOTH).
        match decide_claim(root, "item-1", rsv::now_ms()) {
            ClaimDecision::Claimed(claim) => assert_eq!(claim.owner.as_deref(), Some("owner-a")),
            other => panic!(
                "expected Claimed, got a different outcome (variant index {})",
                match other {
                    ClaimDecision::Claimable(_) => 0,
                    ClaimDecision::NotFound => 1,
                    ClaimDecision::Completed => 2,
                    ClaimDecision::Claimed(_) => 3,
                }
            ),
        }

        // Unknown id.
        match decide_claim(root, "no-such-item", rsv::now_ms()) {
            ClaimDecision::NotFound => {}
            _ => panic!("expected NotFound"),
        }
    }
}
