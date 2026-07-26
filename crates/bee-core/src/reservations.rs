//! reservations — readers for the same-session file-reservation stores,
//! ported from `.bee/bin/lib/reservations.mjs` and `.bee/bin/lib/
//! lease-store.mjs` (rust-port-8, CONTEXT.md D3). Read-only, zero
//! subprocess (D5).
//!
//! multisession-native-16 demoted `.bee/reservations.json` to a
//! rebuildable PROJECTION for legacy readers only — the live storage is
//! now sharded per-resource lease files under `.bee/runtime/leases/`
//! (`.bee/runtime/leases/cells/<cell-id>.json`,
//! `.bee/runtime/leases/paths/<path-hash>.json`). This module reads BOTH,
//! as the cell's panel named ("reservations.json plus sharded lease
//! records listing"): [`read_reservations_projection`] for the legacy
//! single-file shape, [`list_leases`] for the current sharded store.
//!
//! `.bee/bin/lib/reservations.mjs`/`lease-store.mjs` are FROZEN for the
//! duration of the rust-port feature (D1). Every struct round-trips
//! unknown fields via `extra` (D3).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::fsutil::read_json;

pub fn reservations_path(root: &Path) -> PathBuf {
    root.join(".bee").join("reservations.json")
}

pub fn leases_root(root: &Path) -> PathBuf {
    root.join(".bee").join("runtime").join("leases")
}

/// One row of the legacy `.bee/reservations.json` projection:
/// `{agent, cell, path, ttl_seconds, reserved_at, released_at, kind?}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reservation {
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub cell: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub ttl_seconds: Option<Value>,
    #[serde(default)]
    pub reserved_at: Option<String>,
    #[serde(default)]
    pub released_at: Option<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ReservationsStore {
    #[serde(default)]
    reservations: Vec<Reservation>,
}

/// Reads the legacy `.bee/reservations.json` projection. Fail-open: a
/// missing or malformed store reads as an empty list, matching
/// reservations.mjs's `readStore` posture (never a mutator's concern here —
/// this reader never rebuilds or writes the projection).
pub fn read_reservations_projection(root: &Path) -> Vec<Reservation> {
    let raw: Value = read_json(&reservations_path(root), Value::Null);
    if !raw.is_object() {
        return Vec::new();
    }
    serde_json::from_value::<ReservationsStore>(raw)
        .map(|s| s.reservations)
        .unwrap_or_default()
}

/// One `.bee/runtime/leases/{cells,paths}/<hash>.json` record:
/// `{resource, mode, workflow_id, session_id, workspace_id, epoch,
/// acquired_at, expires_at, kind}` (lease-store.mjs module header).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseRecord {
    /// Type-prefixed resource key: `"cell:<cell-id>"` or
    /// `"path:<canonical-path>"`.
    pub resource: String,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub workflow_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub epoch: Option<Value>,
    #[serde(default)]
    pub acquired_at: Option<String>,
    /// ISO timestamp, or `null` meaning "never expires".
    #[serde(default)]
    pub expires_at: Option<Value>,
    /// `'intent'` (advisory broad/glob scope) or `'lease'` (an exact path a
    /// writer is about to touch); defaults to `'lease'` when the source
    /// record omits it, matching lease-store.mjs.
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Lists every readable lease record under `.bee/runtime/leases/cells/`
/// and `.bee/runtime/leases/paths/` (both subdirectories, when present).
/// Fail-open per file and per missing subdirectory — a repo mid-migration
/// with only one of the two subdirs present reads the other as empty
/// rather than erroring.
pub fn list_leases(root: &Path) -> Vec<LeaseRecord> {
    let base = leases_root(root);
    let mut leases = Vec::new();
    for sub in ["cells", "paths"] {
        let dir = base.join(sub);
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let text = match fs::read_to_string(&path) {
                Ok(t) => t,
                Err(_) => continue,
            };
            if let Ok(record) = serde_json::from_str::<LeaseRecord>(&text) {
                leases.push(record);
            }
        }
    }
    leases
}


// Tests live in crates/bee-core/tests/guard_support.rs (this cell's single
// integration target — cargo test -p bee-core --test guard_support) rather
// than here, so every reader's round-trip/logic proof sits in one place
// per must-have.
