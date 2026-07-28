# Validation Diet — Plan

**Feature:** validation-diet
**Lane:** high-risk (hard-gate flag: validation removal)
**Date:** 2026-07-28
**Context:** `docs/history/validation-diet/CONTEXT.md` — D1-D15 locked, cited never reinterpreted.
**Shape:** epic map, 3 slices.

## Approach

### Chosen path

Cut the machine first, the surface second, the doctrine last. Each slice leaves
the repo runnable and green; no slice depends on a later slice's prose to make
sense.

The ordering is forced by one fact: `packages/bee/lib/guards.mjs` and
`packages/bee/hooks/bee-session-close.mjs` both hardcode `"validating"`
independently of the phase enum and fail open (per D3/D13). Until the machine
layer is correct and proven by a test that drives the real state machine, every
prose edit is cosmetic and every green test is potentially vacuous.

**Slice 1 — machine.** Phase enum, both hardcoded guards, the legacy-state
migration, the narrowed deny tail, the merged gate's approval path, and the
derived test migration. After slice 1 the chain runs `planning → swarming`
through one merged gate, and a source write before that gate is proven denied by
a test that walked the real state machine to get there.

**Slice 2 — surface.** Delete `state validation-cache` and the `bee-validating`
skill tree, fold SMALLER PATH and the review wave into `bee-planning`, make
spikes opt-in by change class, repair the three fixtures that reference the
deleted tree, regenerate the four mirror roots.

**Slice 3 — doctrine.** The derived completeness sweep of D11, the new evidence
rule of D9, the `.bee/spikes/` contract narrowing of D10, and the byte fence.

### Resolved at planning time

Both Deferred-To-Planning questions in CONTEXT.md are answered; evidence below.

**Q1 — which phase carries the merged gate.** Answer: **`planning`**. No new
phase, no renamed `validating`.

- `packages/bee/lib/guards.mjs:1307-1308` — the write gate checks
  `record?.approved_gates?.execution === true` for **every** member of
  `GATED_PHASES`, never `shape`. `planning` and `validating` are already treated
  identically. A merged gate flipping both while phase is `planning` satisfies
  this immediately.
- `packages/bee/hooks/test_hook_contracts.mjs:2253-2254,2284` — already fixtures
  `phase: "planning", approved_gates: { shape: true, execution: false }` as a
  legitimate live state, denied on `execution`. Nothing assumes `planning` means
  "shape not yet approved".
- `skills/bee-planning/references/planning-reference.md:202` — the tiny/small
  merged gate already sets both gates while phase is `planning` and hands off
  straight to `swarming`, skipping `validating` entirely. The pattern exists and
  ships today.
- `packages/bee/lib/cells.mjs:1688-1710` — `claimCell` checks only
  `gateApproved(gateSource, 'execution')`; there is no phase check at all. The
  swarming door is gate-keyed, not phase-keyed.
- `packages/bee/lib/state.mjs:2762-2764` — `isDebtGuardedDeparture(from, target)`
  is `String(target) !== String(from)`, derived not enumerated, so it fires
  identically on `planning → swarming`. **No change needed.**

