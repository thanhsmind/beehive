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
  decisions: ["token-thrift D1 (PREAMBLE_BUDGET_BYTES is 9500, below HOST_INLINE_LIMIT_CHARS 10,000)", token-thrift D2 (fixed drop order and one Omitted line that names each fetch path), token-thrift D3 (zero-frontier discovery maps fold into one count line), "token-thrift D4 (dispatch door prints one line per rendered target, roles in full)", "token-thrift D5 (orient promote blocker names at most 3 features, then +N more)", "691fcac4-14a8-40b1-8600-d720ebc47e6f (token-thrift D7: the preamble promote proposal line names at most 3 clear commands, then +N more)", token-thrift D6 (this concept), "8f63adb4-de70-424e-b082-1f4c5ec4b693 (budget-fence-removal D1: budgets bind generated payloads only, never authored instruction text)", "lean-context D1 (prompt-context warns at handoff_context_tokens; token-spend-cuts D1 turned the line into a handoff directive with a 300000 default)", "token-spend-cuts D1 (handoff_context_tokens defaults to 300000; at or past it the prompt-context line tells the leader to hand off at the next clean stop; a green worktree merge ends its next action with the fresh-leader-session handoff)", "token-spend-cuts D4 (lane_budget halves to tiny 4000, small 6000, standard 10000, high-risk 15000, default 10000; no docs/knowledge/index.md fallback in the learned-context block)", "token-spend-cuts D3 (every worker agent template in packages/bee/agents carries the read rule: rg -n first, then a Read range with offset and limit)", "semantic-token-diet D7 (preamble trailer names bee orient for routing and bee status --json only for missing detail; Proof-per-change-type line removed)", "semantic-token-diet D8 (herding brief renders # Proof command only for cell jobs)", "semantic-token-diet D9 (single-command help prints first sentence, flags, then remaining text under details:; heaviest descriptions drop decision ids and history narration)", "reread-thrift D1 (a merge stores the merging session; feature start refuses a second feature in it with handoff_due unless --same-session carries a reason)", "reread-thrift D2 (herding result --job prints a digest of at most 1500 characters; a report opens only on a red, a question or a blocked status)", "reread-thrift D3 (thin worker launches carry SLASH_COMMAND_TOOL_CHAR_BUDGET=1, --disable-slash-commands and --disallowedTools Agent; first call 20739 to 8887)", "reread-thrift D4 (the worker cell prompt carries a search rule; a cell action names each function as path plus symbol)", "reread-thrift D5 (packages/bee/LEADER.md cut from 21685 to 14915 bytes, detail moved to playbooks/leader-detail.md)", "reread-thrift D6 (the fixed cell brief text cut from 15227 to 11413 bytes)"]
  sources: [docs/history/token-thrift/CONTEXT.md, docs/history/token-thrift/reports/hat-wave-synthesis.md, "cell tt-1 (commit 3ca8a49fe, 2026-10-21)", "cell tt-2 (commit 646c14cff, 2026-10-21)", "cell tt-3 (commit 81f81b039, 2026-10-21)", "measured 2026-10-21: the session preamble on this repo was 11,041 to 11,161 bytes before tt-1", docs/history/lean-context/CONTEXT.md, cell lnc-1, docs/history/token-spend-cuts/CONTEXT.md, "token-spend-cuts cells tsc-1, tsc-2, tsc-4 (commits 5a9dae308, c4e633e93, 7674b3f4f, 2026-11-21)", docs/history/semantic-token-diet/CONTEXT.md, "semantic-token-diet cells stdi-1..stdi-6 (commits b519417c6, 66ca1f554, 73d8810d5, be207c310, c611721a7, 2026-11-16)"]
  authoritative_for: "hook-runtime: the size limits on injected session context and why they exist"
  anchors: [packages/bee-rs/crates/bee/src/hooks/prompt_context.rs, packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs, packages/bee-rs/crates/bee/src/hooks/session_init.rs, packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs, packages/bee-rs/crates/bee/src/hooks/model_guard.rs, packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs, packages/bee-rs/crates/bee/src/verbs/fix_at.rs, packages/bee-rs/crates/bee/src/hooks/session_preamble/render.rs, packages/bee-rs/crates/bee/src/herding/mailbox.rs, packages/bee-rs/crates/bee/src/verbs/help.rs, packages/bee-rs/crates/bee/src/generated/registry_payload.json, packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs, packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs, packages/bee-rs/crates/bee/src/herding/run.rs, packages/bee-rs/crates/bee/src/herding/claude_session.rs, packages/bee-rs/crates/bee/src/herding/wave.rs, packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs, packages/bee-rs/crates/bee/src/verbs/feature_group.rs, packages/bee-rs/crates/bee/src/herding.rs, packages/bee/prompts/worker-cell.md, packages/bee/LEADER.md, packages/bee/playbooks/leader-detail.md]
  verified_at: 2026-11-16
  verified_commit: d2ac1ce50e94b747d59335242b9ce380c5f1eebb
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

