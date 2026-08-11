---
name: bee-grooming
description: >-
  Hunt and kill tech debt IN THE CURRENT PROJECT — dead code, stale docs, TODO/stubs, duplication, drifted specs — reported in plain project language. bee's own housekeeping (the entropy score) is a short side-note, and `.bee/`, `.claude/`, `.codex/` are never treated as project debt. Use when the user asks to clean up, find debt, or audit the repo.
metadata:
  version: '0.2'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: Computes the entropy score from bee records via the vendored bee binary. The binary is vendored into the repo by onboarding; no Node runtime is involved.
---

# Grooming — hunt the project's debt

Grooming decides nothing alone and deletes nothing alone: hunt,
propose kills, execute only the approved ones as ordinary cells, then
record what each kill actually bought.

## Scope — the project, not the harness

You are auditing the **project** — its source, docs, tests, config.
Write every finding in plain language a non-bee user understands
("three unused functions in the export module"), never in bee
vocabulary. `.bee/`, `.claude/`, `.codex/`, the `AGENTS.md` bee block,
bee's vendored helpers, `node_modules/`, and build output are never
debt candidates. A genuine bee/harness bug found mid-hunt gets one
line — a harness issue to report upstream to bee — and the hunt moves
on.

## Hive housekeeping — the side-note

Before the hunt, compute bee's own entropy score and its trend vs. the
last run (formula, counting rules, trend record:
`references/grooming-reference.md` ("Entropy Computation")). Report
both in a few lines — never as the headline; the project hunt below is
the main event. The one term about the user's own docs (stale specs)
carries into the hunt; broken tools and bee-lib bugs are harness
health, routed upstream.

## Hunt

Per-source recipes: `references/grooming-reference.md` ("Hunt Checklists").
What you are hunting:

- dead code and unused exports — prove non-use first: dynamic imports,
  reflection, config-driven loading, and external callers all count as
  use; "obviously dead" without evidence is a red flag, not a finding
- stale docs that contradict the code — judging what counts as stale
  and which side to fix: `.bee/expertise/documentation.md`
- stale, missing, or duplicated area truth (specs or knowledge bundle)
- TODO/stub debris — each hit becomes a backlog item or a kill
  candidate, never a comment-shaped promise left in place
- friction clusters, verify-commands that no longer run,
  superseded-but-still-cited decisions, slop patterns in recent diffs
- test-prune candidates — duplicate-logic or low-validation-value
  tests, with evidence of why; the action is merging near-duplicates
  into a table-driven test or deleting a provably dead case, never a
  raw line-count cut — and every touched suite must show green after
  the prune, in the same batch
- structural debt — shallow modules, concept smear, leaking seams —
  hunted only in hot spots and judged with the deletion test
  (`references/grooming-reference.md` ("Architecture lens"),
  `.bee/expertise/architecture.md`)

## Propose

Each kill candidate: **pain** (what it costs today) / **predicted
impact** (what removal buys) / **risk lane** (tiny or small). Rank by
pain × impact and present the top few — never the full dump. Approval
is per-candidate and mandatory: no recorded approval, no kill, however
obvious the candidate looks, and approving one kill never covers its
"related" neighbors.

When the round carries three or more candidates, or any structural
one, render the proposal report
(`references/grooming-reference.md` ("Proposal report")) and hand the
user its viewer URL — the report is for looking, never for deciding:
every approval still happens per-candidate in the conversation.

## Execute

Approved kills run as ordinary tiny/small cells through the
bee-swarming worker loop ("Execute") — one approved kill per cell;
grooming never edits files directly. Mechanical scans delegate to
cheap subagents under the Delegation contract
(`bee-hive/references/routing-and-contracts.md`).

## Close the loop

After each kill, record the actual outcome against the prediction with
`bee backlog add` (fields: `references/grooming-reference.md`
("Outcome Template")). A wrong prediction is signal, not
embarrassment — feed durable lessons to bee-capturing; grooming that
never learns mows the same grass forever.

## Headless

`bee-hive` ("Headless") governs; a headless run audits and proposes
only — score, trend, hunt, ranked proposals — and never executes a
kill or deletes anything.

## References

| File | When to load |
|---|---|
| `references/grooming-reference.md` | Entropy counting rules, hunt checklists, proposal/outcome templates, slop-pattern list |
| `.bee/expertise/documentation.md` | Judging stale docs and spec drift |
