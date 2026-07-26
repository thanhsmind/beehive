//! cells — read-only listing over `.bee/cells/*.json`, ported from
//! `.bee/bin/lib/cells.mjs`'s `cellsDir`/listing semantics (rust-port-8,
//! CONTEXT.md D3). Read-only, zero subprocess (D5): never claims, caps, or
//! writes a cell file.
//!
//! `.bee/bin/lib/cells.mjs` is FROZEN for the duration of the rust-port
//! feature (D1). This module lists the id/status/worker/files shape the
//! guard-support panel named — every other field (title, action,
//! must_haves, verify, trace, tier, ...) survives round-trip via `extra`
//! (D3), so a consumer that needs a field this reader doesn't name
//! explicitly can still reach it without a second reader.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// `cellsDir(root)` — mirrors cells.mjs exactly.
pub fn cells_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("cells")
}

/// `ARCHIVE_DIR_NAME` — the one reserved child of `cellsDir` a default
/// listing must skip (cells.mjs's own comment: a directory entry already
/// fails the `.json` filter, but the guard stays explicit).
pub const ARCHIVE_DIR_NAME: &str = "archive";

/// One `.bee/cells/<id>.json` record. `id`, `status`, and `files` are
/// named explicitly (the guard-support panel's "id/status/worker/files");
/// `worker` lives nested under `trace.worker` in the real mjs shape, so it
/// is read via [`Cell::worker`] against `extra` rather than a top-level
/// field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub id: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Cell {
    /// `trace.worker` — the agent nickname currently holding this cell's
    /// claim, or `None` if unclaimed/absent.
    pub fn worker(&self) -> Option<&str> {
        self.extra.get("trace")?.get("worker")?.as_str()
    }

    pub fn lane(&self) -> Option<&str> {
        self.extra.get("lane")?.as_str()
    }

    pub fn feature(&self) -> Option<&str> {
        self.extra.get("feature")?.as_str()
    }
}

/// Lists every readable `.bee/cells/*.json` record (skipping the `archive`
/// subdirectory and any non-`.json` entry), matching cells.mjs's default
/// listing scope. Fail-open per entry: a corrupt/unreadable cell file is
/// skipped rather than failing the whole listing — same posture as this
/// crate's `fsutil::read_jsonl` per-line tolerance, applied per-file here
/// since each cell is its own file rather than a jsonl line.
pub fn list_cells(root: &Path) -> Vec<Cell> {
    let dir = cells_dir(root);
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut cells = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            continue; // skips `archive/` (and any other subdirectory)
        }
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if path.file_stem().and_then(|s| s.to_str()) == Some(ARCHIVE_DIR_NAME) {
            continue;
        }
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if let Ok(cell) = serde_json::from_str::<Cell>(&text) {
            cells.push(cell);
        }
    }
    cells
}


// Tests live in crates/bee-core/tests/guard_support.rs (this cell's single
// integration target — cargo test -p bee-core --test guard_support) rather
// than here, so every reader's round-trip/logic proof sits in one place
// per must-have.
