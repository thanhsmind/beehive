# Go Mode — Step-by-Step Reference

Load this when executing go mode — the full bee pipeline from raw feature request to compounded learnings, closing verified but `unreviewed`. It chains every skill in sequence with up to **2 human gates** (fewer when the opt-in gate-bypass switch is on — see the end of this file). **Go mode never auto-enters independent review** — `bee-reviewing` and its Gate 3 are a separate, user-invoked flow layered over a completed scope; see the boxed note after the diagram.

Trigger: `/go [feature]`, "run the full pipeline", or "go mode".

**Lane fast paths short-circuit this diagram** (`routing-and-contracts.md` "Lane ceremony in full"): `docs` lane skips the pipeline entirely (announce → write → format-check → capture). `tiny`/`small` collapse Steps 2–5 into: no plan.md (`tiny`) or an opt-in scoping synthesis + plan.md (`small`) → draft cell(s) previewed before persist, inline reality check → **one merged shape+execution gate** → execution (tiny: inline; small: one dispatched execution worker) → orchestrator-authored done-report → capture. The full diagram below is the `standard`/`high-risk` pipeline.

```text
User: "/go [feature]"
       │
       ▼
[BOOTSTRAP] onboarding check, `bee orient`, critical patterns (bundle: docs/knowledge/index.md
            digest; no bundle: critical-patterns.md), recent decisions
       │
       ▼
[STEP 1] bee-shaping (Explore) → docs/history/<feature>/CONTEXT.md
       ▼
[GATE 1] ← HARD STOP
       ▼
[STEP 2] bee-planning (shape) → plan.md (frozen at Gate 2 once approved); discovery.md/approach.md
                                 only for L2+ discovery or high-risk, else plan.md sections
         bee-shaping (Brief)   → implement-plan.md  (high-risk always; standard/small on-demand)
         SMALLER PATH check + review wave run inline before the gate below
       ▼
[GATE 2] ← HARD STOP — approves `shape` AND `execution` together in one call (`bee gate
          --merge`); review the implement plan, or plan.md when no brief was rendered
       ▼
[STEP 3] bee-planning (prep)  → current-slice cells only — the write guard freezes plan.md once shape is approved
         bee-shaping (Brief refresh) → implement-plan.md Affected Files + Steps re-projected
       ▼
[STEP 4] bee-swarming (orchestrator + workers × N) — current slice only
       │
       ├── more approved work remains → return to STEP 3 for the next slice (execution stays
       │   approved feature-wide from Gate 2 — no re-ask per slice)
       ▼
[STEP 5] bee-capturing (Scribe) → knowledge sync: docs/knowledge/ concepts, else docs/specs/<area>.md
                                 (closes unreviewed)
       ▼
[STEP 6] bee-capturing (Compound) → docs/history/learnings/, decision log, review-candidate report
       ▼
DONE — verified, unreviewed, development continues
```

```text
┌─────────────────────────────────────────────────────────────────────────┐
│ Independent review is a SEPARATE, user-invoked flow, not a pipeline     │
│ step. Go mode never dispatches it automatically — not after the final   │
│ slice, not at DONE. When the user explicitly asks for review (any time, │
│ any scope: this feature, a named batch, a commit range), invoke         │
│ bee-reviewing over that immutable scope: P1/P2/P3 findings, artifact    │
│ verification, UAT, then [GATE 3] ← HARD STOP (never auto-merge) inside  │
│ that session, followed by bee-shaping's walkthrough (Brief) for         │
│ standard/high-risk. A merge/ship/release request while candidates       │
│ sit unreviewed/stale reports the count + risk level and asks ONE        │
│ question before ever spending a reviewer token.                         │
└─────────────────────────────────────────────────────────────────────────┘
```

Separately, `standard`/`high-risk` swarming waves also run a semantic checklist judge once per slice at slice close over its capped `behavior_change` cells (table in `bee-hive/references/gates-and-delegation.md`, "Goal-check judge tier") — that is verification of the cells, not the boxed review flow above, and never triggers Gate 3 on its own.

## Pre-Pipeline: Bootstrap

Before invoking `bee-shaping`:

1. Confirm onboarding, then run `bee orient` — it carries phase, gates, blockers, and the next action; open the critical-patterns source only when the preamble digest is missing.
2. Apply the surface-scope-earlier check — clear acceptance criteria plus pattern references may skip Step 1 with user approval.
3. Determine the feature slug (lowercase-hyphenated) and create `docs/history/<feature>/` if missing.
4. Update `.bee/state.json`: `feature: <slug>`, `phase: exploring`, `mode: null` (set at the mode gate).

## Gate Wording (fixed)

- **Gate 1:** "Decisions locked. Approve CONTEXT.md before planning?"
- **Gate 2:** "Work shape is ready. Approve before current-work preparation?" — approves `shape` AND `execution` together in one call (`bee gate --merge`); every lane merges the same way.
- **Gate 3:** P1 > 0 → "P1 findings block merge. Fix before proceeding?" ; P1 = 0 → "Review complete. Approve merge?"

Each gate is one question in the standard CONTEXT / QUESTION / RECOMMENDATION / options format, presented per the **Gate Presentation Contract** (`routing-and-contracts.md`): plain-language layer in chat, in the user's language; full mechanical report written to `docs/history/<feature>/reports/` and linked, never pasted. Gates are asked **one at a time** — Gate 1 and Gate 2 are never batched into a single question. `tiny`/`small` keep the same merge shape at a lighter ceremony (`routing-and-contracts.md` "Lane ceremony in full"): the inline reality check plus one merged question IS the contract there too, and `tiny` closes with a done-report instead of Gate 3. Optional at Gate 2 and Gate 3: a cross-model second opinion; disagreement is quoted to the user, never auto-resolved.

