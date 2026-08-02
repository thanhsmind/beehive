---
type: bee.area
title: "Doctrine Layer — rule placement, propagation, and anchoring"
description: "Which layer a rule belongs on, how much of its mechanics travels with it, how doctrine reaches every project by copy, the anchor tests that stop a rule from disappearing, and the standing check that fails the build when an every-turn rule is reachable only from an on-demand reference."
timestamp: 2026-07-29
bee:
  id: doctrine-layer-placement-and-anchoring
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [areas/doctrine-layer/overview.md]
  decisions: [ba5a35f1-981d-4cb5-8a57-234a187f122d (placement rule), "0023 + 6cd34376 (explicit-tier transport rides critical rule 12, B3a)", "derived-check-hardening E7/E8 (the two live doctrine residuals are cleared, and the completeness criterion becomes a standing check deriving retired stage names from the phase-coercion table)", "tick-contract-inline T1/T5/T6 (a narrower always-loaded summary is a misfiled rule; every-turn rules unreachable from the sheet fail the build, derived from wording not a marker; the check proves reachability, never obedience) -- T3's pay-by-removal rule is RETIRED by budget-fence-removal", "budget-fence-removal D1/D6 + repo decision 8f63adb4 (a size ceiling on instruction text is never a standing rule; a diet is a one-off event leaving no permanent gate; body length is judged per edit by information density, and the surviving standing guards prove meaning, never size)"]
  sources: ["tier-transport-doctrine (cell tier-transport-doctrine-1, 2026-07-13)", "docs/specs/doctrine-layer.md#B1", "docs/specs/doctrine-layer.md#B2", "docs/specs/doctrine-layer.md#B3a", "docs/specs/doctrine-layer.md#B4", "docs/specs/doctrine-layer.md#R1", "docs/specs/doctrine-layer.md#R2", "docs/specs/doctrine-layer.md#E1", "docs/specs/doctrine-layer.md#P1", "docs/specs/doctrine-layer.md#P2", "docs/specs/doctrine-layer.md#P4", "derived-check-hardening cells dch-5/dch-7 (retired-stage currency check with derived tokens; research-brief and write-guard fixture residuals cleared; traces .bee/cells/dch-{5,7}.json, reports docs/history/derived-check-hardening/reports/, 2026-07-29)", "tick-contract-inline (cells tci-1/tci-2/tci-3, decisions T1-T7, traces .bee/cells/tci-{1,2,3}.json, reports docs/history/tick-contract-inline/reports/, 2026-07-29)"]
  authoritative_for: "doctrine-layer: rule placement, propagation, and anchoring"
---

# Doctrine Layer — Rule Placement, Propagation, and Anchoring

The single question that decides a rule's home, the minimum that has to travel
with it, how the layer reaches every governed project, and the only mechanism
that makes a vanished rule a visible event.

## Behaviors & Operations

**B1 — Doctrine reaches every governed project by copy, not by reference.**
Trigger: a project is onboarded or upgraded. What changes: the doctrine block
inside that project's own instruction sheet is replaced with the current one,
in place, leaving any project-authored content outside the block untouched. What
each actor observes: the assistant working in that project reads the new rules
from its very next session, with no action by that project's owner; the owner
sees exactly one bounded region of their instruction sheet change. Why by copy:
a project must carry its rules locally — an assistant reading a project's
instructions cannot be assumed to have access to the workflow's own repository.

**B2 — A rule that must hold when no stage is running is placed on the standing
sheet; anything else may live in a procedure reference.** Trigger: any new rule
is authored. What decides its home: the single question *"does this need to hold
when no workflow stage is running?"* — yes places it on the standing sheet, no
permits a procedure reference. What each actor observes: a rule placed correctly
takes effect in every turn; a rule placed in a procedure reference is **silently
absent from every turn in which its stage is not invoked** — the assistant is not
disobeying it, it is not being told it. This failure is invisible from the rule's
own text: a perfectly written rule in the wrong home behaves exactly like no rule
at all, and the only symptom the human sees is having to repeat the instruction
by hand (decision ba5a35f1).