**Over budget, five optional sections drop in a fixed order.** The builder
assembles the full preamble. While the text is over budget, it removes one
whole section, in this order:

1. Recent decisions
2. Critical patterns (digest)
3. Command surface
4. Project map
5. Fix-at debt (`### Fix-at debt`, registered in `DROPPABLE` in `budget.rs`, mistake-loop-closed D6)

A dropped section takes its heading and its blank separator line with it. The
header, the gates, HANDOFF, the capture-queue counts, the dispatch door, the knowledge
context and the discovery maps never drop. When the five drops are not
sufficient, the builder stops and returns the text that remains. It never
cuts a section in the middle.

**One Omitted line names each fetch path.** When at least one section
dropped, one line goes before the closing trailer:
`### Omitted to fit the host limit: <section> → <fetch>; …`. Each entry names
its own fetch path, because one command does not carry all of them:

- Recent decisions → `.bee/bin/bee status --json`
- Critical patterns (digest) → `docs/knowledge/index.md`
- Command surface → `.bee/bin/bee --help --names`
- Project map → `docs/knowledge/index.md`
- Fix-at debt → `bee backlog fix-at next` (mistake-loop-closed D6)

`bee status --json` carries the recent decisions, but not the pattern digest
or the project map. A quiet repo that fits renders no Omitted line.

**The Fix-at debt block summarizes open fix-at groups when any exist (mistake-loop-closed D6).**
Inserted after the capture queue, `### Fix-at debt` shows the total open count,
per-layer counts (`check`, `architecture`, `rejudge`), recurred count, oldest
open age in days, and a `next` / `promote` pointer. It is registered in
`DROPPABLE` in `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs`
with fetch target `bee backlog fix-at next` so it drops before core gates if
the preamble approaches `PREAMBLE_BUDGET_BYTES`.

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
  `packages/bee-rs/crates/bee/src/hooks/prompt_context.rs` injects again only when the hash changes, or when
  30 minutes (`INJECT_INTERVAL_MS`) have passed since the last injection.
- Per-turn text is appended. It never edits earlier text, so the cached
  prefix of the host request stays the same.
- Each cell runs in a fresh dispatched worker. A worker starts with the brief
  of its cell, not with the history of the session that dispatched it.

**The prompt-context hook tells the leader to hand off once the context passes the mark (token-spend-cuts D1).**
On UserPromptSubmit, the prompt-context hook reads the payload's `transcript_path`
and takes the latest request context (input + cache read + cache creation tokens,
as statusline `context_and_turns` computes it). When that value is at or above
`handoff_context_tokens` from `.bee/config.json` (integer; absent or not a number
means 300000; 0 turns the check off), the injected context carries exactly one extra line:
`context <N>k tokens (mark <T>k): hand off at the next clean stop (a capped cell or a merged worktree), then continue in a fresh leader session`.
The line is a directive, not advice: the leader stops at the next clean stop, never mid-cell.
It replaced the context-mark-advisory line `keep working; hand off only on the user's word or at feature close`
and the 150000 default, because two survey-platform leader sessions ran about 4.5 hours with
12 merges each and peaked at 556k and 425k tokens, and 65 percent of spend was cache read.
A green `bee worktree merge` ends its next action with the same handoff:
`Merged into main at <sha>; capture what settled, then bee close --feature <feature>, then write the handoff and continue in a fresh leader session (one leader session per feature).`
The uat merge variant is unchanged.
Below the mark, with no `transcript_path`, or with an unreadable or usage-free
transcript, nothing is added and the hook never fails or slows the turn.

