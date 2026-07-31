# Harness Refocus — flow into the CLI, craft into the skills

Status: proposed (lead: Claude, owner approval pending)
Date: 2026-07-31

## North star

> Bee is a harness that keeps the AI pointed in the right direction and
> assembles just-right context, so the AI always knows where it is and what
> to do next.

The current product drifted from this: skills became flow manuals for bee's
own state machine, and bee's development history (decision IDs, deleted-rule
changelogs) leaked into the end product. This plan re-centers the product.

## Diagnosis (measured)

- `packages/bee`: ~48k lines runtime JS, ~77k lines tests. CLI surface:
  **122 commands**, all low-level primitives — no high-level flow verbs.
- `skills/`: 17 skills, 77 markdown files, **14 of them `provenance.md`**
  (pure self-justification). Skill bodies cite decision IDs ("exec-speed D7",
  "R55") and bee's own source lines (`lib/state.mjs:2570`).
- CLI `--help` text itself carries provenance tags ("hardening-7",
  "g22-3, D4") and doctrine narration.
- Zero craft-knowledge layer: nothing teaches how to plan, decompose, test,
  or review well. All 2,033 lines of SKILL.md bodies are state-machine
  choreography.
- Root cause: the CLI is 122 plumbing commands, so the skills had to become
  the porcelain. Prose is sequencing the machine; the machine should
  sequence itself.

Contrast (fluent, the reference): ~10 high-level CLI verbs; one main skill
(~200 lines) that says *when* to run them and *when* to stop for a human;
craft lives in an `expertise/` layer (how to write tests, behaviors,
architecture); provenance lives in docs, not the product.

## Target architecture — three layers

```
┌─────────────────────────────────────────────────────────┐
│ MACHINE (CLI + hooks)                                    │
│   owns: flow, state, gates, proof, context assembly      │
│   teaches at point of contact: outputs and errors name   │
│   the next action                                        │
├─────────────────────────────────────────────────────────┤
│ CRAFT (skills + expertise/)                              │
│   owns: how to do the work well — planning, decomposing, │
│   testing, reviewing, capturing knowledge                │
│   universal wording; portable to any repo                │
├─────────────────────────────────────────────────────────┤
│ MEMORY (docs/)                                           │
│   owns: why bee is the way it is — decisions, history,   │
│   provenance maps, specs                                 │
└─────────────────────────────────────────────────────────┘
```

## Design rules (the constitution)

- **R1 — Flow lives in the CLI.** A skill step invokes at most one flow
  verb. If a skill must sequence three or more CLI calls to express one
  intent, that sequence becomes a new verb.
- **R2 — The machine teaches at point of contact.** Every refusal and every
  flow-verb output names the next action in plain language. Prose never
  pre-explains what the machine will say anyway.
