# Reviewing Reference

Load after `bee-reviewing` is selected. Companion to SKILL.md — flow lives there; prompts, schemas, and checklists live here. Every record on this page lands on a review session (`.bee/reviews/<id>.json`) via `node .bee/bin/bee.mjs reviews record --id <id> --kind ...` — a session exists only after an explicit user request created it (SKILL.md Trigger + Scope).

## Scope Resolution in full

The user owns the review boundary. A request resolves to exactly one of five scope types:

1. the current feature, or a named feature
2. a named list of features/cells
3. everything completed and unreviewed since the last review baseline
4. an explicit range with a stated start and end point
5. everything completed within a stated time window (resolved to an explicit list + immutable diff before dispatch)

If the request does not pin one of these, ask exactly ONE boundary question, then proceed — never ask a second question just to re-confirm permission once the scope is already clear.

**Resolving candidates:** `node .bee/bin/bee.mjs reviews candidates` lists completed-but-unreviewed work; `node .bee/bin/bee.mjs reviews status [--feature F]` reports each candidate's derived coverage label (`unreviewed` / `in review` / `reviewed` / `review stale`). For a batch scope (type 3 or 5), resolve the matching candidates through these verbs, then build ONE cumulative diff spanning all of them, with a mapping from each diff region back to its source feature/cell — reviewers read the cumulative diff once so they can see interaction bugs between changes made together, which is the whole point of batching.

**In-progress work is excluded, never swept in:** any cell that is still `open`/`claimed` is excluded from scope with reason "in progress" and stated to the user. Do not wait for it, do not cap it, do not assume it is done. If the runtime cannot hold a review session and an active feature simultaneously, preserve the active state before entering review and restore it exactly afterward — reviewing must never overwrite active work or drop a handoff.

## Scope Freeze in full

Before any reviewer is dispatched, the scope is frozen:

1. Build the scope JSON: `{ id, requested_by, scope_description, included, excluded, baseline, head }`. Each entry in `included`/`excluded` is `{ type: cell|feature|commit, id, reason? }` — the exact shape `normalizeScopeEntry` in `packages/bee/lib/reviews.mjs` accepts.
2. Create the session: `node .bee/bin/bee.mjs reviews create --file <scope.json>`. This runs the verification preflight over every included behavior-change cell and **fails closed** — non-zero exit, zero files written — when evidence is missing. A failed preflight is a stop: surface the error to the user; never dispatch reviewers to compensate for missing verification. Commit-only scope entries (type 4/5 ranges with no mappable cell) carry nothing to preflight — state that explicitly in the preview below rather than implying the same evidence guarantee cell entries get.
3. Only after `create` succeeds, show the user the preview: covered features/cells, baseline/head, what was excluded and why, the expected reviewer count (core + conditional), the review model/tier or external executor that will run, and a warning if the scope is unusually large or has commit-only entries with no preflighted evidence.
4. Record the reviewer manifest once dispatch is decided: `node .bee/bin/bee.mjs reviews record --id <session-id> --kind manifest --file <manifest.json>` (every `record` call requires `--id`).

Reviewer dispatch is impossible before step 2 succeeds and the preview in step 3 has been shown — nothing in this flow spawns a reviewer against an unfrozen or unpreviewed scope.

## Specialist Dispatch

Isolation contract: each reviewer receives the session's cumulative diff (baseline..head, or the mapped multi-feature diff), `docs/history/<feature>/CONTEXT.md`, and `docs/history/<feature>/plan.md` for every feature in scope — nothing else, never session history. All reviewers run in parallel; the orchestrator synthesizes only after every one has returned (SKILL.md §2 — synthesis is orchestrator work, never a dispatched reviewer). Precedent is already in `plan.md` (planning's bootstrap owns the learnings search).

Common prompt shape:

```text
You are the <X> reviewer. Review only your focus area. Lead with findings.
For each: severity, file/line evidence, failure scenario, smallest credible fix.
Do not rewrite code.
```

Per-reviewer focus lines (append to the shape):

| Reviewer | Focus line |
|---|---|
| `code-quality` | Correctness, readability, type safety, error handling. Cite file/line evidence for every claim. |
| `architecture` | Boundaries, coupling, API design, maintainability, drift from plan.md structure. |
| `security` | Auth, authorization, secrets in code or logs, injection, permissions, data exposure. |
| `test-coverage` | Missing edge cases, regression paths, weak or tautological assertions, untested behavior changes. |

