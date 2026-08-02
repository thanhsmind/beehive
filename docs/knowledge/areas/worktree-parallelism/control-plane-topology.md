---
type: bee.area
title: "Worktree Parallelism — resolveContext, the control-plane/workspace-local split, and the single write owner per workspace"
description: "The single topology resolver every store now goes through, the declared plane split that keeps coordination stores shared at main while cells/backlog/decisions stay per-checkout, the migrate-or-fail-loud door for worktree-local records stranded by the split, the workspace registry's single write owner, the write-policy default that auto-isolates a second write session instead of refusing it, and the issue-#56 acceptance suite proving the write guard's identity-mandatory checks."
timestamp: 2026-07-25
bee:
  id: worktree-parallelism-control-plane-topology
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [areas/worktree-parallelism/the-trust-model.md, areas/worktree-parallelism/store-tiers-and-where-it-lives.md, areas/workflow-state/sessions-lanes-and-identity.md]
  decisions: ["multisession-native D2 (control plane / data plane split — resolveContext(cwd) replaces resolveRoots; controlRoot is session records, workflow state, claims, leases, and the workspace registry, shared across all worktrees; workspaceRoot is the physical checkout; localRuntimeRoot never needs sharing — docs/history/multisession-native/CONTEXT.md, decision e1ceca12)", "multisession-native D3 (second write session defaults to isolation: observe/shared-disjoint/isolated write-policy modes; a workspace has exactly one write_owner_session, others attach read-only — docs/history/multisession-native/CONTEXT.md, decision e1ceca12)", "multisession-native re-slice decision 89a4a87b (msn-18 honest block: resolveContext was dead code with zero production call sites until every coordination-store call site is swept — re-sliced into 18a state.mjs mapping, 18b cells/reservations/recovery/compaction/state-projection sweep, 18c the standalone bee dispatcher sweep, 18d onboard migrate-or-fail-loud)", "multisession-native advisor-digest-slice4 conditions 1-7 (docs/history/multisession-native/reports/advisor-digest-slice4.md — file anchors fixed at authoring; msn-18 must close the guard's lane/workflow read in-cell and migrate-or-fail-loud on pre-existing worktree-local sessions/claims/leases, never silently orphaning in-flight data; grant and write ownership compose and never subsume each other; resolveContext becomes the single git-common-dir resolver, with herding.mjs's own standalone resolver tracked rather than reconciled; auto-isolation needs one-line cost disclosure and CLI-owned-allowlisted register/create writes so isolation cannot self-deadlock)", "multisession-native D9 invariant 15 (issue #56 3.9 — write-capable ops refuse without identity where identity is mandatory today: lease acquire and workspace ownership claim; legacy carve-outs — unfenced lease renew/release, sessionless calls proceeding untouched, the C1 no-workflow handoff fallback — are named explicitly, not silently tightened)"]
  sources: ["multisession-native cell multisession-native-17 (resolveContext(cwd) added beside resolveRoots in state.mjs, controlRoot mapped to <mainRoot>/.bee/runtime/control; trace .bee/cells/multisession-native-17.json, commit bd8f755, 2026-07-25)", "multisession-native cell multisession-native-18a (controlRoot corrected to mainRoot itself; state.mjs's own claims/sessions/workflow-store call sites and guards.mjs's resolveWriteRecord re-rooted; trace .bee/cells/multisession-native-18a.json, commit 5d0ec3c, 2026-07-25)", "multisession-native cell multisession-native-18b (cells.mjs/reservations.mjs/recovery.mjs/compaction.mjs/state-projection.mjs claims/sessions/leases call sites re-rooted via controlRootFor; reservations.mjs carries its own fail-open findMainRoot/controlRootFor replica to avoid an import cycle; trace .bee/cells/multisession-native-18b.json, commit a1431448, 2026-07-25)", "multisession-native cell multisession-native-18c (bee's own claims/workflow-store call sites and state-projection.mjs's rebuild reads re-rooted; packages/bee/hooks/adapter.mjs gains its own import-light controlRootFor(root) and ctx.controlRoot; trace .bee/cells/multisession-native-18c.json, commit d69d81e, 2026-07-25)", "multisession-native cell multisession-native-18d (bee onboard detectWorktreeMigration/applyWorktreeMigration — migrate-or-fail-loud, all-or-nothing; trace .bee/cells/multisession-native-18d.json, commit c90dd37, 2026-07-25)", "multisession-native cell multisession-native-19 (workspace-store.mjs: registerWorkspace/unregisterWorkspace/claimWriteOwnership/attachWorkspace; wired into worktree-store.mjs and claims.mjs; bee-session-init.mjs re-roots session creation onto controlRoot; trace .bee/cells/multisession-native-19.json, commit 09e1ed0, 2026-07-25)", "multisession-native cell multisession-native-20 (applyWritePolicy: observe/shared-disjoint/isolated; wired into startFeature's default path; trace .bee/cells/multisession-native-20.json, commit 84376ce, 2026-07-25)", "multisession-native cell multisession-native-21 (guards.mjs checkWrite unified onto one resolveContext resolution; workspace-scoped lease deny; workspace-ownership deny; trace .bee/cells/multisession-native-21.json, commit 3f56916, 2026-07-25)", "multisession-native cell multisession-native-23 (test_msn_invariants.mjs invariant 15 — fresh identity-mandatory refusal proofs for lease acquire and workspace ownership claim, reusing test_guards.mjs's existing deny-class-(c) proof, legacy carve-outs named explicitly in the invariant's own PASS-line output; trace .bee/cells/multisession-native-23.json, commit 06cd209, 2026-07-25)", "docs/history/multisession-native/reports/advisor-digest-slice4.md (conditions 1-7, verdict proceed-with-conditions)"]
  authoritative_for: "worktree-parallelism: the control-plane/workspace-local topology resolver, the workspace registry's write ownership, and the write-policy isolation default"