## Gate Presentations

Templates below are the **human layer** — fill them in the user's language, in the user's terms. Square-bracket content is plain prose — default, not table dumps or jargon.

**GATE 1** — after shaping:

```text
What we decided: [the feature in one plain sentence] — [N] choices locked, [M] questions still open.
The key choices, in plain words: [max 3, one line each; more → "full list in CONTEXT.md"]
If a choice is wrong: everything after this builds on it — fixing it now costs a conversation, fixing it later costs redone work.
You are deciding: whether these choices match what you meant, before any planning starts.
Full record: docs/history/<feature>/CONTEXT.md
Decisions locked. Approve CONTEXT.md before planning? (yes / revise / show full CONTEXT.md)
```

Revise → return to shaping for the specific gray areas, update CONTEXT.md in place, re-present.

**GATE 2** — after the planning shape pass, approves `shape` AND `execution` together:

```text
What I plan to build: [the shape in one plain sentence]. Size: [mode, glossed — e.g. "standard — a normal mid-size feature"].
Why this size: [one plain sentence — the least workflow that honestly protects the work].
If the shape is wrong: preparation gets built against it — revising now is cheap, revising after prep is not.
You are deciding: whether this is the right thing and the right size — and whether I may start editing real files, this slice of work only.
Full plan: docs/history/<feature>/plan.md
Work shape is ready. Approve before current-work preparation? (yes / revise / show full plan.md)
```

Approval flips `approved_gates.shape` AND `approved_gates.execution` together (`bee gate --merge`) and covers the **current slice only**; later slices of the same feature build on it without a re-ask (a `plan-rev bump` is what revokes it). Revise → return to the shape pass, update `plan.md` content (unapproved — pre-Gate-2 content edits are allowed; frozen only once `approved_gates.shape` is set), re-present.

**GATE 3** — inside a user-invoked `bee-reviewing` session only (never at the end of go mode's default chain):

```text
What was built: [the shipped change in one plain sentence].
Review found: [P1 count] problems that block merge — [each named in plain words] — plus [P2+P3 count] smaller issues filed for later.
If we merge now: [the consequence in user terms — "nothing known breaks" or "X would ship broken for users who Y"].
You are deciding: whether this goes into the main branch.
Full review: docs/history/<feature>/reports/
```

- P1 > 0 → "P1 findings block merge. Fix before proceeding? (a) fix now (b) show details (c) explicit user override" — silence is not acknowledgment.
- P1 = 0 → "Review complete. Approve merge? (yes / show P2s first / no)"

Fix cells created for P1s run through swarming, then reviewing re-runs (targeted to the fix diff) before Gate 3 is re-presented. Repeat until P1 = 0 or explicit override.

## The Slice Loop

After each slice's swarm completes: later approved work remains → return to Step 3 (planning prep for the next slice), which hands straight to Step 4 (swarming) — the merged Gate 2 already covers execution for the rest of the feature, so it is never re-asked per slice. Final slice done → Step 5 (bee-capturing, Scribe) directly. `bee-reviewing` is never part of this loop — it is a separate flow the user invokes on demand, over whatever scope they choose, independent of slice boundaries.

## Fallback Paths

- **Spike returns NO** (opt-in by change class — migration, security, external side effect, or no in-repo precedent): STOP before Gate 2. Present "Spike [id] failed: [reason]. Current work is blocked." Options: revise approach / descope the risky part / change mode or boundaries. A workaround that "probably works" is not a path — plausibility is not evidence.
- **SMALLER PATH check fails:** default is to redraft the shape before presenting Gate 2, rather than persist-then-preview.
- **Review-wave BLOCKER still open after the second pass** (bee-planning's Review Wave): escalate — present both positions to the user and ask "Return to planning with these specific concerns?". A third pass needs a recorded reason.
- **Context hits ~65% mid-swarm:** write `.bee/HANDOFF.json`, present "[X] cells capped, [Y] in flight. Resume in a new session." End gracefully.
- **User rejects at any gate:** identify what feels wrong, return to the owning stage, update the artifact in place, re-present the same gate.

## Close-out

After compounding: set state `phase: idle`, `feature: null`, `mode: null`, summary "Go mode complete for <feature>", and delete `.bee/HANDOFF.json` if present. Report the completion line from `bee_status` (verified/unreviewed candidate count) — never state or imply the feature was reviewed unless a review session actually ran and approved it.

## Headless Go Mode

`mode:headless` runs stages headlessly **between** gates only. Every gate still stops the pipeline and reports "awaiting Gate N approval" in the terminal report (`gates-and-delegation.md`, "Headless mode").

## Gate bypass in go mode (opt-in)

Separate from headless. When `gate_bypass` is on, go mode does not stop at a bypassed Gate 1-2: the agent takes the RECOMMENDATION, records the approval, logs a one-line audit decision, posts a short `⚡ auto-approved Gate N` line, and continues. How far the level reaches — and what still stops, `off` included — is the table in `gates-and-delegation.md` ("Gate bypass mode").

Gate 3 sits outside this entirely: bypass never creates or auto-approves a review session, so go mode reaching DONE never triggers it. If the user later invokes `bee-reviewing`, bypass may auto-approve the merge question only once P1 = 0 and every UAT item passed; any P1 or UAT fail/skip always stops for the human inside that session.
