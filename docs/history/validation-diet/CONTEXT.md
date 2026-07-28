# Validation Diet — Context

**Feature slug:** validation-diet
**Date:** 2026-07-28
**Exploring session:** complete
**Scope:** Deep
**Domain types:** CALL (bee CLI surface), RUN (workflow chain + hooks), READ (doctrine docs)

## Feature Boundary

Remove bee's predictive validation layer — the `bee-validating` skill, the
`validating` phase, the feasibility matrix, the delta rule, and the
`state validation-cache` verbs — folding the one survivor (SMALLER PATH) and the
review wave into `bee-planning`, merging Gate 3 into Gate 2, and replacing
probe-then-delete evidence with a doctrine that evidence is what the build
already emits. Ends when the chain runs `planning → briefing → swarming` green,
the pre-execution write guard is proven still closed by a real state-machine
test, and a repo left in the removed phase migrates without bricking or
un-gating. Does not touch the executing-side proof-tier matrix (R55) or the cap
evidence contract.

## Feature Origin

Owner directive, this session. The owner's position, verbatim in substance:
all effort belongs before a complete plan; once the plan exists, ship, and look
at the result at the end rather than challenge the solution before building it.
The owner explicitly rejects re-litigating that stance — it is an input, not an
open question.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | `skills/bee-validating/` is deleted outright — SKILL.md, both references, CREATION-LOG.md, `agents/openai.yaml`, and all four rendered mirrors. Its one survivor, the reality gate's SMALLER PATH check, moves into `bee-planning` as a single inline check. | The `spike` lane already exists in the hive lane table, so opt-in spikes need no skill of their own. Three live fixtures reference the deleted tree and all three move in the same cell as the deletion, never after: `scripts/tests/test_gate_bypass_doctrine.mjs:30` (pins prose inside the file; fails outright on absence via `:54`), `scripts/skill_lint.mjs:112-124` (unguarded `readFileSync` of both the SKILL and its reference — advisory only, `check()` catches and the script always exits 0, so it degrades to a warning and will **not** catch a miss), and `scripts/skill-body-budget.json` at **both** `:19` (budgets entry) and `:37` (the `migrated[]` array). |
| D2 | Gate 3 merges into Gate 2. One gate at the end of briefing approves shape and execution together, flipping `approved_gates.shape` and `approved_gates.execution` in one call. Both fields survive in `.bee/state.json`; only the question merges. Preconditions and revocation: D14, D15. | There is no `briefing` phase in the enum — briefing runs inside phase `planning` — so a separate Gate 3 would need `PHASE_GATE` (`hooks/bee-session-close.mjs:26`) to become one-to-many. Merging keeps it one-to-one. `bee state gate` takes a single `--name` (`lib/command-registry.mjs:724-725`), so the merged approval is a new code path, not a config change. Precedent: `skills/bee-planning/references/planning-reference.md:202` already merges shape+execution for tiny/small and records both. Under the owner's `gate_bypass: total` both already auto-approve, so the merged *question* changes no observed behavior for this repo — the approval *mechanics* do change, which is why D14/D15 exist. |
| D3 | `"validating"` is removed from `PHASES` and `KNOWN_PHASES` (`packages/bee/lib/state.mjs:41-52` and its `.bee/bin/` mirror). `GATED_PHASES` (`lib/guards.mjs:142`) and `PHASE_GATE` (`hooks/bee-session-close.mjs:26`) are retargeted to the phases that now carry the merged gate. | Both guards hardcode the literal `"validating"` independently of the enum and **fail open** — a partial cut silently stops gating pre-execution source writes instead of erroring. See D4. |
| D4 | D3 ships with at least one test that drives the **real state machine** to the pre-execution phase and asserts a source write is denied — never a hand-built `phase:` fixture. The completeness criterion is a derived property, not a list: **at close, no test may assert gating behavior from a hand-written phase literal, and no test may construct a `phase:` value the state machine can no longer produce.** The set is computed at planning time (`rg -lw validating` over the test estate returns 12 files today), never copied from this document. `scripts/tests/test_conformance.mjs:113` is migrated first — it is bee's canonical conformance proof that a source write before the execution gate is denied, and it builds `phase: "validating"` by hand via `buildStoreFixture` (`:81-93`), so after the cut it would drive an unknown phase straight into the fall-through path D13 closes. | Repo critical patterns: "A coverage gate derives its ground truth; it never compares two hand-authored lists" and "A scan scope set from assumption passes green while hiding the very bug it was built to catch". An enumerated list in a locked decision reproduces the exact defect the decision exists to prevent — the fresh-eyes pass found 7 fixture sites beyond the 5 originally listed. |
| D5 | The review wave (the merged structure + cold-pickup reviewer) moves into `bee-planning`, dispatched when the shape is drafted, findings held until the merged gate. Cost stays `max(reviewer, planning)`, never the sum. | It is review value, not feasibility. Repo critical pattern: "Pre-code gates filter spec defects; only diff review catches implementation defects" — dropping it entirely would lose the spec-defect layer with nothing downstream replacing it. |
| D6 | The feasibility matrix and the delta rule are deleted, prose and all, with no replacement artifact. | Zero machine coverage exists: `FEASIBILITY MATRIX` / `feasibility_matrix` return no hits under `packages/bee/` or `scripts/tests/`. Nothing to migrate. |
| D7 | The `state validation-cache` verbs are removed entirely — command surface, implementation, exports, tests, and the gitignore-managed cache file. No deprecation window, no dormant code. | Zero callers outside the deleted skill: the only invocations in the repo are prose in `bee-validating/SKILL.md:40` and `validation-reference.md:50`. The current call sites are inventoried under Existing Code Context, not locked here — planning re-derives them, because a locked decision that carries line numbers becomes false the moment the file shifts and cannot be silently corrected. |
| D8 | Spikes become opt-in by change class, routed through the existing `spike` lane rather than a phase step. A spike is owed only when the change is `migration`, `security`, or reaches an external system with a side effect, **or** when it uses an API/library/technique with no in-repo precedent. Everything else builds directly. | Mirrors R55's shape (PBI p-8afb88a4), which already narrowed red-first proof by change class. Revert is cheap under one-commit-per-cell plus the empirical merge gate; it is not cheap for a migration already run, a release already published, or an external call already made. |
| D9 | New evidence doctrine, stated positively in `AGENTS.md` and `packages/bee/AGENTS.block.md`, scoped to **evidence**: **never author an artifact whose only purpose is to be deleted as evidence.** Evidence is what the build already emits — red test output, stack trace, verify output, `git diff`, `git show` of the prior state. A red-first repro is written at the real path where it will ship, run red, and kept; it is never a throwaway probe. Opt-in feasibility spikes (D8) are explicitly outside this rule. | The "as evidence" scope is load-bearing: an unscoped ban would contradict D8, D10, `AGENTS.block.md:85` ("`.bee/spikes/<feature>/` <- disposable feasibility proofs") and `lib/guards.mjs:1106,1119` in the same shipped file — a feasibility spike is by definition an artifact authored to be deleted. Verified as an addition, not a removal: no live doc binds red-first evidence to `.bee/spikes/` — `worker-details.md:98` defines the "before" as `git show <pre-change-commit>:<file>` or a failing pre-change check, `:182` as a scoped test-filter run; a full live sweep of `spikes` ∩ `red\|evidence\|proof\|repro` returns only feasibility uses. The probe-then-delete pattern was agent improvisation filling a gap, and deleting the probe makes the evidence unauditable, since the commit never carries it. |
| D10 | `.bee/spikes/` survives with a narrowed contract: opt-in feasibility spikes (D8) and exploring's SEE mocks only. It is never an evidence directory. The generic write-guard fallback messages at `lib/guards.mjs:1106,1118-1119` are reworded so they stop advertising it as the home for disposable proof. | Exploring's SEE mock lives at `.bee/spikes/<feature>/mocks/` (`bee-exploring/SKILL.md:27`) and the path is a scratch home in `lib/guards.mjs:103`; retiring the directory would break an unrelated, still-wanted mechanism. |
| D11 | Every doctrine surface naming the removed layer is updated in the same feature, not deferred. The completeness criterion is derived, not listed: **at close, no live file may describe the validating stage, the `validating` phase, or a standalone Gate 3 as current behavior.** Exception set — the only places the old names may survive: `docs/decisions/**`, `docs/history/**`, and D13's legacy-coercion code path. Planning computes the set (`rg -lw validating` over live surfaces returns ~45 files today, versus the 15 originally listed here). | A half-migrated doctrine layer is the repo's own named failure: "A system contradicting itself hides in the seam, because each half passes inspection alone." The fresh-eyes pass found ~30 unlisted surfaces, including four that describe the code D3 edits: `docs/07-contracts.md:99` (the written contract for `GATED_PHASES`), `docs/02-architecture.md:234` (the phase enum verbatim), `docs/knowledge/areas/workflow-state/overview.md:61` (the closed vocabulary, in the state layer read first), and `docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md:70` (R16b, a live rule about the removed layer). Also unlisted: `AGENTS.block.md:38` critical rule 3, which names Gate 3 and mirrors `lib/cells.mjs:1706-1707`. |
| D12 | `docs/decisions/*` and `docs/history/**` are not edited. Historical `auto-approved Gate 3` audit lines and closed feature records stay exactly as written. | Append-only history. Rewriting it would destroy the provenance the decision log exists to hold. |
| D13 | An existing `.bee/state.json` or `.bee/lanes/*.json` holding `phase: "validating"` is coerced at read to the phase carrying the merged gate — following the existing precedent at `lib/state.mjs:3299` (`isKnownPhase(phase) ? phase : 'idle'`). In the same change, the write guard's fall-through tail flips from `return { allow: true }` to a **deny** on any unrecognized phase. **Citation corrected at planning time:** the true tail is `lib/guards.mjs:1366`, not `:1318` (`:1319` is the `GATED_PHASES` branch's own local return). **Scope narrowed at planning time, decision intent unchanged:** "unrecognized" means `!isKnownPhase(phase)` (`lib/state.mjs:54-56`) — the four known-but-unhandled phases `reviewing`, `scribing`, `compounding`, `grooming` keep falling through to allow. A blanket deny would hard-block every write during ordinary post-approval scribing and compounding work, since that branch carries no `underAllowedPrefix`/`idle_gate` carve-out. | Without this the cut bricks live repos and un-gates their writes at once: `bee.mjs:2931-2934` refuses `state set` before any write when the *pre-mutation* phase is unknown, and `state set` is the only verb that changes phase — so a repo in `validating` cannot leave it; meanwhile an unknown phase matches neither `TERMINAL_PHASES` nor `GATED_PHASES` nor `'swarming'` (`lib/guards.mjs:1294-1318`) and falls through to allow, so source writes pass ungated and silently, and `hooks/bee-session-close.mjs:395-398` (`if (!gate) return null`) stops covering the repo too. The tail flip closes the fail-open door permanently — the root cause behind D3/D4's danger — and may surface other paths currently passing by accident, which is a finding, not a regression. |
| D14 | The merged gate inherits the high-risk advisor-consult precondition: it refuses for `mode: high-risk` when `advisor_ref` is missing or stale, exactly as the standalone execution gate does today (`bee.mjs:3292-3301`, AO3/AO13). The consult therefore has to happen during planning/briefing rather than validating. | The precondition is asymmetric today — it guards `execution` and not `shape` — so a naive merge would silently drop it for every high-risk feature. Inheriting keeps the guarantee and only moves when it fires. |
| D15 | `state plan-rev bump` revokes **both** `shape` and `execution`, and re-approval re-asks the one merged question. The `approved_for_plan_rev` stamp extends to cover both fields. | Today only `execution` carries the stamp (`bee.mjs:3310-3330`, `lib/state-projection.mjs:132-140`), so after a bump a "merged" gate would sit half-revoked — execution false, shape true — with no `validating` phase left to return to. A plan-rev bump means the shape itself changed, so revoking shape is the honest reading, not a tightening. |

