---
type: bee.area
title: "Worktree Parallelism — the shared holds ledger that makes an island's writes visible"
description: "One path-keyed ledger in the main store that mirrors every reservation, acquired under a single lock so two checkouts can never both believe they hold a path, renewed by a live session's heartbeat, read by three taps whose write-time answer now splits by resource — exclusive resources still hard-deny, every other path is advisory — and released by cell rather than by holder."
timestamp: 2026-08-05
bee:
  id: worktree-parallelism-cross-worktree-holds
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [areas/worktree-parallelism/the-trust-model.md, areas/worktree-parallelism/control-plane-topology.md]
  decisions: ["cross-worktree-holds D1-D6 (the shared ledger, 2026-07-20)", "hardening-1-7-10 (acquisition is one atomic step; holds renew on heartbeat, 2026-07-21)", "a0ab91b6 (release is scoped by cell, never by holder alone — live incident)", "multisession-native D4 (slice 3, issue #56: a foreign hold on a normal path downgrades to advisory allow+warning at write time; only a built-in-plus-configured exclusive-resource list keeps the old hard deny — docs/history/multisession-native/CONTEXT.md)", "multisession-native D2 (slice 4, msn-21: the write guard resolves topology exactly ONCE per call via resolveContext, replacing the two separate walks — resolveWriteRecord's own call and resolveHoldTopology's own resolveRoots call — this exclusive-resource-list class of deny previously required; the class's own policy stays byte-for-byte unchanged — docs/history/multisession-native/CONTEXT.md, decision e1ceca12)", worktree-reclaim D3 (a teardown either finishes or does not start — the shared teardown helper every worktree-removing path now calls through), "hold-holder-attribution hha-1..3 (a hold belongs to the cell's owning worktree; one resolver seam asked by reserve/release/cap-release/claim-next; write-guard unchanged, 2026-08-14)", "41a67ee2 (a guard whose scope is 'this checkout' resolves that scope from the checkout it is RUNNING in: the concurrent-worker git guard reads this ledger for a granted worktree's sibling cohort, combined by max not sum, 2026-08-25)"]
  sources: [docs/history/worktree-feature-parallelism/, "docs/specs/worktree-parallelism.md#S-cross-worktree-holds-the-shared-ledger-cross-worktree-holds-d1-d6-2026-07-20", "multisession-native cell multisession-native-14 (advisory cross-worktree hold on normal paths; exclusive-resource list; trace .bee/cells/multisession-native-14.json, commit 5bee967, 2026-07-25)", "multisession-native cell multisession-native-21 (guards.mjs checkWrite unified onto one resolveContext resolution feeding this class plus two new classes; trace .bee/cells/multisession-native-21.json, commit 3f56916, 2026-07-25; see areas/worktree-parallelism/control-plane-topology.md)", commit 430ec952 (lift teardown into one helper); packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs (teardown_worktree), "wave-guard-gaps cell wgg-2 (the guard's fourth read tap; resolve_live_worker_count in packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs; trace .bee/cells/wgg-2.json, commit 336b3641, merged ed4f3b62, 2026-08-25)", "docs/history/wave-guard-gaps/CONTEXT.md (the two-invocation reproduction: exit 2 from main, exit 0 from the worktree, identical state)"]
  authoritative_for: "worktree-parallelism: the cross-worktree holds ledger, its acquisition, renewal, taps and release"
---

# Worktree Parallelism — Cross-Worktree Holds

A granted worktree's store is an island by design — which used to mean its reservations were
invisible to main and to sibling worktrees, and overlapping edits surfaced only at merge. The
shared holds ledger closes that gap at WRITE time:

## Cross-worktree holds (the shared ledger — cross-worktree-holds D1-D6, 2026-07-20)

- **One ledger, main store only:** `<mainRoot>/.bee/runtime/cross-worktree-holds.json`,
  beside the grant registry (same trust model: the main store is the authority; a worktree
  reaches it via `resolveRoots().mainRoot`). Path-keyed rows
  `{holder, feature, session, cell, acquired_at, expires_at}`; holder is the git-verified
  worktree id or `"main"`. TTL-expired rows are pruned on every read; all mutations run under
  the main store's `cross-worktree-holds` lock with atomic tmp+rename writes.
- **Mirror on reserve:** a reservation in any checkout mirrors into the ledger (an ungranted
  linked worktree never double-mirrors — its reservations already live in the shared store).
  Before reserving, the seam consults the ledger and refuses with a typed `FOREIGN_HOLD`
  result naming the holding checkout, feature, and expiry.
