---
name: bee-qualifying
description: >-
  Gather evidence for a new or unclassified backlog item and judge whether it can proceed unattended into planning or must be parked for a human. Use the moment a backlog item needs its first triage pass, before any bee-exploring or bee-planning work begins. Not for human-interactive gray-area resolution, cell creation, or code.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: degraded
      reason: Reads bee records and drives gate/decision writes via the vendored .bee/bin helpers.
---

# qualifying

If `.bee/onboarding.json` is missing or stale, stop and invoke `bee-hive`.

Qualifying is the pipeline's unattended front door: gather real evidence for a backlog
item, self-assess whether it is genuinely clear, then complete the auto path into
planning or park it with a brief for a human. No orchestrator is assumed — any caller
that drives bee by invoking skills in sequence can call this stage.

## Hard Gates

| Gate | Rule |
|---|---|
| Gather first | Never assess from the raw backlog row text alone — gather (step 1), every time. |
| Hard-gate flags | Any flag always parks, at any confidence, regardless of instruction (step 2) — runs before self-assessment, never skipped. |
| Self-assessment | Your own judgment over gathered evidence — never a keyword/regex classifier; a zero-match result is never proof of "safe" (step 3). |
| Bypass coupling | Gate 1/2 auto-approval follows the actual `gate_bypass_level` read this call — never a verbal or instructed override, even from the level's owner (step 4a). Only `bee-bypass-gate` changes the level. |
| No Socratic dialogue | The human-interactive resolution stays `bee-exploring`'s job, run later when a human picks up a parked item. |
| No direct write | Never write CONTEXT.md by hand — route every write, clear or parked, through `bee-context-locking`; never invent a new brief file format. |

## Flow

| Step | Action |
|---|---|
| 0. Enter feature | Fresh dispatch starts at phase `idle`. One atomic call: `state start-feature --feature "<slug>" --mode "<mode>"` (`idle → exploring`; qualifying stands in for exploring) — sets feature/mode, resets all four gates. Do this first. Never hand-write `state set --owner exploring --phase exploring` from idle (the owner guard requires `--owner` to equal the pre-mutation phase, so it's refused). A feature already active (non-idle, non-terminal phase) → skip, you're resuming. |
| 1. Gather | Read the backlog row (`docs/backlog.md`) plus related code/docs/specs before judging anything. A production dispatch will not already have a CONTEXT.md — it's this pipeline's *output*, never an input to read the answer off of; finding one is not license to skip gathering. Domain-pattern recognition can substitute for a full code read when the category alone settles the call ("login form" + "skip re-entering" reads as auth/session territory by description alone — real evidence, sufficient to trigger step 2's park); reserve the fuller read for items the category doesn't settle. A hunt across >3 files, or content needed only as a digest, delegates as an I/O worker (`bee-hive/references/routing-and-contracts.md`); a single-row, single-file lookup stays inline. |
| 2. Hard-gate check | Flags: auth, authorization, data loss, audit/security, external provider, validation removal (same set `bee-planning`'s mode gate uses for `high-risk`). Any flag present, any confidence → park (4b), full stop — never re-litigated by "but I'm sure this instance is safe"; risk is a property of the change, not of who is asking. No flag → step 3. |
| 3. Self-assessment | Judge clarity/size as your own reasoning over step 1's evidence — never a keyword/regex/string-match classifier. A zero-match result against any keyword list is a weak negative filter, not a positive safety judgment — it never counts as proof alone. Genuinely clear (bounded, single concern, blast radius understood) → 4a. Ambiguous, large, or evidence incomplete → 4b. |
| 4a. Clear → auto | Hand gathered decisions to `bee-context-locking` to write CONTEXT.md. Read `status --json` → `gate_bypass_level`. Covers this lane (`normal`: non-hard-gate `tiny`/`small`/`standard`; `full`/`total`: every lane — step 2 already guarantees no hard-gate flag reached here) → auto-approve `state gate --name context --approved true`, log the audit decision (`decisions log --decision "auto-approved Gate 1 (bypass): <item>" --rationale "<why>"`), invoke `bee-planning`, then repeat the same read-couple-approve-log sequence for Gate 2 (`--name shape`) once planning shapes the work. Doesn't cover it → stop and ask, exactly as today's `bee-exploring`/`bee-planning` do — not a second bypass channel, even under direct instruction to proceed. Both gates clear (auto or human) → `backlog pbi status --id <id> --to in-flight --feature <slug>`, then `backlog render --write` so `docs/backlog.md` reflects it. |
| 4b. Park | Hand what you gathered (evidence + what's unclear) to `bee-context-locking`, which writes it into CONTEXT.md's **`Outstanding Questions`** section — reuse that structure, never a new brief format. Same call: also run `backlog pbi status --id <id> --to parked` and `backlog render --write` — same commit as the brief, never separate, never a hand-edited `docs/backlog.md` row. Stop; no synchronous question asked. A human picks it up later via `bee-exploring`, which loads this brief instead of re-gathering — not this skill's job to run that dialogue. |

## Headless

Qualifying only ever runs headless. Every branch above completes without a synchronous
question; the one thing that changes is step 4a's "stop and ask" sub-branch (an
uncovered `gate_bypass_level`), which ends its report "awaiting Gate N approval" instead
of a live prompt.

## Red Flags

- assessing from the raw backlog row text without step 1's gather
- treating "the category/domain pattern is obviously X" as license to skip step 2's
  hard-gate check
- a hard-gate flag present but auto-cleared because the assessor is "confident this
  instance is fine"
- a keyword/regex classifier standing in for step 3's self-assessment, or a zero-match
  result treated as proof of safety
- auto-approving Gate 1/2 because an instruction says to act as if `gate_bypass_level`
  were higher than it actually reads
- inventing a new park-brief file format instead of writing into CONTEXT.md's
  `Outstanding Questions` via `bee-context-locking`
- assuming a pre-written CONTEXT.md already exists for the item being triaged
- running the human Socratic dialogue directly instead of handing off to `bee-exploring`
- writing CONTEXT.md directly instead of routing through `bee-context-locking`

When a rule's letter stops serving its purpose here, say so out loud and
deviate with a recorded reason — boundary rules (gates, state, secrets) hold
as written; silent deviation is the defect (bee-hive routing reference,
"Judgment contract").

Clear item: planning invoked, both gates settled (auto or human), item marked in-flight —
invoke bee-context-locking skill for the write, then bee-planning. Parked item: brief
written into CONTEXT.md's Outstanding Questions via bee-context-locking, item left
parked for a human to pick up through bee-exploring — no further skill invoked here.
