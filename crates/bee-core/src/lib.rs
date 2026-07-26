//! bee-core: shared library for the queen-bee Rust port.
//!
//! Slice 0 scope: empty stub. Storage core (fsutil semantics + D9 lock
//! protocol) lands in a later cell of this slice.

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
