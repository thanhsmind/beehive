// bee worktree — native port of the `worktree` verb group.
//
// Ported argv shapes (everything else returns None BEFORE any output and the
// whole command re-runs under Node):
//   worktree list                       [--json]
//   worktree register   --feature F     [--json]
//   worktree unregister [--id ID]       [--json]
//   worktree new    --feature F [--base-ref R] [--with-companion] [--json]
//   worktree merge  --id ID [--cleanup] [--queue-wait-ms N]        [--json]
//
// `worktree new` — NATIVE (R6), including the creating path from the MAIN
// checkout: createFeatureWorktree whole, inside ONE 'worktree-admin' hold —
// the eight zero-mutation refusals, the real `git worktree add`, the post-add
// block (git-verified id, grant write, store bootstrap, `rev-parse HEAD`,
// registerWorkspace, COMPANION START, skill-tree sync) and the
// ORDER-SENSITIVE rollback ladder. The port that unblocked it is
// verbs/workspace_store.rs: the registerWorkspace/unregisterWorkspace pair the
// post-add block and rungs R1/R2 of the ladder need, with Node's
// `workspace:<id>` lock name.
//
// Both of `new`'s former gates are CLOSED (R6 cutover wave):
//   * `--with-companion` is native. runCompanionStart spawns the project's own
//     `commands.worktree_companion_start`, symlinks the declared worktreePath
//     at `<worktreeRoot>/<commands.worktree_companion_mount>` and writes the
//     `.bee/companion-session.json` marker; every failure arm folds into the
//     SAME post-add rollback ladder. It could not delegate because all of
//     those arms fire AFTER `git worktree add`, so the two that embed a
//     Node-only string are DIVERGENCES now, named below.
//   * wcg-3's shared-nested-checkout guard is native, whole, via
//     crate::nested_checkout — which imports guards.mjs's verification
//     predicates from hooks/write_guard.rs (widened to pub(crate) for exactly
//     this) rather than re-deriving them, so the two enforcement surfaces
//     cannot drift apart (C5).
// A bare `--base-ref` (no value) still delegates: Node stringifies `true` into
// a ref name, an unproven shape.
//
// `worktree merge` — NATIVE (R6), the whole three-phase staged transaction
// (mergeFeatureWorktree's P1/P2/P3), integration-queue.mjs's drainer
// (crate::integration_queue, built on the now-landed crate::lease_store) and
// bee.mjs's own text/JSON rendering. Its three recorded blockers were
// re-measured at this port and NONE of them survived:
//
//   (a) "the lease store is unported, so composing merge on it would fork
//       store-mutation logic" — DISSOLVED. src/lease_store.rs now carries
//       acquireLeases (hash-sorted batch + rollback), renewLease,
//       releaseLease, both fence checks, sweep and list, so
//       crate::integration_queue's tryBecomeProcessor / renewProcessorLease /
//       releaseProcessorLease / checkProcessorLeaseEpoch call THE port rather
//       than a second copy. This was a sequencing blocker; the sequence has
//       arrived.
//   (b) `runVerifyChild`'s libuv spawn error (`spawn cmd.exe ENOENT`)
//       concatenated into MERGE_VERIFY_RED's `output_tail` — PRE-CHECKED
//       away. That byte exists only when the SHELL ITSELF cannot start, so
//       `shell_launchable()` probes the shell BEFORE P1 stages anything: a
//       failed probe returns None with zero mutations and zero locks taken,
//       and a passing probe proves the real spawn will launch, making the arm
//       unreachable. (A delegation is only sound before a mutation — after
//       the merge is staged nothing here may fall back, which is exactly why
//       this had to become a pre-check rather than a late bail.)
//   (c) the un-null-guarded `runGit(...).stdout.trim()` sites
//       (mainUntouchedProof ×2, checkMergeFence ×2, preMergeHead,
//       stagedTreeHash, postCommitStatus) whose `TypeError: Cannot read
//       properties of null` V8 text `processAsOwner` PERSISTS into the queue
//       record — UNREACHABLE BY CONSTRUCTION, for the same reason `new`'s
//       single site is. Every one of them sits after
//       `mergeFeatureWorktreeStage`'s FIRST git call, `isTreeDirty(mainRoot)`,
//       which routes a never-launched spawn through `gitStatusPorcelain`'s
//       own throw — a DETERMINISTIC message (`"git status --porcelain" failed
//       in <root>: exit null`, reproduced natively) raised with zero
//       mutations. So a repo where git cannot be spawned refuses before any
//       merge is staged, and by the time a `.trim()` site runs git has
//       provably launched from that same cwd. The residual — git going away
//       BETWEEN two calls of one merge — is the same race class
//       verbs/workspace_store.rs documents for a record going unreadable
//       between its probe and its in-lock read.
//
// Two DOCUMENTED DIVERGENCES, both in the class state_group.rs's prune
// approximation and lease_store.rs's LEASE_CORRUPT residual already
// established:
//   * `runVerifyChild` resolves on Node's `exit` event, so output still
//     sitting in a pipe at exit can be LOST; this port joins its reader
//     threads and therefore always captures the full stream. It can only ADD
//     trailing bytes to `output_tail`, never change `status`, the red/green
//     verdict, or any `.bee/` record — and the Node side is not
//     reproducible run-to-run anyway.
//   * a verify spawn that fails DESPITE `shell_launchable()` (a race) keeps
//     Node's `status: null` verdict but carries Rust's io text instead of
//     libuv's, exactly as `node_fs_error_message` approximates elsewhere here.
//
// Both of `merge`'s former gates are CLOSED too (same wave):
//   * a COMPANION worktree (a parsed `.bee/companion-session.json` marker) is
//     native. `teardownCompanionIfPresent` spawns
//     `commands.worktree_companion_end` and unlinks the mount after every
//     zero-mutation refusal has cleared but BEFORE the merge is staged, and
//     the worktree dirty-check switches to `gitStatusPorcelainExcluding`'s
//     pathspec form (which has to be a pathspec, never text filtering — see
//     that function). On a companion worktree the TEARDOWN, not the staged
//     merge, is the first mutation, so every `MErr::Ex` probe in `run_merge`
//     is what keeps the no-fallback-after-mutation rule true.
//   * `--queue-wait-ms` is native: `js_string_to_number` is the full
//     `Number(string)` conversion, validate()'s finiteness gate is reproduced,
//     and the handler's positive-only filter decides whether the value
//     replaces DEFAULT_WAIT_BOUND_MS. Only a value validate() itself REFUSES
//     (a bare flag, an empty string, `Infinity`, a non-numeric literal) still
//     returns before any output — that is the dispatcher's own generic
//     machinery, shared by every verb, not this flag's arm.
//
// A BROKEN linked-worktree link (WorktreeLinkInvalidError) is native for all
// five verbs here, through crate::link_invalid — see that module for why the
// timing wrapper made it undelegatable-but-unreproducible before, and for the
// single named timing-line divergence.
//
// DELEGATED to Node, by design:
//
//   * A grants registry file that exists but does not parse with serde:
//     Node's readGrants swallows the parse error and reads `{}`, but V8's
//     JSON grammar is not provably identical to serde's, so "unparseable
//     here" cannot be turned into "Node saw {} too".
//   * An `Exotic` root resolution (a `.git` that vanished between existsSync
//     and statSync) — a V8-worded ENOENT.
//
// THREE MORE DOCUMENTED DIVERGENCES, all of them companion arms that fire
// after a mutation and therefore could never have been delegations:
//   * runCompanionStart's unparseable-stdout refusal interpolates serde's
//     parse message where Node interpolates V8's. Every other byte of that
//     sentence, including the 500-UTF-16-unit raw-stdout tail, is Node's.
//   * runCompanionStart's symlink failure carries an errno-CLASS-accurate
//     approximation of libuv's message (`EPERM: operation not permitted,
//     symlink '<target>' -> '<path>'` for a win32 host without
//     SeCreateSymbolicLinkPrivilege) — same class as `node_fs_error_message`.
//   * a companion marker that parses to a truthy value with no string
//     `mountPath` makes the .mjs die with a V8 TypeError from
//     `mountPath.replace(...)`; here it is an explicit typed refusal
//     (WORKTREE_MERGE_COMPANION_MARKER_INVALID) taken before any mutation.
//
// What IS native is the part the linked-worktree root port unlocked: all
// three grant-registry verbs, plus the two "you are inside a worktree"
// refusals, work from INSIDE a linked worktree — the classification, the
// bidirectional gitdir validation and the grant lookup all come from
// crate::roots::resolve_roots_core (state.mjs resolveRootsCore), not from a
// second copy of the walk.
//
// Provenance: bee.mjs handleWorktreeList / handleWorktreeRegister /
// handleWorktreeUnregister / handleWorktreeNew / handleWorktreeMerge /
// resolveMainRoot / requireFlag / emit / emitError, plus lib/worktree-store.mjs
// readGrants / listGrants / writeGrant / writeGrantCore / removeGrant /
// removeGrantCore / grantsFile / writeGrantsFileAtomic /
// bootstrapWorktreeStore / writeCreationIdentity, and lib/lock.mjs
// withStoreLock.
//
// Locking: writeGrant/removeGrant serialize on the SAME cross-process lock
// file Node uses — lock name "worktree-admin" on the MAIN root — through
// crate::lock::acquire_store_lock with lock.mjs's own MAX_ATTEMPTS, so the
// two runtimes serialize against each other mid-campaign (contract C1).
// `merge` adds two more of Node's names, unchanged: "integration-queue" on
// the CONTROL root for every queue write, and lease-store's per-resource
// `lease:<sha256(file)>` for the processor record. It also re-acquires
// 'worktree-admin' for P3 rather than holding one lock end to end — Node
// does, so a single hold would drop a `result: "acquired"` row from
// .bee/logs/contention.jsonl, which is `.bee/` state, not just telemetry. A
// LOCK_BUSY refusal is reproduced natively (never delegated: it is reached
// AFTER a lock attempt, and delegating would double the contention
// telemetry).
//
// One accepted residual, mirroring verbs/reservations.rs: `register` can
// still return a late None if a file copy fails mid-bootstrap. Every step it
// has taken by then is idempotent (writeGrant merges, mkdir is recursive,
// the copies are copy-if-absent), so the Node re-run reproduces the same
// error message over the same store, exactly like two sequential CLI calls.












/// worktree-store.mjs's lock name, byte-identical (hardening-4b).
const WORKTREE_ADMIN_LOCK: &str = "worktree-admin";

/// worktree-store.mjs FRESH_STATE_SCHEMA_VERSION.
const FRESH_STATE_SCHEMA_VERSION: &str = "1.0";

#[cfg(test)]
mod tests;

mod registry;
mod git;
mod companion;
mod create;
mod merge;
mod phases;
mod handlers;
pub(crate) use self::registry::*;
pub(crate) use self::git::*;
pub(crate) use self::companion::*;
pub(crate) use self::create::*;
pub(crate) use self::merge::*;
pub(crate) use self::phases::*;
pub(crate) use self::handlers::*;
