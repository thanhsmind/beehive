//! The ported ledger command groups (slice 4, `rust-port-ledgers`).
//!
//! One module per registry group. Each exposes a single `group()` returning
//! its [`crate::dispatch::GroupDef`], and [`crate::groups::register_all`]
//! wires it in with ONE line — that is the whole seam contract from rpl-1:
//! `main.rs` and `dispatch.rs` do not change per group, so a red in a group
//! is a red in that group and never in the dispatcher.

pub mod capture;
pub mod intent;

use serde_json::Value;

use crate::dispatch::Call;

/// `bee.mjs:289` `requireFlag`. The refusal text is user-visible output —
/// `bee.mjs:7200` routes a thrown handler error straight into `emitError` —
/// so it is diffed byte-for-byte by the parity harness.
///
/// The three refused shapes are mjs's exactly: absent, the empty string, and
/// the boolean `true` (a flag written bare when the schema wanted a value).
pub(crate) fn require_flag(call: &Call, name: &str) -> Result<String, String> {
    match call.flag(name) {
        Some(Value::String(s)) if !s.is_empty() => Ok(s.clone()),
        _ => Err(format!("Missing required flag --{name}.")),
    }
}

/// The `flags.<name> ? String(flags.<name>) : null` idiom every intent
/// handler uses for its optional flags. JS truthiness, so the empty string is
/// `null` — not `Some("")`.
pub(crate) fn optional_flag(call: &Call, name: &str) -> Option<String> {
    match call.flag(name) {
        Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
        // Unreachable through the registry (no intent flag is flag-alone),
        // but `String(true)` is "true" and a port that silently dropped it
        // would diverge if one ever became flag-alone.
        Some(Value::Bool(true)) => Some("true".to_string()),
        _ => None,
    }
}
