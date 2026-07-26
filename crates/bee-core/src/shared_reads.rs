//! shared_reads — the PER-INVOCATION shared-read memo (rust-port-23,
//! CONTEXT.md D3/D5, decision e119fc8b). One load point per store per
//! invocation, supplied by the CALLER — the same
//! "caller-supplied, never re-derived inside bee-core" convention
//! rust-port-16 and rust-port-20 established for `control_root` and the
//! resolved session id, applied to the three stores `build_status` used to
//! re-read four, six and two times respectively.
//!
//! ## Why a memo and not an eagerly-loaded struct (advisor note 1)
//!
//! Every field below is a [`OnceCell`], filled on FIRST USE by calling the
//! ordinary reader for that store — never at construction. That is not a
//! micro-optimization, it is the correctness property the hook path
//! depends on: `chain_nudge` calls [`crate::cells::scribing_debt`], which
//! returns before touching any cell when no feature resolves, and
//! `bee-chain-nudge` runs on EVERY SubagentStop event. An eagerly-loaded
//! shared struct would turn that conditional scan into an unconditional
//! one on every hook event, with every correctness test still green —
//! rust-port-22 baselined both hook entry points (`crates/queen-bee/tests/
//! read_accounting.rs`) precisely so that regression cannot ship quietly.
//! A lazy memo preserves every existing gate by construction: a consumer
//! that never runs never triggers a read.
//!
//! ## Why the loaders call the ordinary readers (rust-port-22's judge)
//!
//! `cells()` calls [`crate::cells::list_cells`]; `decisions()` calls
//! [`crate::decisions::active_decisions`] (which funnels through
//! [`crate::fsutil::read_jsonl`]); `capture_events()` calls `read_jsonl`.
//! Every read-accounting counter lives inside those functions, NOT at the
//! filesystem — a loader that reimplemented one (a raw `fs::read_dir` of
//! `.bee/cells`, a direct `read_to_string` of the journal) would be
//! invisible to the instrument and the "1 per store" proof would be false.
//! The rule for any future field here: load THROUGH the existing reader,
//! never around it.
//!
//! ## Per invocation, never wider (plan-slice3.md, REJECTED alternatives)
//!
//! This type is created by the caller, borrowed for the length of one
//! invocation, and dropped. There is no process-global cache, no lazily
//! initialized static, and no on-disk cache: these stores change on every
//! bee write, so the staleness guard that makes `reviews.rs`'s on-disk
//! cache safe does not exist here. `reviews::GitMemo` (a pass-local memo)
//! is the precedent this extends upward. [`OnceCell`] rather than
//! `OnceLock` is deliberate — it makes the "one invocation, one thread"
//! scope a type-level fact (`SharedReads` is `!Sync`), matching the
//! thread-local read counters it is measured by.
//!
//! ## What this does and does NOT guarantee (validation W5, advisor note 3)
//!
//! Each store is read at ONE instant per invocation; cross-store
//! consistency remains unguaranteed, exactly as today. This is NOT
//! snapshot semantics: decisions, cells and transcript roots are still
//! loaded at distinct moments, and a journal read racing an in-flight
//! append is still possible. `status` takes no D9 locks, so the lock and
//! lease conformance surface is untouched.

use std::cell::OnceCell;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::capture::capture_queue_path;
use crate::cells::{self, Cell};
use crate::decisions::active_decisions;
use crate::fsutil::read_jsonl;

/// One invocation's shared view of the stores that were being re-read.
///
/// Construct one per invocation ([`SharedReads::new`]), pass it by
/// reference to every reader that accepts one, and drop it when the
/// invocation ends.
pub struct SharedReads {
    root: PathBuf,
    decisions: OnceCell<Vec<Value>>,
    capture_events: OnceCell<Vec<Value>>,
    cells: OnceCell<Vec<Cell>>,
}

impl SharedReads {
    /// A fresh, empty memo for `root`. Reads nothing — every store is
    /// loaded on first use.
    pub fn new(root: &Path) -> Self {
        SharedReads {
            root: root.to_path_buf(),
            decisions: OnceCell::new(),
            capture_events: OnceCell::new(),
            cells: OnceCell::new(),
        }
    }

    /// The store root this memo was built for.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// `activeDecisions(root, {})` — the full active list, newest first.
    /// A caller wanting only the newest N takes them off the front (that
    /// is exactly what `active_decisions`'s own `recent` argument does),
    /// rather than asking for a second, differently-truncated read.
    pub fn decisions(&self) -> &[Value] {
        self.decisions.get_or_init(|| active_decisions(&self.root, None, false))
    }

    /// `.bee/capture-queue.jsonl`'s raw events.
    pub fn capture_events(&self) -> &[Value] {
        self.capture_events.get_or_init(|| read_jsonl::<Value>(&capture_queue_path(&self.root)))
    }

    /// `listCells(root)` — the ACTIVE-ONLY inventory (`.bee/cells/*.json`,
    /// archive subdirectory skipped), in directory order.
    ///
    /// # This inventory is not a dependency resolver
    ///
    /// [`crate::cells::read_cell`] falls back to
    /// `.bee/cells/archive/<feature>/<id>.json` when a cell is absent from
    /// the active directory, and `list_cells` skips `archive/` entirely.
    /// Resolving a cell dependency against THIS slice would therefore read
    /// an archived, capped dep as uncapped and silently drop ready cells —
    /// with byte-parity still green, because no parity fixture has an
    /// archived dep. Dependency resolution keeps going through `read_cell`
    /// (see [`crate::cells::ready_cells_from`]); this inventory is for
    /// listing, filtering and counting only.
    pub fn cells(&self) -> &[Cell] {
        self.cells.get_or_init(|| cells::list_cells(&self.root))
    }
}
