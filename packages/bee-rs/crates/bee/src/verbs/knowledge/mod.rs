// bee knowledge — native port of the knowledge verb group (bee.mjs
// handleKnowledgeCheck/Index/List/Context + lib/knowledge.mjs).
//
// Verbs served natively (exact argv shapes only — see the probe):
//   knowledge check   [--strict] [--json]
//   knowledge index   [--check] [--json]
//   knowledge list    [--type T] [--lifecycle L] [--area A] [--json]
//   knowledge context --work W (--budget N | --lane tiny|small|standard|high-risk) [--json]
//
//   knowledge promote --work W [--json]
//
// Nothing in this group is left permanently delegated. `promote` mines the
// capped cell traces of a work item and renders the delivery/area/pattern
// proposals; it NEVER writes (writes: [] is the contract) and its two typed
// refusals (missing_work / unknown_work) are deterministic, so both are
// native. Its extra delegation triggers: a .bee/cells/*.json file serde
// refuses but V8 might parse, and a trace `verification_evidence` string in
// the same class — BOTH RETIRED at the cutover below.
//
// Additional delegation triggers (None before any output/write):
//   - --help anywhere, unknown flags, non-flag tokens, validate()-failing
//     shapes (missing --work/--budget, non-numeric --budget, --strict=x, ...)
//   - a configured non-empty `product_root` (repo-divorce topology: Node's
//     resolveProductRoot warn/path semantics are not replicated here)
//   - corrupt .bee/config.json or state files (Node warns with V8 text)
//   - bundle file/dir names carrying chars >= U+E000 (JS sorts by UTF-16
//     code units; Rust by UTF-8 bytes — they disagree only across that range)
//   - --budget values outside the plain decimal/scientific grammar that JS
//     Number() also accepts (hex, Infinity, ...)
//   - any emitted value failing the JS number round-trip guard
//
// CUTOVER (2026-08-01) — the arms that existed only because Node's text
// would have carried V8/libuv bytes are native now:
//   - a frontmatter quoted scalar with a lone-surrogate escape (U+D800..
//     U+DFFF) is no longer "a shape only V8 could decide": it is an
//     undecodable quoted scalar, and takes the same bad_quoted_string
//     finding every other one takes.
//   - an unreadable bundle file mid-walk pushes checkBundle's own
//     `unreadable` finding and keeps walking (the Rust io message stands
//     where Node put the libuv one).
//   - JSON-looking text that serde refuses — a cell file in promote's walk,
//     a trace `verification_evidence` — takes Node's OWN catch branch
//     (silently skipped / kept as raw text) instead of delegating: with one
//     parser left, which branch ran is no longer in doubt.
//
// DIVERGENCE NOTES (documented, unreachable-different for real bee data):
//   - relevance scores use Rust's libm ln() vs V8's fdlibm port — equal for
//     all practical inputs, possibly one ulp apart in razor-edge ties, and
//     toFixed(6) here rounds half-to-even where JS toFixed rounds ties up
//     (binary doubles essentially never land on exact decimal midpoints).
//   - toLowerCase in relevance tokens uses Rust's Unicode lowercasing, which
//     can differ from JS on a handful of special-cased code points (same
//     accepted approximation decisions.rs documents).
//   - `knowledge index` write failures surface a Rust io message where Node
//     would print the V8 message (partial writes make delegation unsafe).
//
// Provenance: bee.mjs handleKnowledgeCheck/handleKnowledgeIndex/
// handleKnowledgeList/handleKnowledgeContext + resolveKnowledgeContextLaneBudget,
// lib/knowledge.mjs (CONCEPT_TYPES/PROFILE_REQUIRED/KEY_RE/RESERVED_BASENAMES/
// bundleDir/emitFrontmatter/parseFrontmatter/listBundleMarkdown/
// isIsoDateHeading/checkIndexFile/checkLogFile/readPath/resolveInsideBundle/
// checkBundle/collectConcepts/listConcepts/computeIndexFiles/
// knowledgeIndexDrift/renderKnowledgeIndexes/CONFUSABLE_FOLD/foldEncoding/
// normalizeSubject/
// CONTEXT_ESTIMATOR/estimateTokens/KNOWLEDGE_CONTEXT_LANE_BUDGETS/beeOf/dirOf/
// normalizeBundleTarget/CRITICAL_RELEVANCE/RELEVANCE_STOPWORDS/relevanceTokens/
// conceptBody/metaTextOf/scoreCriticalRelevance/buildContextManifest),
// lib/state.mjs resolveProductRoot (delegating branch only).
//
// This file also hosts the pub(crate) dispatch frame (GCtx / g_prelude)
// shared by the R3-wave-2 group files intent_group.rs / reviews.rs /
// tmp_group.rs — the same root → drift → emit/fail/timing shape
// reservations.rs's Ctx implements, with the no-root path keyed on the
// PRE-parse --json scan exactly like bee.mjs main().













#[cfg(test)]
mod tests;

mod frame;
mod frontmatter;
mod walk;
mod check;
mod index;
mod anchor;
mod context;
mod search;
mod routing;
mod promote;
mod bootstrap;
pub(crate) use self::frame::*;
pub(crate) use self::frontmatter::*;
pub(crate) use self::walk::*;
pub(crate) use self::check::*;
pub(crate) use self::index::*;
pub(crate) use self::anchor::*;
pub(crate) use self::context::*;
pub(crate) use self::search::*;
pub(crate) use self::routing::*;
pub(crate) use self::promote::*;
pub(crate) use self::bootstrap::*;