### Agent's Discretion

- Slice boundaries, cell count, and dependency order — planning's call, subject to
  D4 (the enum/guard cut is its own cell, never bundled).
- Exact replacement wording for `AGENTS.md` critical rule 1, provided it stays
  numbered `1.`, carries its full text rather than a pointer (it is one of
  `TERMINAL_HOME_RULES` in `scripts/tests/test_agents_budget.mjs:45-51`), and the
  file stays under the 15000-byte hard fence / 14000-byte warn line.
- Whether the four rendered skill mirrors regenerate per cell or once at the end,
  provided `.bee-render.json` sidecars end consistent.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Predictive gate | A check run before code exists that reasons about whether the plan will work. What this feature removes. |
| Empirical gate | A check run against real built code. `worktree merge --no-ff --no-commit` + verify + abort (`lib/worktree-store.mjs:1851`) is the one bee already has, and the reason the predictive gate is redundant for revert-cheap work. |
| Build-emitted evidence | An artifact the build produces anyway — red test output, stack trace, verify output, `git diff`. The only evidence class D9 accepts for red-first. |
| Throwaway probe | Code authored solely to produce an evidence string and then deleted. Banned by D9. |
| SMALLER PATH | The one reality-gate check that survives: is there a cheaper shape than the one planned. Moves into `bee-planning` §5 per D1. |

