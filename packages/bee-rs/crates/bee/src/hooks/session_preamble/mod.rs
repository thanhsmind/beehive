// session_preamble — native port of packages/bee/lib/inject.mjs's
// `buildSessionPreamble` (plus the three shared renderers it exports:
// onboardingLine, bypassBannerLines, handoffBlockLines, firstOpenGate).
//
// CUTOVER MODULE. Until the Node runtime was deleted this whole surface was
// `Outcome::Delegate` in hooks/session_init.rs — the preamble pulls
// essentially the entire vendored lib closure, and contract C2 demanded the
// bytes match Node's. C2 retired with Node, and a delegation with nowhere to
// delegate is a crash, so the renderer is native here.
//
// The one deliberate wording divergence, taken FOR the cutover: the closing
// line named the Node entry point (`node .bee/bin/bee.mjs status --json`).
// It now names the binary (`.bee/bin/bee status --json`), which is the only
// spelling that still exists. Same for the knowledge-context command
// (inject.mjs:260). The emitted preamble carries NO `.mjs` spelling anywhere;
// `no_mjs_spelling_survives_anywhere_in_the_preamble` pins that.
//
// ─── Ported inject.mjs surface ─────────────────────────────────────────────
//
//   buildSessionPreamble           -> build_session_preamble  (pub)
//   onboardingLine                 -> onboarding_line         (pub)
//   bypassBannerLines              -> bypass_banner_lines     (pub)
//   handoffBlockLines              -> handoff_block_lines     (pub)
//   firstOpenGate / visibleGates   -> first_open_gate / visible_gates (pub/priv)
//   gatesLine                      -> gates_line
//   knowledgeContextLines          -> knowledge_context_lines
//   knowledgeContextBudgetForMode  -> knowledge_context_budget_for_mode
//   projectMapLines / specProjectMapLines / bundleProjectMapLines
//   criticalPatternsDigest / legacy… / bundle…
//
// ─── Re-derived Rust (the "may not edit that file" rule) ───────────────────
//
// Every lib helper the preamble consumes is ALREADY ported in this tree, but
// each lives behind a private fn inside a module this cell may not touch
// (verbs/status_full.rs, verbs/knowledge.rs, verbs/decisions.rs,
// hooks/chain_nudge.rs, ...). Following the precedent verbs/drivers.rs set
// with its `kctx` lift, the helpers below are lifts of those ports — not
// second implementations — each carrying a provenance line naming BOTH the
// .mjs source and the Rust port it was lifted from. Only two things are
// reused directly, because they are already `pub` on a module this cell may
// edit: `crate::state::{bypass_level, ship_visibility}` and
// `crate::fsutil::{read_json, warn_corrupt_json}`.
//
// ─── Fail-open discipline (inject.mjs's whole posture) ─────────────────────
//
// "Orientation is never a place to fail a session." Every `try {} catch {}`
// in inject.mjs is preserved as a Rust fallback, and Node's
// `readJson(file, fallback)` warn-and-fall-back becomes
// `warn_corrupt_json(&file)` + the same fallback. This module is TOTAL: it
// has no delegate arm, no error type, and no panicking path.
//
// ─── Documented divergences (beyond the two command spellings) ─────────────
//
//   * A bundle whose filenames are not valid UTF-8, or sort differently under
//     UTF-16 than UTF-8, delegated in verbs/knowledge.rs. Here they are read
//     lossily and sorted by UTF-8 order — a preamble may not delegate.
//   * A frontmatter scalar carrying a lone-surrogate escape delegated there
//     too; here it reads as an unparseable concept (empty data), which is the
//     same direction collectConcepts already takes for an unreadable file.
//   * `pipelineRecord.route.flags.join(',')` throws in Node when `flags` is
//     not an array (only `state route --set`, which validates, writes that
//     field). Totality wins: a non-array renders `flags=undefined []`.
//   * resolveContext's WorktreeLinkInvalidError would propagate out of
//     resolvePipeline in Node and crash the hook. Here it falls back to the
//     given root — the same direction resolvePipeline already takes for every
//     other unresolvable binding.




use serde_json::{Map, Value};




type JMap = Map<String, Value>;

// ─── constants (state.mjs / cells.mjs / knowledge.mjs / backlog.mjs) ───────


/// state.mjs GATE_NAMES.
const GATE_NAMES: [&str; 4] = ["context", "shape", "execution", "review"];

/// state.mjs COMMAND_KEYS.
const COMMAND_KEYS: [&str; 4] = ["setup", "start", "test", "verify"];

/// state.mjs normalizeCommands' companion slots (normalized alongside, never
/// listed by the preamble — COMMAND_KEYS is what it iterates).
const WORKTREE_COMPANION_COMMAND_KEYS: [&str; 3] = [
    "worktree_companion_start",
    "worktree_companion_end",
    "worktree_companion_mount",
];

/// state.mjs NO_TEST_SENTINEL.
const NO_TEST_SENTINEL: &str = "none";

/// cells.mjs CEILING_MAX_SHARE / SCARCITY_MIN_TIERED.
const CEILING_MAX_SHARE: f64 = 0.4;

const SCARCITY_MIN_TIERED: i64 = 3;

/// cells.mjs MODEL_TIERS.
const MODEL_TIERS: [&str; 3] = ["extraction", "generation", "ceiling"];

/// inject.mjs NO_WORK_PHASES.
const NO_WORK_PHASES: [&str; 2] = ["idle", "compounding-complete"];

/// inject.mjs CRITICAL_PATTERNS_HEADING.
const CRITICAL_PATTERNS_HEADING: &str = "## Critical patterns";

/// inject.mjs PROJECT_MAP_FILES.
const PROJECT_MAP_FILES: [(&str, &str); 2] =
    [("system-overview.md", "System overview"), ("reading-map.md", "Reading map")];
/// knowledge.mjs KNOWLEDGE_CONTEXT_LANE_BUDGETS / _DEFAULT_BUDGET.
const KNOWLEDGE_CONTEXT_LANE_BUDGETS: [(&str, i64); 4] =
    [("tiny", 8000), ("small", 12000), ("standard", 20000), ("high-risk", 30000)];
const KNOWLEDGE_CONTEXT_DEFAULT_BUDGET: i64 = 20000;

/// backlog.mjs PBI_STATUSES / BACKLOG_STATUSES.
const PBI_STATUSES: [&str; 5] = ["proposed", "in-flight", "parked", "done", "declined"];

const BACKLOG_STATUSES: [&str; 3] = ["proposed", "in-flight", "done"];

// ─── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;

mod jsval;
mod state;
mod store;
mod knowledge;
mod render;
mod budget;
pub(crate) use self::jsval::*;
pub(crate) use self::state::*;
pub(crate) use self::store::*;
pub(crate) use self::knowledge::*;
pub(crate) use self::render::*;
pub(crate) use self::budget::*;
