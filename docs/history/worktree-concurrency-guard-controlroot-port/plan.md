---
artifact_contract: bee-plan/v1
mode: high-risk
approved_gate2: 2026-07-26T12:32:00Z
---

# Plan: Worktree Concurrency Guard — Port to bee 1.18.2

Mode: `high-risk` — hard-gate flag (data loss prevention) carries over from the original feature; also multi-domain (build-time regen chain + 3 security-relevant runtime files + 2 test files) and touches a covered contract (PR #64's existing, reviewed behavior must not regress).
Why this is the least workflow that protects the work: this is a live, unresolved merge conflict across ~20 files including bee-core's own concurrency-safety guard — a wrong resolution could silently reintroduce either of the two P1s the independent review already found and fixed once.

## Requirements (from CONTEXT.md)

- Port-D1: merge (not rebase) origin/main into the branch.
- Port-D2: mechanical/generated conflicts resolved via regen or deletion-matching-upstream, never hand-merged.
- Port-D3: minimal, behavior-preserving re-application — no redesign, no scope expansion.
- Port-D4: concurrency check uses `controlRoot`; filesystem scan stays on `root`.
- Port-D5: write-guard reuses `ctx.controlRoot`, no new topology resolution.
- Port-D6: worktree-new derives its own `controlRoot` via `controlRootFor(mainRoot)` + `resolveSessionId`.
- Port-D7: refusal drops the `[CODE]` prefix, matches the new plain-`Error` convention.
- Port-D8: test rows re-applied at new paths, same harness idioms.

## Discovery

L1 quick verify (folded in, no separate discovery.md per the fan-out table): confirmed `render_plugin_skill_trees.mjs` still exists and runs, but its `TARGET_ROOTS` dropped `.agents/skills` and plain `.claude/skills` — only `.claude-plugin/skills` and `.codex-plugin/skills` remain (`git show origin/main:scripts/render_plugin_skill_trees.mjs:38-41`). `onboard_bee.mjs` relocated to `packages/bee/scripts/onboard_bee.mjs`, same role. This changes the mechanical-conflict resolution step: `.agents/`/`.claude/` skill-tree conflicts resolve by **accepting upstream's deletion**, not by re-running a regen that no longer targets them.

## Approach

See `docs/history/worktree-concurrency-guard-controlroot-port/approach.md` (high-risk lane — fan-out per decision 0009). Its risk map flags the controlRoot-signature shape and the scan-vs-concurrency-root scoping as the two things most likely to go wrong; both get explicit test coverage in the current slice.

## Shape — epic map

**Feature outcome:** `wt/worktree-concurrency-guard` merges onto `origin/main` (bee 1.18.2) with zero conflicts, PR #64 shows `mergeable: MERGEABLE`, and every proven behavior (D1-D6 of the original feature, plus both P1 fixes) still holds — now correctly scoped to `controlRoot` where the new architecture requires it.

**Repo-reality basis:** confirmed via direct reads of `origin/main`'s actual files (not inferred) — exact line numbers for every integration point are already cited in CONTEXT.md's Existing Code Context.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| E1 | The merge itself | A live, uncommitted `git merge --no-commit` is one shared working-tree state — every other epic below is a SECTION of resolving it, not an independently claimable cell | 1 slice, all epics below are sub-steps of the same cell | Full verify green before the merge commit lands |

**Why one cell, not five:** approach.md's "Rejected alternatives" already rules out splitting this across multiple claimed cells — the uncommitted merge state cannot be safely shared between independent claims/reservations. The "epics" (mechanical conflicts, guards.mjs, bee-write-guard.mjs, bee.mjs, tests) are sections inside one cell's `action`, executed in the stated order by one worker, committed once at the end.

**Slice queue:** one slice, one cell (`port-1`). Feasibility status: proven — every integration point is cited with real evidence, both flagged risks have explicit proof requirements below.

**Current slice to prepare:** `port-1` — the entire merge resolution, in the order approach.md's "Files and order" states.

## Test matrix

High-risk lane — probes written out per edge dimension that applies:

- **controlRoot vs root scoping:** a target physically under `root` whose session records live under a DIFFERENT `controlRoot` (simulating a linked-worktree topology) — the filesystem scan still finds it (scoped to `root`), the concurrency check correctly consults `controlRoot`'s session records, not `root`'s.
- **Regression — original feature:** every existing row from `hooks/test_write_guard.mjs` (85 rows) and `scripts/test_worktree_companion.mjs` (17 rows) passes unchanged at the new paths.
- **Regression — P1 fixes:** self-exclusion (wcg-fix-1) and fail-closed-on-error (wcg-fix-2) both still hold at the new call sites.
- **Mechanical/generated files:** `ledger_parity.mjs --check`, `release_manifest.mjs --check` (or its 1.18.2 equivalent — verify the command still exists under this name), `knowledge check`, `backlog render --check` (or write + diff) all clean after resolution.
- **Error-convention consistency (Port-D7):** the new refusal's message is grep-able/distinguishable in practice (even without a `[CODE]` prefix) — confirmed fresh-eyes review already checked no test depends on the old prefix.
- **Deletion-matching-upstream:** `.agents/skills/bee-hive/{scripts,templates}` and `.claude/skills/bee-hive/{scripts,templates}` are deleted (matching upstream), not regenerated.

## Out of scope

- Deeper integration with the new workspace-ownership system beyond composing alongside it (Port-D3, Deferred Ideas).
- Any change to the original feature's locked D1-D6 product decisions — this is a relocation, not a redesign.
- Resolving the pre-existing `.bee/cells/rel1150-1.json` stray-record conflict beyond whatever is mechanically needed to unblock the merge (tracked separately as `worktree-scaffolding-cell-leak`, `p-9c48a67c`).
