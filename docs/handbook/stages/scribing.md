# Stage: scribing (`bee-capturing` — "Scribe")

**Purpose** — bee's business analyst. Keeps technology-agnostic specs of every area
current (in the `docs/knowledge/` bundle, or `docs/specs/`) so a human understands
the system without the code and an agent can rebuild it on another stack.

**When it runs** — **Deferred, never dropped.** A green `bee close` records capture
as pending and the reminder stands until this runs, so several closed features are
often scribed in one session. It is **also self-triggering** — the moment any rule,
behavior, or value settles, in any phase — and runs on user request to document an
area, including a legacy area that has code but no spec.

## Inputs
- Capped `behavior_change` [cells](../register.md#beecellsfeature-njson) and their
  recorded outcomes, test records, UAT records, and the user's own answers.
  **Deltas come from evidence — never from `plan.md`, never from memory.**
- Gate-locked `CONTEXT.md` + active decisions.
- The existing concepts of the touched areas, read first, so the write extends the
  layer instead of restating it.
- Code reading (harvest mode, for legacy areas).

## Outputs
- Updated `docs/knowledge/areas/<area>/*.md` concepts (or `docs/specs/<area>.md` in
  a repo without a bundle), plus the regenerated index (`bee knowledge index`).
- Capture stubs, logged decisions, a `state scribing-run` record.

## Gate
None.

## State touched
[`decisions log`](../register.md#beedecisionsjsonl),
[`capture add/flush`](../register.md#beecapture-queuejsonl),
[`backlog add` / `backlog pbi add/status/amend`](../register.md#beebacklogjsonl),
`bee knowledge index` (regenerate) / `check` (grade) / `list` (locate),
`state scribing-run` (stamps the workflow record, advances phase to `compounding`,
and appends to the durable ledger `.bee/logs/scribing-runs.jsonl`).

## Key rules
- **The bar is the rebuild test** — given only the spec with its Pointers section
  deleted, a stranger rebuilds the same behavior on another stack.
- **NEVER invent.** A claim backed by neither evidence nor a decision enters the
  spec only as an **Open Gap**. A partial spec that states its gaps beats an
  invented-complete one.
- **One area = one spec, forever** — locate before you create, update in place,
  never fork a `-v2`. The machine helps you *locate* (`bee knowledge list --area`)
  and *grade* (`bee knowledge check`), but nothing refuses a fork at write time:
  this is a convention you are accountable for, and breaking it is said out loud.
- **A contradicted line is replaced, never kept alongside**; present tense only,
  and history lives in git, not in the prose.
- **Tech-agnostic** — no language, framework, class, table, or file name outside a
  Pointers section, and deleting Pointers must remove no business meaning.
- **`docs/specs/` is a read-only compatibility surface in bee's own repo.** New
  prose belongs in the knowledge bundle; a fence fails the chain if it lands in the
  old location.
- **Do not skip scribing** when `behavior_change` cells were capped — in any lane,
  tiny included. Lanes scale ceremony, never memory.
- Secrets and PII never enter a spec, decision, learning, or backlog row.

## Source
`skills/bee-capturing/SKILL.md` ("Scribe") + `references/{area-spec, citations}.md`;
craft in `.bee/expertise/documentation.md` and `.bee/expertise/knowledge.md`
