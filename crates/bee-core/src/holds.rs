//! holds — reader + `findForeignHolds`/`holdsStoreCorrupt` port for the
//! cross-worktree holds ledger, from `.bee/bin/lib/worktree-holds.mjs`
//! (rust-port-8, CONTEXT.md D3). Read-only, zero subprocess (D5): this
//! module never mirrors, releases, sweeps, or renews a hold — only the two
//! read-time checks guards.mjs's `checkWrite` imports.
//!
//! `.bee/bin/lib/worktree-holds.mjs` is FROZEN for the duration of the
//! rust-port feature (D1). The store always lives at the MAIN checkout's
//! `.bee/runtime/cross-worktree-holds.json` (never a worktree's own
//! `.bee/`) — callers pass `main_root` explicitly, mirroring the mjs
//! source's own `mainRoot` parameter naming.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::fsutil::read_json;
use crate::jsdate::parse_iso_ms;

pub fn holds_ledger_path(main_root: &Path) -> PathBuf {
    main_root.join(".bee").join("runtime").join("cross-worktree-holds.json")
}

/// One row of the cross-worktree holds ledger:
/// `{path, holder, feature, session, cell, ttl_seconds, mirrored_at,
/// released_at}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hold {
    pub path: String,
    pub holder: String,
    #[serde(default)]
    pub feature: Option<String>,
    #[serde(default)]
    pub session: Option<String>,
    #[serde(default)]
    pub cell: Option<String>,
    #[serde(default)]
    pub ttl_seconds: Option<Value>,
    #[serde(default)]
    pub mirrored_at: Option<String>,
    #[serde(default)]
    pub released_at: Option<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct HoldsStore {
    #[serde(default)]
    holds: Vec<Hold>,
}

/// Fail-open read: a missing/malformed store reads as an empty ledger,
/// matching worktree-holds.mjs's `readStore`.
fn read_store(main_root: &Path) -> Vec<Hold> {
    let raw: Value = read_json(&holds_ledger_path(main_root), Value::Null);
    if !raw.is_object() {
        return Vec::new();
    }
    serde_json::from_value::<HoldsStore>(raw).map(|s| s.holds).unwrap_or_default()
}

/// `normalizePath` — byte-identical to worktree-holds.mjs's private
/// helper (itself a duplicate of reservations.mjs's own, per that module's
/// header comment on deliberate small duplication for import-isolation).
/// `pub(crate)`: guards.rs (rust-port-9) reuses it for `isHardConflict`,
/// whose mjs source uses reservations.mjs's identical copy.
///
/// rust-port-9 fix (deviation, bug in touched code): the mjs regex
/// `/^\.\/+/` strips ONE leading `.` followed by a run of slashes — a
/// single anchored replace, not a loop — so `"././x"` normalizes to
/// `"./x"`, not `"x"`. The earlier `while strip_prefix("./")` loop here
/// over-stripped that (edge-case-only) spelling.
pub(crate) fn normalize_path(value: &str) -> String {
    let backslashes_replaced = value.replace('\\', "/");
    let no_leading_dot_slashes = match backslashes_replaced.strip_prefix("./") {
        Some(rest) => rest.trim_start_matches('/').to_string(),
        None => backslashes_replaced,
    };
    let mut chars: Vec<char> = no_leading_dot_slashes.chars().collect();
    while chars.last() == Some(&'/') {
        chars.pop();
    }
    chars.into_iter().collect()
}

/// `pathsOverlap(a, b)` — reused verbatim from reservations.mjs (imported
/// there by worktree-holds.mjs, cells.mjs, schedule.mjs, state.mjs alike);
/// re-derived here since bee-core has no reservations-module dependency
/// edge into this file beyond what `reservations.rs` already exposes
/// separately for the projection/lease readers.
pub fn paths_overlap(a: &str, b: &str) -> bool {
    let left = normalize_path(a);
    let right = normalize_path(b);
    if left.is_empty() || right.is_empty() {
        return false;
    }
    if left == right {
        return true;
    }

    let left_glob = left.ends_with('*');
    let right_glob = right.ends_with('*');
    let strip_glob_and_slash = |s: &str| -> String {
        let mut t = s.trim_end_matches('*').to_string();
        while t.ends_with('/') {
            t.pop();
        }
        t
    };
    let left_base = if left_glob { strip_glob_and_slash(&left) } else { left.clone() };
    let right_base = if right_glob { strip_glob_and_slash(&right) } else { right.clone() };

    if left_base == right_base {
        return true;
    }
    if left_base.is_empty() || right_base.is_empty() {
        return true; // bare "*" covers everything
    }
    left_base.starts_with(&format!("{right_base}/")) || right_base.starts_with(&format!("{left_base}/"))
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// `isExpired(entry, nowMs)`.
fn is_expired(hold: &Hold, now_ms: i64) -> bool {
    let ttl = match &hold.ttl_seconds {
        Some(Value::Number(n)) => n.as_f64(),
        _ => None,
    };
    let ttl = match ttl {
        Some(t) if t.is_finite() && t > 0.0 => t,
        _ => return false,
    };
    let mirrored_ms = match &hold.mirrored_at {
        Some(s) => match parse_iso_ms(s) {
            Some(ms) => ms,
            None => return false,
        },
        None => return false,
    };
    (mirrored_ms as f64 + ttl * 1000.0) <= now_ms as f64
}

/// `isActive(entry, nowMs)`.
fn is_active(hold: &Hold, now_ms: i64) -> bool {
    let released = !matches!(hold.released_at, None | Some(Value::Null));
    !released && !is_expired(hold, now_ms)
}

/// `findForeignHolds(mainRoot, holder, paths)`: active holds NOT owned by
/// `holder` whose `path` overlaps any of `paths`.
pub fn find_foreign_holds(main_root: &Path, holder: &str, paths: &[&str]) -> Vec<Hold> {
    let requested: Vec<&str> = paths.iter().copied().filter(|p| !p.is_empty()).collect();
    if requested.is_empty() {
        return Vec::new();
    }
    let acting = holder.trim();
    let now = now_ms();
    let store = read_store(main_root);
    store
        .into_iter()
        .filter(|hold| {
            is_active(hold, now)
                && hold.holder != acting
                && requested.iter().any(|p| paths_overlap(&hold.path, p))
        })
        .collect()
}

/// `holdsStoreCorrupt(mainRoot)`: `false` when the ledger file is absent
/// (missing store = today's open behavior) or valid JSON; `true` only when
/// the file exists and fails to parse.
pub fn holds_store_corrupt(main_root: &Path) -> bool {
    let file = holds_ledger_path(main_root);
    let text = match fs::read_to_string(&file) {
        Ok(t) => t,
        Err(_) => return false,
    };
    serde_json::from_str::<Value>(&text).is_err()
}


// Tests live in crates/bee-core/tests/guard_support.rs (this cell's single
// integration target — cargo test -p bee-core --test guard_support) rather
// than here, so every reader's round-trip/logic proof sits in one place
// per must-have.
