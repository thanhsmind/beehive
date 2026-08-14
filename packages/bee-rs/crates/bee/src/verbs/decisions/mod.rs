// bee decisions — native port of the `decisions` verb group.
//
// Ported argv shapes (everything else returns None BEFORE any output and the
// whole command re-runs under Node):
//   decisions active  [--recent N] [--tag T] [--scope S|--area S] [--since D]
//                     [--all] [--untagged] [--cell C] [--feature F] [--json]
//   decisions search  [--text T] [--tag T] [--scope S|--area S] [--since D]
//                     [--all] [--untagged] [--cell C] [--feature F] [--json]
//   decisions log     --decision D --rationale R [--alternatives A]
//                     [--scope S] [--source S] [--confidence N] [--tags T]
//                     [--json]
//   decisions tag     --target ID --tags T [--scope S] [--json]
//   decisions tag     --stdin [--json]        (the JSON batch protocol)
//   decisions redact  --id ID --reason R [--json]
//   decisions archive --before ISO [--json]
//   decisions render  [--all] [--check] [--json]
//   decisions supersede --id ID --decision D --rationale R [--tags T]
//                     [--scope S] [--json]
//
// `decisions tag --stdin` USED to be delegated: a probe had to choose native
// vs Node before the pipe was consumed, and it could not choose without
// reading it. With no Node to choose, the batch is read and validated here —
// `flags.stdin === true` strictly, then handleDecisionsTag's two exact
// refusals ("input is not valid JSON." / "input must be a JSON array of
// {target, tags, scope?}.") and tagDecisionsBatch's per-row validation. Every
// remaining delegate trigger (argv shape, `prelude`'s root resolution) is
// settled BEFORE the read, as verbs/cells.rs run_add/run_update already do.
//
// Still delegating: any unknown flag, missing required flag, or --help, all
// before any output. `supersede` additionally delegates when a superseded id
// leaves the calibrated region — see run_supersede's ASCII guard
// (localeCompare collation over free prose, deliberately out of the
// corrupt-JSON cutover's scope). `render` no longer has such a guard:
// retire-collation-guard D1 removed `collation_safe`, since the comparator it
// gated (`lc_primary_key`) is total and the Node oracle it preserved parity
// with is gone from this build. `render` can still return Ex through
// `active_decisions`/`build_tag_overlay` on a null event or an inconsistent
// date comparator — that family is filed, not fixed (D3).
//
// Provenance: bee.mjs handleDecisionsLog/Active/Search/Tag/Redact/Archive/
// Supersede/Render + lib/decisions.mjs supersedeDecision/
// sweepDecisionCitations/collectSweepFiles/escapeRegExp/
// buildDecisionIndexBody/formatIndexLine/decisionIndexContent/
// renderDecisionIndex/decisionIndexDrift/writeTextAtomic +
// lib/capture.mjs addCaptureStub/normalizeList/assertSafeContent +
// filterDecisionEvents/matchesWholeToken/resolveScopeFilter/
// resolveSinceFilter/formatDecision/splitList, lib/decisions.mjs
// (SECRET_CONTENT_PATTERNS/INJECTION_PATTERNS/assertSafe/normalizeTags/
// TAG_PATTERN/classifyDecisionTags/loadTaxonomy/
// appendTaxonomyCandidatesSync/logDecision/redactDecision/
// tagDecisionsBatch/resolveTagTarget/normalizeTagEventTags/
// decisionTargetCandidates/buildTagOverlay/applyTagOverlay/activeDecisions/
// archiveDecisions/writeJsonlAtomic/appendJsonlBatch/
// withDecisionsLockSync/DecisionsLockBusyError/datamark) and lib/fsutil.mjs
// readJsonl.
//
// Locking: every store write serializes on the SAME cross-process lock file
// Node uses — lock name "decisions" (decisions.mjs DECISIONS_LOCK_NAME),
// through crate::lock::acquire_store_lock_once wrapped in the same bounded
// 15-retry/20ms loop as withDecisionsLockSync (~300ms worst case), with the
// DecisionsLockBusyError message replicated byte-for-byte.
//
// The atomic-jsonl-rewrite primitives are ported faithfully: archive appends
// qualifying events to .bee/decisions-archive.jsonl FIRST, then rewrites the
// pruned active file via write_jsonl_atomic (unique tmp + rename, best-effort
// tmp cleanup on failure) — the same crash-ordering decisions.mjs documents.
//
// Regex-free matching: the secret/injection/datamark patterns are hand-ported
// scanners (no regex crate in this workspace). Word boundaries use JS \w
// ([A-Za-z0-9_]); case-insensitive comparisons are ASCII-folding for the
// ASCII literals the patterns contain (V8's canonicalize differs only on
// exotic non-ASCII case pairs, e.g. U+017F — accepted approximation, noted
// here). toLowerCase in filters uses Rust's Unicode lowercasing, which can
// differ from JS on a handful of special-cased code points — same class of
// documented approximation.
//
// CUTOVER (2026-08-01) — the corrupt-JSON delegations are gone:
//   * an unparseable JSONL line is SKIPPED (Node's own readJsonl behavior)
//     and reported once via crate::fsutil::warn_corrupt_jsonl_line. It used
//     to delegate because serde's grammar and V8's differ on lone-surrogate
//     escapes; nothing here can decode those, so they are corrupt.
//   * a corrupt taxonomy.json warns once and reads as "no taxonomy" —
//     `readJson(file, null)`'s own fallback, so classification stays
//     optional and `decisions log` takes its warn-only branch, unchanged.
//   * numbers >= 1e21 were retired upstream (jsjson::js_f64_to_string now
//     implements the full ECMA Number::toString).
// Delegation beyond argv shape that REMAINS: `null` events in the active
// store (a JS property-access crash in activeDecisions' default branch),
// non-string/non-ISO date values wherever Date.parse runs, and mixed
// finite/NaN dates feeding a sort comparator (V8's TimSort with an
// inconsistent comparator is unspecified) — none of these is a V8-text
// matter.








use std::path::{Path, PathBuf};




const DECISIONS_LOCK_NAME: &str = "decisions";

const DECISIONS_LOCK_RETRY_ATTEMPTS: u32 = 15;

const DECISIONS_LOCK_RETRY_DELAY_MS: u64 = 20;

fn decisions_path(root: &Path) -> PathBuf {
    root.join(".bee").join("decisions.jsonl")
}

fn decisions_archive_path(root: &Path) -> PathBuf {
    root.join(".bee").join("decisions-archive.jsonl")
}

fn taxonomy_path(root: &Path) -> PathBuf {
    root.join("docs").join("decisions").join("taxonomy.json")
}

// ─── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;

mod store;
mod scanners;
mod read;
mod verbs_read;
mod verbs_write;
mod render;
mod supersede;
pub(crate) use self::store::*;
pub(crate) use self::scanners::*;
pub(crate) use self::read::*;
pub(crate) use self::verbs_read::*;
pub(crate) use self::verbs_write::*;
pub(crate) use self::render::*;
pub(crate) use self::supersede::*;
