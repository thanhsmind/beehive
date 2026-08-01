# Stage: scribing (`bee-capturing` — "Scribe")

**Purpose** — bee's business analyst. Keeps technology-agnostic specs of every area
current (in the `docs/knowledge/` bundle, or `docs/specs/`) so a human understands
the system without the code and an agent can rebuild it on another stack.

**When it runs** — Chain default after swarming execution completes with capped
`behavior_change` cells. **Also self-triggering** — the moment any rule, behavior,
or value settles, in any phase — and on user request to document an area.

## Inputs
- Capped `behavior_change` [cells](../register.md#beecellsfeature-njson) +
  `verification_evidence`.
- Gate-locked `CONTEXT.md` + active decisions, worker reports / UAT records.
- Code reading (harvest mode, for legacy areas).

## Outputs
- Updated `docs/knowledge/areas/<area>/*.md` concepts (or `docs/specs/<area>.md`).
- `system-overview.md` sync, `reading-map.md` refresh.
- Capture stubs, logged decisions, a `state scribing-run` record.

## Gate
None.

## State touched
[`decisions log`](../register.md#beedecisionsjsonl),
[`capture add/flush`](../register.md#beecapture-queuejsonl),
[`backlog pbi add/status/amend`](../register.md#beebacklogjsonl),
`state scribing-run` (stamps the workflow record, advances phase to `compounding`,
and appends to the durable ledger `.bee/logs/scribing-runs.jsonl`).

## Key rules
- **NEVER invent** — unverified claims become *Open Gaps*, never asserted fact.
- **One area = one file/concept, forever** — never fork. This is a rule the
  scribe follows, NOT a gate the machine enforces: `scribingTarget` has no
  runtime caller and no CLI verb, so nothing refuses a fork at write time.
  Read it as a convention you are accountable for, and say so if you break it.
- **Tech-agnostic** — no language, framework, or library named outside a Pointers
  section.
- **Do not skip scribing** when `behavior_change` cells were capped — in any lane,
  tiny included. Lanes scale ceremony, never memory.

## Source
`skills/bee-capturing/SKILL.md` ("Scribe")
