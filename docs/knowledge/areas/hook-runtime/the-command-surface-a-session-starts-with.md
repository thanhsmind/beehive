---
type: bee.area
title: Hook Runtime — the command surface a session starts with
description: "The briefing's command index: why every session opens carrying the whole command surface as flag names rather than leaving it behind an on-demand lookup, what that section states and omits, and the limit it inherits from the catalog it is generated from."
timestamp: 2026-08-06
bee:
  id: hook-runtime-command-surface-in-briefing
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: ["cli-surface-in-context Gate 2: the name-and-type index is carried every session and the description-per-flag variant is priced and declined (2026-08-06)", "8ef2bae6 (cli-ergonomics D1): exhaustive refusal — every problem named in one message"]
  sources: ["cells csc-1 through csc-4: docs/history/cli-surface-in-context/plan.md, 2026-08-06 — full suite 1341 passed, 3 ignored", "measured 2026-08-06: five invented flag names across two agents in one working session, every one a wrong name and none a misunderstood meaning", "measured 2026-08-06: 142 commands, 143 distinct flag names, rendered section 7,950 characters"]
  authoritative_for: "hook-runtime: the command surface carried in the session briefing"
---

# Hook Runtime — the command surface a session starts with

## Behaviors & Operations

**Carry the whole command surface, every session.** Trigger: any session
opening. What changes: the briefing gains a section listing every command the
catalog publishes, each with its own flag names, the declared type of each, and
a marker on the ones the catalog calls required. What the agent observes: it
never has to guess a flag name, because every name it could need is already in
front of it before its first command.

The section is generated from the embedded catalog at render time, never from a
checked-in copy. A copy would be correct on the day it was written and wrong on
the day a verb changed, and the failure would be silent — the reader cannot tell
a stale list from a current one.

**State the near-universal flag once.** The machine-readable-output flag is
accepted by almost every command. Repeating it on every line costs roughly a
seventh of the section for no information, so it is named once in the section
header and omitted from every line. A command with no other flag renders as its
bare name, never as a name followed by an empty separator.

## Business Rules

- **R1 — Names, not meanings.** The section carries what each flag is CALLED
  and what type it takes; it does not carry what each flag does. This is the
  measured shape of the problem: every observed failure was a wrong name, none
  was a misunderstood meaning. An agent that knows a flag exists can ask what it
  does; an agent that does not know it exists invents a spelling and is refused.

- **R2 — The cost is stated and bounded.** The full name-and-type index costs
  roughly a seventh of what a description-per-flag variant would, and that
  variant was priced and declined: the cost falls on every session of every
  project, including the many that never touch the command surface. A size
  budget guards the section, so a future growth in commands is caught rather
  than silently paid for at every startup.

- **R3 — Honesty is inherited, not asserted.** The required marker reflects what
  the catalog DECLARES, which is not always what a handler ENFORCES: most
  commands declare no required flags at all, and some of those still refuse at
  run time for a flag the catalog never named. The section is therefore exactly
  as truthful as the catalog beneath it, and closing that gap is the catalog's
  work, not the briefing's.

- **R4 — A refusal names the flag it refused.** When a call is declined for a
  flag the command does not accept, the refusal names every unrecognised flag
  and offers the nearest declared spelling when one is close enough to be worth
  printing. A refusal that only says "some optional flag was wrong" makes the
  caller pay another round to learn what the refusing side already knew.

- **R5 — The vocabulary ratchets.** The count of distinct flag names across the
  catalog is pinned by a test. Nothing is renamed — the existing divergences
  stand, including the same role being called one thing by one command and
  another by its neighbour — but introducing a NEW spelling for an existing idea
  now requires deliberately moving a number, which makes it a decision instead
  of an accident.

## Edge Cases Settled

- A command whose only flag is the machine-readable-output one renders as its
  bare name. There is no empty flag list and no trailing separator.

- The section's placement is fixed between the project's declared commands and
  the documentation links. The briefing's closing bytes are asserted elsewhere,
  so nothing may be appended after them.

## Open Gaps

- The required marker under-reports wherever the catalog under-declares (R3).
  Filed as debt against the catalog.

## Pointers (implementation)

- Section render + size budget: `hooks/session_preamble/budget.rs`
  (`PREAMBLE_BUDGET_BYTES`), tests in `hooks/session_preamble/tests.rs`.
  Cell `csc-1`, commit 7fc5439b.
- Unknown-flag refusal + nearest spelling: `unknown_flags` / `nearest_flag` in
  `router.rs`, sharing `catalog::distance`. Cells `csc-2` (commit 0576f250) and
  `csc-4` (commit e40cc311).
- Vocabulary ratchet: the distinct-flag-name pin in `catalog.rs`. Cell `csc-3`,
  commit b16669ef.
