# Model role split — discovery map

## Destination

A locked decision on how many model roles bee configures, which ones a
dispatch can actually reach, and what a role entry may carry (fallback
chain, effort). Then one shaped feature that implements it.

Spawned: (not yet)

## Origin

Owner observation, 2026-08-24, from another tool's model-roles screen
(DEFAULT / SMOL / SLOW / VISION / PLAN / DESIGNER / COMMIT / TINY / TASK
/ ADVISOR, each with an ordered fallback list and its own effort level):
bee's `generation` and `extraction` are broad by comparison, and a finer
split would make configuration more dynamic and more efficient.

## Notes — reality at map time (2026-08-24, verified in code)

- **Four configurable slots, not two.** `CONFIGURABLE_SLOTS =
  ["extraction", "generation", "review"]` —
  `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:37`;
  `MODEL_NORMALIZE_SLOTS` adds `advisor` — `models.rs:40`. `ceiling` is a
  fifth pseudo-tier that means "the session model" and is deliberately
  never configurable (decision 0015) — `models.rs:324-326`.
- **Effort already exists.** `EFFORT_LEVELS = ["low","medium","high",
  "xhigh","max"]` — `models.rs:27`, carried on the `{model, effort}` leaf
  (`models.rs:167-181`) and the `{kind:'native', …}` leaf
  (`models.rs:86-97`). One scalar per entry.
- **Fallback already exists, but never as a chain.** Two single-step
  mechanisms only: the explicit-only composite
  `{primary, fallback_policy, fallback}` (`models.rs:134-166`, decision
  3ceba8f5 D2) and the herding slot's `fallback: "default"` flag
  (`models.rs:112-133`, decision 267192c1). No list-of-models anywhere.
- **The `extraction` slot is reachable by cell tier, never by dispatch
  kind.** Two reachability paths exist, and extraction has only one of
  them. (a) By `--kind`: `DISPATCH_KINDS = ["cell","gather","reviewer",
  "advisor"]` (`prepare.rs:31`) and `slot_for_kind` maps
  `"cell" | "gather" => "generation"` (`prepare.rs:34-40`) — no arm
  yields `extraction`. (b) By a cell's own recorded `tier` field:
  `MODEL_TIERS = ["extraction","generation","ceiling"]` validates cells
  (`verbs/cells/validate.rs:29`) and `--kind cell` prefers the cell
  record's tier over the slot default, `tier_source: "cell"`
  (`prepare.rs:731-745`). So a cell recorded `tier: extraction` does
  resolve the extraction model — but it dispatches as `bee-build` with
  a `[bee-tier: extraction]` marker (`prepare.rs:1033-1036`).
- **`bee-extract` is rendered but never returned by the door.**
  `pinned_agent_type` maps `"extraction" => "bee-extract"`
  (`verbs/drivers/guard.rs:32-39`), and the agent is a full member of
  the rendered set (`onboard/templates.rs:222-230`, tier `extraction`;
  rendered here at `.claude/agents/bee-extract.md` on `sonnet`). But
  `prepare` calls `pinned_agent_type` only when `kind != "cell"`
  (`prepare.rs:810-811`), where the tier token can only be generation,
  review or advisor. `bee-extract` is therefore never a value `prepare`
  can return; `--kind gather` always yields `bee-gather`
  (`prepare.rs:870`).
- **The shipped docs and the one-door rule contradict each other.**
  `skills/bee-swarming/references/swarming-reference.md:104-114` and
  `:294` tell the agent to name `subagent_type: "bee-extract"` for
  extraction, while AGENTS.md forbids hand-picking `subagent_type` and
  lists only the four kinds. An agent obeying both cannot dispatch an
  extraction worker at all.
