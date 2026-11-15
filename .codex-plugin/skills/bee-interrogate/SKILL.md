---
name: bee-interrogate
description: >-
  Run independent reviewers adversarially over a change, then judge their findings as the lead and return one verdict, applying nothing. Use when the user types /bee-interrogate or asks to interrogate, stress-test or tear apart a change. Not for the review gate (bee-reviewing).
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: The reviewers go out through the vendored bee binary's dispatch door. Absent, the leader reviews the change itself, one lens after the other, and the account says so.
---

# Interrogate

Spawn several reviewers to adversarially review a change. Every reviewer gets the same prompt and rubric. The adversarial signal comes from independence, not assigned personas. Models differ in blind spots, priors, and reasoning patterns. Agreement across reviewers on different models is high-confidence signal; lone-reviewer findings are worth reading but lower confidence.

The deliverable is a synthesized verdict. Do NOT auto-apply changes. This is advice before a gate: the review gate and its record stay `bee-reviewing`'s.

## Step 1, Determine Scope

Identify what to review from context:

- If the user points at specific files or a diff, use that
- If on a feature branch, run `git diff main...HEAD` (or the appropriate base branch) for the full changeset
- If the user's message references recent work, gather the relevant files

Package the diff (or file contents) plus any surrounding context files the reviewers need to understand the code.

## Step 2, State the Intent

Before dispatching reviewers, state the intent explicitly. What is this code trying to accomplish? Derive this from:

- The user's message
- Commit messages
- The pull request description if one exists
- The code itself

Write one clear paragraph. Reviewers challenge whether the work achieves the intent well, not whether the intent itself is correct. If you're unsure about the intent, ask the user before proceeding.

## Step 3, Dispatch Reviewers

Issue one `bee dispatch prepare --runtime claude --kind reviewer --json` per reviewer, all in the same turn, with no `--role`, and run every returned payload in the background. Use each payload exactly as it returns. Never pick a model or a subagent type by hand. Which model a reviewer runs on is a setting of the review role in the bee team table, never a choice made here.

Three reviewers is the default. A small change takes two. Label them Reviewer A, B, C and so on.

Read `references/reviewer-prompt.md` and fill in the template with:
1. The stated intent
2. The diff or file contents
3. The review rubric from `references/rubric.md`
4. The code-quality lens from `references/code-quality-review.md`

The same filled template goes to every reviewer, so every reviewer applies the code-quality lens. The brief carries the artifact, the constraints and the open question, never your own verdict, leaning or expected findings.

When a payload names the same model for every reviewer, the verdict says so: reviewers on one model share its blind spots, and their agreement is not independent corroboration.

Each reviewer produces structured findings as described in the prompt template, with severity in `bee-reviewing`'s vocabulary:

- **P1**: security breach, data loss, breaking change, production blocker. Blocks approval.
- **P2**: real performance, architecture, reliability, or test gap.
- **P3**: cleanup, docs, future debt.

A return that breaks the finding schema or cites a path that does not resolve gets one re-dispatch with the failure named. A second failure is listed as an uncovered reviewer in the verdict.

## Step 4, Synthesize

As results come back, build a unified picture:

1. **Parse all findings** from the reviewers
2. **Identify consensus**. Findings raised by 2+ reviewers independently are highest signal.
3. **Identify lone-reviewer findings**. Still worth reading, but weight accordingly.
4. **Deduplicate**. Different reviewers may describe the same issue differently. Merge these and note which reviewers raised it.
5. **Note disagreements**. If one reviewer flags something and another explicitly says the opposite, that's useful context for the verdict.

## Step 5, Lead Judgment

You are the lead reviewer, a pragmatic senior engineer, not a neutral aggregator.

Read `references/lead-judgment.md` for the full framework. Reviewers only see a slice of the codebase. You have the full context (the goal, the constraints, the timeline, which tradeoffs were already considered). Use that context aggressively. A finding about behavior reaches the verdict only after you reproduced it or traced it.

Categorize every finding using these buckets:

- **Act on**. Real issues affecting correctness, security, or maintainability given the actual goals. These would block a real merge. Every surviving P1 is here and blocks approval.
- **Consider**. Legitimate points, but you're not sure they outweigh the cost of addressing them right now. Worth the user's attention.
- **Noted**. Technically valid but not actionable. Context-dependent, premature optimization, or low-impact given the current stage.
- **Dismissed**. Wrong, nitpicky, or missing context. Brief explanation why, and what was run or traced.

For each finding, include:
- Which reviewer(s) raised it
- The severity (P1, P2 or P3) and the category (act on / consider / noted / dismissed)
- A one-line rationale for the categorization

## Output Format

Present the verdict in this structure:

### Intent
> [The stated intent paragraph from Step 2]

### Reviewers
- Reviewer [label]: [model name the payload named, or "not named"], [N findings] (one bullet per reviewer)

### Act On
[Findings that should be addressed. For each: description, severity, which reviewers raised it, why it matters. Say plainly when a P1 blocks approval.]

### Consider
[Findings worth thinking about. For each: description, severity, which reviewers raised it, tradeoff involved.]

### Noted
[Valid but low-priority. Brief list.]

### Dismissed
[Rejected findings with brief rationale. This shows the user what was filtered out and why, so they can override your judgment if they disagree.]

### Agreement Map
[Where did reviewers agree, where did they diverge, and what does the pattern of agreement/disagreement tell us? Say whether the reviewers ran on one model or several.]

## Handoff

Verdict delivered, nothing applied. Invoke bee-hive skill.
