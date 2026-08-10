---
type: bee.area
title: Hook Runtime — the command surface a session starts with
description: "The briefing's command index: why every session opens carrying the whole command surface as grouped command names — flags demoted to per-command help after measurement — what that section states and omits, and the limit it inherits from the catalog it is generated from."
timestamp: 2026-08-06
bee:
  id: hook-runtime-command-surface-in-briefing
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: ["cli-surface-in-context Gate 2: the name-and-type index is carried every session and the description-per-flag variant is priced and declined (2026-08-06)", "8ef2bae6 (cli-ergonomics D1): exhaustive refusal — every problem named in one message"]
  sources: ["cells csc-1 through csc-4: docs/history/cli-surface-in-context/plan.md, 2026-08-06 — full suite 1341 passed, 3 ignored", "measured 2026-08-06: five invented flag names across two agents in one working session, every one a wrong name and none a misunderstood meaning", "measured 2026-08-06: 142 commands, 143 distinct flag names, rendered section 7,950 characters", "preamble-surface-slim cell pss-1 (grouped-name renderer in budget.rs, 7.9KB to 1.7KB, supersede decision logged; capture stub 6e9734f6, 2026-08-07)"]
  authoritative_for: "hook-runtime: the command surface carried in the session briefing"
---

# Hook Runtime — the command surface a session starts with

## Behaviors & Operations

**Carry the whole command surface, every session — as grouped NAMES
(preamble-surface-slim pss-1, 2026-08-07, superseding the flag-per-line
shape).** Trigger: any session opening. What changes: the briefing gains a
section listing every command the catalog publishes as grouped, dotted
command names (one line per verb group), with a header pointer naming
`bee <command> --help` as where the flags live. What the agent observes: it
never has to guess whether a COMMAND exists, and it asks per-command help
for the flags — measured, the flag-per-line rendering cost 7,950 characters
per session start and the grouped-name shape costs about 1,700 for the same
does-it-exist answer. The original flag-name index (the shape this concept
first recorded) is superseded: flag names are no longer carried in the
briefing at all.

The section is generated from the embedded catalog at render time, never from a
checked-in copy. A copy would be correct on the day it was written and wrong on
the day a verb changed, and the failure would be silent — the reader cannot tell
a stale list from a current one.

**State the near-universal flag once.** The machine-readable-output flag is
accepted by almost every command, so it is named once in the section header
and never per command. The count of commands accepting it rides the header
line (e.g. "133 of 145"), generated from the catalog like everything else.

## Business Rules

- **R1 — Names, not meanings — now at COMMAND grain (pss-1).** The section
  carries what each command is CALLED; it carries neither flag names nor
  meanings. The measured failure mode was invented spellings; the invented
  spellings were curable one level cheaper than first thought: an agent that
  knows the command exists reaches its full flag surface through
  `bee <command> --help` (which since harness-audit-hardening hah-4 renders
  EVERY declared flag, not only required ones). Flag-name recall moved from
  the always-loaded briefing to on-demand help.

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