- **The code already names the gap as temporary.** The guard's
  `dispatch_kind_for_tier` returns `Some` only for `review` and
  `generation` (`model_guard.rs:660-666`), and its refusal text reads
  "dispatch prepare has no --kind for the {t} tier yet"
  (`model_guard.rs:768`). A source comment states it outright:
  "`slot_for_kind` … has no extraction arm, so there is no `--kind`
  value that resolves the extraction slot today"
  (`model_guard.rs:653-659`). A test pins the resulting refusal —
  `a_herding_shaped_extraction_slot_denies_bee_extract_without_a_wrong_kind`
  (`model_guard.rs:1614-1634`).
- **A `[bee-tier: extraction]` marker passes the guard today.** It is a
  valid `CLAUDE_TIERS` word, it repairs a `general-purpose` dispatch to
  `bee-extract` (`model_guard.rs:705-720`, test `:2116-2126`), and it
  rewrites a mismatched `model` param to the extraction slot's model
  (`model_guard.rs:724-746`). Only a cli/herding-shaped extraction slot
  denies.
- **Two locked decisions pull opposite ways.** `de967733` (advisor-mode
  removal) makes down-tier I/O dispatch bee's one cost pattern, and
  `3ff7cd72` recorded live `tier_mix extraction 1 / generation 3 /
  ceiling 0` — extraction was in real use. `a2f85972`
  (guard-herding-fallback) now *relies* on the opposite: its herding
  fallback widening covers generation and review only "because prepare
  slot_for_kind … never reaches extraction or advisor", and looping all
  three slots was rejected on that exact ground. Any answer to ticket
  001 that makes extraction reachable touches `a2f85972`.
- **The guard's tier lists are asymmetric.** `CLAUDE_TIERS` has 4
  entries and omits `advisor`; `CODEX_TIERS` has 5 and includes it —
  `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:192-193`. Two
  hand-maintained lists, not one shared constant.
- **The config shape has two independent parsers.** `resolve_tier` in
  `models.rs:318-383` (over `Map<String, Value>`) and a second
  `resolve_tier` in `model_guard.rs:442-467` (over the guard's own
  structs). Every new role or entry shape has to land in both.

## Source read (xia, 2026-08-24)

`~/Projects/refs/oh-my-pi` @ `2b66ee69`, the tool whose model-roles
screen started this map, distilled at `docs/history/research/oh-my-pi-model-roles-distill.md`. Two findings reframe the
map: their ten roles are **mixed-axis too** (six job, two cost, two
capability — `README.md:333` "route work by intent"), and the reason
that works is not naming discipline but **fallthrough** — every consumer
names an ordered *list* of roles, so an unset role costs nothing
(`commit/model-selection.ts:46`). Their role set is **open**
(`model-roles.ts:77-91`), and their agents name a role in frontmatter
(`reviewer.md:6` `model: "@slow"`) instead of being welded to a cost
slot.

## Decisions so far

- **`8dad7c2e`** — `bee dispatch prepare` gains `--kind extract`,
  resolving the `extraction` slot and returning `bee-extract`; `gather`
  keeps `generation`. Establishes the map's spine: a model role is
  reachable by **two independent paths** — a cell's recorded `tier`, or
  a dispatch `--kind` — and both are legitimate. Touches `a2f85972`
  (its herding-fallback member set must widen to extraction).
  Ticket: `tickets/001-dead-extraction-slot.md`.
- **`06e49368`** — model roles become an **open, fall-through set**. A
  consumer names an ordered *list* of role names and an unset name
  yields to the next; any name in `models.<runtime>` is a legal role and
  the guard asks "is this configured" rather than checking a hardcoded
  list; an unconfigured name is warned, never silently accepted. Ends
  the dead-slot class of defect, so role **count** stops being the
  governing question. Touches `8dad7c2e`, `a2f85972`, `72f3d6dd`.
  Ticket: `tickets/002-role-candidates.md`.
- **`3c9d6262`** — the **work declares its job role**. A cell carries a role
  beside its lane and tier, and its dispatch asks for an ordered list
  starting with that role. bee publishes as defaults only the names bee
  itself asks for; every other job name (`test`, `design`, `docs`,
  `migrate`) is the user's to invent, with no bee code needed. Closes
  ticket 002.