Rejected: a new post-shape phase (briefing never writes `state.phase` —
`skills/bee-briefing/SKILL.md:37,81` — and adding one forces `PHASE_GATE`
one-to-many, the exact shape merging exists to avoid, plus `TERMINAL_PHASES`,
`SCRIBING_RUN_FROM`, every skill's phase list, and the enum byte-identity test).
Rejected: keeping `validating` under a new name (reintroduces the vocabulary
D1/D3/D11 delete, for zero enforcement benefit).

Exact new values:

- `packages/bee/lib/state.mjs:41-51` — `PHASES` drops `'validating'`;
  `KNOWN_PHASES` derives unchanged.
- `packages/bee/lib/guards.mjs:142` — `GATED_PHASES = new Set(['exploring','planning'])`.
- `packages/bee/hooks/bee-session-close.mjs:26` — `PHASE_GATE = Object.freeze({ planning: "execution" })`.
  Note the value change from `"shape"` to `"execution"`: every other enforcement
  surface for phase `planning` already reads `execution`, and D15 keeps the two
  flipped in lockstep, so this removes the last surface treating them as
  asynchronous.

**Q2 — what D13's tail flip breaks.** Answer: **a blanket deny is unsafe; the
deny narrows to `!isKnownPhase(phase)`.**

- The true tail is `packages/bee/lib/guards.mjs:1366`, not `:1318` as CONTEXT.md
  originally cited (`:1319` is the `GATED_PHASES` branch's local return). The
  correction is recorded inline in D13.
- `reviewing`, `scribing`, `compounding`, `grooming` are real `PHASES` members
  matched by no branch — they reach the tail and are allowed today, with **no**
  `underAllowedPrefix` or `idle_gate` carve-out on that path. A blanket deny
  hard-blocks every write during ordinary post-approval work.
- The dangerous case is already safe: no `.bee/state.json`, corrupt state, or
  absent/empty `phase` all collapse to `'idle'` (`guards.mjs:1240`,
  `state.mjs:1067,1086-1092`) and are handled by `TERMINAL_PHASES`
  (`guards.mjs:151,1294-1305`). The tail is never reached.
- A bound lane whose record is missing or corrupt never reaches the dispatch at
  all — `guards.mjs:1130-1133` short-circuits with `{ allow: false, kind: 'lane' }`.
- One caller: `packages/bee/hooks/bee-write-guard.mjs:1195`. A deny becomes
  `process.exitCode = 2` at `:1340-1345` and is **not** swallowed by the outer
  fail-open catch at `:1293-1296` (that guards thrown exceptions only). The flip
  is a real hard block, not advisory.
- One test flips by design, not by accident: `packages/bee/tests/test_guards.mjs:1036-1059`
  ("unknown phase falls through open") builds `phase: 'executing'` — not even an
  enum member — and asserts allow. It moves to asserting deny.

### Risk map

| Risk | Where | Mitigation |
|---|---|---|
| Silent un-gating: an incomplete cut leaves writes ungated while every test stays green | `guards.mjs:142`, `bee-session-close.mjs:26` — both hardcoded, both fail open | vd-4's real-state-machine test (D4). No fixture-built phase may prove gating. |
| A live repo in `phase: "validating"` cannot leave it | `bee.mjs:2931-2934` refuses `state set` on an invalid pre-mutation phase | vd-2's read-coercion (D13), with a test starting from a legacy state file |
| Deny tail over-reaches into normal work | `guards.mjs:1366` | Narrowed to `!isKnownPhase` (Q2); the four in-enum phases keep allowing, pinned by a new test |
| High-risk features silently lose the advisor consult | `bee.mjs:3292-3301` guards `execution` only | vd-3 (D14) |
| Half-revoked merged gate after `plan-rev bump` | `bee.mjs:3310-3330` stamps `execution` only | vd-3 (D15) |
| Doctrine half-migrated — each half passes inspection alone | ~45 live files | vd-12's derived criterion (D11), run as a command, not a checklist |
| Mirror drift between `packages/bee/` and `.bee/bin/` | 10 vendored files | Twins move in the same commit; `test_misc.mjs:1881` catches one-sided edits |

### Files and order

Slice 1 touches `packages/bee/lib/state.mjs`, `lib/guards.mjs`,
`hooks/bee-session-close.mjs`, `bee.mjs`, their `.bee/bin/` twins, and the
derived test set. Slice 2 touches `lib/state.mjs` (cache block),
`lib/command-registry.mjs`, `bee.mjs`, `skills/bee-validating/**` (deleted),
`skills/bee-planning/**`, `scripts/skill_lint.mjs`,
`scripts/skill-body-budget.json`, `scripts/tests/test_gate_bypass_doctrine.mjs`,
and the four mirror roots. Slice 3 touches `packages/bee/AGENTS.block.md`, root
`AGENTS.md`, `skills/bee-hive/**`, and the `docs/` surfaces D11's criterion
selects.

## Slice Map

### Slice 1 — machine (current slice)

| Cell | Work | change_class | deps |
|---|---|---|---|
| vd-1 | Drop `'validating'` from `PHASES`; `GATED_PHASES = {'exploring','planning'}`; `PHASE_GATE = { planning: "execution" }`. Both `.bee/bin/` twins in the same commit. | migration | — |
| vd-2 | D13: coerce a legacy `validating` phase at read via the `isKnownPhase(phase) ? phase : …` precedent; narrow the `guards.mjs:1366` tail to deny only `!isKnownPhase(phase)`. | migration | vd-1 |
| vd-3 | D2/D14/D15: merged shape+execution approval path in `state gate`; inherit the high-risk advisor-consult refusal; extend `approved_for_plan_rev` to stamp both fields. | api | vd-1 |
| vd-4 | D4: derive the fixture set (`rg -lw validating` over the test estate), migrate or retire every hand-written phase literal starting with `scripts/tests/test_conformance.mjs:113`, and add the real-state-machine gating test. | test | vd-1, vd-2, vd-3 |

`migration` change class means vd-1 and vd-2 owe red-first proof in every lane
per R55 — the "before" is the current behavior each one alters, captured from
`git show` or a pre-change run, never a throwaway probe (D9).

### Slice 2 — surface

| Cell | Work | change_class |
|---|---|---|
| vd-5 | D7: remove the `state validation-cache` verbs entirely — surface, implementation, exports, tests, gitignore entry. Call sites re-derived, not read off CONTEXT.md. | api |
| vd-6 | D1/D5/D6/D8: delete `skills/bee-validating/`; fold SMALLER PATH and the merged review wave into `bee-planning`; make spikes opt-in by change class; delete the feasibility matrix and delta rule. | behavior |
| vd-7 | Repair the three fixtures referencing the deleted tree (`test_gate_bypass_doctrine.mjs:30`, `skill_lint.mjs:112-124`, `skill-body-budget.json:19` **and** `:37`), then regenerate the four mirror roots and their `.bee-render.json` sidecars. | refactor |
| vd-8 | Slice-tail test over slice 2's net behavior. | test |

### Slice 3 — doctrine

| Cell | Work | change_class |
|---|---|---|
| vd-9 | `packages/bee/AGENTS.block.md`: critical rule 1, critical rule 3, the chain line, the working-files table, plus D9's new evidence rule. Re-render root `AGENTS.md` through onboarding, never by hand. Byte fence: 15000 hard / 14000 warn, 12692 today. | behavior |
| vd-10 | `skills/bee-hive/**` (priority rule 6, the Four Gates block, routing-and-contracts, go-mode STEP 4) and the remaining skills naming the removed stage. | behavior |
| vd-11 | The `docs/` surfaces D11's criterion selects — handbook, numbered docs, knowledge bundle, README, specs. `docs/handbook/stages/validating.md` deleted whole. | behavior |
| vd-12 | D10: reword `guards.mjs:1106,1118-1119` so they stop advertising `.bee/spikes/` as the home for disposable proof. Then run D11's derived criterion as a command and record its output as the completeness evidence. | behavior |
| vd-13 | Slice-tail test over slice 3's net behavior. | test |

## Open Questions

None blocking. Both Deferred-To-Planning items are resolved above with evidence.

## Rollback

One commit per cell with the cell id — `git revert` per cell in reverse
dependency order. Slice 1 is the only slice with a persisted-state effect;
vd-2's coercion is additive (it reads a value it did not write), so reverting it
restores the prior read path without leaving migrated data behind. Slices 2 and
3 are deletions and prose; reverting restores the files byte-identically, and
the four mirror roots regenerate from source at any time.