Tiers: specialists = the review slot (SKILL.md §1). Where the runtime cannot select per-agent models, fall back to read budgets and output caps.

Orchestrator synthesis (after all reviewers return): deduplicate overlaps, mark cross-reviewer corroboration (promotes one severity level), attach known-pattern notes from the precedent in `plan.md`, classify each finding's autofix_class, and present counts by severity.

## Conditional Reviewers (selected by diff analysis)

Before dispatch, scan the diff ONCE and spawn any conditional reviewer whose trigger matches, in the same parallel wave as the always-on four. Same isolation contract, same prompt shape, same review slot — only the focus line differs. Personas stay thin lens contracts: no failure-mode catalogs (the model already knows the domain; the trigger and the lens are the value).

| Reviewer | Spawn when the diff touches | Focus line |
|---|---|---|
| `performance` | ORM/query calls inside loops, caching layers, pagination, hot-path data access | Query patterns, N+1 exposure, cache correctness, unbounded result sets. Flag only measurable risks with the triggering code cited. |
| `api-contract` | routes, serializers, public response shapes, exported type signatures, versioned endpoints | Client-visible breaking changes, envelope drift, missing versioning, silent field removals — checked against locked decisions (D-ids). |
| `data-migration` | **spawn gate:** only if the diff includes migration files or schema definitions (`**/migrations/**`, `db/migrate/*`, `schema.*`, `*.sql` DDL) | Destructive DDL, backfills on large tables, NOT NULL without default, irreversibility, deploy-order coupling. |
| `reliability` | retries, timeouts, queues, background jobs, webhooks, external service calls | Failure paths: what happens on timeout, partial failure, replay, and double delivery. Missing idempotency and dead-letter handling. |

Rules:

- Triggers are mechanical — grep the diff's file paths and hunks; do not spawn on vibes, and do not skip a matched trigger to save time.
- Cap the wave at 6 reviewers total (4 core + 2 conditionals). If more triggers match, fold the extra lens into the closest always-on reviewer's focus line and say so in the synthesis.
- A `security` overlap (auth/payments/data-mutation files with ≥50 changed lines) is also the signal for the optional cross-model second opinion at Gate 4 (see 06-runtime-integration.md) — surface the option to the user; never auto-run it.

## Finding Schema

Every distinct issue becomes one finding:

```markdown
### [P<N>] <problem title>   (autofix_class: gated_auto | manual | advisory)

## Plain-Language Summary
<1-3 sentences a non-specialist understands>

## What The Code Does Today
- <current behavior, with source>

## Why This Is A Problem
- <requirement, locked decision (D-id), or invariant broken>

## Concrete Failure Scenario
- <realistic steps and the incorrect outcome>

## Evidence
File: `path`
Line(s): <line>
Snippet: <small relevant snippet>
Why this proves the issue: <one sentence>

## Proposed Fix
Recommended: <smallest credible fix>
Tradeoff: <if any>

## Acceptance Criteria
- [ ] <specific testable condition>
```

Synthesis rules recap: uncertain → P2; independent corroboration promotes one level; disagreement → the more conservative route; `autofix_class` routes work (gated_auto = concrete fix applied after orchestrator judgment; manual = needs design input; advisory = report-only) but never bypasses judgment or the gate.

## Review Cells and Backlog Routing

| Severity | Route | Blocking? |
|---|---|---|
| P1 | fix cell on the current feature (lane tiny/small; verify command required), then re-review the fix | yes — Gate 4 |
| P2 | `.bee/backlog.jsonl` entry; grooming cell if the fix is already concrete | no |
| P3 | `.bee/backlog.jsonl` entry | no |

Backlog entry format (one JSON object per line):

```json
{"ts":"<ISO>","type":"review-finding","feature":"<feature>","severity":"P2","title":"<problem title>","autofix_class":"manual","evidence":"<file:line one-liner>","predicted_impact":"<what it costs if left>","source":"reviewing"}
```

P2/P3 entries carry the feature name for traceability but must NOT be wired as blockers of the current work. If any filing write fails, append the full finding to `docs/history/<feature>/reports/residual-findings.md` — nothing evaporates.