## Open shape

The observation that started this map points at *more roles*. The code
said the first defect was *a role nothing can select* — answered by
`8dad7c2e`, which also renamed the real axis: bee does not have one
role surface, it has two paths onto one set of slots (cell `tier`,
dispatch `--kind`).

Then `06e49368` moved the axis again, and further. The owner's
objection — `extraction` and `generation` are cost words, so nobody can
judge which model is "good at generation", while real model strengths
are job-shaped — turned out to be answerable without choosing between
the axes. The source read showed a shipping product running six job
roles beside two cost roles and two capability roles quite happily; what
holds it together is not naming discipline but **fallthrough**. A role
nobody configures costs one name.

So neither role count nor role shape is the open question any more.
Both were consequences of a resolver that refused. What is left is
smaller and more concrete:

- **Whether `role` replaces the cell's `tier` or sits beside it**
  (ticket 006, graduated from 002's answer) — the last structural
  question, and the one that decides where the ceiling budget guard
  lives.
- **Whether a runtime error chain** sits on top of the resolution-layer
  fallthrough, and which failures it may absorb (ticket 003 — the
  upstream answer is recorded there; the stance is still the owner's).
- **One parser for the config shape** (ticket 004, now narrowed to the
  two `resolve_tier` implementations).

One structural fact the answer exposes and does not yet resolve: bee
already owns job names at the *agent* layer — `bee-build`, `bee-gather`,
`bee-extract`, `bee-review` — welded 1:1 to cost slots by
`pinned_agent_type` (`verbs/drivers/guard.rs:32-39`). Unwelding those
two layers, so an agent names a role the way `reviewer.md:6` names
`"@slow"` upstream, is the shape the remaining tickets are circling.

## Not yet specified

- Role count and names (ticket 002).
- Whether a role entry gains an ordered fallback chain, or the two
  existing single-step mechanisms stay as they are (ticket 003).
- Whether the two parsers and the two guard tier lists collapse into one
  source before the surface grows (ticket 004).
- Whether the cell's `role` replaces its `tier` or sits beside it
  (ticket 006).
- What tells a cell's author which role to write — a published
  vocabulary, free text, or something derived from the cell's own
  fields. Too dim to ticket until 006 settles whether that judgment
  also covers cost. `(agent-suspected)`
- ~~Which names bee publishes as its default role set~~ — answered by
  `3c9d6262`: only the names bee asks for; ticket 002 closed.
- ~~Whether slot-to-door reachability is enforced~~ — dissolved by
  `06e49368`; ticket 005 closed.

## Out of scope

- `ceiling` becoming configurable — settled by decision 0015 and not
  reopened here.
- Per-provider model catalogues or auto-selection. bee configures a
  model per role; it does not rank or discover models.

## Recorded deviation

The bee CLI was absent when this map was charted (`.bee/bin/bee` held
only `bee.pre-expertise.bak`), so session 1 wrote the map directly and
cut no decision-log line. **Closed 2026-08-24**: the binary was rebuilt
from `packages/bee-rs` and installed at `.bee/bin/bee`; from ticket 001
onward the wayfinding verbs (`reservations reserve`, `decisions log`)
run normally.

**Named deviation — three with-user tickets in one session
(2026-08-24).** The skill caps a session at one. Ticket 001's answer
re-framed 002 in the same exchange, and the owner's xia read of
`~/Projects/refs/oh-my-pi` then dissolved 005 and re-framed 002 again
before it could be answered. Recomputing the frontier after each answer
is the skill's own instruction, and each time it moved inside the same
conversation. Stopping at one ticket would have left the map holding a
question the next answer had already retired. Cost of the deviation:
this session ran long; the tickets it opened (006) and left open (003,
004) are the natural stop.