## Existing Code Context

### Integration Points

- `packages/bee/lib/state.mjs:41-52` — `PHASES` / `KNOWN_PHASES`, the enum D3 edits. Mirrored byte-identically at `.bee/bin/lib/state.mjs`, enforced by `test_misc.mjs:1881`.
- `packages/bee/lib/guards.mjs:142` — `GATED_PHASES`, hardcoded, fail-open. The D3/D4 danger point.
- `packages/bee/hooks/bee-session-close.mjs:26` — `PHASE_GATE`, hardcoded, fail-open. The second D3/D4 danger point.
- `packages/bee/lib/state.mjs:3299` — `isKnownPhase(phase) ? phase : 'idle'`, the existing read-coercion precedent D13 follows.
- `packages/bee/bee.mjs:2931-2934,2941-2949` — `state set` refuses on an invalid *pre-mutation* phase and requires `--owner` to equal the current phase. The reason a legacy `validating` repo cannot leave the phase without D13.
- `packages/bee/lib/guards.mjs:1294-1318` — the phase dispatch and its `return { allow: true }` tail. D13 flips the tail.
- `packages/bee/hooks/bee-session-close.mjs:395-398` — `if (!gate) return null`, the Stop-net's silent opt-out on an unknown phase.
- `packages/bee/bee.mjs:3292-3301` — the high-risk advisor-consult refusal on the `execution` gate (D14).
- `packages/bee/bee.mjs:3310-3330`, `packages/bee/lib/state-projection.mjs:132-140` — `approved_for_plan_rev` stamping, execution-only today (D15).
- `packages/bee/lib/command-registry.mjs:724-725` — `bee state gate` takes a single `--name`; the merged approval of D2 is a new code path.
- **`state validation-cache` call sites (D7), inventory only — planning re-derives, this list is not locked:** logic `lib/state.mjs:2267-2576` (~310 lines) with exports `VALIDATION_CACHE_VERSION`, `VALIDATION_SOURCE_ABSENT_SENTINEL`, `validationCacheCheck`, `writeValidationCache`; census `test_misc.mjs:1011-1014`; registry `lib/command-registry.mjs:1173-1207`; handlers/wiring `bee.mjs:4676-4758`, `:7668-7692`, `:7879-7880`; tests `test_bee_cli.mjs:3453-3730`; gitignore entry `scripts/onboard_bee.mjs:178` and its fixture `scripts/test_onboard_bee.mjs:1448`.
- `packages/bee/lib/cells.mjs:1706-1707` — claim refusal text naming Gate 3; wording follows D2.
- `packages/bee/scripts/onboard_bee.mjs:2033,2986-2997` — `renderAgentsBlock()` merges `packages/bee/AGENTS.block.md` into root `AGENTS.md` between `<!-- BEE:START -->`/`<!-- BEE:END -->`. One direction: template → render. Edit the template, never the rendered block.

