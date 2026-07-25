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

Reviewing is an independent inspection session over a completed, immutable scope — the same kind of scrutiny a second team gives a pull request or a release candidate. It is not an automatic stage every feature passes through; it runs only when the user asks for it (decision `565e68d0-327f-404e-b49e-d1c61ba81bfd`).

Reviewing is not verification. Verification (bee-executing's cap rules: a real verify command, recorded output, `verification_evidence` for behavior-change cells) is mandatory for every cell and proves a completed unit of work meets its locked requirement. It happens with or without a review session and it is NOT this skill. Review status is independent of implementation status: a change can be `completed`/verified and simultaneously `unreviewed`. Cell closure or feature closure is not proof the feature has been reviewed.

## Trigger — explicit user intent only

Dispatch this skill only when the user names one of these intents (R1):

- "review this / review this feature"
- "review all of today's work"
- "review feature A and B" (or any named list)
- "review the diff from X to Y"
- "review everything unreviewed before release"

None of the following are triggers, no matter how tempting the alignment feels:

- a cell, slice, feature, or working day finishing — verification completing is not a review request
- the words "merge", "ship", or "release" on their own (7.4/A9): when the user asks to merge/ship/release while unreviewed or stale work exists, report the count and risk level (`node .bee/bin/bee.mjs reviews status`), then ask exactly ONE question — does the user want a review session for that scope? Only an explicit yes starts a session; silence or a non-answer means no dispatch, and the work stays labeled `unreviewed` — never described as review-approved.
- gate bypass being on — bypass never creates or auto-approves a review session (R8)

## Scope Resolution

The user owns the review boundary (R4). A request resolves to exactly one of five scope types:

1. the current feature, or a named feature
2. a named list of features/cells
3. everything completed and unreviewed since the last review baseline
4. an explicit range with a stated start and end point
5. everything completed within a stated time window (resolved to an explicit list + immutable diff before dispatch)

If the request does not pin one of these, ask exactly ONE boundary question, then proceed — never ask a second question just to re-confirm permission once the scope is already clear.

**Resolving candidates:** `node .bee/bin/bee.mjs reviews candidates` lists completed-but-unreviewed work; `node .bee/bin/bee.mjs reviews status [--feature F]` reports each candidate's derived coverage label (`unreviewed` / `in review` / `reviewed` / `review stale`). For a batch scope (type 3 or 5), resolve the matching candidates through these verbs, then build ONE cumulative diff spanning all of them, with a mapping from each diff region back to its source feature/cell (7.3) — reviewers read the cumulative diff once so they can see interaction bugs between changes made together, which is the whole point of batching.

**In-progress work is excluded, never swept in:** any cell that is still `open`/`claimed` is excluded from scope with reason "in progress" and stated to the user (A6). Do not wait for it, do not cap it, do not assume it is done. If the runtime cannot hold a review session and an active feature simultaneously, preserve the active state before entering review and restore it exactly afterward (7.5) — reviewing must never overwrite active work or drop a handoff.

## Scope Freeze and Preview

Before any reviewer is dispatched, the scope is frozen (R5):

1. Build the scope JSON: `{ id, requested_by, scope_description, included, excluded, baseline, head }`. Each entry in `included`/`excluded` is `{ type: cell|feature|commit, id, reason? }` — the exact shape `normalizeScopeEntry` in `packages/bee/lib/reviews.mjs` accepts.
2. Create the session: `node .bee/bin/bee.mjs reviews create --file <scope.json>`. This runs the verification preflight over every included behavior-change cell and **fails closed** — non-zero exit, zero files written — when evidence is missing (A10). A failed preflight is a stop: surface the error to the user; never dispatch reviewers to compensate for missing verification. Commit-only scope entries (type 4/5 ranges with no mappable cell) carry nothing to preflight — state that explicitly in the preview below rather than implying the same evidence guarantee A10 gives cell entries.
3. Only after `create` succeeds, show the user the preview: covered features/cells, baseline/head, what was excluded and why, the expected reviewer count (core + conditional), the review model/tier or external executor that will run, and a warning if the scope is unusually large or has commit-only entries with no preflighted evidence.
4. Record the reviewer manifest once dispatch is decided: `node .bee/bin/bee.mjs reviews record --id <session-id> --kind manifest --file <manifest.json>` (every `record` call requires `--id`).

Reviewer dispatch is impossible before step 2 succeeds and the preview in step 3 has been shown — nothing in this flow spawns a reviewer against an unfrozen or unpreviewed scope.

## Lane Scaling — the SESSION's scope sets review depth, not the originating feature's lane

No lane auto-runs a reviewer at feature close (goal 1: zero reviewer tokens spent without a request). `tiny`'s done-report stays entirely inside `bee-swarming`'s single-execution-worker dispatch (the orchestrator authors it from the worker's diff plus its own verify re-run, AO14) — that is verification, not independent review, and it never substitutes for a session. Once a session is requested, its panel scales to the SCOPE's own risk, independent of any single feature's lane:

| Scope risk | Review | Gate 4 |
|---|---|---|
| small scope (single small change, low blast radius) | 1 correctness reviewer (review slot, isolated context: cumulative diff + CONTEXT.md/plan.md only) | asked normally |
| standard scope | 4 core reviewers (§1 table) | asked normally |
| scope with high-risk content (auth, authorization, audit/security, migration, data loss, external provider) | full wave + conditional reviewers, cap 6 | asked normally, UAT always |

A scope containing any high-risk content warrants the full wave regardless of how small the rest of the batch is. None of these depths are ever reduced by gate bypass or by the originating feature having been `tiny`.

Everything below runs the pre-existing full-review contract **unreduced** — same reviewer count, same models, same severity rules, same UAT obligations (goal 5) — it now simply executes over the session's frozen, immutable diff instead of an ad hoc "final slice" diff.

## Required Inputs

- the review session: `node .bee/bin/bee.mjs reviews show --id <session-id>` (scope, baseline/head, included/excluded)
- `docs/history/<feature>/CONTEXT.md` and `docs/history/<feature>/plan.md` for every feature in scope
- the session's cumulative diff (baseline..head, or the mapped multi-feature diff from Scope Resolution)
- capped cells and traces: `node .bee/bin/bee.mjs cells list --feature <feature>`
- current state: `node .bee/bin/bee.mjs status --json`

Missing CONTEXT.md or plan.md for any feature in scope → stop and return to the stage that owns it.

**Delegation:** the required-inputs gather, §3 evidence-gate mining, and the §4 artifact EXISTS/SUBSTANTIVE scan delegate as extraction/generation-tier I/O workers per the Delegation contract (D2/D3, `bee-hive/references/routing-and-contracts.md`); WIRED judgment and severity synthesis stay on the orchestrator.

## 1. Specialist Review

Dispatch reviewers with ISOLATED context: the session's cumulative diff + CONTEXT.md + plan.md ONLY. Never session history.

**Spawn contract:** spawn every reviewer as the runtime's default/general subagent type with the persona prompt from the reference pasted inline. NEVER use an agent type registered by another plugin, even when its name matches the role (`*-correctness-reviewer`, `*-security-reviewer`, …) — a same-named agent carries a different contract (finding format, severity scale, report paths), silently breaks bee's synthesis rules, and makes the run depend on which plugins happen to be installed on this machine.

| Reviewer | Focus | Slot | Order |
|---|---|---|---|
| `code-quality` | correctness, readability, type safety | review | parallel |
| `architecture` | boundaries, coupling, API design, maintainability | review | parallel |
| `security` | auth, secrets, injection, permissions, data exposure | review | parallel |
| `test-coverage` | missing edge cases, regression paths, weak assertions | review | parallel |

Precedent arrives pre-loaded: planning's bootstrap owns the `docs/history/learnings/` search, and its hits land in `plan.md`, which every reviewer receives — no review-time precedent agent exists. Synthesis (§2) is the orchestrator's own work after all reviewers return, never a dispatched reviewer.

**The `review` slot (P16, decision 0021):** reviewers resolve `resolveTier(root, 'review', runtime)` — a dedicated, per-repo-editable model for review work, default `opus` on Claude (independent reviewer > self-review: the model that reviews should not be the model that implemented). A `null` review slot falls back to `generation`; a `{kind:'cli'}` value dispatches an external adversarial reviewer (e.g. GPT via codex CLI) as a **read-only gather**, resolved with the purpose-scoped 4-arg form `resolveTier(root, 'review', runtime, {for:'gather'})` through the Delegation contract's cli gather branch (`bee-hive/references/routing-and-contracts.md`) — a bare 3-arg resolve of a cli-shaped review slot now refuses (AO12/B1, plan 2A-ii). Conditional reviewers (below) use the same slot.

**Conditional reviewers** join the same parallel wave when the diff mechanically matches their trigger: `performance` (queries in loops, caching), `api-contract` (routes, public shapes), `data-migration` (spawn gate: migration/schema files only), `reliability` (retries, queues, external calls). Scan the diff once before dispatch; spawn every matched trigger; cap the wave at 6 (4 core + 2 conditionals — the cap tracks the roster). Trigger table and focus lines in `references/reviewing-reference.md`.

Full prompts in `references/reviewing-reference.md`.

## 2. Severity and Synthesis

- **P1** — security breach, data loss, breaking change, production blocker. Blocks session approval.
- **P2** — real performance, architecture, reliability, or important test gap.
- **P3** — cleanup, docs, future debt.

The orchestrator performs synthesis itself, only after every reviewer has returned — the old synthesis agent ran on the orchestrator's own model anyway, so dispatching it added a hop, not a mind.

Rules: uncertain → P2. Reviewers score independently; corroboration across independent reviewers promotes a finding one level. On disagreement, take the more conservative route. Every finding carries an `autofix_class` — `gated_auto` (concrete fix, apply after judgment), `manual` (needs design input), `advisory` (report-only) — as a routing SIGNAL, never an apply gate.

Finding format, in this order: plain-language summary → what the code does today → why it matters → concrete failure scenario → file/line evidence → smallest credible fix. Schema in the reference. Record every finding to the session: `node .bee/bin/bee.mjs reviews record --id <session-id> --kind finding --file <finding.json>`.

## 3. Verification-Evidence Gate

For every capped cell in scope with `behavior_change: true`, inspect the recorded `verification_evidence` in the cell trace. Missing or vague evidence ("tests pass", "should be covered") is itself a P1 finding — the work goes back; it does not pass forward.

This is now a **backstop, not the primary catch** (decision 0009): the cap helper already refuses a `behavior_change` cell without a "before" characterization (`red_failure_evidence`, or a `deliberate_exceptions` note for a genuinely new surface), and `bee.mjs reviews create`'s own preflight (Scope Freeze and Preview, step 2) already fails closed on missing evidence before this session could even exist — so an assertion-capped cell should not reach review at all. If one does, treat it as a double bypass and a P1. Do **not** raise a P1 whose only remedy is "record the missing before-state in a new evidence cell" — that backfill loop is exactly what cap-time and create-time enforcement exist to prevent; a real evidence gap means the behavior was never actually proven, which the worker fixes by re-verifying, not by writing a document. Read evidence from the cell trace — the single source — never from a parallel `reports/*-evidence.*` file.

**Frozen-judge flags (P12, decision 0018):** any cell the orchestrator flagged with judge hits — undeclared test/CI/lockfile/verify-config changes (`node .bee/bin/bee.mjs cells judge --id <id>`) — is reviewed assuming the judge was *moved*, not passed: diff each flagged file; verify no assertion weakened, no test skipped or deleted, no verify command softened, no dependency silently repinned. A weakened judge is a P1 (it invalidates the wave's evidence), never a cleanup note.

## 4. Artifact Verification

For everything CONTEXT.md and plan.md promised across every feature in scope, verify three levels:

- **EXISTS** — the artifact is present
- **SUBSTANTIVE** — not a stub, placeholder, TODO-only, fake static path, or empty handler
- **WIRED** — imported and used on the integration path

All three = OK. EXISTS + SUBSTANTIVE only = P2. Missing or EXISTS-only = P1.

## 5. Human UAT

Walk the user through every SEE/CALL/RUN decision in CONTEXT.md, for every feature in scope (wording in the reference). Failure → P1 fix cell + rerun the item. A skip requires its reason in the UAT item itself; record that session-local outcome with `node .bee/bin/bee.mjs reviews record --id <session-id> --kind uat --file <uat-item.json>`. Independent review owns no active routing state and never calls generic `state set`. UAT failures are never logged as passes.

## 6. Delta Re-Review (fix protocol, R9/A12)

After a P1 fix is capped:

1. Re-review the fix delta AND sweep the whole scope diff for the finding's defect class — not just the line that changed (critical pattern 20260711: grill deltas).
2. Record the resolution to the session: `node .bee/bin/bee.mjs reviews record --id <session-id> --kind finding --file <finding-update.json>`.
3. Do not re-run the full panel for the whole batch unless the fix crosses a scope boundary, changes a public contract, or destabilizes an assumption the rest of the scope relied on. When it does, propose the expanded re-review to the user rather than silently choosing either the minimal or the maximal option.
4. A concrete, localized P1 fix that stays inside its own boundary only needs its own delta re-reviewed and its defect class swept (A12) — it does not force a full-panel re-run for content that never changed.

## 7. Finishing

1. Run the project build/test/lint gates; quote fresh command output — never claim "passing" without it.
2. P2/P3 findings → `node .bee/bin/bee.mjs backlog add --type review-finding --severity P2|P3 --layer <layer> --title "<finding>" --feature <feature>` (plus grooming cells where warranted) with non-blocking traceability to the feature(s) in scope. They never block the current session.
3. If filing a residual finding anywhere fails, write it to `docs/history/<feature>/reports/residual-findings.md` so nothing evaporates.
4. Close the session: `node .bee/bin/bee.mjs reviews record --id <session-id> --kind decision --file decision.json` (status `pending`, `blocked`, or `approved`). This closes the REVIEW, not any feature — every feature in scope already reached its own close through execution → scribing → compounding independently (§11.1), and session closeout leaves that feature state untouched (7.5). Do not run `bee.mjs state set --phase ...` as if a review were a workflow phase transition for the covered features.

## Gate 4 (wording is fixed) — lives only inside a session

Gate 4 exists ONLY inside a review session (R8) — there is no "Gate 4" after a feature merely finishes execution, and no empty/automatic Gate 4. Present per the Gate Presentation Contract (bee-hive routing reference): plain-language layer in chat — what was built / what review found in plain words / consequence of merging now / what you are deciding — in the user's language, with full findings linked from `docs/history/<feature>/reports/`, never pasted as a findings table. Then verbatim:

- P1 > 0 → "P1 findings block merge. Fix before proceeding?"
- P1 = 0 → "Review complete. Approve merge?"

Never continue past open P1s without explicit user acknowledgment. Silence is not acknowledgment. A session stays `blocked` (A11) until every P1's fix and delta re-review (§6) pass.

`tiny` lane exception (Lane Scaling table): with a clean self-review, Gate 4 is the done-report inside `bee-swarming` — no merge question there, and that done-report is never itself an independent-review session.

**Gate bypass never covers session creation or approval (R8, decision 0010 boundary).** `.bee/config.json` `gate_bypass: true` NEVER creates or auto-approves a review session — a session only ever exists because a user explicitly requested one (Trigger, above). Once a session already exists and reaches its human UAT/merge question, the pre-existing bypass carve-out still applies unchanged: the §5 UAT items are always presented to the human, any P1 finding always stops, and bypass may auto-approve the **merge** question only when P1 = 0 **and** every UAT item was confirmed pass by the human — then record the review gate, log a one-line audit decision, and post a short `⚡ auto-approved merge (bypass)` line instead of asking. Any P1, or any UAT fail/skip, stops Gate 4 for the human as normal. Secret reads during review always require human approval regardless of bypass.

**No re-dispatch for an unchanged, already-approved range (R6/A7):** before creating a new session, check `node .bee/bin/bee.mjs reviews status` — a candidate already reporting `reviewed (covered by <review-id>)` for an unchanged range is not re-reviewed; only genuinely new or `review stale` delta gets a new session, unless the user explicitly asks for a re-review.

## Headless

`mode:headless` = report-only, and still requires the explicit Trigger before it starts a session at all: run all reviewers, both verification gates, and artifact checks; emit every finding in a structured terminal report with UAT items and ambiguous severities deferred to an `Outstanding Questions` section. Gate 4 still requires the human — headless never self-approves merge, and headless never invents a review request the user didn't make.

## Red Flags

- a full reviewer wave spawned for a small/single-change scope (Lane Scaling: small scope = one correctness reviewer)
- a reviewer dispatched before `bee.mjs reviews create` succeeded and the scope preview was shown
- a session created, or Gate 4 auto-approved, by gate bypass (bypass never creates or approves sessions)
- a finished cell/slice/feature, or the words "merge"/"ship"/"release" alone, treated as a review trigger
- a tiny defect waved through because "it's just the fast path" — the fast path never ships a known defect
- continuing past a P1 without explicit user acknowledgment
- UAT failure marked pass, or a skip without a recorded reason
- artifact verification skipped because "the cells are capped"
- a `behavior_change` cell accepted with vague verification evidence
- synthesis started before every reviewer returned
- P2/P3 filed as blocking work on the current session
- a full panel re-run for a scope that did not cross a boundary (§6) — or, conversely, a boundary-crossing fix re-reviewed only at the delta
- a reviewer dispatched with session history in its context
- a reviewer spawned as another plugin's registered agent type instead of the default type + inline persona
- "should work" accepted as evidence
- re-dispatching a full panel for a range `status` already reports as `reviewed (covered by <id>)` and unchanged

Violating the letter of these rules is violating the spirit of these rules.

## Handoff

Session complete: record the decision (`record --id <id> --kind decision --file decision.json`) and close the session — this closes the REVIEW, not the feature. Every feature in scope already went through its own execution → scribing → compounding close independently (§11.1); a review session is never a precondition for that chain, and closing one does not re-trigger it. For `standard`/`high-risk` scope, invoke `bee-briefing` in walkthrough mode to write `docs/history/<feature>/walkthrough.md` per feature in scope, as an audit artifact of what the session found. If a P1 fix inside the session settled new behavior worth documenting, that triggers `bee-scribing` under its own standing self-triggering rule (AGENTS.md rule 9) — because a decision settled, not as an automatic hop from this skill.

| Reference | When to Load |
|---|---|
| `references/reviewing-reference.md` | specialist prompts, finding/session-record schema, UAT wording, session-record checklist, delta re-review protocol |
