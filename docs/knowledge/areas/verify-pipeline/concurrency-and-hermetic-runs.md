---
type: bee.area
title: Verify Pipeline — concurrency safety and hermetic runs
description: "Keeping whole-tree regeneration lock-serialized and atomic-swapped, keeping every child suite hermetic to session identity, and the deterministic race/isolation proofs that back both claims."
timestamp: 2026-07-22
bee:
  id: verify-pipeline-concurrency-and-hermetic-runs
  lifecycle: active
  areas: [verify-pipeline]
  required_context: [areas/verify-pipeline/suite-topology-and-discovery.md]
  decisions: [verify-parallel-runner, F4-D9]
  sources: ["contention-split cells cs-1/cs-2a/cs-2b/cs-3/cs-4 (locked tmp-swap render; traces in .bee/cells/, 2026-07-20)", "verify-parallel-runner (parallel pool, 2026-07-20, commit 6caceb4)", "hardening-1-7-10 cells 1710-1..1710-11 (2026-07-21 — session-id env scrubbing at both runner and bootstrap for hermetic local/CI parity; deterministic fs-barrier claim-race negative control, 10/10 both env modes; chmod-based write-failure simulations skip loudly under root; nested-clone isolation regression pins root resolution off the parent repo's git config)", "docs/specs/verify-pipeline.md#R4", "docs/specs/verify-pipeline.md#R5", "docs/specs/verify-pipeline.md#E2", "docs/specs/verify-pipeline.md#E3", "docs/specs/verify-pipeline.md#E4", "docs/specs/verify-pipeline.md#P5", "okf-integration-close-f4 cell f4-4 (the third identity variable — the agent-name prefix bee's own swarm rule mandates was manufacturing a false red in the coordination guards; trace in `.bee/cells/`, 2026-07-22)"]
  authoritative_for: "verify-pipeline: concurrency safety and hermetic runs"
---

# Verify Pipeline — Concurrency Safety and Hermetic Runs

Two features working in parallel must not need to edit the same test
artifact unless they genuinely change the same module, and a developer's
local run must never diverge from what CI would report just because the
local shell happened to inherit ambient session identity. This concept owns
the run's own safety: locked, atomic-swapped regeneration; multi-worker
checkout etiquette; and hermetic construction proven by deterministic race
and isolation suites. How suites are shaped and discovered in the first
place is `suite-topology-and-discovery.md`.

## Behaviors & Operations

- **Concurrency-safe artifact regeneration.** The plugin-tree render locks
  (`plugin-render` store lock), builds into a tmp sibling dir, and swaps by
  rename; concurrent renders serialize and converge. Proven by a permanent
  race suite whose deliberate-red twin replays the old torn-tree behavior.
- **Hermetic by construction, not by convention (hardening-1-7-10).** Every
  child suite the verify runner launches has `CLAUDE_CODE_SESSION_ID` and
  `BEE_SESSION_ID` scrubbed from its own environment before it starts, and any
  suite whose subject matter is session-sensitive (claim/session/identity
  behavior) scrubs the same variables again at its own bootstrap, in case
  something downstream of the runner re-introduces them. The effect is that a
  developer's own local shell — which may well be sitting inside a live
  harness session with one or both variables set — cannot produce a run that
  is only green because it inherited that ambient identity: a local run and a
  CI run now see the identical (absent) session identity, so a local green
  cannot silently diverge from what CI would report.

- **The scrubbed identity is THREE variables, not two — and the third one bee
  itself mandates.** Alongside the two session variables, the agent-name
  variable is scrubbed by the runner and at the bootstrap of the write-guard
  suite. It is the sharpest of the three because two of bee's own rules collide
  on it: the swarm discipline requires prefixing write-heavy shell commands with
  the acting agent's name, and the session discipline requires running the
  configured verify command. A worker that obeys both leaks its own agent name
  into every spawned suite, where it becomes the *acting agent identity* inside
  the write guard's cross-session hold check — and the assertion "the acting
  session's own hold must never block its own write" inverts. The result is a
  **false red in the coordination guards**, which is the worst possible place
  for one: it reads exactly like a real cross-session defect. The acceptance
  test is therefore stated as a property, not a run: **the chain passes with the
  agent-name prefix set and without it**, identically.

## Business Rules

- **R4** — Whole-tree regeneration steps must be lock-serialized and
  atomic-swapped; a crashed regen may never leave a torn tree.
- **R5** — Multi-worker checkout etiquette: cap → commit (own hunks only) →
  release reservations, in that order; aggregate regen artifacts
  (render sidecars, release manifest, onboarding state) ride a
  consolidated pass, not per-cell commits.

## Edge Cases Settled

- **E2** — Known WSL2 host flakes under heavy concurrent load: `test_store_lock`,
  `test_render_race` — a flake is rerun once and both runs reported;
  a clean rerun is acceptable, hiding a flake is not.
- **E3** — The claim-race negative control (proving exactly one winner among
  simultaneous claimants) no longer relies on timing to force the race: a
  deterministic filesystem-barrier handshake holds every racer at the same
  starting line and releases them together, proven 10/10 clean under both
  session-id-present and session-id-absent modes. A chmod-based simulation of
  a write failure is a separate, narrower proof (permission denial, not a
  race) and skips loudly — rather than reporting a false pass or a false fail
  — when the suite itself is running as root, since root ignores the
  permission bits a chmod-based simulation depends on (hardening-1-7-10).
- **E4** — A nested-clone isolation regression pins down a scoping requirement that
  worktree/root-resolution code must hold everywhere: cloning the repository
  into a nested directory and running root resolution or a worktree operation
  from inside that nested clone must never read or write the PARENT
  repository's own git configuration. Without this pin, a root-resolution bug
  could silently walk past the nested clone's own `.git` and act on the
  outer repository instead (hardening-1-7-10).

## Pointers (implementation)

- **P5** — `scripts/render_plugin_skill_trees.mjs`, `scripts/tests/test_render_race.mjs` —
  locked tmp-swap render + race proof.