### Established Patterns

- Four rendered skill mirrors (`.claude/skills/`, `.agents/skills/`, `.claude-plugin/skills/`, `.codex-plugin/skills/`) regenerate mechanically via `scripts/render_plugin_skill_trees.mjs`, tracked by per-root `.bee-render.json` sidecars. Never hand-edit a mirror.
- `packages/bee/lib/**`, `hooks/**`, and `bee.mjs` are vendored 1:1 into `.bee/bin/`. Both copies move in the same commit — repo critical pattern: "A test that runs the canonical source can never catch vendoring drift."
- `packages/bee/tests/` and `packages/bee/scripts/` have no `.bee/bin/` mirror.

## Canonical References

- `packages/bee/lib/worktree-store.mjs:1851,2002` — the empirical gate this feature leans on: `merge --no-ff --no-commit`, verify, then commit; red or drift aborts and proves main untouched.
- `docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md:210` — R55, the change-class × lane proof matrix D8 mirrors.
- `docs/backlog.md:82` — PBI p-8afb88a4, the prior narrowing of red-first by change class.
- `scripts/tests/test_agents_budget.mjs:42-51,123-136` — the byte fence (15000 hard / 14000 warn), the `EXPECTED_RULE_COUNT = 15` roster guard, `TERMINAL_HOME_RULES = [1,5,6,11]`, and the AGENTS.md ↔ AGENTS.block.md byte-identity assertion.
- `scripts/tests/test_gate_bypass_doctrine.mjs:30,248-299` — pins prose inside the file D1 deletes.
- `docs/knowledge/index.md` `## Critical patterns` — in particular: "A state name that ASSERTS history, with nothing checking it, becomes the shortcut"; "A system contradicting itself hides in the seam"; "Pre-code gates filter spec defects; only diff review catches implementation defects"; "A removal is verified by its invariants, not the names it deletes".