---

# Worktree Parallelism — resolveContext, the Control-Plane/Workspace-Local Split, and the Single Write Owner

`the-trust-model.md` and `store-tiers-and-where-it-lives.md` describe a worktree
as an island that gets its own store only when granted. That is still true for
**workspace-local** state — cells, backlog, decisions — but it stopped being
true for **coordination** state. This concept owns the resolver that draws that
line, the migration door that closes the gap the line opened, and the two
mechanisms that decide who may write into a shared checkout: the workspace
registry's single write owner, and the write-policy default that offers
isolation instead of a wait.

## `resolveContext(cwd)` — the single topology resolver (multisession-native D2)

`resolveContext(cwd)` in `state.mjs` returns `{projectRoot, controlRoot,
workspaceRoot, localRuntimeRoot, gitCommonDir, workspaceId, worktreeId}` from
one canonical git-worktree classification. `resolveRoots()` — every existing
caller's entry point — is refactored into a byte-identical compat wrapper over
the same internal core (`resolveRootsCore`) `resolveContext` is built from, so
no existing caller changed behavior the day this landed.

- **`controlRoot`** always resolves to the MAIN checkout itself (accepted,
  after re-slicing, as the least-churn D2 reading over an earlier
  `<mainRoot>/.bee/runtime/control` shape — see "Landed dead, then swept"
  below). Every linked worktree's `controlRoot` is the same path main's own
  resolves to; solo/main checkouts are byte-identical by construction.
- **`workspaceRoot`** is the physical checkout — unchanged mechanism from the
  trust model's existing grant read: `workspaceId` reuses
  `worktree-store.mjs`'s `decideWorktreeStore`/`readGrants`, so an
  *unregistered* linked worktree still reports `workspaceId: 'main'` while a
  *registered* one reports its own id. `worktreeId` always reflects
  git-worktree-ness regardless of registration — it answers "is this a linked
  worktree at all", never "is it granted".
