# Compounding Reference

Load after `bee-compounding` is selected. Protocol lives in SKILL.md; prompts and templates live here.

## Analyst Prompts

Each analyst receives: the feature name, CONTEXT.md, plan.md, the cell list with traces, review findings, and the commit log for the feature. Nothing else — never session history. They return findings as structured text to the orchestrator and never write files.

```text
You are the pattern extractor. From the evidence provided, identify reusable
code, process, or integration patterns that worked. For each: name the pattern,
cite the concrete file/command/flow where it appeared, and state when a future
agent should reuse it. Return findings only; write no files.
```

```text
You are the decision analyst. From the evidence provided, identify the important
choices made: what was decided, what alternatives existed, what tradeoff was
accepted, and what surprised us. Flag any decision future planning must honor
or supersede. Return findings only; write no files.
```

```text
You are the failure analyst. From the evidence provided, identify blockers,
wrong assumptions, regressions, verification gaps, and friction recorded in cell
traces. For each: what happened, the root cause, and the check that would have
caught it earlier. Return findings only; write no files.
```

Tiers: pattern extractor = extraction; decision and failure analysts = generation; synthesis = ceiling (the orchestrator itself).

**Spawn type & waiting (SKILL.md §2).** Dispatch each analyst as the runtime's **read-only** agent type (Claude Code: `Explore`), never `general-purpose` — the analysts only read evidence and return text, and "write no files" in the prompt is not a tool restriction. Waiting is event-driven: launch all three, end the turn, let completions notify you; never poll liveness. A dispatch denied/errored at creation (e.g. model-guard on a missing `[bee-tier: …]` marker) created no subagent — surface it, fix the cause, re-dispatch that one **once**, then synthesize from whatever returned. Synthesis never requires three-of-three; never loop a failing dispatch or wait on a subagent that was never created.

## Learnings File Template

Path: `docs/history/learnings/YYYYMMDD-<slug>.md`. Slug: `YYYYMMDD-<primary-topic>-<secondary-topic>`, lowercase hyphens only.

```markdown
---
date: YYYY-MM-DD
feature: <feature-name>
categories: [pattern, decision, failure]
severity: critical | standard
tags: [tag1, tag2]
---

# Learning: <Concise Title>

**Category:** pattern | decision | failure
**Severity:** critical | standard
**Tags:** [tag1, tag2]
**Applicable-when:** <when future agents should use this>

## What Happened

<2-4 concrete sentences. Name files, commands, tools, or flows.>

## Root Cause

<Why it happened, or why the pattern worked.>

## Recommendation

<Imperative rule: "When X, do Y." Specific enough to act on.>
```

Multiple findings from one feature go in one dated file as repeated Learning sections — not one file per finding.

## Promotion Decision Tree

1. Seen twice (review finding, user correction, repeated deviation) AND it clears the three promotion criteria below? If not, it stays a learning entry.
2. Mechanizable? A grep/lint line in a verify command, a `bin/lib` guard, a hook denial → **promote as the check**, note the check's location in the learnings file, done.
3. Not mechanizable (judgment, taste, product intent) → promote as prose below.

## Critical Promotion Format

Only lessons passing all three criteria (multi-feature relevance, meaningful waste prevented, generalizable) get promoted. **With a bundle**, a promoted lesson is authored as a `bee.pattern` concept under `docs/knowledge/patterns/` and picked up by the generated root index's `## Critical patterns` section on the next `bee.mjs knowledge index` — never appended to `critical-patterns.md`, which in a bundled repo is a pointer stub carrying no lessons. **With no bundle**, today's guidance stands, unchanged: append the summary block below to `docs/history/learnings/critical-patterns.md`:

```markdown
## [YYYYMMDD] <Learning Title>
**Category:** pattern | decision | failure
**Feature:** <feature-name>
**Tags:** [tag1, tag2]

<2-4 sentence summary: what happened, root cause, and the future rule.>

**Full entry:** docs/history/learnings/YYYYMMDD-<slug>.md
```

critical-patterns.md is injected into every session preamble — every low-signal block you add taxes every future session. When in doubt, do not promote.

## Decision Logging

```
node .bee/bin/bee.mjs decisions log --decision "..." --rationale "..." [--alternatives "..."] [--confidence N]
```

- Log only decisions with forward force (conventions adopted, approaches rejected with reasons, constraints discovered).
- Include `--alternatives` whenever real alternatives were weighed; add `--confidence N` when the evidence was partial.
- To change a past decision: `node .bee/bin/bee.mjs decisions supersede --id UUID --decision D --rationale R`. Never rewrite the log.
- The logger rejects secret-like content and injection patterns; do not try to work around a rejection — redact instead.

## State-Layer Guard

The area-concept, area-spec, and reading-map templates live in `bee-scribing/references/scribing-reference.md` — compounding never writes the state layer itself (`docs/knowledge/` when the repo has a bundle, else `docs/specs/`). The guard check reads `.bee/state.json` for the feature's scribing record; absent while `behavior_change` cells were capped → invoke `bee-scribing`, then resume.

## Friction Backlog Entry

Unresolved friction (from cell `trace.friction` or the session) appends to `.bee/backlog.jsonl`:

```json
{"ts":"<ISO>","type":"friction","feature":"<feature>","title":"<short name>","detail":"<what kept hurting>","predicted_impact":"<what it will cost if left>","layer":"<spec|context|environment|verification|state>","source":"compounding"}
```

