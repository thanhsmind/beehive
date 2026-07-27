//! registry — loader for the JSON bridge produced by
//! `scripts/dump_command_registry.mjs` (rust-port-8), which snapshots
//! `.bee/bin/lib/command-registry.mjs`'s `COMMAND_REGISTRY` (116 defs, 19
//! group prefixes at scout time — CONTEXT.md, D7's flip-unit list) to
//! `.bee/cache/command-registry.json` alongside a `source_sha256` of the
//! mjs file it was generated from.
//!
//! `command-registry.mjs` is FROZEN for the duration of the rust-port
//! feature (D1). This loader NEVER re-derives the registry itself and NEVER
//! spawns the dump script — it only reads the JSON the dump script already
//! wrote, and separately re-hashes the live `command-registry.mjs` bytes to
//! decide freshness. A stale dump (source file edited since the dump ran)
//! surfaces as a typed [`Registry::Stale`] result — this loader never
//! silently validates CLI shape against a registry snapshot that no longer
//! matches the source of truth (must-have: "registry lookup serves all 116
//! entries and flags staleness when command-registry.mjs sha differs").
//!
//! Delivery note (decision 2026-07-26, cell rust-port-8): at final flip this
//! JSON becomes a release/onboard-generated managed artifact — this loader
//! shape is final; only the producer moves.

use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// One `COMMAND_REGISTRY` entry, byte-shape-identical to
/// `command-registry.mjs`'s own header comment: "only {name, invoke,
/// description, parameters, examples, deprecated} make up an entry" — a
/// closed, not flattened, shape (unlike the store readers elsewhere in this
/// crate) because the mjs source itself documents this as the exact and
/// only field set every entry carries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandRegistryEntry {
    pub name: String,
    pub invoke: String,
    pub description: String,
    /// JSON-Schema shaped exactly like Claude Code's own tool definitions
    /// (`{type, properties, required}`, per command-registry.mjs's header
    /// comment) — kept as raw JSON rather than a hand-modeled schema type
    /// since bee-core has no reason to interpret JSON-Schema itself.
    pub parameters: Value,
    pub examples: Vec<String>,
    /// `null` for every entry at scout time; a future deprecation note is a
    /// string. Kept as raw JSON so either shape round-trips.
    pub deprecated: Value,
}

/// The `.bee/cache/command-registry.json` document shape written by
/// `scripts/dump_command_registry.mjs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRegistryDump {
    /// sha256 (lowercase hex) of `.bee/bin/lib/command-registry.mjs`'s bytes
    /// at the moment the dump ran — the staleness anchor.
    pub source_sha256: String,
    pub generated_at: String,
    pub entries: Vec<CommandRegistryEntry>,
}

impl CommandRegistryDump {
    /// Look up an entry by its exact `name` (e.g. `"cells.update"`).
    pub fn find(&self, name: &str) -> Option<&CommandRegistryEntry> {
        self.entries.iter().find(|e| e.name == name)
    }
}

/// Outcome of [`load_registry`]: either the dump is fresh (its embedded
/// `source_sha256` still matches the live source file), or it is stale and
/// the caller must regenerate it — this loader itself never regenerates.
#[derive(Debug)]
pub enum Registry {
    Fresh(CommandRegistryDump),
    Stale {
        dump: CommandRegistryDump,
        embedded_sha256: String,
        live_sha256: String,
    },
}

impl Registry {
    /// The dump payload either way — a stale registry is still readable,
    /// just flagged; callers that need to refuse on staleness check the
    /// variant explicitly.
    pub fn dump(&self) -> &CommandRegistryDump {
        match self {
            Registry::Fresh(d) => d,
            Registry::Stale { dump, .. } => dump,
        }
    }

    pub fn is_fresh(&self) -> bool {
        matches!(self, Registry::Fresh(_))
    }
}

fn sha256_hex_of_file(path: &Path) -> io::Result<String> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

/// Loads `dump_path` (the JSON `scripts/dump_command_registry.mjs` wrote)
/// and compares its embedded `source_sha256` against a fresh hash of
/// `source_mjs_path` (`.bee/bin/lib/command-registry.mjs`). Returns
/// [`Registry::Fresh`] when they match, [`Registry::Stale`] when they
/// don't — never panics or silently drops the mismatch.
pub fn load_registry(dump_path: &Path, source_mjs_path: &Path) -> io::Result<Registry> {
    let text = fs::read_to_string(dump_path)?;
    let dump: CommandRegistryDump = serde_json::from_str(&text)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let live_sha256 = sha256_hex_of_file(source_mjs_path)?;
    if live_sha256 == dump.source_sha256 {
        Ok(Registry::Fresh(dump))
    } else {
        let embedded_sha256 = dump.source_sha256.clone();
        Ok(Registry::Stale { dump, embedded_sha256, live_sha256 })
    }
}


// Tests live in crates/bee-core/tests/guard_support.rs (this cell's single
// integration target — cargo test -p bee-core --test guard_support) rather
// than here, so every reader's round-trip/logic proof sits in one place
// per must-have.
