//! Tell cargo that this crate's compiled output depends on the plugin
//! manifest.
//!
//! `src/version.rs` embeds `.claude-plugin/plugin.json` with `include_str!`,
//! and that file lives OUTSIDE this package. Cargo fingerprints such a path
//! unreliably: on a release bump it rebuilt the integration test but not the
//! `bee` bin (or the reverse, run to run), so `bee version` and the manifest
//! disagreed and `scripts/release.sh`'s test gate refused a release that was
//! in fact fine — twice, in opposite directions, which is what pointed at the
//! fingerprint rather than at either target.
//!
//! One `rerun-if-changed` makes the dependency explicit, so a version bump
//! rebuilds what embeds it. Declared here rather than worked around in the
//! release script, because CI and every developer's incremental build have
//! the same hole.
//!
//! The path is relative to this package's root
//! (`packages/bee-rs/crates/bee/`), which is where cargo resolves it from.

fn main() {
    println!("cargo:rerun-if-changed=../../../../.claude-plugin/plugin.json");
}
