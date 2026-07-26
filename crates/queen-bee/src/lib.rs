//! queen-bee: the single compiled binary that replaces bee.mjs + hooks in
//! host repos (CONTEXT.md D2). This crate ships both a `[lib]` and a
//! `[[bin]]` target so `crates/queen-bee/tests/*.rs` integration tests can
//! call the ported hook runtime directly (e.g. the adapter's output-encoding
//! functions) as well as drive the compiled binary as a subprocess for the
//! full stdin/exit-code/side-effect fixtures (rust-port-7, D7b conformance
//! rig).
//!
//! `adapter` is a line-by-line port of `.bee/bin/hooks/adapter.mjs` (frozen
//! per D1 — ported, never "improved"). `hookconfig` replicates the narrow
//! slice of `.bee/bin/lib/state.mjs`'s `readConfig`/`hookEnabled` that the
//! hook runtime needs. `hooks` holds the individual ported hook handlers.

pub mod adapter;
pub mod hookconfig;
pub mod hooks;
