---
date: 2026-09-17
feature: pi-statusline-cache-by-model
categories: [failure]
severity: standard
tags: [planning, dispatch, pi]
---

# Learning: Post-cap Routing Must Be Planned

## Learning: A Mandatory Verification Stage Needs a Role-Plan Row

**Category:** failure
**Severity:** standard
**Tags:** [planning, semantic-judge]
**Applicable-when:** A standard or high-risk behavior plan assigns workflow stages to configured roles.

### What Happened

The first approved plan assigned test execution and independent review roles. It omitted the mandatory semantic goal-check stage.

After the cell capped, dispatch preparation refused the judge. The refusal said: `stage "test-and-live-proof" requires role "test", got "review"`.

The plan had to return to planning, unlock, add the review-role stage, and pass the gates again.

### Root Cause

Planning covered implementation stages but did not include the required post-cap verification dispatch.

### Recommendation

For each standard or high-risk behavior plan, add a required `semantic-goal-check` stage with the review role.

## Learning: Pi Escalation Must Be Verified at the Dispatch Payload

**Category:** failure
**Severity:** standard
**Tags:** [dispatch, escalation, pi]
**Applicable-when:** A Pi cell needs the session-model rescue rung after its configured worker fails.

### What Happened

The cell had `escalate: true` after `bee cells escalate`. The next Pi dispatch still requested `gemini-3.8-flash-high` and used `--agent agy-flash`.

The original provider quota remained the active limit. A backlog finding now tracks this mismatch.

### Root Cause

The escalation flag and the prepared Pi herding payload did not select different execution resources.

### Recommendation

After Pi escalation, inspect the prepared payload. Do not claim a session-model rescue unless its model or transport changed.
