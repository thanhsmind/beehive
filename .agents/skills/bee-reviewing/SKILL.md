---
name: bee-reviewing
description: >-
  Run the multi-agent review gate — severity findings, artifact verification, and user acceptance — over an immutable scope the user explicitly asked to review. Use only when the user requests an independent review: "review this", "review today's work", "review feature A and B", "review the diff from X to Y", "review everything unreviewed before release". A finished cell, slice, or feature is never a trigger by itself, and neither is "merge"/"ship"/"release" alone.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: degraded
      reason: Reads bee records (cells, state, backlog, reviews) via the vendored .bee/bin helpers.
---

# Reviewing (inspector bees)

Independent inspection session over a completed, immutable scope — the scrutiny a second team gives a pull request. Runs only on explicit request, never automatic. A cell can be verified and still `unreviewed`. Rules stated bare: `references/provenance.md`.

## Trigger — explicit user intent only

- "review this / review this feature"
- "review all of today's work"
- "review feature A and B" (or any named list)
- "review the diff from X to Y"
- "review everything unreviewed before release"

Never a trigger: a cell/slice/feature/day finishing · "merge"/"ship"/"release" alone (report count+risk via `reviews status`, ask exactly ONE yes/no question, silence stays `unreviewed`) · gate bypass being on.

## Flow

| Step | What happens |
|---|---|
| Scope | user-owned boundary: named feature(s), unreviewed-since-baseline, explicit range, or time window; ambiguous → one question |
| Freeze | scope JSON → `reviews create` (fails closed on missing evidence) → preview → record manifest; no dispatch before this |
| Dispatch (§1) | 4 core + matched conditional reviewers, isolated context, review-tier model |
| Synthesis (§2) | orchestrator only, after every reviewer returns |
| Evidence gates (§3-4) | verification-evidence + frozen-judge + artifact checks |
| UAT (§5) | every SEE/CALL/RUN decision, with the human |
| Delta re-review (§6) | P1 fix caps → re-review delta + sweep defect class |
| Finish (§7) | build/test/lint gates, file P2/P3, close session |

Full mechanics + required inputs/delegation: `references/reviewing-reference.md` ("Scope Resolution", "Scope Freeze", "Required Inputs").

## Lane Scaling

| Scope risk | Review | Gate 4 |
|---|---|---|
| small | 1 correctness reviewer, isolated context | asked normally |
| standard | 4 core reviewers | asked normally |
| high-risk content (auth, authz, audit/security, migration, data loss, external provider) | full wave + conditionals, cap 6 | asked normally, UAT always |

Never reduced by bypass or the feature's lane; `tiny`'s clean self-review stays inside `bee-swarming`'s done-report, never a session — reference ("Lane Scaling").

## 1-2. Review & Synthesis

Spawn every reviewer as the default subagent type + inline persona, never another plugin's type. Core four (parallel, review-tier, default `opus`): `code-quality`, `architecture`, `security`, `test-coverage`. Conditional (`performance`, `api-contract`, `data-migration`, `reliability`) join on a matched trigger; cap 6 total. Reference: ("Specialist Dispatch", "Conditional Reviewers").

- **P1** — security breach, data loss, breaking change, production blocker. Blocks approval.
- **P2** — real performance, architecture, reliability, or important test gap.
- **P3** — cleanup, docs, future debt.

Orchestrator synthesizes only after every reviewer returns: uncertain → P2, corroboration promotes one level, disagreement takes the conservative route. `autofix_class` routes work, never bypasses judgment. Schema: reference ("Finding Schema").

## 3-4. Evidence Gates

Every capped `behavior_change: true` cell: inspect `verification_evidence` in the trace, never a parallel file. Missing/vague evidence is a P1 — no backfill doc. Frozen-judge flags (`cells judge --id <id>`) assume the judge moved, not passed — always P1. Detail: reference ("Verification-Evidence Gate", "Frozen-Judge Flags").

Artifact check: EXISTS+SUBSTANTIVE+WIRED=OK; EXISTS+SUBSTANTIVE only=P2; missing/EXISTS-only=P1.

## 5-6. UAT, Delta Re-Review

Walk every SEE/CALL/RUN decision with the human; fail → P1 fix cell + rerun; a skip needs a recorded reason; intermittent failure is a Fail, not a Skip. Record: `reviews record --kind uat`. Wording: reference ("Human UAT").

After a P1 fix caps: re-review the delta AND sweep the scope for the defect class, not just the changed line. A localized fix needs only delta+sweep; a boundary-crossing fix gets an expanded re-review proposed to the user. Record: `reviews record --kind finding`. Protocol: reference ("Delta Re-Review").

## 7. Finishing

1. Run build/test/lint gates; quote fresh output — never claim "passing" without it.
2. P2/P3 → `backlog add --type review-finding --severity P2|P3 --layer <layer> --title "<finding>" --feature <feature>` (+ grooming cell if concrete), non-blocking.
3. Filing fails anywhere → write to `docs/history/<feature>/reports/residual-findings.md`.
4. Close: `reviews record --kind decision --file decision.json` (`pending`/`blocked`/`approved`). Closes the REVIEW, not any feature — every feature already closed independently; leave that state untouched, never `state set --phase ...` here.

Checklist: reference ("Finishing Checklist").

## Gate 4 (wording fixed) — lives only inside a session

Exists ONLY inside a review session — never after a feature merely finishes. Plain-language layer first (built / found / consequence of merging now / the decision), findings linked from `docs/history/<feature>/reports/`, never pasted as a table. Then verbatim:

- P1 > 0 → "P1 findings block merge. Fix before proceeding?"
- P1 = 0 → "Review complete. Approve merge?"

Never continue past open P1s without explicit acknowledgment — silence isn't acknowledgment; session stays `blocked` until every P1's fix + delta re-review pass. `tiny` exception: a clean self-review's Gate 4 is `bee-swarming`'s done-report — no merge question, never a review session.

Bypass never covers session creation or approval — only an explicit request creates one. Once one exists, bypass may auto-approve only the merge question when P1 = 0 and every UAT item passed; any P1/UAT fail or skip stops Gate 4 normally, secret reads always need human approval. Mechanics: reference ("Gate 4 Bypass Mechanics").

## Headless

Report-only; still requires the explicit Trigger. Gate 4 still requires the human — never self-approves merge or invents a request. Full text: reference ("Headless").

## Red Flags

full wave for a small scope · reviewer dispatched before `reviews create`+preview · session or Gate 4 approved by bypass · finished work or bare merge/ship/release as a trigger · a defect waved through as "the fast path" · past a P1 with no acknowledgment · UAT passed with no confirm, or a skip with no reason · artifact check skipped because "cells are capped" · `behavior_change` accepted on vague evidence · synthesis before every reviewer returned · P2/P3 filed as blocking · full panel re-run inside an unchanged boundary, or a boundary-crossing fix re-reviewed only at the delta · reviewer given session history · reviewer spawned as another plugin's agent type · "should work" as evidence · re-dispatching an already-`reviewed`, unchanged range

Violating the letter of these rules is violating the spirit of these rules.

## Handoff

Record the decision and close the session — closes the REVIEW, not the feature; every feature already closed independently. `standard`/`high-risk`: `bee-briefing` walkthrough mode writes `walkthrough.md` per feature. A P1 fix settling new behavior triggers `bee-scribing` (AGENTS.md rule 8): a settled decision, not a chain hop.

| Reference | When to Load |
|---|---|
| `references/reviewing-reference.md` | Scope mechanics, specialist prompts, session-record schema, UAT wording, delta re-review + bypass |
| `references/provenance.md` | Decision IDs + rationale per body rule |
