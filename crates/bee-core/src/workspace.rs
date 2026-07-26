//! workspace — reader for `.bee/runtime/workspaces/<id>.json` records,
//! ported from `.bee/bin/lib/workspace-store.mjs`'s `readWorkspace`/
//! `workspacePath` (rust-port-8, CONTEXT.md D3). Read-only, zero
//! subprocess (D5): this module never registers, attaches, or mutates a
//! workspace record.
//!
//! `.bee/bin/lib/workspace-store.mjs` is FROZEN for the duration of the
//! rust-port feature (D1).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::fsutil::read_json;

pub fn runtime_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("runtime")
}

pub fn workspaces_dir(root: &Path) -> PathBuf {
    runtime_dir(root).join("workspaces")
}

/// `workspacePath(root, id)` — note this reader does not replicate
/// `requireWorkspaceId`'s validation-and-throw; an id containing path
/// separators or otherwise unsafe characters is the CALLER's concern
/// (this is a read-only reader, never a write path).
pub fn workspace_path(root: &Path, id: &str) -> PathBuf {
    workspaces_dir(root).join(format!("{id}.json"))
}

/// `.bee/runtime/workspaces/<id>.json`:
/// `{id, type, root, branch, base_sha, write_owner_session, fence_epoch,
/// attached_sessions, created_at}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub workspace_type: String,
    pub root: String,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub base_sha: Option<String>,
    #[serde(default)]
    pub write_owner_session: Option<String>,
    #[serde(default)]
    pub fence_epoch: Value,
    #[serde(default)]
    pub attached_sessions: Vec<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// `readWorkspace(root, id)`: `None` when the record is missing or
/// malformed — fail-open, matching workspace-store.mjs's `readWorkspaceRecord`
/// (a caller that needs the typed `WorkspaceStoreError` refusal instead
/// should stay on the mjs path for now; this reader only serves the
/// guard-support "does a workspace record exist and what does it say"
/// checks named by this cell).
pub fn read_workspace(root: &Path, id: &str) -> Option<WorkspaceRecord> {
    let raw: Value = read_json(&workspace_path(root, id), Value::Null);
    if !raw.is_object() {
        return None;
    }
    serde_json::from_value::<WorkspaceRecord>(raw).ok()
}


// Tests live in crates/bee-core/tests/guard_support.rs (this cell's single
// integration target — cargo test -p bee-core --test guard_support) rather
// than here, so every reader's round-trip/logic proof sits in one place
// per must-have.