**The worker brief's learned-context block is smaller and never falls back to the knowledge index (token-spend-cuts D4).**
`lane_budget` in `verbs/drivers/prepare.rs` caps the knowledge pack by lane:
tiny 4000, small 6000, standard 10000, high-risk 15000, and 10000 for any other or absent lane.
These halved the old 8000 to 30000 range. When the manifest ranks no concept for the
cell, `bundle_learned_lines` returns an empty list; it no longer lists `docs/knowledge/index.md`,
which survey-platform workers were handed 31 times and read 0 times. Workers still
reach knowledge through `bee knowledge search`. The knowledge A/B holdout is unchanged.

**The preamble trailer points routing at `bee orient` (semantic-token-diet D7).**
The closing `TRAILER_LINE` reads: `Everything above is already read — do not
re-fetch it. Run .bee/bin/bee orient when you route work (claim, plan, change
phase); run .bee/bin/bee status --json only for detail this block does not
carry.` It drops the sentences `Never hand bee commands to the user.` and
`Route via bee-hive.` because `LEADER.md` already holds them. The preamble
also drops the `Proof-per-change-type` line because `agents-proof-at-cap`
already owns that rule.

**A herding brief renders `# Proof command` only for a cell job (semantic-token-diet D8).**
`render_brief` in `packages/bee-rs/crates/bee/src/herding/mailbox.rs` emits
`# Proof command` only when `spec.cell_id` is `Some`. A non-cell job such as a
reviewer or advisor has no cell verify to prove, so printing the project test
command added noise to its brief.

**Every worker agent template carries one read rule (token-spend-cuts D3).**
Each template in `packages/bee/agents` (`bee-worker`, `bee-comb`, `bee-guard`,
`bee-forager`, `bee-scout`, `bee-elder`) tells its agent to locate with `rg -n`
first, then Read a range with offset and limit, never to dump a file with
`cat`, `head`, `tail` or `sed`, and to read a whole file only when the work
edits most of it or it is short. The rule sits in the template because a
template reaches every project that installs bee. Find the rule with
`rg -n "Read rule"` in that directory.

**Reread thrift: the leader and its workers stop re-reading what they never use.**
Read when: a leader session grows past the mark, a worker's first call is heavy, or a brief, a report or a search floods a context.
Each rule below came from one measured session of 103.4M tokens, where the leader read its own context again on 334 calls.

- Thin worker switches (reread-thrift D3). A thin worker launch gets `SLASH_COMMAND_TOOL_CHAR_BUDGET=1` with `THIN_PEER_CLAUDE_ENV`. Where bee builds the claude argument list, it also adds `--disable-slash-commands` and `--disallowedTools Agent`. The first-call context was 20739 tokens, 14803 with the env switch, and 8887 with all three. A switch stays only when it cuts at least 2000 tokens. A paseo worker gets the env switch alone, because `paseo run` passes only `--env`. The SessionStart, PreToolUse and Stop hooks still fire.
- Handoff door (reread-thrift D1). A merge stores the merging session. `bee feature start` refuses a second feature in that session with `handoff_due`. `--same-session` with a reason passes and goes to the audit log. A caller with no session id is not refused.
- Result digest (reread-thrift D2). `bee herding result --job <id>` prints at most 1500 characters of the result file and never the report body. The leader opens a report only on a red, a question or a blocked status, and starts a diff review with the stat view.
- Search rule (reread-thrift D4). The worker cell prompt says to locate with `rg -n` and read a range. A cell action names each function it changes as path plus symbol. `rg` and `fd` output was 41 percent of worker tool output.
- LEADER.md size limit (reread-thrift D5). `packages/bee/LEADER.md` went from 21685 to 14915 bytes against a limit of 15000. Every rule marker stays with its boundary sentence, and the cut detail sits in full in `packages/bee/playbooks/leader-detail.md`.
- Brief size cut (reread-thrift D6). The fixed text of a cell brief went from 15227 to 11413 bytes, a cut of 25 percent, measured on one rendered cell brief before and after. Every result field and status the parser accepts is still named.