**B3a — A standing rule carries whatever mechanics compliance requires; only the
elaboration may be referenced.** Trigger: a rule on the standing sheet orders an
action that has a required form — a mandatory parameter, a marker, a naming
convention — and a guard rejects the action when that form is missing. What is
placed on the standing sheet: the order *and* the minimum needed to obey it
correctly on the first attempt. What may stay in a procedure reference: the
rationale, the tiers, the full contract. What each actor observes when the split
is wrong: the rule fires in turns where the reference is not loaded, the action
is attempted in its bare form, and the guard denies it — so the assistant learns
the requirement only from the rejection, and pays one wasted attempt per session
to do so. This is B2's failure at half-scale: the order travelled to the always-
loaded layer and its transport did not (observed with the delegation rule and the
subagent tier marker; decision 0023).

**B4 — A doctrine rule is pinned by an anchor the suite enforces.** Trigger: the
suite runs. What is checked: for each rule that must never disappear, a
distinctive phrase from it is asserted present on both the master copy of the
standing sheet and this project's own. What happens on failure: the suite fails,
naming the missing rule. Why: doctrine has no runtime — nothing *executes* a
rule, so nothing fails loudly when one goes missing. The anchor test is the only
mechanism that makes deleting or relocating a rule a visible event.

**B5 — A retired workflow stage may not be described as current anywhere a
reader is told is current behavior.** Trigger: the suite runs. What is checked:
every document that describes how the system behaves today — the standing
sheet, the procedure references and their bodies, and the state layer — is read
for any description of a workflow stage that has been retired, written as
though the stage still exists. What happens on failure: the suite fails, naming
the document. How the retired names are known: they are derived from the record
layer's own table of renamed phase values rather than written into the check, so
the check keeps working the next time a stage is retired without anyone
remembering to teach it the new name. What is exempt: exactly the two
append-only trees where past decisions and past features are recorded by design,
and the single path whose whole job is translating the retired name — nothing
else, and the exception set is never widened to turn a real finding green. Why a
standing check rather than a sweep: this criterion had been run by hand once and
never again, and by the time anyone looked, two live documents were still
routing readers to a stage that no longer existed. A criterion with no
enforcement is a note, not a criterion (derived-check-hardening E7/E8).

**B6 — A rule whose own wording says it applies every turn must be named by the
standing sheet, and which rules those are is derived from the wording itself.**
Trigger: the suite runs. What is checked: every section of every on-demand
procedure reference is read, and any section whose wording states that its rule
applies on every turn must also be named by the always-loaded sheet, through a
pointer the sheet itself carries. What happens on failure: the suite fails,
naming the offending section by document and line. How the rule set is known: it
is derived from the rules' own wording — a small vocabulary of the ways English
states per-turn scope — and never from a list of rules an author marked. A marker
would have to be applied by the same author who has just filed an every-turn rule
in a reference, so its failure mode is **silence**: precisely the defect this
check exists to catch, relocated one level up. Derivation fails the other way —
toward a false positive, a red build a human clears by moving the rule or by
rewording a claim that was never meant to read as every-turn — which is the safe
direction for a gate to be wrong in. What this establishes is **reachability**
only: that an agent loading nothing but the standing sheet still arrives at the
rule. It says nothing whatever about whether the rule was then obeyed
(tick-contract-inline T5/T6).

## Business Rules

- **R1** — A rule that must hold when no workflow stage is running belongs on the
  standing instruction sheet. A procedure reference is never an acceptable home
  for it (ba5a35f1).
- **R2** — Every doctrine rule that must never disappear carries a suite-enforced
  anchor. A rule without one may be deleted or relocated with no signal.
- **R3** — No document describing current behavior may present a retired
  workflow stage as though it still exists. The check derives the retired names
  from the record layer's own rename table instead of naming them, so it
  survives the next retirement unedited, and its only exemptions are the two
  append-only history trees and the translation path itself
  (derived-check-hardening E7/E8).