- **Acquisition is one atomic step, not three sequential ones (hardening-1-7-10).** Checking
  the ledger for a conflicting foreign hold, reserving the path in the local (worktree or
  main) store, and inserting the mirrored row into the shared ledger all run under the SAME
  single lock acquisition at the main root — never as separate lock-check, lock-reserve,
  lock-insert steps that could interleave with another checkout's attempt on the identical
  path. Two checkouts racing the same path can therefore never both come away believing they
  hold it; exactly one wins, and the loser gets the typed `FOREIGN_HOLD` refusal instead of a
  hold it does not actually have.
- **Holds renew, not just expire (hardening-1-7-10).** A hold's TTL ceiling (1 hour) is the
  same failure-recovery backstop as before, but a live session's own heartbeat now refreshes
  the timestamp on every cross-worktree hold it owns — a try-once refresh that never blocks
  the session's primary work. A long-running live worker therefore keeps its holds protected
  for as long as it stays genuinely active; TTL expiry now fires only for a session that has
  actually gone silent (a dead or abandoned worker), not for one still working past the old
  fixed ceiling.
- **A hold belongs to the cell's owning worktree, not the checkout that typed the command
  (hold-holder-attribution hha-1..3, 2026-08-14).** Holder identity resolves through one
  seam — the cell's granted worktree decides who owns the hold — and every mutating tap asks
  that same seam: reserve, release, cap-time release, and claim-next's overlap skip. Before
  this, a granted worktree's own worker could be denied by a hold the ledger attributed to
  the checkout that happened to run the command (a false FOREIGN_HOLD against itself). The
  write-guard's read tap deliberately does NOT change: its by-resource policy below stays
  byte-identical.
- **Three read taps, one voice — but the write-guard's own voice now splits by resource
  (revised multisession-native D4, slice 3, msn-14).** (1) `reservations reserve` — typed
  refusal before any local row is written, unchanged; (2) `claim-next` — silently skips a cell
  whose declared files overlap a foreign hold, so a session always picks conflict-free work
  instead of waiting, unchanged; (3) the write-guard — phase-independent, unconditional on
  session id, same placement as before, but a foreign hold on a DIFFERENT-workspace path no
  longer hard-denies unconditionally. It denies only when the path matches an
  **exclusive-resource list** — built-in defaults: any-depth `migrations/` directories, common
  lockfiles (`package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`, `Cargo.lock`,
  `composer.lock`, `Gemfile.lock`, matched both at repo root and nested), the beehive
  release-manifest path, `.bee/onboarding.json`, and any-depth `generated/` directories —
  extended (never replaced) by `.bee/config.json`'s `guards.exclusive_paths` array, matched
  with a small `**`-aware glob-to-regex translator distinct from this store's own
  trailing-star `pathsOverlap` containment check. Every OTHER cross-worktree overlap
  downgrades to `allow: true` plus a non-blocking warning naming the holding checkout, its
  feature, and the expiry — the same facts the old hard-deny reason carried — plus the
  merge-time consequence: `bee worktree merge`'s own drift check is what actually surfaces a
  real collision on a normal path, not this write-time check. Same-workspace conflicts
  (cross-session holds, swarming reservations, and the intent/lease split on those — see
  `workflow-state/holds-and-the-coordination-lock.md`) are completely untouched; only this
  cross-worktree branch's policy changed. Red-first proof: stashing the write-guard back to
  its pre-cell content and rerunning its suite failed exactly the two tests this cell adds
  (the normal-path-advisory case, and the config-extension case) while every
  same-workspace/cross-session/swarming regression test stayed green.
- **The write-guard now resolves this tap's own topology exactly ONCE per call, alongside
  two entirely new deny classes it never carried before (multisession-native D2, slice 4,
  msn-21).** Before this cell, `checkWrite` walked topology twice to answer this exclusive-
  resource-list question at all: `resolveWriteRecord`'s own `resolveContext` call (gated on a
  session id being present) plus this ledger's own separate `resolveHoldTopology`/
  `resolveRoots` call, with a THIRD, lane-path-only re-resolution on top. All three collapse
  into one upfront `resolveWriteTopology` (wrapping `resolveContext(root)`, fail-open) that
  every branch reads from — this exclusive-resource-list class's own policy is unchanged
  byte-for-byte by the collapse (same list, same advisory fallback, same merge-time backstop);
  only the plumbing that resolves WHERE main is got cheaper and singular. The other two deny
  classes the guard now carries — a same-workspace cross-session exact-path lease, and a
  non-owner default-path write in a workspace a different live session already owns — are
  genuinely new, scoped to same-workspace and same-checkout situations respectively, and
  never touch this cross-worktree tap's own exclusive-resource-list branch. Full three-class
  breakdown and the resolver itself: `areas/worktree-parallelism/control-plane-topology.md`.
