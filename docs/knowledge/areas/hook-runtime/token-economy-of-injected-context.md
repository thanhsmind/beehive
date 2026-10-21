---
type: bee.area
title: Hook Runtime — the token economy of injected context
description: "How bee saves host tokens when it does not own the host request: it controls only what it injects and what it prints. The session-start budget under the Claude Code inline limit, the fixed drop order and the Omitted line, the folded discovery maps, the grouped dispatch door, the capped orient promote blocker, the savings that were already in place, and the boundary that budgets bind generated payloads only."
timestamp: 2026-10-21
bee:
  id: hook-runtime-token-economy-of-injected-context
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: ["token-thrift D1 (PREAMBLE_BUDGET_BYTES is 9500, below HOST_INLINE_LIMIT_CHARS 10,000)", token-thrift D2 (fixed drop order and one Omitted line that names each fetch path), token-thrift D3 (zero-frontier discovery maps fold into one count line), "token-thrift D4 (dispatch door prints one line per rendered target, roles in full)", "token-thrift D5 (orient promote blocker names at most 3 features, then +N more)", "691fcac4-14a8-40b1-8600-d720ebc47e6f (token-thrift D7: the preamble promote proposal line names at most 3 clear commands, then +N more)", token-thrift D6 (this concept), "8f63adb4-de70-424e-b082-1f4c5ec4b693 (budget-fence-removal D1: budgets bind generated payloads only, never authored instruction text)"]
  sources: [docs/history/token-thrift/CONTEXT.md, docs/history/token-thrift/reports/hat-wave-synthesis.md, "cell tt-1 (commit 3ca8a49fe, 2026-10-21)", "cell tt-2 (commit 646c14cff, 2026-10-21)", "cell tt-3 (commit 81f81b039, 2026-10-21)", "measured 2026-10-21: the session preamble on this repo was 11,041 to 11,161 bytes before tt-1"]
  authoritative_for: "hook-runtime: the size limits on injected session context and why they exist"
---

# Hook Runtime — the token economy of injected context

bee runs inside a host: Claude Code, Codex or Pi. The host builds the request
to the model. bee cannot trim or drop history that is already in that request.
bee controls two things only: the text it injects, and the text its commands
print. So "save tokens" in bee means three things. Inject less. Keep each
injection small enough that the host shows it in full. Print less.

## Behaviors & Operations

**The host inline limit sets the budget.** Claude Code keeps hook output of
more than 10,000 characters out of the context. It saves that output to a
file and shows a 2 KB preview. Thus an injection over the limit costs tokens
and the agent still does not read most of it. Before this feature, the
session preamble on this repo was 11,041 to 11,161 bytes, and the old budget
was 14,336 bytes. The constant `HOST_INLINE_LIMIT_CHARS` (10,000) records the
host limit. `PREAMBLE_BUDGET_BYTES` is 9500. The budget counts bytes. A byte
count is never smaller than the character count of the same text, so a
render under 9500 bytes is always under 10,000 characters. A test asserts
that the budget stays below the limit.

**The budget binds the whole session-start stdout, on every runtime.** The
host counts all of the hook output, not only the preamble. The session-start
hook first computes the belt line and the intent anchor. It then subtracts
their bytes and their separators from the budget, and it gives the remainder
to `build_session_preamble_within`. A long anchor on a `resume` start thus
makes the preamble smaller; it never pushes the stdout over the limit. All
runtimes use the same builder, so the smallest host limit sets the budget
for all of them.

**Over budget, four optional sections drop in a fixed order.** The builder
assembles the full preamble. While the text is over budget, it removes one
whole section, in this order:

1. Recent decisions
2. Critical patterns (digest)
3. Command surface
4. Project map

A dropped section takes its heading and its blank separator line with it. The
header, the gates, HANDOFF, the debt counts, the dispatch door, the knowledge
context and the discovery maps never drop. When the four drops are not
sufficient, the builder stops and returns the text that remains. It never
cuts a section in the middle.

**One Omitted line names each fetch path.** When at least one section
dropped, one line goes before the closing trailer:
`### Omitted to fit the host limit: <section> → <fetch>; …`. Each entry names
its own fetch path, because one command does not carry all four:

- Recent decisions → `.bee/bin/bee status --json`
- Critical patterns (digest) → `docs/knowledge/index.md`
- Command surface → `.bee/bin/bee --help --names`
- Project map → `docs/knowledge/index.md`

`bee status --json` carries the recent decisions, but not the pattern digest
or the project map. A quiet repo that fits renders no Omitted line.

**Discovery maps with no frontier ticket fold into one line.** A map with
frontier tickets keeps its own line. All maps with zero frontier tickets fold
into one count line: `### Open discovery map(s): <n> with no frontier ticket —
bee discovery list`. The count is the signal. A map with nothing to resume
is not an action, and one line per map was a list that grew with the store.