- **R4** — A rule that applies every turn is named by the standing sheet. One
  that is reachable only from an on-demand procedure reference fails the build,
  named. The check derives which rules are in scope from their wording, never
  from a maintained marker (tick-contract-inline T5).
- **R5** — A standing-sheet line earns its place by changing what an agent
  does. A line that changes no behavior belongs in an on-demand reference, and
  a line that changes behavior every turn belongs on the sheet however long the
  sheet becomes. Length is judged per edit, by that question alone — never
  against a recorded size, and no gate measures it (budget-fence-removal D1/D6,
  superseding tick-contract-inline T3's pay-by-removal rule).
- **R6** — A size ceiling on instruction text is never a standing rule. A
  deliberate compression pass is legitimate work and leaves nothing behind; a
  permanent ceiling is not, because it makes an author fund a correct addition
  by cutting correct text elsewhere. What survives as a standing guard is
  meaning — that no rule vanished, and that a rendered copy still matches its
  master — never size (budget-fence-removal D1).

## Edge Cases Settled

- **A perfectly written rule can be perfectly ineffective.** The delegation rule
  was fully specified, and cited by every workflow stage, for its entire life
  before this settlement — and the human still had to repeat it by hand, because
  every citation of it lived somewhere that ordinary conversation never loaded.
  Completeness of a rule's *text* says nothing about its *reach*. When a rule is
  being ignored, check its placement before rewriting its wording.

- **A check can be green for an accidental reason.** One fixture asserted a
  gate's behavior using a stage name that had already been retired. It passed —
  reliably, every run — only because the read path underneath it silently
  rewrote the retired name into a live one before the assertion ever saw it. The
  fixture was proving the wrong thing and proving it consistently. A retired
  value reaching a check through a translation path is not evidence the check
  works; it is evidence that nothing is exercising the real value. The fixture
  now names a value the state machine can actually produce, so it exercises the
  gate policy rather than the translation (derived-check-hardening E7).

- **A summary can be narrower than the rule it stands in for, and still read as
  faithful.** The standing sheet did name the per-step reporting rule — as a
  one-line summary — and that summary was narrower on two axes at once: it read
  as conditional on a mode the reader might not be in, and it named four kinds
  of event where the rule in fact covers every perceivable step. An agent
  following the summary exactly still broke the contract, and nothing in the
  summary's own text revealed the shortfall. A summary standing in for a rule is
  judged by whether it is as **wide** as the rule, never by whether it mentions
  it — B2's failure mode surviving a correct-looking placement
  (tick-contract-inline T1).

- **Reachability is not obedience, and the check that proves the first must not
  be cited for the second.** B6 proves an every-turn rule can be arrived at from
  what is always loaded. Whether it was then followed is observed nowhere in
  this project. Recording the check as though it closed that gap would hand the
  surface a claim of coverage it does not have, and a check that overstates its
  reach is how the next gap goes unnoticed (tick-contract-inline T6).

- **A deliberate re-read is when stale cross-references surface — the trim was
  never the thing that found them.** Trimming the sheet to fund a new rule once
  put a reader's eye on citations nobody had re-read in a long time, and two of
  them pointed one rule off from the rule they named. That was read as an
  argument for paying by removal. It was not: what found the rot was reading the
  whole document, and a size law is an expensive and lossy way to schedule a
  read. Removing the law and running one deliberate sweep instead found 18
  defects of that class where the incident-driven approach had surfaced 2
  (budget-fence-removal D9). Schedule the re-read directly.

## Pointers (implementation)

- Master copy of the standing sheet: `packages/bee/AGENTS.block.md`;
  the rendered per-project copy sits between the `<!-- BEE:START -->` /
  `<!-- BEE:END -->` markers in each host's root `AGENTS.md`.
- B1's copy-into-project step: `packages/bee/scripts/bee onboard`
  (`update_agents_block` plan item).
- Anchor tests (B4/R2): `packages/bee/tests/test_lib.mjs` — the
  `census:` checks, including the delegation-layer anchor and the on-demand
  review anchors, plus the native Codex empty-wait anchor across the master,
  root, canonical procedure, and writable `.claude` surfaces.
- Retired-stage currency check (B5/R3): `scripts/tests/test_scan_set_hygiene.mjs`,
  check 2. Retired tokens are derived from `LEGACY_PHASE_COERCIONS` in
  `packages/bee/lib/state.mjs` (`{ validating: 'planning' }` today), never
  hard-coded. Scanned surfaces: `skills/**/SKILL.md`, `skills/**/references/**`,
  `docs/knowledge/**`, `docs/specs/**`, `AGENTS.md`, `CLAUDE.md`. Exceptions:
  `docs/decisions/**`, `docs/history/**`, and the coercion path itself. The two
  residuals it was written for: `skills/bee-researching/references/research-brief-template.md`
  (proof obligations now route to `bee-planning`'s shape gate) and
  `packages/bee/hooks/test_write_guard.mjs` (the fixture row that hand-built the
  retired phase value). Evidence: traces `.bee/cells/dch-5.json`,
  `.bee/cells/dch-7.json`.
- Every-turn reachability check (B6/R4): `scripts/tests/test_always_loaded_rules.mjs`
  — blocking (exits 1 on a finding), picked up by the chain's `test_*.mjs` glob
  over `scripts/tests/`, so no registration in `scripts/run_verify.mjs` was
  needed. Its one seed is `EVERY_TURN_PHRASES` (six wording patterns); the rule
  set, the reference corpus (`*.md` under any `references/` directory beneath
  `skills/`), and the pointer set are all read off the tree at run time — no rule
  name, heading, or rule count is hardcoded. The two always-loaded locations it
  reads, `AGENTS.md` and `packages/bee/AGENTS.block.md`, are the only paths it
  fixes. Deliberately out of scope: `skills/*/SKILL.md` bodies (a different tier
  with a different pointer convention). Provenance ledgers are skipped only when
  a file self-declares as a rule→decision map, never by filename, so the
  exclusion fails toward flagging.
- Drift proof for B6: reverting to `c3c67005^`, and separately stripping only
  critical rule 17 from the block, each turn the suite red naming
  `skills/bee-hive/references/routing-and-contracts.md` "Progress ticks —
  worked examples" as the unreachable every-turn section.
- Size-law regression guard (R6): `scripts/tests/test_instruction_size_law.mjs` —
  fails the build if any byte or line ceiling on instruction text reappears
  anywhere under `scripts/`, and separately proves the two surviving meaning
  guards still exit non-zero on a seeded violation. It matches on the shape of
  a size law, not on the names of the deleted files, so the same rule under a
  new identifier is still caught; its own negative controls prove that.
- Standing-sheet meaning guard (R5): `scripts/tests/test_agents_budget.mjs` —
  guards the 17-rule roster and the byte-identical render between the master
  block and the rendered `AGENTS.md`, no size threshold anywhere in the suite;
  `EXPECTED_RULE_COUNT` moved 16→17 by appending, so
  `TERMINAL_HOME_RULES [1, 5, 6, 11]` was unaffected. The rule cost ~1,850 bytes
  and was funded by removing 835 bytes of restated prose plus the two stale
  citations, at a time when the suite still carried a byte fence; that fence
  was later deleted outright (budget-fence-removal D2/D5).
- The two corrected cross-references: block rules 9 and 10 each carried a
  `Full rule: bee-hive skill, critical rule N` citation off by one; both were
  removed outright rather than repointed, the block stating those rules more
  fully than the hive one-liners it was citing.
- Evidence: traces `.bee/cells/tci-1.json`, `.bee/cells/tci-3.json`; reports
  under `docs/history/tick-contract-inline/reports/`.