## Session Record Checklist

A review session (`.bee/reviews/<id>.json`) minimally carries these fields — `create` writes the first eight at freeze time (SKILL.md, Scope Freeze and Preview); the rest fill in as the session progresses via `record`:

| Field | Set by | Notes |
|---|---|---|
| `id` | `create` | stable, never reused |
| `requested_by` / `requested_at` | `create` | proves this is a user request, and when |
| `scope_description` | `create` | how the user described the boundary |
| `included` | `create` (frozen) | feature/cell/commit entries actually in scope |
| `excluded` | `create` (frozen) | related work left out, with reason (e.g. "in progress") |
| `baseline` / `head` | `create` (frozen) | the two immutable diff endpoints |
| `reviewer_manifest` | `record --kind manifest` | reviewers, model/tier/executor actually dispatched |
| `verification_preflight` | `create`, then `record --kind preflight` if re-checked | evidence check result before reviewer spend |
| `findings` | `record --kind finding` (append) | severity, evidence, status, fix/re-review reference |
| `uat` | `record --kind uat` (append) | item, pass/fail/skip, skip reason |
| `decision` | `record --kind decision` | `pending`/`blocked`/`approved` + Gate 4 record |

`record` refuses any payload touching `baseline`/`head`/`included`/`excluded` — those four are frozen at `create` and no sub-record kind legitimately needs to touch them. Before creating a new session for a scope that might already be covered, run `node .bee/bin/bee.mjs reviews status` — an unchanged range already reported `reviewed (covered by <id>)` is not re-reviewed.

## Delta Re-Review Protocol

After a P1 fix caps:

