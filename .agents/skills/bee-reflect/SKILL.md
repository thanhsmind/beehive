---
name: bee-reflect
description: >-
  Mine the session transcript with three lens reviewers, synthesize the learnings and route each to a recorded edit. Use when the user types /bee-reflect or says reflect or retrospective. Not for a mistake noticed mid-task (bee mailbox reflect) or a rule just settled (bee-capturing).
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: The three reviewers and the synthesizer go out through the vendored bee binary's dispatch door. Absent, the leader applies each lens itself, one after the other, and the account says so.
---

# Reflect

Mine the current conversation for durable learnings, then route them into recorded edits. Nothing is applied before the user approves it.

## When to invoke

- The user said "reflect" or "/bee-reflect".
- A complex task (5+ tool calls) just landed cleanly and the recipe is worth keeping.
- The agent hit dead ends, found the working path, and the path generalizes.
- The user corrected the agent's approach mid-task.
- A non-trivial workflow emerged that isn't captured anywhere.

Skip when the conversation is trivial, off-topic, or already covered by an existing skill the parent followed correctly. One-offs are not learnings.

A single mistake noticed mid-task is not this skill's job: write it the moment you see it with `bee mailbox reflect --wrong "<what went wrong>" --better "<what would have been better>" --fix-at <architecture|check|doctrine|none>`. A rule that just settled goes through `bee-capturing` the same turn.

## Process

### 1. Locate the active transcript

The parent finds its own transcript file before fanning out. Use the path the host or the system prompt names for the active session. Never glob across other projects' transcript directories: that crosses workspace boundaries and reads private chats from unrelated projects.

For each candidate, read the first line and check that it contains the conversation's opening user prompt. Take the matching path. If no path resolves, write a tight digest of the session and pass that instead.

### 2. Dispatch three reviewers in parallel

Issue one `bee dispatch prepare --runtime claude --kind reviewer --json` per reviewer, all three in the same turn, and run every returned payload in the background. Use each payload exactly as it returns. Never pick a model or a subagent type by hand. Which model a reviewer runs on is a setting of the review role in the bee team table, never a choice made here.

| Lens | Prompt template |
|---|---|
| Judgment | `references/judgment-reviewer.md` |
| Tooling | `references/tooling-reviewer.md` |
| Divergent | `references/divergent-reviewer.md` |

Pass each template verbatim, substituting the transcript path or digest where marked. The prompt forbids file writes; the parent applies edits. Reviewers return findings in their response. A reviewer that looks up context referenced in the transcript (tickets, chat threads, traces) uses whatever tools its payload gives it.

A return that is not a numbered list of Principle, Evidence and Routing, or that cites a skill path that does not resolve, gets one re-dispatch with the failure named. A second failure is listed as an uncovered lens in the summary.

When the payloads name the same model for all three lenses, say so in the summary: the lenses then differ by prompt only, and agreement across them is weaker corroboration.

### 3. Synthesize

Issue one `bee dispatch prepare --runtime claude --kind reviewer --json` for the synthesizer, and run the returned payload as is. Use `references/synthesizer.md` verbatim, with each reviewer's full output inlined where marked. The synthesizer's quality check includes spot-verifying citations. It returns a structured Accepted / Rejected / Backlog list.

### 4. Structural enforcement check

Sanity-check the synthesizer's Accepted list. For any item that would be enforced more reliably by a lint rule, script, metadata flag, or runtime check, move it from Accepted to Backlog. The synthesizer already applies this criterion; this is a final pass before edits land. See `bee-principle-encode-lessons-in-structure`.

### 5. Apply

Before applying any Accepted edit, present the synthesizer's full Accepted/Rejected/Backlog output to the user and wait for explicit approval. The user picks which subset to apply and may redirect routings. Skill changes affect every future agent in the org; do not auto-apply.

Backlog items file with `bee backlog add`. Those are backlog entries, not skill edits. Only the Accepted list waits for approval.

For each approved Accepted item, follow the Routing field exactly:

- Trivial existing-skill edit (a one-line bullet, a tightened sentence, a stale fact corrected): the parent does it directly.
- Substantive existing-skill edit (a new section, a new pattern table, more than ~10 lines): hand to `bee-writing-skills` and run its draft, pressure-test and iterate loop.
- `tune description: <skill path>` (the skill exists but didn't trigger when it should have): hand to `bee-writing-skills` and run its description loop.
- `new skill via bee-writing-skills: <kebab-name>`: hand creation to `bee-writing-skills`. Do not invent the shape ad hoc.

If your environment ships a SKILL.md validator, run it on every touched skill before declaring done. Skip this step if it doesn't.

Record each approved learning before the edit lands, by what it is:

- A mistake the session made: `bee mailbox reflect --wrong "<what went wrong>" --better "<what would have been better>" --fix-at <architecture|check|doctrine|none>`. Name a concrete file, command or output in the first part, and pick the strongest layer someone would build now.
- A rule, behavior or value that settled: `bee decisions log`, with the evidence pointer in its why. `bee-capturing` owns the rest of that route.

In the bee repo a learning about how agents work is a change request to the framework (rule `agents-feedback-changes-framework`). Record it in the bee store, change the doctrine, hook or skill that owns the behavior through the bee chain (`bee-shaping`, `bee-planning`, `bee-swarming`), and release it. An agent's private memory may hold a pointer, never the only copy.

### 6. Summarize for the user

Short list, no preamble:

- Edits applied: `<skill path>`. What changed, one line each.
- New skills created: `<skill path>`. One line each (rare).
- Recorded: the `bee mailbox reflect` or `bee decisions log` entry for each approved learning, one line each.
- Backlog filed: `<issue title>` (`<tags>`). One line each.
- Dropped: one line per rejected finding + reason from the synthesizer.

## Handoff

Learnings routed, nothing applied without approval. Invoke bee-hive skill.