- **`localRuntimeRoot`** stays per-checkout, always distinct from
  `controlRoot` even once the control plane collapses onto main — proven by a
  dedicated BINDING topology test, because this is the one root the D2 split
  must never accidentally merge.
- `herding.mjs`'s own `resolveHerdingMainRoot` and a descriptive comment at
  `command-registry.mjs:1947` are deliberately left unreconciled: the former
  exists to mirror `dispatch-interlock.mjs` (a standalone, zero-bee-dependency
  script) byte-for-byte, and `resolveContext`'s walk-up-and-validate algorithm
  is not proven identical to that script's raw `git rev-parse
  --git-common-dir` call in every edge case — tracked, not merged, per
  advisor-digest-slice4 condition 5.

**Landed dead, then swept (the msn-18 honest block).** `multisession-native-17`
shipped `resolveContext` with zero production call sites reading `controlRoot`
for anything — every coordination-store module still resolved its own root the
old way. The worker assigned to wire it in blocked honestly instead of
half-landing a re-root across roughly a dozen call sites and six modules in one
cell; the re-slice that followed (decision `89a4a87b`) split the sweep into
`18a` (state.mjs's own call sites plus the write guard's lane read — the one
branch advisor condition F3 named as the biggest risk, since an unresolvable
lane there is a typed hard deny) → `18b` (cells/reservations/recovery/
compaction/state-projection) → `18c` (the `bee` dispatcher itself,
standalone and never folded into the write-guard cell) → `18d` (the onboarding
migration below). Only after `18d` closed did `multisession-native-19`
(workspace registry) begin — every later cell in this concept depends on the
sweep being complete, not partial.

## Coordination stores are control-plane; cells/backlog/decisions stay workspace-local (declared plane split)

The split is a declared, tested boundary, not a inference from file location:

- **Control-plane (always `controlRoot`, i.e. main):** session records, the
  durable workflow record and its lock (`workflow-records-and-projections.md`),
  claims (`claims-and-ownership.md`), leases/reservations
  (`workflow-state/holds-and-the-coordination-lock.md`), the per-workflow
  handoff mailbox (`handoff.md`), and the workspace registry (below). A linked
  worktree — granted or not — reads and writes every one of these at main's
  real `.bee/` tree, never its own.
- **Workspace-local (stays at `workspaceRoot`):** cells, backlog, decisions.
  These are the surfaces `multisession-native-18b` deliberately left alone —
  `cells.mjs`'s own cell-file reads (`readCell`/`claimCell`/`readyCells`) are
  untouched by the re-root, on purpose.
- Solo checkouts and the main checkout itself are byte-identical under this
  split by construction: `controlRoot === workspaceRoot === projectRoot` when
  there is no linked worktree in play.
- One read-path exception is documented in-file rather than swept:
  `state-projection.mjs`'s workflow/lane/handoff-mailbox **reads** stayed
  workspace-local through `18b` on purpose, because their WRITE paths (the
  gate/plan-rev/lane handlers in `bee`) were not yet re-rooted — re-rooting
  only the read side first would have desynced from an in-flight worktree
  write instead of just staying stale. `18c` closed this by re-rooting both
  sides of `bee`'s own sweep together.

## Onboarding migrates stranded worktree-local coordination records — all-or-nothing (multisession-native-18d)

Before this cell, `bee onboard` had zero worktree-awareness: a granted
linked worktree that had — under the pre-D2 model — accumulated its own
`.bee/sessions`, `.bee/claims`, `.bee/runtime/{workflows,leases,handoffs}`
records now had those records silently stranded the moment `18a`-`c` re-rooted
every reader onto `controlRootFor(root)`; bee would never look there again.

`detectWorktreeMigration`/`applyWorktreeMigration` close that gap at
onboard/upgrade time:

- Scan a linked worktree's own coordination stores. For each record: no
  target present at main, or an identical duplicate → migrate (or clean up the
  duplicate locally). Any record that conflicts with a **different** record
  already at that id in main's store → abort the **entire** apply before a
  single byte moves, naming every stranded record (path, id, kind, reason) as
  a `blocked_worktree_migration_conflict` status.
- **All-or-nothing across the whole migration set, not per-store or
  per-record** — a partial migration would leave some records reachable and
  others still stranded with no signal either way.
- Deliberately does not import `state.mjs`'s real `resolveContext`: this
  file's own test suite ships a minimal fake `state.mjs` to pin a controlled
  version for skill-sync tests, and a static import naming a binding that
  module does not export fails the whole ESM load, uncatchable, before any
  code runs. Migration detection therefore carries its own minimal, cycle-safe
  git-worktree walk-up replica — the same precedent `reservations.mjs`
  already set for the identical reason in `18b`.
- Idempotent: a re-run over an already-migrated worktree is a no-op, and an
  ordinary/main checkout gets zero footprint from this logic at all.

## The workspace registry: one write owner per workspace (multisession-native-19)

`workspace-store.mjs` (`controlRoot/.bee/runtime/workspaces/<id>.json`) is a
new store, structurally isolated the same way `workflow-store.mjs` and
`lease-store.mjs` already are (no import of `claims.mjs`):

- One record per physical checkout: `{id, type, root, branch, base_sha,
  write_owner_session, fence_epoch, attached_sessions}`.
- `registerWorkspace`/`unregisterWorkspace` are idempotent.
- `claimWriteOwnership` is strict and O_EXCL-lock-fenced: it refuses typed
  `WORKSPACE_OWNED`, naming the current holder, when a *different* live
  session already owns the workspace. `attachWorkspace` is forgiving — it
  records a read-only attach instead of throwing.
- Ownership reclaim follows the same heartbeat-staleness rule every other
  coordination-store liveness check uses (`holds-and-the-coordination-lock.md`
  B21): a caller-supplied `isOwnerLive` predicate decides, keeping the module
  free of a `claims.mjs` import.
- **Grant and ownership are proven independent and composable, never
  subsuming** (advisor condition 4, all four combinations tested): a worktree
  can be granted its own workspace-local store with no live write owner yet
  (nobody has written there), or have a live write owner while its grant is
  still pending — the two answer different questions (store topology vs.
  live-session concurrency) and neither implies the other.
- Wired in: `createFeatureWorktree` registers a workspace (rolled back on a
  later failure); `performCleanup` unregisters it alongside the grant.
  `claims.mjs`'s `createSession`/`claimCellFile` stamp and carry `workspace_id`
  on every session and claim, auto-looked-up from the acting session's own
  record. `packages/bee/hooks/bee-session-init.mjs` re-roots session creation onto
  `resolveContext.controlRoot` — closing the gap `18c`'s adapter comment had
  flagged as deferred — and lazily auto-registers the workspace on first
  touch.

## Write-policy resolution: observe / shared-disjoint / isolated (multisession-native D3, msn-20)

`applyWritePolicy` (`state.mjs`) resolves, per write-capable entry point, what
a second session in the same checkout is allowed to do — replacing the old
`startFeature` refusal ("N active worker session(s) remain … wait") on its
DEFAULT (non-lane) path:

- **`observe`** (`config.guards.write_policy`) skips workspace ownership
  entirely — unlimited concurrent read/analyze/review sessions in one
  checkout.
- **`shared-disjoint`** (opt-in only) requires an existing **exact-path**
  reservation before a write proceeds — never a broad or glob one — refusing
  typed `LEASE_REQUIRED` otherwise.
- **`isolated`** (the default whenever a write-capable session already lives)
  claims workspace-store write ownership for the acting session. A
  *different* live session already owning the workspace gets a typed
  `WORKSPACE_ISOLATION_REQUIRED` refusal naming the exact `--isolate`
  one-liner — the full message once per session, a shorter repeat after —
  or, with `--isolate` / `config.guards.auto_isolate` set, bee auto-creates a
  fresh feature worktree via `createFeatureWorktree` and attaches the caller
  as its owner, returning a redirect result so the caller's own write never
  lands in the contended checkout. The loud one-line cost disclosure fires
  exactly once per session (advisor condition 6).
- **Deliberately NOT wired into the lane path or into `cells claim`/
  `claim-next`'s actual enforcement** (`enforceIsolation: false` there):
  CONTEXT.md's "Scope boundaries" locks lanes-as-UX unchanged, and swarming's
  concurrent-claim mechanism is bee's own already-coordinated pattern for many
  workers sharing one workspace — blanket ownership enforcement there would
  silently reintroduce "another session is active, wait" against multi-worker
  swarming under a new name. `observe`/`shared-disjoint` still apply at every
  entry point regardless of this carve-out.
- Isolation's own register/create writes never invoke `checkWrite` (the write
  guard hook only intercepts Edit/Write/Bash tool calls, never `bee`'s own
  internal fs I/O) — proven by a dedicated test that creates an isolated
  worktree successfully under the most restrictive state (terminal phase,
  every gate unapproved), closing the self-deadlock risk advisor condition 6
  named.

## The write guard: one topology resolution, three deny classes (multisession-native-21)

`guards.mjs`'s `checkWrite` used to walk topology twice per call —
`resolveWriteRecord`'s own `resolveContext` call (gated on `sessionId`) and
`resolveHoldTopology`'s own separate `resolveRoots` call, plus a third,
lane-path-only `resolveContext` re-resolution. `multisession-native-21`
collapses all three into **one** upfront `resolveWriteTopology` (wrapping
`resolveContext(root)`, fail-open) that every branch — lane-bound or not —
now reads from, feeding `resolvePipeline`'s session → bound-lane → default
routing uniformly. `packages/bee/hooks/bee-write-guard.mjs` passes the adapter's
already-resolved `ctx.controlRoot` straight through instead of leaving
`guards.mjs` to re-derive it.

From that single resolution, exactly three deny classes fire — the same
three cited in the settled-behavior list this feature closes on:

- **(a) Cross-worktree exclusive-path holds** — unchanged msn-14 policy; see
  `cross-worktree-holds.md` "Three read taps, one voice" for the exclusive-
  resource list and the advisory-warning fallback for every other path.
- **(b) A same-workspace cross-session exact-path lease** — a lease taken in
  a *different* physical checkout no longer hard-blocks a same-named
  repo-relative path in another workspace, since every lease now carries the
  acting session's stamped `workspace_id` (msn-19); see
  `holds-and-the-coordination-lock.md` for the lease record itself.
- **(c) A non-owner default-path write in a workspace a different LIVE
  session already owns** — new with this cell, read-only (it never claims
  ownership itself): scoped exactly to where `applyWritePolicy`'s `'isolated'`
  mode governs — a real session, `write_policy` resolved to `'isolated'`, the
  DEFAULT (non-lane) pipeline, and `phase !== 'swarming'`.

Legacy/solo repos (no `workspace_id` anywhere, defaulting to `'main'` on both
sides) stay byte-identical under (b) and (c). The guard never acquires a
store lock to answer any of the three — `readWorkspace`/`readSession` are
plain reads — proven by a regression test with a genuinely held lock file
present; the guard never waits.

**The issue-#56 acceptance suite proves identity-mandatory checks and their legacy
carve-outs, not just the deny classes above (multisession-native D9 invariant 15,
msn-23).** `test_msn_invariants.mjs` (the numbered, fail-loud index over every
multisession-native invariant — see `returning-and-the-merge-gate.md` for the suite's
own design) devotes invariant 15 to a narrower claim than "the guard denies": that
write-capable operations refuse **without identity** specifically where identity is
already mandatory today — lease acquire and workspace ownership claim — while
everywhere else identity remains genuinely optional, and says so out loud rather than
quietly tightening it. Its own PASS-line output names three legacy carve-outs
explicitly, deliberately NOT closed by this cell (post-feature work, D9): (1)
`renewLease`/`releaseLease` omitting `presentedEpoch` stays byte-unchanged UNFENCED
legacy behavior; (2) a sessionless call to `checkWrite`/`applyWritePolicy` proceeds
untouched even into a workspace a live session owns — the workspace-ownership deny
class (c) above only ever fires for a session-identified caller; (3) `bee`'s
`writeHandoff` no-workflow fallback (see `areas/workflow-state/handoff.md`) stays one
more release per advisor digest slice5 condition E, reclassified as a projection
writer rather than removed. The workspace-ownership deny half reuses
`test_guards.mjs`'s existing deny-class-(c) proof rather than re-deriving it; the lease
half is fresh. Evidence: trace `.bee/cells/multisession-native-23.json`, commit
06cd209; advisor digest
`docs/history/multisession-native/reports/advisor-digest-slice5.md`.

## Where it lives (reading map)

- Resolver: `resolveContext`/`resolveRootsCore`/`resolveRoots` (compat
  wrapper) in `packages/bee/lib/state.mjs`. `controlRootFor(root)`
  (same file) is the shared helper every re-rooted call site in this concept
  uses; `reservations.mjs` and `packages/bee/hooks/adapter.mjs` each carry their own
  minimal, cycle-safe, fail-open replica for structural-isolation reasons
  documented above.
- Onboarding migration: `detectWorktreeMigration`/`applyWorktreeMigration` in
  `packages/bee/scripts/bee onboard`; five fixture scenarios in
  `test_bee onboard` (happy path, conflict abort with zero mutations,
  re-run idempotency, identical-content dedup, zero footprint on an
  ordinary/main checkout).
- Workspace registry: `workspace-store.mjs`
  (`registerWorkspace`/`unregisterWorkspace`/`claimWriteOwnership`/
  `attachWorkspace`); wiring in `worktree-store.mjs` and `claims.mjs`;
  `packages/bee/hooks/bee-session-init.mjs`. Tests: `test_workspace_store.mjs`,
  `test_worktree_store.mjs`, plus 3 rows in `test_claims.mjs` — including a
  real exactly-one-owner race proven through the lock.
- Write policy: `applyWritePolicy` in `state.mjs`; `startFeature`'s default
  path in `bee`. Tests: `test_write_policy.mjs` (11), two rewritten +
  one new CLI-level test in `test_cli_state.mjs` (the retired "any live
  heartbeat blocks a start" test replaced by two tests distinguishing that
  from the new actually-live-owner behavior).
- Unified write guard: `resolveWriteTopology`/`checkWrite` in `guards.mjs`;
  `ctx.controlRoot` threaded from `packages/bee/hooks/bee-write-guard.mjs`. 7 new tests in
  `test_guards.mjs` (same-workspace lease scoping, ownership deny + its exact
  scoping boundaries, legacy no-records byte-identical, corrupt-workspace
  fail-closed, linked-worktree topology, never-waits regression).
- Advisor consult for this whole slice:
  `docs/history/multisession-native/reports/advisor-digest-slice4.md`
  (conditions 1-7, verdict proceed-with-conditions); the re-slice decision is
  `.bee/decisions.jsonl` id `89a4a87b`.
- Acceptance suite (D9 invariant 15, msn-23):
  `packages/bee/tests/test_msn_invariants.mjs` (see
  `returning-and-the-merge-gate.md`'s Pointers for the suite's full index).
  Evidence: trace `.bee/cells/multisession-native-23.json`, commit 06cd209;
  advisor digest
  `docs/history/multisession-native/reports/advisor-digest-slice5.md`.
