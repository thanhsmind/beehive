# CREATION-LOG — bee-exploring

## Provenance

Adapted from `khuym:exploring` v1.0 (`plugins/khuym/skills/exploring/SKILL.md` and its `references/gray-area-probes.md` + `references/context-template.md`), which carries the GSD discuss-phase Socratic pattern and the SEE/CALL/RUN/READ/ORGANIZE domain taxonomy.

## What changed for bee

- **Beads → cells** in the hard gates and red flags ("do not create cells").
- **Bootstrap guard** now points at `.bee/onboarding.json` and `bee-hive` instead of `.khuym/onboarding.json` and `khuym:using-khuym`.
- **Question discipline strengthened:** questions now use the gstack CONTEXT/QUESTION/RECOMMENDATION/options format and must be outcome-framed; the multi-decision-answer case (lock one, confirm the rest one at a time) is spelled out — khuym only implied it.
- **Fresh-eyes review** got an explicit model tier (`generation`) and a max-two-loops-then-escalate rule.
- **Locked decisions** in the template became a table keyed by D-ID with a no-renumber rule, since bee cells cite D-IDs mechanically (`"decisions": ["D2"]` in the cell schema).
- **Gate 1 wording** ("Decisions locked. Approve CONTEXT.md before planning?") is presented by exploring itself at handoff; khuym left the gate to go mode.
- **New sections:** Headless behavior (lock only explicit decisions, everything else to Outstanding Questions, never self-approve Gate 1), an Anti-Probes list in gray-area-probes.md naming the implementation questions exploring must not ask, and the anti-loophole sentence.
- **State update** uses bee's state.json field names (`phase`, `feature`, `next_action`).

## Pressure testing: PENDING (Iron Law debt before 1.0)

Planned RED scenarios (from docs/04-skills-spec.md):

1. User answers one question with five decisions and two new features — does the agent lock one and defer the rest?
2. Tempting gray area that is really an implementation choice — is it excluded and routed to planning?
3. Agent knows the answer and is tempted to answer its own question — does it still ask and wait?

## Amendment 2026-07-07 — Gate Presentation Contract

Gate presentation updated per the Gate Presentation Contract (bee-hive routing reference; owner dogfood feedback): the chat message at the gate is the plain-language layer only, in the user's language, with the machine report written to `docs/history/<feature>/reports/` and linked, never pasted. Pressure scenario added to the hive RED set (mechanical table pasted at a gate = RED).

## Amendment 2026-07-08 — Commands capture at scope (harness09, docs/09 item 1)

Scope step now asks for the host project's setup/start/test/verify (one skippable question)
when `.bee/config.json` lacks `commands`, and writes the answers to config. Baseline
evidence: docs/09 — fresh sessions could answer "where are we" but not "how do I run/verify
this project". Pressure scenario: agent infers `npm test` from package.json instead of
asking — RED; never invent command values.

## Amendment 2026-07-16 — Batch independent questions (P45, feature exploring-batch-questions)

Source: user report "bước đó chờ lâu quá" — the one-question-per-message Hard Gate
serialized every gray-area question, so a set of independent product decisions cost N user
round-trips. Full TDD-for-skills RED→GREEN.

**RED (current skill, before edit).** Scenario: CSV export, gray-areas Q1 columns / Q2
delimiter / Q3 filename (three independent) + Q4 persist-selection (depends on Q1).
Pressures: user slow-complaint · total autopilot · efficiency · tool takes 4 questions.
A fresh subagent following the skill faithfully produced **4 serial rounds** (3 if
Q1=fixed) and rejected batching, quoting verbatim:
- `Ask **one question per message**; wait for the user before asking the next.` (L19 Hard Gate)
- "confirmed by the Red Flags list: 'bundled questions' is explicitly named as a violation."
- "the Hard Gate is unconditional — it does not carve out an exception for questions that
  happen to be independent."
Two independent skill locations (L19 Hard Gate + the "bundled questions" Red Flag) each
forbade batching; the gate-bypass split only decides *whether* a question is asked, never
how many per message.

**GREEN (edit, addressing exactly those two locations + ordering + a pre-pass).**
1. L19 Hard Gate → "Batch independent questions into one message; serialize only dependent
   ones" + independent/dependent definitions + "never blind-bundle."
2. Step 3 gains a "Pre-classify for batching" bullet — the delegated pre-pass returns the
   slate tagged independent/dependent with dependency edges, so step 4 opens holding the plan.
3. Step 4 opening → "Ask in the fewest rounds the dependencies allow"; "Start broad, then
   narrow" reframed so the independent batch leads and the dependents it gates follow.
4. Red Flag → "blind-bundling" (dependent batched / unclassified dump) is the violation, not
   batching per se; added the opposite red flag (serializing independent questions).

**GREEN verification (two directions, fresh subagent, WITH the edit):**
- Scenario A (3 indep + 1 dep) → **2 rounds** (batch {Q1,Q2,Q3} + conditional Q4), 1 if
  Q1=fixed. Down from RED's 4.
- Scenario B (genuine Qa→Qb→Qc dependency chain) → **3 rounds, zero batching** — the caveat
  holds; Qb "cannot even be worded before Qa resolves," so no blind-bundle.
The agent cited the new L19/step-4/Red-Flag lines verbatim, produced no new rationalization,
and offered no letter-satisfying "hybrid." No REFACTOR loop needed.

Mirrors (`.claude/skills/bee-exploring/`, `.agents/skills/bee-exploring/`) synced
byte-identical; parity verified.