1. Re-review the fix delta itself.
2. Sweep the whole scope diff for the finding's defect class — the same category of bug, anywhere else in scope, not just the line that changed.
3. Record the outcome: `node .bee/bin/bee.mjs reviews record --id <session-id> --kind finding --file <finding-update.json>` (status moves to resolved, with the fix's evidence).
4. Decide whether the fix stayed inside its own boundary:
   - **stayed inside** (localized fix, no public-contract change, no destabilized assumption elsewhere in scope) → only the delta + defect-class sweep is required; the full panel does not re-run.
   - **crossed a boundary** (touches another feature's contract, changes a public/API shape, or invalidates an assumption the rest of the scope relied on) → propose an expanded re-review to the user; do not silently pick either the minimal or the maximal option.
5. A session stays `blocked` until every open P1's delta re-review passes.

## Verification-Evidence Gate (behavior_change cells)

For each capped cell with `behavior_change: true`, the trace's `verification_evidence` must name: tests inspected, tests added/changed, red-failure or characterization evidence, the verification run, and any deliberate exception. Missing field, or prose like "covered by existing tests" with no test named → P1 finding; the cell's work goes back.

This is a backstop, not the primary catch: the cap helper already refuses a `behavior_change` cell without a "before" characterization (`red_failure_evidence`, or a `deliberate_exceptions` note for a genuinely new surface), and `bee.mjs reviews create`'s own preflight already fails closed on missing evidence before a session could even exist — so an assertion-capped cell should not reach review at all. If one does, treat it as a double bypass and a P1. Do **not** raise a P1 whose only remedy is "record the missing before-state in a new evidence cell" — that backfill loop is exactly what cap-time and create-time enforcement exist to prevent; a real evidence gap means the behavior was never actually proven, which the worker fixes by re-verifying, not by writing a document. Read evidence from the cell trace — the single source — never from a parallel `reports/*-evidence.*` file.

## Frozen-Judge Flags

Any cell the orchestrator flagged with judge hits — undeclared test/CI/lockfile/verify-config changes (`node .bee/bin/bee.mjs cells judge --id <id>`) — is reviewed assuming the judge was *moved*, not passed: diff each flagged file; verify no assertion weakened, no test skipped or deleted, no verify command softened, no dependency silently repinned. A weakened judge is a P1 (it invalidates the wave's evidence), never a cleanup note.

## Human UAT

For each SEE/CALL/RUN decision in CONTEXT.md:

```text
UAT Item <i>/<n> - Decision <D-id>:
"<deliverable>"
Can you confirm this works? [Pass / Fail / Skip]
```

- Fail → create a P1 fix cell, then rerun this UAT item after the fix caps.
- Skip → record the user's reason in `.bee/state.json` before moving on.
- Intermittent failure is a Fail, not a Skip.

## Finishing Checklist

- [ ] all P1 fix cells capped and their findings re-verified (delta re-review + defect-class sweep)
- [ ] project build/test/lint gates run, fresh output quoted
- [ ] P2/P3 → backlog entries (+ grooming cells where concrete), non-blocking
- [ ] residual-findings fallback written if any filing failed
- [ ] UAT results (and skip reasons) recorded on the session (`record --kind uat`) and in `.bee/state.json` where a skip reason is needed
- [ ] session closeout: `node .bee/bin/bee.mjs reviews record --id <session-id> --kind decision --file decision.json` (`pending`/`blocked`/`approved`) — this closes the SESSION, not a workflow phase; every covered feature already reached its own close via execution → scribing → compounding independently, and that feature state is left untouched. Do not set `next_action: "Invoke bee-compounding."` here — there is no automatic chain hop out of a review session.

## Gate 4 Bypass Mechanics

Gate bypass (`.bee/config.json` `gate_bypass: true`) NEVER creates or auto-approves a review session — a session only ever exists because a user explicitly requested one (SKILL.md Trigger). Once a session already exists and reaches its human UAT/merge question, the bypass carve-out applies: the UAT items are always presented to the human, any P1 finding always stops, and bypass may auto-approve the **merge** question only when P1 = 0 **and** every UAT item was confirmed pass by the human — then record the review gate, log a one-line audit decision, and post a short `⚡ auto-approved merge (bypass)` line instead of asking. Any P1, or any UAT fail/skip, stops Gate 4 for the human as normal. Secret reads during review always require human approval regardless of bypass.

## Lane Scaling in full

No lane auto-runs a reviewer at feature close (zero reviewer tokens spent without a request). `tiny`'s done-report stays entirely inside `bee-swarming`'s single-execution-worker dispatch (the orchestrator authors it from the worker's diff plus its own verify re-run) — that is verification, not independent review, and it never substitutes for a session. Once a session is requested, its panel scales to the SCOPE's own risk, independent of any single feature's lane, per the Lane Scaling table in SKILL.md. A scope containing any high-risk content warrants the full wave regardless of how small the rest of the batch is. None of these depths are ever reduced by gate bypass or by the originating feature having been `tiny`. Everything runs the full review contract **unreduced** — same reviewer count, same models, same severity rules, same UAT obligations — executing over the session's frozen, immutable diff.

## Required Inputs and Delegation

- the review session: `node .bee/bin/bee.mjs reviews show --id <session-id>` (scope, baseline/head, included/excluded)
- `docs/history/<feature>/CONTEXT.md` and `docs/history/<feature>/plan.md` for every feature in scope
- the session's cumulative diff (baseline..head, or the mapped multi-feature diff from Scope Resolution)
- capped cells and traces: `node .bee/bin/bee.mjs cells list --feature <feature>`
- current state: `node .bee/bin/bee.mjs status --json`

Missing CONTEXT.md or plan.md for any feature in scope → stop and return to the stage that owns it. The required-inputs gather, the Verification-Evidence Gate mining, and the Artifact Verification EXISTS/SUBSTANTIVE scan delegate as extraction/generation-tier I/O workers per the Delegation contract (`bee-hive/references/routing-and-contracts.md`); WIRED judgment and severity synthesis stay on the orchestrator.

## Headless in full

`mode:headless` = report-only, and still requires the explicit Trigger before it starts a session at all: run all reviewers, both verification gates, and artifact checks; emit every finding in a structured terminal report with UAT items and ambiguous severities deferred to an `Outstanding Questions` section. Gate 4 still requires the human — headless never self-approves merge, and headless never invents a review request the user didn't make.

## Red Flags

- P1 passed on user silence
- UAT failure logged as pass, or skip without reason
- artifact verification skipped
- synthesis started before every reviewer returned
- P2/P3 blocking the current session
- findings dropped because a write failed (use residual-findings.md)
- a session closeout that sets `next_action: "Invoke bee-compounding."` as if review were a chain stage a feature must pass through
- a new session created for a range `bee.mjs reviews status` already reports `reviewed (covered by <id>)` and unchanged