- **Release is scoped by cell AND session, never by holder alone.** All agents in the main
  checkout share `holder:"main"`; an early cut that released by holder wiped a concurrent
  worker's mirrored holds (live incident, same day, decision a0ab91b6). A second live
  incident (GH #87, gh-fix-batch cell gfb-1, 2026-07-28) showed cell-only scoping still
  wipes a *different session's* active hold on the same cell id: release now derives
  `{cell, session}` pairs from the acting agent's own active reservation rows and passes
  the session filter into `releaseHolds` whenever the row carries one; a sessionless
  (legacy) row falls back to the exact cell-only scoping — strictly narrowing, byte-identical
  for the single-session case. Every teardown of a worktree releases every row for its id,
  best-effort after the grant is removed — worktree merge cleanup (default now, on a merge
  that merged something; worktree-reclaim D1), `bee worktree prune`'s removal of a
  classified-dead worktree, and `bee worktree unregister` all reach this release through the
  one shared teardown helper (`entering-creating-and-registering.md`, `returning-and-the-
  merge-gate.md`).
- **A lease names the checkout that wrote it, so a conflict refusal is not anonymous
  (cell wtf-2, commit 41556fb1, 2026-08-12).** A path lease record carries its writing
  checkout — `main`, or a granted worktree's git-verified id, and the field is omitted for an
  ungranted linked worktree, whose writes already live in the shared store. A conflict refusal
  then names WHERE the holder sits instead of only which agent nickname holds it. Leases are
  control-plane and shared across every checkout, so the record itself is the only place that
  answer can come from — the refusing side cannot see the holder's filesystem. Reporting only:
  conflict detection, release scoping, and sweep scoping are unchanged, and a lease written
  before this still round-trips (a missing field reads as "unknown checkout", never as main).
- **Failure discipline:** missing ledger = empty (byte-identical to pre-ledger behavior);
  unparseable ledger = typed deny (`worktree-holds-unreadable`, mirroring the reservation
  corrupt-store rule); unresolvable topology = fail-open with crash-log. Both runtime files
  (`cross-worktree-holds.json`, `worktree-grants.json`) are direct-edit-denied in every
  phase — CLI-only writes.
- **A worktree session that reserves via the main checkout mirrors a hold against ITSELF
  (knowledge-search cell ks-2, live incident, 2026-08-10; second instance after the wr-2 handoff
  note).** A session working in a granted worktree that runs its reservations by cd-ing to the
  main checkout lands rows under `holder: "main"` — the holder is derived from the invoking
  checkout, not the session's home. Those rows then read as FOREIGN holds from the worktree side
  and hard-block that same session's own commit when the paths are exclusive-resource-listed. The
  remedy is self-release before the commit: `reservations release --agent <worker> --cell <id>`
  for the session's own rows. The durable rule: reserve from the checkout you write in, so the
  holder matches the writer.
- **A fourth read tap: the ledger is how a granted worktree learns how many workers share its
  index (wave-guard-gaps cell wgg-2, decision 41a67ee2, 2026-08-25).** The concurrent-worker git
  guard refuses a whole-tree verb — a bare `git commit` with no pathspec, `commit -a`, `stash`,
  `reset` and the rest — when more than one worker is live in the checkout. Inside a granted
  worktree it had never once fired, for the whole life of worktree-based parallelism, because its
  worker count resolved to ZERO there by two independent routes: the worktree's own lease store is
  empty (reservations mirror to the main store, not the island's), and live worker sessions are
  filtered by workspace, so workers *executing in* the worktree under an orchestrator session
  stamped `main` were every one of them skipped. Reproduced with identical live state: the same
  hook payload exits 2 from main and exits 0 from the worktree. The guard now asks this ledger for
  the distinct cells holding active mirrored rows whose `holder` is that worktree's id, and
  combines that with its existing count by **max, never a sum** — the cohort is an independent view
  of the *same* workers, so a solo worker seen through both views still counts as one and is never
  blocked. Read-only, like the write-guard tap above: this adds a reader, never a mutation, and the
  main-checkout verdict is byte-identical before and after. An unparseable ledger keeps the
  existing fail-safe shape — unresolved counts as more-than-one, never as zero.
- **Waiting model:** fail-fast, never blocking — an exclusive-resource refusal names who and
  until when; the session takes other open work (`claim-next` already routed it away) and the
  hold lapses by TTL if its owner dies. On a normal path (revised multisession-native D4)
  there is nothing to wait on at write time at all — the write proceeds with a warning, and
  the merge verify-gate stays the final semantic net for that path instead.
