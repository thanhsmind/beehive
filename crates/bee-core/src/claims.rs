//! claims — `readSession`/`heartbeatStale` port over `.bee/sessions/*.json`,
//! from `.bee/bin/lib/claims.mjs` (rust-port-8, CONTEXT.md D3). Read-only,
//! zero subprocess (D5): this module never resolves a session id from
//! env/flags, touches a heartbeat, or claims/releases anything — only the
//! two read-time checks guards.mjs's `checkWrite` imports.
//!
//! `.bee/bin/lib/claims.mjs` is FROZEN for the duration of the rust-port
//! feature (D1).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::fsutil::read_json;
use crate::jsdate::parse_iso_ms;

/// `DEFAULT_HEARTBEAT_STALE_SECONDS`.
pub const DEFAULT_HEARTBEAT_STALE_SECONDS: i64 = 900;

pub fn sessions_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("sessions")
}

pub fn claims_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("claims")
}

/// `sessionPath(root, sessionId)`. Note this reader does not replicate
/// `requireId`'s validation-and-throw (`readSession` itself catches that
/// and returns `null` on a malformed id — the "fail-open: a malformed id
/// reads as 'no session'" comment on `readSession` below) — a caller here
/// gets `None` from [`read_session`] for the same malformed-id case rather
/// than a panic.
pub fn session_path(root: &Path, session_id: &str) -> PathBuf {
    sessions_dir(root).join(format!("{session_id}.json"))
}

pub fn claim_path(root: &Path, cell_id: &str) -> PathBuf {
    claims_dir(root).join(format!("{cell_id}.json"))
}

/// `.bee/sessions/<id>.json`: `{id, started_at, last_heartbeat, ...}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub last_heartbeat: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// `readSession(root, sessionId)`: `None` when the record is missing,
/// malformed, or its own `id` field doesn't match the requested id
/// (mirrors mjs's `session.id !== String(sessionId).trim()` guard, which
/// catches a stale/misnamed file). A blank/empty `session_id` is likewise
/// `None` (mirrors the malformed-id fail-open branch).
pub fn read_session(root: &Path, session_id: &str) -> Option<Session> {
    let trimmed = session_id.trim();
    if trimmed.is_empty() {
        return None;
    }
    let raw: Value = read_json(&session_path(root, trimmed), Value::Null);
    if !raw.is_object() {
        return None;
    }
    let session: Session = serde_json::from_value(raw).ok()?;
    if session.id != trimmed {
        return None;
    }
    Some(session)
}

/// `listSessionRecords(root)`: every readable `.bee/sessions/*.json`
/// record. Fail-open per file and for a missing directory (empty list),
/// same posture as `readSession`.
pub fn list_session_records(root: &Path) -> Vec<Session> {
    let dir = sessions_dir(root);
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut sessions = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if let Ok(session) = serde_json::from_str::<Session>(&text) {
            sessions.push(session);
        }
    }
    sessions
}

/// `heartbeatStale(session, nowMs, staleSeconds)`: `true` when the
/// session is absent/malformed, has no parseable `last_heartbeat`, or the
/// heartbeat is older than `stale_seconds`.
pub fn heartbeat_stale(session: Option<&Session>, now_ms: i64, stale_seconds: i64) -> bool {
    let session = match session {
        Some(s) => s,
        None => return true,
    };
    let beat_ms = match session.last_heartbeat.as_deref().and_then(parse_iso_ms) {
        Some(ms) => ms,
        None => return true,
    };
    beat_ms + stale_seconds * 1000 <= now_ms
}


// Tests live in crates/bee-core/tests/guard_support.rs (this cell's single
// integration target — cargo test -p bee-core --test guard_support) rather
// than here, so every reader's round-trip/logic proof sits in one place
// per must-have.
