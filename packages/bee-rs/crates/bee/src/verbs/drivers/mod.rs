// bee drivers — native port of the two porcelain DRIVER verbs: `bee dispatch
// prepare` (the worker-prompt / payload assembler) and `bee close` (the
// feature-close driver). Both are "drivers" in the porcelain sense: they own
// no store of their own, they compose other subsystems into one gesture.
//
// ─── Argv shapes served NATIVELY ───────────────────────────────────────────
//
//   close --feature <F> [--json]
//   close --feature <F> --dry-run [--json]
//   dispatch prepare --runtime <claude|codex> --kind <cell|gather|reviewer|advisor>
//                    [--cell <C>] [--worker <W>] [--force-ownership] [--json]
//                    [--claim [--session-id <S>]]
//
// Everything else returns None BEFORE any output, any lock, and any write, and
// the whole command re-runs under Node (campaign rule 1 — conservative argv
// routing; the dispatcher's validate()/nearest-match machinery is never
// reproduced here).
//
// ─── Shapes DELEGATED, and why ─────────────────────────────────────────────
//
//   * (CLOSED at R6) `dispatch prepare --claim` is now NATIVE. The blocker
//     was never the logic — both mutating doors were already ported, just
//     PRIVATE inside their own modules. They are now shared, exactly as
//     bee.mjs shares them, rather than re-derived (a second copy of ~1.5k
//     lines of store-mutation code is the precise drift C1 exists to
//     prevent):
//       - `cells::claim_cell_from_flags` — bee.mjs's own "one claim door for
//         cells.claim and dispatch.prepare --claim": the write-policy
//         resolution (enforceIsolation:false, so only observe /
//         shared-disjoint can act), the O_EXCL claims-store protocol with
//         fence epochs and session adoption, the per-cell store lock, the
//         budget checks, the byte-identical claim refusal and the route
//         soft-warning.
//       - `reservations::reserve_path_atomic` — the ONE reserve door, now
//         returning a TYPED verdict (ForeignHold / Conflicts / Reserved /
//         Thrown) so each caller renders its own text, mirroring bee.mjs's
//         "presentation stays with each caller" split.
//       - `reservations::release_reservations_for_agent` +
//         `cells::unclaim_cell` for the conflict UNWIND, run in reverse
//         order so the refusal can truthfully claim the repo is back in its
//         pre-call state.
//     Campaign rule 2 is honoured by front-loading: the claim door's own
//     prescans, the reserve prechecks, and a dry-run `prepare_dispatch` pass
//     all run BEFORE the first O_EXCL write, so nothing delegates after a
//     lock attempt. With `--claim` that dry-run pass is a DELEGATION PROBE
//     ONLY — bee.mjs sequences the claim BEFORE the payload build, so its
//     verdict over the still-unclaimed cell is discarded.
//     Still delegated inside this branch: a non-`true` `--claim` spelling
//     and a bare `--session-id` (both `String(true)`-shaped, unproven), and
//     — unchanged — `--session-id` WITHOUT `--claim`, which bee.mjs
//     documents as ignored.
//
//   * `dispatch prepare --runtime codex` on a host that HAS a
//     `.bee/native-transport-probe.json` whose `schema` matches. Beyond that
//     point readNativeTransportClassification shells out to `codex --version`
//     and `codex features list` and hashes ~/.codex config scope — process
//     probes with no native port. An ABSENT / unparseable / schema-mismatched
//     probe record short-circuits to `native_budget_only` with no subprocess
//     at all, which IS ported (the overwhelmingly common host shape).
//
//   * `dispatch prepare --stdin`-family and every unknown/missing/invalid
//     flag, `--help` anywhere, non-UTF-8 argv, stray positionals: Node's
//     validate()/emitError machinery owns those bytes.
//
//   * (CLOSED) `close --feature <F>` when `.bee/lanes/<F>.json` exists no
//     longer delegates. The lane record's own `last_scribing_run` now joins
//     scribingDebt's threshold (close.rs `best_scribing_stamp_ms`, reusing
//     `workflow_store::read_lane_display` — the same fail-open display read
//     `status`'s own port already uses) and a corrupt one warns, naming the
//     path, instead of throwing. Workflow records are still NOT part of this
//     path — nothing close reads (readState / listCells / captureQueue /
//     readConfig) consults `.bee/runtime/workflows/`.
//
//   * Linked-worktree roots (crate::roots -> NeedsNode). This also makes
//     bee.mjs's `grantedWorktreeContext()` provably `null` on every shape this
//     file serves, so close's merge-back line is never rendered natively —
//     the granted-worktree branch is Node's.
//
//   * `dogfood_repos` entries (normalizeDogfoodRepos console.warns per dead
//     repo), a configured non-empty `product_root` (repo-divorce topology),
//     and the remaining shapes the lifted knowledge helpers flag (bundle-path
//     normalization, collation-sensitive ranking — see their provenance
//     banner below).
//
//     CUTOVER (2026-08-01): corrupt JSON on a read path is NO LONGER one of
//     them. `rj` (readJson(file, null)) warns once via
//     crate::fsutil::warn_corrupt_json and returns the `null` fallback, so
//     every caller treats the record exactly as it treats an absent one —
//     which is what Node's own `!cell` / `?? null` guards did with that
//     fallback. A corrupt `.bee/config.json` likewise reads as "no config"
//     inside crate::state::read_config_raw, so `declared_test_commands`
//     reports tests undeclared instead of routing the command to Node. And a
//     lone-surrogate escape in a quoted frontmatter scalar — the one shape
//     only V8 could decode — is now the typed `bad_quoted_string` failure
//     every other undecodable quoted scalar already was.
//
//   * `close` without a POSIX shell on win32 (Node falls back to cmd.exe).
//
// ─── Provenance (every ported lib function) ────────────────────────────────
//
//   bee.mjs            handleDispatchPrepare (~7495), handleClose (~7643),
//                      renderTestCommandLines (~7601), buildCloseReportDoors,
//                      renderCloseDoorLines, CLOSE_TESTS_UNDECLARED_DETAIL,
//                      requireFlag, grantedWorktreeContext,
//                      readNativeTransportClassification (delegating slice),
//                      nativeTransportProbePath, doctorSafeReadJson,
//                      emit/emitError/main's dispatch frame.
//   lib/dispatch-prepare.mjs
//                      DISPATCH_RUNTIMES/DISPATCH_KINDS, slotForKind,
//                      purposeForKind, checkCellClaimOwnership, oneLine,
//                      PRIOR_ROUNDS_MAX_EVENT_LINES, priorRoundEventLines,
//                      LEARNED_CONTEXT_MAX_LINES, bundleLearnedLines,
//                      learnedContextLines, cellPromptBody, promptBodyFor,
//                      appendPrepareRecord, prepareDispatch.
//   lib/prompt-renderer.mjs
//                      loadPrompt, render (the C4 byte-identity pin).
//   lib/dispatch-guard.mjs
//                      PINNED_AGENT_TYPE, deriveEconomics,
//                      NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE,
//                      NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY.
//   lib/state.mjs      DEFAULT_MODELS, EFFORT_LEVELS, RUNTIMES,
//                      CONFIGURABLE_SLOTS, MODEL_NORMALIZE_SLOTS,
//                      normalizeTierValue, normalizeModels, nativeResolved,
//                      resolveTier, resolveAdvisor, readState (the slice
//                      scribingDebt consumes), readConfig (via
//                      crate::state::read_config_raw).
//   lib/test-runner.mjs
//                      declaredTestCommands, runDeclaredTests,
//                      spawnDeclaredCommand, posixShell, firstFailureLine,
//                      TEST_RESULTS_RELATIVE, FAILURE_EXCERPT_MAX_CHARS.
//   lib/cells.mjs      readCell, ID_PATTERN, ARCHIVE_DIR_NAME, listCells,
//                      scribingDebt, bestScribingStampMs,
//                      scribingRunStampMs, readScribingLedger.
//   lib/capture.mjs    captureQueuePath, pendingCaptureStubs, captureQueue.
//   lib/knowledge.mjs  bundleDir, bundleMode, collectConcepts,
//                      buildContextManifest (+ its whole ranking closure),
//                      KNOWLEDGE_CONTEXT_LANE_BUDGETS/DEFAULT_BUDGET.
//
// ─── Re-derived Rust (the "may not edit that file" rule) ───────────────────
//
// The knowledge-context builder lives in verbs/knowledge/ (anchor.rs,
// context.rs, promote.rs, walk.rs, frame.rs, frontmatter.rs); prepare.rs
// calls it directly (crate::verbs::knowledge::…) rather than carrying a
// second, byte-parity copy — R6: the standing "may not edit that file" rule
// was worked around by widening the knowledge module's own function
// visibility to pub(crate), the one edit that removes the need for a lift.
// Same for the test runner (verbs/test_runner.rs) and readCell/listCells
// (verbs/cells.rs), each re-derived here with a per-function provenance line
// naming BOTH the .mjs source and the Rust port.
//
// ─── Documented divergences ────────────────────────────────────────────────
//
//   * `dispatch_id` is crypto.randomUUID() in Node; here it is
//     crate::verbs::reservations::pseudo_uuid_v4 (the same v4 SHAPE, a
//     different entropy source — already the campaign's convention, and the
//     twin-diff harness masks ids).
//   * A test-record write failure after close's run reports Rust's io message
//     where Node reports V8's (the .bee/logs dir is pre-flighted, so this is
//     a hard-race-only path). Same for a dispatch.jsonl append failure —
//     except that one is fail-open on BOTH sides, so it is unobservable.
//   * Node's spawnSync 64 MiB maxBuffer kill and its 10s shell probe timeout
//     are not replicated (inherited from verbs/test_runner.rs).
//   * The prompt templates are COMPILED IN (include_str!) rather than read off
//     disk. A runtime skew guard byte-compares the embedded template against
//     the on-disk one whenever the repo ships prompts (canonical
//     `packages/bee/prompts/` or vendored `.bee/bin/prompts/`) and delegates
//     on any mismatch — the same "lib skew ⇒ delegate" discipline R2's
//     write-guard uses, so a prompt can never render from stale bytes.














/// This argv/repo/store shape belongs to the Node runtime.
#[derive(Debug)]
pub(crate) struct Delegate;

type D<T> = Result<T, Delegate>;

impl From<Delegate> for crate::verbs::reservations::Err2 {
    fn from(_: Delegate) -> Self {
        crate::verbs::reservations::Err2::Ex
    }
}

// R6 (CLOSED): the `kctx` module — a hand-kept byte-for-byte lift of
// verbs/knowledge/{walk,frame,frontmatter,anchor,context}.rs, carried here
// only because those functions were PRIVATE to the knowledge module — is
// folded away. Their visibility is now pub(crate) at the source, so
// prepare.rs (below) calls crate::verbs::knowledge::… directly: one surface,
// not two independently-drifting answers to the same question.

// ═══ tests ═════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests;

mod jsprim;
mod prompt;
mod models;
mod guard;
mod prepare;
mod brief_lint;
mod context_table;
mod close;
pub(crate) use self::jsprim::*;
pub(crate) use self::prompt::*;
pub(crate) use self::models::*;
pub(crate) use self::guard::*;
pub(crate) use self::prepare::*;
pub(crate) use self::brief_lint::*;
pub(crate) use self::context_table::*;
pub(crate) use self::close::*;