- **R3 — Single source for every rule.** A rule enforced by the machine is
  deleted from prose. Prose keeps only the *intent* ("cap with proof; the
  CLI refuses otherwise and tells you how to fix it").
- **R4 — Provenance exile.** No decision IDs, no deleted-rule changelogs,
  no bee-source `file:line` citations in skills or CLI help. The
  rule→decision map lives in `docs/decisions/`. `provenance.md` files leave
  `skills/`.
- **R5 — Portability litmus.** Every skill must make sense dropped into a
  foreign repo. If a sentence only makes sense to someone developing bee
  itself, it belongs in `docs/`, not the product.
- **R6 — Craft over choreography.** Skill bodies teach judgment (what makes
  a good cell, a good plan, a good review finding) and delegate mechanics
  to the CLI.

## CLI changes

### 1. Porcelain / plumbing split

Introduce a small porcelain set (target: **~15 flow verbs**); demote the
rest to a `bee internal <group> <verb>` namespace (still available, hidden
from default `--help`). Nothing is deleted in this phase — only re-shelved.

Porcelain sketch (names to be settled during implementation):

| Verb | Replaces (prose today) | Does |
|---|---|---|
| `bee orient` | skill bootstrap sections, reading orders | One context packet: phase, feature, locked decisions digest, open cells, blockers, exactly one recommended next action. Supersedes the "read these 5 files in this order" paragraphs. |
| `bee route` | bee-planning §1 mode gate tables | Classify intake (flags, lane) interactively from evidence the agent supplies; persists the route record. |
| `bee shape` | Gate 2 mechanics across bee-planning | Validates a drafted shape, runs the machine-checkable parts (file caps, dep graph, test-cell presence), emits the gate question text. |
| `bee dispatch <cell>` | bee-swarming/executing dispatch protocol | Claim + reserve + build the full worker prompt (cell JSON inlined, nickname, state line). Extends existing `dispatch.prepare`. |
| `bee finish <cell>` | worker steps 5–8 (commit→cap→release) | Cap with trace, release reservations, validate the status token, in one verb. |
| `bee close` | feature-verify + scribing/compounding handoffs | Run the feature verify, settle debts, emit the capture checklist. |
| `bee gate` | gate presentation contracts | Render the pending gate in the approved wording; record approval/bypass with audit line. |

### 2. Outputs that teach

Every porcelain verb and every guard refusal is rewritten to the shape:
*what happened → why it's refused/what it means → the exact next command or
choice.* This is what lets skill prose shrink: the knowledge moves to the
moment it's needed.

### 3. Help-text scrub

All 122 command descriptions rewritten in plain product language. Provenance
tags move to `docs/decisions/cli-provenance.md`.

## Skills consolidation — 17 → 7

| New skill | Absorbs | Body becomes |
|---|---|---|
| `bee-hive` | bee-hive, bee-bypass-gate | Thin router: `bee orient`, follow it. Gate etiquette. ≤60 lines. |
| `bee-shaping` | bee-exploring, bee-qualifying, bee-context-locking, bee-briefing | Craft of turning fuzzy intent into locked decisions: interviewing, vocabulary pinning, actor/event/state mapping, inversion questions ("what must it NOT do"), when to park. |
| `bee-planning` | bee-planning | Craft of shaping work: smallest honest shape, walking skeleton, cell decomposition quality, coverage judgment before test authoring. Mode-gate mechanics → `bee route`/`bee shape`. |
| `bee-swarming` | bee-swarming, bee-executing | Orchestrator + worker contracts, each ≤50 lines: delegation judgment, deviation rules, headless discipline. Mechanics → `bee dispatch`/`bee finish`. |
| `bee-reviewing` | bee-reviewing | Craft of review: severity calibration, evidence standards, adversarial reading. |
| `bee-capturing` | bee-scribing, bee-compounding | Craft of knowledge capture: what settles vs what's noise, spec-writing for humans, decision hygiene. |
| `bee-researching` | bee-xia | Evidence-labeled research method. |

Out of the product, re-homed:
- `bee-writing-skills`, `bee-evolving` → `docs/handbook/` (they are about
  developing bee, not using it).
- `bee-grooming`, `bee-herding` → keep as skills but slimmed under the same
  constitution (they are genuinely user-invoked workflows).

## Expertise layer (new)

`expertise/` at repo root, referenced by skills, written in universal terms
(structure inspired by fluent's layer; content authored fresh for bee):

- `planning.md` — decomposition, smallest honest shape, dependency thinking
- `tests.md` — what to test, coverage judgment, test smells
- `decisions.md` — what makes a lockable decision, granularity, superseding
- `review.md` — finding quality, severity, verification
- `documentation.md` — specs a human can rebuild from
- `debugging.md` — repro-first, instrument-before-guessing

## Migration phases

Each phase lands independently; the product works after every phase.

- **P1 — Provenance exile** (mechanical, low risk). Move 14 provenance.md
  → `docs/decisions/skills/`; strip D-IDs and bee-source citations from
  skill bodies and CLI help; no behavior change. Acceptance: `rg` for
  decision-ID patterns in `skills/` and command-registry descriptions
  returns zero.
- **P2 — Porcelain verbs.** Implement `orient`, `dispatch` (extend),
  `finish`, `gate` first (the worker chain); then `route`, `shape`,
  `close`. Plumbing demoted to `internal`. Outputs written to teach.
  Acceptance: worker completes a cell end-to-end using only porcelain.
- **P3 — Pilot rewrite: the worker chain.** `bee-swarming` (merged with
  executing) rewritten under the constitution against the new verbs.
  Acceptance: merged skill ≤120 lines total; a cold worker session
  completes a tiny cell reading only AGENTS.md + the dispatch prompt +
  CLI outputs.
- **P4 — Roll the pattern.** Remaining consolidations (shaping, planning,
  capturing, reviewing, researching); re-home writing-skills/evolving.
- **P5 — Expertise layer.** Author the six guides; wire skill references.
- **P6 — Docs sweep.** AGENTS.md trimmed to boundaries + pointers;
  `docs/decisions/` becomes the single provenance home; specs updated.

## Definition of done (product-level)

1. Portability: any skill body dropped into a foreign repo reads as sound
   guidance for that repo.
2. No decision IDs outside `docs/`.
3. Skill bodies total ≤ ~800 lines (from 2,033) with *more* craft content
   than today, not less.
4. Default `bee --help` shows ~15 verbs.
5. A cold agent completes a tiny task from AGENTS.md + CLI outputs alone —
   no skill preloading required.

## Non-goals

- No platform change (stays Node).
- No behavior change to gates, proof rules, or state storage in P1–P3.
- Fluent content is a structural reference only; no text is copied.
