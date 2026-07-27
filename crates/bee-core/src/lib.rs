//! bee-core: shared library for the queen-bee Rust port.
//!
//! Slice 0 scope: fsutil storage primitives (rust-port-5) and the D9
//! cross-process lock protocol (rust-port-3), oracle/interop-checked against
//! the frozen mjs layer. rust-port-8 adds the guard-support readers
//! (config, state, cells, reservations/leases, holds, workspace, claims),
//! the command-registry JSON bridge loader, and the tokenizer port — see
//! each module's own doc comment for its mjs source of truth.

pub mod backlog;
pub mod capture;
pub mod cells;
pub mod claims;
pub mod config;
pub mod decisions;
pub mod dispatch_guard;
pub mod fsutil;
pub mod guards;
pub mod holds;
pub mod intent;
pub mod jsdate;
pub mod lock;
pub mod read_accounting;
pub mod recovery;
pub mod registry;
pub mod reservations;
pub mod reviews;
pub mod shared_reads;
pub mod source_identity;
pub mod state;
pub mod state_projection;
pub mod tokenize;
pub mod validate_args;
pub mod workflow_store;
pub mod workspace;

/// Crate version string, re-exported so consumers (e.g. `queen-bee
/// --version`) don't hardcode it in two places.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_non_empty() {
        assert!(!VERSION.is_empty());
    }
}