**Single-command help prints flags before details (semantic-token-diet D9).**
For a single command, `render_help_text` in
`packages/bee-rs/crates/bee/src/verbs/help.rs` prints the invoke line, the first
sentence of `description`, the `flags:` block, and then any remaining text
under `details:`. Multi-command help (`--all`) keeps its existing layout. The
heaviest descriptions in `registry_payload.json` drop decision IDs and history
narration while keeping every current behavior; `herding control-loop` help
names all five roles (`dispatch`, `merge`, `route`, `supervisor`, `broker`).

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
  `packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs`
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
- Handoff context warning: `compute_handoff_warning` and
  `handoff_context_tokens` in
  `packages/bee-rs/crates/bee/src/hooks/prompt_context.rs`. Tests:
  `transcript_past_handoff_mark_carries_warning_line`,
  `transcript_below_handoff_mark_adds_no_line`,
  `handoff_context_tokens_zero_adds_no_line_at_any_size`,
  `missing_or_unreadable_transcript_adds_no_line_and_exits_zero`.
  The green merge next action lives in
  `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs`. Find the contract
  tests with `rg -n handoff_mark` in the crate.
- Knowledge pack budget: `lane_budget` and `bundle_learned_lines` in
  `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`. Find the contract
  tests with `rg -n knowledge_pack` in the crate.
- Preamble trailer: `TRAILER_LINE` in
  `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs`
  (semantic-token-diet D7).
- Cell-only proof command in herding briefs: `render_brief` in
  `packages/bee-rs/crates/bee/src/herding/mailbox.rs`
  (semantic-token-diet D8).
- Flags-before-details single-command help: `render_help_text` in
  `packages/bee-rs/crates/bee/src/verbs/help.rs` and command descriptions in
  `packages/bee-rs/crates/bee/src/generated/registry_payload.json`
  (semantic-token-diet D9).
- Reread thrift: thin worker env in `THIN_PEER_CLAUDE_ENV` and `thin_peer_env` in
  `packages/bee-rs/crates/bee/src/herding/run.rs`; the argument list in `spawn_argv` in
  `packages/bee-rs/crates/bee/src/herding/claude_session.rs` and the pane argv in
  `packages/bee-rs/crates/bee/src/herding/wave.rs`; the merge session in
  `merge_text_lines` in `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs`
  and the `handoff_due` refusal in `run_start` in
  `packages/bee-rs/crates/bee/src/verbs/feature_group.rs`; the digest in
  `envelope_result` in `packages/bee-rs/crates/bee/src/herding.rs`; the brief in
  `render_brief` in `packages/bee-rs/crates/bee/src/herding/mailbox.rs` and
  `packages/bee/prompts/worker-cell.md`. Find the numbers again with
  `bee report usage` on a session and `claude -p` with the thin env.
- Worker read rule: `packages/bee/agents/bee-worker.md.tmpl` and its five
  sibling templates in `packages/bee/agents` (token-spend-cuts D3).
- Locked decisions: `docs/history/token-thrift/CONTEXT.md`,
  `docs/history/lean-context/CONTEXT.md` (D1),
  `docs/history/semantic-token-diet/CONTEXT.md` (D7–D9),
  `docs/history/token-spend-cuts/CONTEXT.md` (D1, D3–D5),
  `docs/history/reread-thrift/CONTEXT.md` (D1–D6).