**The dispatch door prints one line per rendered target.** The door lists
each distinct target once, as `- <target>: <role>, <role>, …`, in the order
of first appearance. Each role appears on exactly one line, in full, with no
`…` cut. In the test fixture, 18 roles print as 4 lines. Before, the door printed one
line per role and cut each line at 60 characters, so long herding agent
names ended in `…`. The compaction re-inject reads the same function, so both
surfaces print the same lines.

**The orient promote-proposal blocker names at most three features.** The
blocker names three features with their clear commands, then `+N more`. The
count stays the total. Three or fewer features render the same bytes as
before. Before, the line named every feature (74 on this repo). The
preamble's `### Unapplied promote proposal(s)` line has the same cap: it keeps
the count and the newest path, names at most 3
<!-- bee:not-a-deferral: names the clear command, promises no later action -->
`bee deferred-queue complete`
<!-- /bee:not-a-deferral -->
clear commands, then `; +N more` (token-thrift D7).

**Already in place, before this feature:**

- The per-turn reminder is deduped by hash. `should_inject` in
  `hooks/prompt_context.rs` injects again only when the hash changes, or when
  30 minutes (`INJECT_INTERVAL_MS`) have passed since the last injection.
- Per-turn text is appended. It never edits earlier text, so the cached
  prefix of the host request stays the same.
- Each cell runs in a fresh dispatched worker. A worker starts with the brief
  of its cell, not with the history of the session that dispatched it.

## Business Rules

- R1 — A byte budget on an injected payload stays below the smallest host
  inline limit. The test pins `PREAMBLE_BUDGET_BYTES < HOST_INLINE_LIMIT_CHARS`
  (token-thrift D1).
- R2 — A cap must hold for any store. A section that grows with the store
  either drops whole in the fixed order, folds into a count, or names a fixed
  number of items and then `+N more` (token-thrift D2, D3, D5, D7).
- R3 — A dropped section is never silent. The Omitted line names it and the
  path that fetches it again (token-thrift D2).
- R4 — Budgets bind generated payloads only: the preamble, the door, the
  orient blockers. They never bind authored instruction text, such as skills
  or AGENTS.md (decision 8f63adb4-de70-424e-b082-1f4c5ec4b693,
  budget-fence-removal D1).

## Edge Cases Settled

- When the sections that never drop are over budget alone, the stdout stays
  over budget. The builder drops all four optional sections and returns. Cell
  tt-1 first measured the pathological fixture with the four sections
  removed, and the result was under 9500 bytes. A live render on this repo
  then measured 11,444 bytes with all four sections gone, because the promote
  proposal line carried 74 clear commands. D7 capped that line.

## Out of Scope

- Trimming or dropping host history. The host owns the request.
- Wait timing tuned to the cache TTL. The host owns its wakeup tool, and the
  TTL is different for each host and plan.
- Byte budgets on skills or AGENTS.md. Decision 8f63adb4 forbids them.

## Open Gaps

- The compact capsule (`build_compact_capsule_with_runtime`) has no budget
  yet. It is a separate path, and nobody measured it over the limit.

## Pointers (implementation)

- Budget, host limit, drop order and Omitted line:
  `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs`
  (`HOST_INLINE_LIMIT_CHARS`, `PREAMBLE_BUDGET_BYTES`, `DROPPABLE`,
  `fit_blocks`, `compose_with_drops`, `build_session_preamble_within`,
  `open_maps_lines`). The stdout budget: `compose_output_budgeted` in
  `packages/bee-rs/crates/bee/src/hooks/session_init.rs`. Tests:
  `hooks/session_preamble/tests.rs`
  (`the_preamble_stays_inside_its_budget_however_big_the_store_gets`).
- Grouped dispatch door: `dispatch_door_lines` in
  `packages/bee-rs/crates/bee/src/hooks/model_guard.rs`. Tests:
  `the_door_prints_one_line_per_target`,
  `dispatch_door_groups_all_18_roles_by_target_in_full`.
- Reminder dedupe: `should_inject` and `INJECT_INTERVAL_MS` in
  `packages/bee-rs/crates/bee/src/hooks/prompt_context.rs`. Test:
  `should_inject_dedup_rules`.
- Orient promote blocker cap: `PROMOTE_PROPOSALS_SHOWN` in
  `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs`.
- Preamble promote proposal line cap: `promote_proposal_lines` and
  `PROMOTE_CLEAR_COMMANDS_SHOWN` in
  `packages/bee-rs/crates/bee/src/hooks/session_preamble/render.rs`.
- Locked decisions: `docs/history/token-thrift/CONTEXT.md`.