`layer` is optional but valuable: attribute the friction to exactly one harness layer —
`spec` (the task was underspecified), `context` (the right information wasn't provided),
`environment` (the tooling/setup failed), `verification` (feedback was missing or wrong),
`state` (continuity/records failed). Grooming aggregates these to find the bottleneck
layer; entries without `layer` stay valid.

## State Update

```json
{
  "phase": "compounding-complete",
  "summary": "Compounding complete. Learnings captured for the next feature.",
  "next_action": "Start the next feature or reopen deferred follow-up work.",
  "last_compounding_run": {
    "feature": "<feature-name>",
    "date": "YYYY-MM-DD",
    "learnings_file": "docs/history/learnings/YYYYMMDD-<slug>.md",
    "critical_promotions": 0,
    "scribing_verified": true
  }
}
```

Merge these fields into `.bee/state.json`; do not drop `approved_gates` or other existing fields.

## Red Flags

- skipping compounding for meaningful work
- promoting everything as critical
- writing vague advice such as "test more carefully"
- inventing findings when evidence is thin
- an analysis subagent writing durable files directly
- an analyst spawned with a write-capable agent type, or a failing/denied dispatch looped or waited-on forever instead of synthesizing from what returned (§2 spawn/wait contract)
- unredacted secrets or PII in any durable record
- compounding writing the state layer itself (`docs/knowledge/` or `docs/specs/`) instead of invoking bee-scribing

## Guard the state layer

`bee-scribing` owns the state layer — `docs/knowledge/` when the repo has a bundle, else `docs/specs/`; compounding only verifies the handoff happened:

1. Check `.bee/state.json` for the feature's scribing record ("scribing: N specs synced" or "scribing: no sync needed").
2. Record present → note it in the run summary and move on.
3. Record absent while `behavior_change` cells were capped → **invoke bee-scribing now**, then resume compounding. Never merge specs inline "to save a step" — the BA-grade template, sources, and rebuild check live in scribing, and a shortcut sync produces exactly the shallow spec this rule exists to prevent. **This is mechanically enforced: `state set --phase compounding-complete` is REFUSED while any capped `behavior_change` cell is unscribed, and the refusal names every one.** You cannot close around it. If the behavior genuinely belongs in no spec, `--waive-scribing-debt` is the sanctioned door — it permits the close and logs a decision naming every cell you waived.

**Backlog done-flip fallback:** confirm the feature's `docs/backlog.md` row flipped to `done` with a `docs/history/<feature>/` link. Scribing owns that flip at sync; when scribing legitimately NOOPed (no `behavior_change` cell, nothing to sync), compounding is the last close point — do the done-flip here, under the identical per-clause CoS check as scribing, never looser: enumerate every CoS clause and cite delivered evidence per clause. Any clause without evidence means no flip — the row stays `in-flight` with a `Delivered:`/`Remaining:` annotation naming the subset still owed, and the remainder may split into a new row when the delivered subset is independently shippable; silent full-flip on partial delivery is never allowed here either, so no shipped feature leaves a stale `in-flight` row wearing an unearned `done`. The backlog done-flip is prose-ruled; the **scribing record** it sits next to is mechanically enforced at the close.

**Review candidate at close:** the feature closes without independent review — that is the normal path, not a shortcut. Register the completed change set so it can be picked up by a later user-invoked review: `node .bee/bin/bee.mjs reviews candidate add --feature <feature> --head "$(git rev-parse HEAD)" --mode <lane>` (`<lane>` is the feature's lane — tiny/small/spike/standard/high-risk). Then post the completion line: "Completed and verified: N cells. Independent review not requested; the change set was added to review candidates." Never describe the close as reviewed or approved — the feature is truthfully `unreviewed` until a user-invoked review session covers this head.

## Feedback digest

After the learnings file is written, refresh the local feedback digest so the evolving-loop telemetry stays current:

```
node .bee/bin/bee.mjs feedback digest
```

Run this unprompted at every close — it is part of compounding, not an optional extra, and no user, teammate, or missing skill mention excuses skipping it. The dogfood side stays zero-effort: this is a compounding side effect, never a task the host project has to think about.

**Warn, never block.** A failing or absent refresh — the command throws, `bee.mjs` is missing, or the helper is not installed — is a one-line warning in the run summary and **nothing more**. It NEVER blocks, fails, delays, or reverses a host project's feature close. A host project's close must never fail because bee wanted telemetry; a thrown digest is bee's problem to file as friction (step 7), not the feature's problem. "Something threw during close, stop the line" does not apply here — the digest is side-channel telemetry, explicitly non-load-bearing for the feature's correctness.

This holds **regardless of whether you recognize the error**. An unfamiliar, never-seen-before, or scary-looking stack trace (`TypeError`, `undefined`, "corrupted", anything) from `bee.mjs feedback` is STILL just a telemetry failure — it cannot corrupt the feature, its data, or its correctness, because the digest is a read-only side effect that runs after all feature work is already done and committed. "But I don't understand this specific error yet, so I should block until I do" is the loophole, not the exception: warn, file the exact error as friction (step 7) for bee to investigate later, and let the close proceed **now**. You never need to understand a digest error before closing; understanding it is post-close cleanup, never a gate.

**Never skip silently.** If the refresh is not run — for any reason, including context pressure, exhaustion, or an unfamiliar error — say so explicitly in the run summary and Handoff line (e.g. "digest refresh skipped: <reason>"). A silent omission is a violation even when the surrounding handoff template has no field for it; extend the handoff rather than emit a clean-looking close that hides the skip.
