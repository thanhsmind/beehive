//! bee-core: shared library for the queen-bee Rust port.
//!
//! Slice 0 scope: fsutil storage primitives (rust-port-5) and the D9
//! cross-process lock protocol (rust-port-3), oracle/interop-checked against
//! the frozen mjs layer.

pub mod fsutil;
pub mod lock;

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