## Outstanding Questions

### Deferred To Planning

- [ ] Which phase carries the merged gate under D2/D3 — `planning` throughout, or a new terminal-of-planning marker — and what `GATED_PHASES` / `PHASE_GATE` become exactly. Answered by reading the transition guards and the Stop-hook net together. This also fixes D13's coercion target.
- [ ] What D13's tail flip (`allow` → `deny` on an unrecognized phase) breaks elsewhere. Any path currently passing only because the tail allows is a pre-existing hole, not a regression caused here — but it has to be found and sized before the cell lands.
- [ ] Whether the ~140 rendered skill-mirror files and 10 vendored `.bee/bin/` files count as cut work or mechanical regen, and how that shapes cell boundaries.
- [ ] Whether `bee-xia`'s routing of proof-obligation claims to `bee-validating` retargets to planning or drops.

## Deferred Ideas

- Renaming `.bee/spikes/` to something that does not read as an evidence store — deferred; D10 narrows the contract by prose, and a path rename would touch the guard's scratch-home list and every mirror for cosmetic gain.
- Deriving `GATED_PHASES` and `PHASE_GATE` from `PHASES` so no future phase rename can silently desync them — real hardening, and the root cause behind D4's danger, but larger than this cut. File as a PBI.
- Auditing the other 171 orphaned scribing-debt cells surfaced at session start — unrelated to this feature.
- `skills/bee-executing/references/worker-details.md:58` still requires "spike-evidence links where the plan recorded constraints" for high-risk cells, which D8 makes optional. The Feature Boundary deliberately excludes the cap evidence contract, so this seam ships open by design — file as a follow-up rather than widening this cut.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
D4 is the highest-risk decision in this document: both write guards fail open, so
an incomplete cut passes every existing test while leaving source writes ungated.
