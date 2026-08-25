# 03 — Workflow Contract

The bee chain, stage by stage: what each skill reads, writes, and must never do. This is the normative spec that the SKILL.md files implement.

## The chain and the three gates

```
bee-hive
  -> bee-shaping (Explore → Lock)           [GATE 1] approve CONTEXT.md
  -> bee-planning (shape + reality check + review wave)
  -> bee-shaping (Brief: render implement-plan)  [GATE 2] approve shape AND execution together (reviews the brief)
  -> bee-planning (current-work prep) + bee-shaping (Brief refresh)
  -> bee-swarming (orchestrator + Execute workers × N)
  -> more approved work remains? -> back to planning for the next slice
  -> bee-capturing (Scribe)                 (BA spec sync — state layer; feature closes unreviewed)
  -> bee-capturing (Compound)               (candidate report: verified/unreviewed/in review/reviewed/review stale)
  (on demand, any time) bee-capturing       capture a settled outcome; harvest a legacy area
  (on demand, any time the hive is idle) bee-grooming

  on user request, any time: bee-reviewing  [GATE 3] independent review over a user-chosen scope; P1s block; else approve merge
    -> bee-shaping (Brief: walkthrough)     (standard/high-risk: walkthrough.md, implement-plan status → Shipped)
```

Independent review is never a default pipeline stage (decision 565e68d0): execution always closes through scribing and compounding, verified but `unreviewed`, and further development is not blocked on it. `bee-reviewing` runs only when the user explicitly asks, over a scope of their choosing — this feature, a named batch, or a commit range.

There is no standalone validating stage and no `validating` phase (validation-diet D1/D3): its one reality-check survivor (SMALLER PATH) and its review wave folded into `bee-planning`, the feasibility matrix and delta rule were deleted with no replacement, and its old standalone execution gate merged into Gate 2 (validation-diet D2, D5, D6).

Gate wording (fixed, from khuym):

- **Gate 1:** "Decisions locked. Approve CONTEXT.md before planning?"
- **Gate 2:** "Work shape is ready. Approve before current-work preparation?" — now also the execution approval: the old standalone execution gate ("Feasibility validated. Approve execution?") folded into this same gate, flipping `approved_gates.shape` and `approved_gates.execution` together (`bee state gate --merge`; validation-diet D2).
- **Gate 3:** P1 > 0 → "P1 findings block merge. Fix before proceeding?" ; P1 = 0 → "Review complete. Approve merge?" — asked only inside a user-invoked `bee-reviewing` session (SPEC R8, decision 565e68d0), never automatically after any lane's execution completes.

Lane exceptions (lane scaling v2, decision d02a6bc6): the `docs` lane has no gates; every lane merges the old shape and execution approvals into one question — tiny/small ask it inline before cells persist, standard/high-risk ask it once shape and the brief are ready. Every lane — `tiny` through `high-risk` — closes through Gates 1-2 by default and ends `unreviewed`; Gate 3 is never part of that default chain for any lane, it exists only inside an on-demand review session. A defect found in any review session still stops for the human.

**Gate Presentation Contract** (owner feedback, dogfood): a gate is presented in two layers. The chat message is the plain-language layer only, in the user's language — *what I'm about to do / why it's trustworthy / if it goes wrong / what you are deciding* — followed by the fixed question. The full mechanical material (reality-gate tables, matrices, plan-checker findings, cell lists) goes to `docs/history/<feature>/reports/` and is linked, never pasted. Litmus: the user can restate what they are approving in their own words — a gate the user cannot restate is a dead gate that manufactures false confidence. Normative text in `bee-hive/references/routing-and-contracts.md`.

Optional at Gates 2 and 3: a **cross-model second opinion** (gstack). If the other runtime's model is available, ask it to challenge the artifact. Agreement → mention it. Disagreement → quote both positions to the user. Never auto-resolve.

## Priority rules (hive law)

1. P1 review findings always block.
2. Context budget always applies; at ~65%, write `.bee/HANDOFF.json` and pause.
3. `CONTEXT.md` is the source of truth; locked decisions are cited, never reinterpreted.
4. Gate 2's execution approval (the old standalone execution gate, now folded in) is the critical one; no source-editing execution before it.
5. A failed reality gate or a NO spike halts the pipeline and returns to planning.
6. Never skip the reality check (SMALLER PATH) — including in tiny mode (it collapses to a one-line inline question, it does not disappear).
7. `docs/history/learnings/critical-patterns.md` and recent active decisions (`bee decisions active --recent 3`) are mandatory context before planning or executing.
8. Evidence before claims: any "done/passing/fixed" statement requires fresh command output in the same message.
9. Lanes scale ceremony, never memory: a capped `behavior_change` cell obliges a `bee-capturing` (Scribe) sync in every lane, and a settled discussion outcome is captured the moment it settles (vision principle 11). An explicit user settlement signal — "chốt", "final", "ok ship it" — is a mandatory same-turn capture trigger, never deferred to feature close (decision 0003). What same-turn capture *costs* is lane-scaled (decision 0017): `high-risk` = full spec sync inline; every other lane = decision log + a one-line stub in `.bee/capture-queue.jsonl`, with the full merge at a flush point (wrap-up, PreCompact warning, next session's offer) — durability is never deferred, only elaboration.
10. Critique passes (fresh-eyes review in exploring, the merged reviewer's review wave in planning) run in the background where the runtime supports it (decision 0017): the main loop keeps working; the pass blocks only the gate it feeds, and that gate is never presented with the pass still outstanding.

## Modes and lanes (the mode gate)

Every planning pass starts by classifying the work. Classification is **mechanical**, using repository-harness risk flags:

> Flags: auth · authorization · data model · audit/security · external systems · public contracts · cross-platform · existing covered behavior · weak proof around the area · multi-domain

| Mode | Trigger | Workflow |
|---|---|---|
| `docs` | every touched file is knowledge, not runtime: `docs/`, specs, README, sample/example configs, plans | announce one line → write → format check → capture stub/decision if an outcome settled. No cells, no gates, no reviewers (lane scaling v2, decision d02a6bc6) |
| `tiny` | 0–1 flags, ≤2 files, no API/data change, one direct task | short plan note → inline 2-min reality check → **one merged shape+execution gate** → solo in-session execution (one cell, no worker) → self-review + done-report (diff + fresh verify output; no merge question) → scribing sync if behavior changed → compound only if a lesson emerged |
| `spike` | one yes/no proof decides whether the plan is real | spike cell in `.spikes/` → answer → return to planning |
| `small` | 0–1 flags, ≤3 files, no gray areas | light plan → inline reality gate (SMALLER PATH check only, no subagent dispatch) → merged shape+execution gate → solo in-session execution → self-checks (no auto reviewer — the correctness reviewer moves inside an on-demand review session) → scribing sync if behavior changed |
| `standard` | 2–3 flags, or story-sized behavior | full chain; phase plan or epic map, whichever explains the work honestly |
| `high-risk` | 4+ flags **or any hard-gate flag** (auth, authorization, data loss, audit/security, external provider, validation removal) | epic map → current-story pack → opt-in feasibility spikes (only migration/security/external-side-effect/no-precedent, validation-diet D8) → the merged shape+execution gate → detailed traces |

Rule of use: **the least workflow that honestly protects the work**. A tiny fix that spawns epic ceremony is a red flag; a hard-gate change routed as `small` is a worse one.

**Lanes scale ceremony, never memory** (vision principle 11): in every lane — tiny included — a capped `behavior_change` cell obliges a `bee-capturing` (Scribe) sync before the work is considered closed. The sync itself scales with the lane (a tiny fix usually means one replaced line in one spec); skipping it does not.

## Stage contracts

### bee-hive (bootstrap & routing)

- **On every session start / after compaction:** verify onboarding (`.bee/onboarding.json`), run `.bee/bin/bee status --json`, surface `HANDOFF.json` if present and **wait**, read `critical-patterns.md`, surface recent active decisions.
- **Routing:** vague/new feature → shaping; clear-scope research → planning; small fix → planning in tiny/small mode; review request → reviewing; "clean up / debt / audit" → grooming; capture learnings → capturing; improve bee's own skills → the maintainer guide [handbook/writing-skills.md](handbook/writing-skills.md); `/go` → go mode (full chain with the three gates).
- **Surface scope earlier** (compound-engineering): if the request already contains concrete acceptance criteria *and* references to existing patterns, offer to skip the Explore step — "Found clear requirements. Jump straight to planning, or explore alternatives first?" — and on approval route to planning with a one-paragraph scoping synthesis in place of CONTEXT.md gray-area work (the decisions still get D-IDs).
- **Scout contract (just-enough reading):** phase × lane matrix with token budgets — tiny ≈ 2K tokens of harness context, standard ≈ 5K, high-risk ≈ 10K. Retrieval triggers, not reading lists: "touching schema → read schema decisions first", "touching auth → read auth decisions + high-risk template".
- **Never:** auto-resume a handoff, skip a gate, or let a stage start with stale onboarding.

### bee-shaping — Explore and Lock (scout bees)

- **Reads:** user conversation, critical-patterns, a *quick scout only* (`rg` keyword pass + 2–3 files, cited in questions).
- **Does:** classify scope and domain types (SEE/CALL/RUN/READ/ORGANIZE); generate 2–4 gray areas that would otherwise make planning guess; Socratic locking — **one question per message**, preferably single-choice, outcome-framed ("what breaks for users if…"); assign stable IDs D1, D2…; scope creep → mark deferred and return.
- **Writes:** `docs/history/<feature>/CONTEXT.md` — boundary, domain types, locked decisions, deferred ideas, scout paths. Concrete language; no placeholders. One fresh-eyes reviewer pass (max two loops), run in the background where the runtime supports it — blocks only Gate 1's presentation (decision 0017).
- **Never:** research implementation, propose architecture, create cells, write code, bundle questions, answer its own question.
- **Handoff:** Gate 1 → "Invoke bee-planning."

### bee-planning (the waggle dance)

- **Reads:** CONTEXT.md, critical-patterns, active decisions, bee_status scout.
- **Does:**
  1. **Discovery** at the right research level (gsd): L0 skip / L1 quick verify / L2 standard (2–3 options) / L3 deep dive — using the three-layers framing (tried-and-true, new-and-popular, first-principles).
  2. **Mode gate** (mechanical flags, above).
  3. **Synthesis:** `approach.md` — chosen path, risks, proof needs, files, open questions still owed by the time the reality check runs.
  4. **Shape:** write `plan.md` (`artifact_readiness: requirements-only`) — direct note / spike question / small plan / phase plan / epic map. At the moment the shape is drafted: the **SMALLER PATH check** (validation-diet D1, the sole reality-gate survivor — one inline question, one line of file/command evidence, never a report: is there a cheaper shape that still honors every locked decision?), and, standard/high-risk, the **review wave** (validation-diet D5 — a merged reviewer covering structure and cold-pickup, findings held until the gate). **Stop at Gate 2.**
  5. **Prep (after approval):** enrich the *same* `plan.md` to `artifact_readiness: implementation-ready` and create cells for the *current* slice only. Cells are executable prompts: files, read-first, directive action citing D-IDs, `must_haves`, verify command. Every cell that changes observable behavior is marked `behavior_change: true`. Future-slice cells are prohibited.
- **Quality rules:** no scope reduction of locked decisions (SPLIT instead); no pseudo-cells in markdown; every cell has a testable exit; test matrix informed by the 12 edge dimensions (claudekit) at a depth matching the lane.
- **Spikes:** opt-in by change class, not a default gate step (validation-diet D8) — owed only when the change is `migration`, `security`, or reaches an external system with a side effect, or uses an API/library/technique with no in-repo precedent. One spike = one yes/no question; disposable code in `.bee/spikes/<feature>/`; NO → return to planning with the failed assumption; YES → record constraints. Spike code never silently becomes production code. The feasibility matrix and delta rule that used to gate this are deleted outright, no replacement (validation-diet D6).
- **Handoff:** invoke `bee-shaping` ("Brief") to render the implement plan (`small`+), then "Invoke bee-swarming" once Gate 2 (shape + execution) is approved.

### bee-shaping — Brief (the beekeeper's brief) — decision 0008

- **Reads:** CONTEXT.md decisions, approach.md, plan.md, current-slice cells, planning's reality-check evidence (SMALLER PATH + review wave), `state.json` gates.
- **Does:** render **one** `docs/history/<feature>/implement-plan.md` per feature — a human-legible consolidation of the truth artifacts that Gate 2 links as the review object. It **projects** every section from a named source and **authors only two**: the Technical Design narrative and the Rollback Plan (bee's only rollback discipline). Lane-scaled: no brief for `tiny`/`spike`, a ~15-line mini-brief for `small`, the full template with empty sections dropped for `standard`, Rollback + Security mandatory for `high-risk`. Three modes: **render** (before prep), **refresh** (after prep patches the Validation Plan with the reality-check evidence), **walkthrough** (after Gate 3 on `standard`/`high-risk`: `docs/history/<feature>/walkthrough.md` reconstructed from execution records — capped cell traces, review findings, UAT — never from the plan; sets the implement plan `status: Shipped`), **on-demand** (any phase). Status frontmatter mirrors the gates (`Draft → Ready for Review → Approved → Needs Revision → Shipped`); a source change after approval flips `Needs Revision`.
- **Truth model (extends D12):** the brief is the human-layer projection of the truth artifacts, never their master. Human feedback on the brief flows back into `CONTEXT.md`/`plan.md` (a locked decision is superseded by D-ID) and the brief re-renders — it is never hand-edited as the sole change site.
- **Never:** originate a decision/scope/approach/cell; invent content to fill a section (source silent → Open Question); assert a validation result that has not run; in walkthrough mode, summarize the plan instead of reconstructing from execution records, claim verification broader than the evidence, or omit deferred findings/deviations; fork a `-v2`/dated brief; paste the whole brief into a gate chat message (link it).
- **Handoff:** render/refresh → return to the caller (`bee-planning`, for Gate 2 — shape and execution together); walkthrough → "Invoke bee-capturing (Scribe)." The Brief step presents no gate itself.

There is no standalone validating stage (validation-diet D1) — its reality gate, feasibility matrix, delta rule, and spikes are gone or folded into `bee-planning` above: the SMALLER PATH check and the review wave moved in (D1/D5), the feasibility matrix and delta rule were deleted with no replacement (D6), and opt-in-by-change-class spikes replaced the old mandatory-for-high-risk spike step (D8). The decision vocabulary it used to speak (READY / READY WITH CONSTRAINTS / NOT READY – RUN SPIKE / NOT READY – RETURN TO PLANNING) goes with it — a failed SMALLER PATH check or spike simply returns to planning.

### bee-swarming (orchestrator)

- **Preconditions:** Gate 2's execution component approved (`approved_gates.execution: true` — the old standalone execution gate, folded into Gate 2); current-slice cells open; reservations swept.
- **Does:** wave analysis over the cell dependency graph (parallel within a wave, sequential across waves); assign exactly one cell per worker; spawn with the isolation contract — cell id, CONTEXT.md path, global constraints, reservation identity, status-token protocol, **nothing else, never session history**; pick the worker model by declared tier (compound-engineering): `extraction` = cheapest capable (retrieval, mechanical edits), `generation` = mid (implementation, test writing), `ceiling` = the orchestrator's own model (integration, architecture, final review) — state the model explicitly, and where the runtime can't select per-agent models, fall back to read budgets and output caps; record workers in `state.json`; tend results; rescue or re-dispatch `[BLOCKED]` with more context or a stronger tier; write HANDOFF at ~65% context.
- **Never:** implement cells itself; let workers self-select; resolve file conflicts by "being careful" (fix reservations or cell scope instead); send routine mid-flight pings (silence is not failure).
- **Handoff:** phase clean → next planning slice, or final slice done → "Invoke bee-capturing (Scribe)." `bee-reviewing` is never part of this handoff (SPEC R1/R3, decision 565e68d0) — it is a separate flow the user invokes on demand, over whatever scope they choose, at any later point.

### bee-swarming — Execute (worker bee)

Loop: **Initialize → Accept assigned cell → Reserve → Implement → Verify → Cap → Release → Return.**

- Initialize: read AGENTS.md, bee status, CONTEXT.md, the cell (`bee cells show <id>`).
- Reserve every file/glob before writing; conflict → `[BLOCKED]`.
- Implement: read before editing; match existing patterns and locked decisions; no stubs, TODO-placeholders, or dead code. **Deviation rules (gsd):** auto-fix bugs / missing critical functionality / blocking issues; STOP and report for architectural changes; package installs always checkpoint.
- Verify: run the cell's verify command exactly; diff-aware test mapping where the project suite is big (claudekit); two serious failures → `[BLOCKED]` with command, failure summary, diagnosis.
- Cap: `bee cells cap <id>` (refuses without a recorded verify pass); one commit per cell with the cell id; record the lane-scaled trace depth (outcome, files, deviations, friction if a trigger fired); if the cell is marked `behavior_change: true`, the trace must include structured `verification_evidence` — tests inspected, tests added/changed, red-failure or characterization evidence, verification run, any deliberate exception (compound-engineering); release reservations.
- Return exactly one of `[DONE] [BLOCKED] [HANDOFF] [NOOP]` plus a report file in `docs/history/<feature>/reports/`.
- **Never:** edit outside reserved scope, handle multiple cells, wait silently, cap without verification.

### bee-reviewing (inspector bees)

**On-demand only (SPEC R1/R7/R8, decision 565e68d0):** `bee-reviewing` never launches automatically — not after a final swarm slice, not after a feature closes, and not for a merge/ship/release request that hasn't explicitly asked for review (that case reports the unreviewed/stale count and risk level, then asks one question before spending any reviewer token — 7.4/A9). It runs only when the user names a scope: a feature, a named batch, or a commit range. Status vocabulary distinguishes `verified`, `unreviewed`, `in review`, `reviewed`, and `review stale` (R10) — a completed, verified change may sit `unreviewed` indefinitely without blocking further development.

- **Does:**
  1. Dispatch specialist reviewers with isolated context (diff + CONTEXT.md + plan.md only): `code-quality`, `architecture`, `security`, `test-coverage` in parallel; `learnings-researcher` searches `docs/history/learnings/` for precedent related to the touched modules (compound-engineering); `learnings-synthesizer` runs after all of them.
  2. Findings → severity P1 (security/data-loss/breaking/blocker — blocks merge), P2 (real perf/architecture/reliability/test gap), P3 (cleanup/docs/future debt). Uncertain → P2. **Synthesis rules** (compound-engineering): reviewers score independently; corroboration across independent reviewers promotes a finding one level; each finding carries an `autofix_class` — `gated_auto` (concrete fix, apply after judgment), `manual` (needs design input), `advisory` (report-only) — as routing *signal, not an apply gate*; on disagreement take the more conservative route. Each finding: plain-language summary, what the code does today, why it matters, concrete failure scenario, file/line evidence, smallest credible fix.
  3. **Verification-evidence gate:** for any capped cell with `behavior_change: true`, check the recorded `verification_evidence`; missing or vague evidence is itself a P1 finding — the work goes back, it does not pass forward.
  4. **Artifact verification** for everything CONTEXT.md/plan.md promised: EXISTS → SUBSTANTIVE (no stub/TODO/fake path) → WIRED (imported and used on the integration path). All three = OK; EXISTS+SUBSTANTIVE = P2; missing or EXISTS-only = P1.
  5. **Human UAT** walk-through for SEE/CALL/RUN decisions; failure → P1 fix cell and re-run; skip requires a recorded reason.
  6. Finish: project build/test/lint gates; P2/P3 → backlog/grooming cells with traceability (never blocking the current epic); if filing a residual finding anywhere fails, write it to `docs/history/<feature>/reports/residual-findings.md` so nothing evaporates; close out state.
- **Handoff:** Gate 3 → `standard`/`high-risk`: "Invoke bee-shaping (Brief walkthrough)," which then hands to the Scribe sync; `tiny`/`spike`/`small`: "Invoke bee-capturing (Scribe)."

### bee-capturing — Scribe (scribe bees, the BA)

- **Owns:** the state layer at BA grade (decision 0002) — `docs/specs/<area>.md` + `reading-map.md`. An area is **domain-general**: a screen/form, an API, a background job, an integration, a pipeline, a business process — any unit with observable behavior that outlives features. Acceptance test is the **rebuild bar**: an agent given only the spec, minus its Pointers section, rebuilds the same observable behavior on another stack; a human understands the area without the code. **Tech-agnostic rule:** no language/framework/library/file names outside the quarantined `Pointers (implementation)` section.
- **Reads:** capped `behavior_change` cells + `verification_evidence`, UAT records and worker reports (→ behaviors, data, per-actor visibility); gate-locked CONTEXT.md + active decisions (→ business rules, cited by D-ID); code + user interview in harvest mode.
- **Does:** four modes. **Sync** (chain, after Gate 3): merge the feature's behavior deltas into the touched areas' specs — entry points & triggers (links/screens; schedules/events/calls), data dictionary (every enum value's business meaning; display order for UI; chosen config values with their D-ID), behavior/operation blocks (blocked-when or runs-when / what changes / side effects / what each actor or consumer observes; failure behavior for operations), actors & access matrix, business rules; refresh the reading map; run the rebuild self-check. **Capture** (any phase, on demand): whenever a discuss → build → test → adjust loop settles an outcome — a rule agreed, a behavior confirmed by test, a threshold tuned — it is logged via `bee decisions` same turn; the spec merge is inline for `high-risk`, and a queued stub (`bee capture add`) for every other lane (decision 0017) — discussion knowledge never waits for feature close. **Flush** (decision 0017): at wrap-up, the PreCompact/close warning, or the next session's offer, drain the capture queue oldest-first — full merge per stub, mark flushed, record the scribing run. **Harvest** (on demand or from grooming): first spec for a pre-bee area — code yields observable behavior only; meanings and rules are asked (one Socratic question per message) or filed as honest `Open Gaps` with `coverage: partial`.
- **Never:** invent — evidence → behavior, approved decision → rule, neither → open gap; copy from plan.md; state a not-yet-implemented rule as current behavior; narrate history; leak technology above Pointers; skip when `behavior_change` cells were capped.
- **Handoff:** continue to the same skill's Compound section.

### bee-capturing — Compound (honey)

- **Reads:** feature history, cells and traces, review findings, commit history. Missing artifacts → session summary + git diff; never fabricate.
- **Does:** three parallel analysis subagents — pattern extractor, decision analyst, failure analyst; orchestrator synthesizes (subagents never write durable files); write one dated `docs/history/learnings/YYYYMMDD-<slug>.md` (what happened / root cause / imperative future rule); promote only genuinely critical, cross-feature lessons to `critical-patterns.md`; log durable decisions to `bee decisions log` (with rationale + alternatives + a
declared `--relation`); **guard the state layer** — verify the Scribe sync ran for the feature (state record); if not, run it, never sync specs inline (decisions 0001/0002); file unresolved friction into `.bee/backlog.jsonl` with predicted impact.
- **Never:** skip compounding for meaningful work; promote everything as critical; write "test more carefully"-grade advice; close out while a spec is older than the behavior it describes (`behavior_change` cells capped but the Scribe sync never ran).

### bee-grooming (undertaker bees) — on demand

- **Audit:** compute the entropy score (orphaned cells ×10, unverified cells ×5, stale decisions ×5, stale specs ×5, backlog-without-outcome ×2, stale work ×3, broken tools ×8; cap 100) and report the trend.
- **Hunt:** cluster friction from traces and backlog; scan for dead code, unused exports, stale docs vs code, stale or missing area specs (decisions 0001/0002 — proposed sync/harvest work routes through `bee-capturing` (Scribe)), TODO/stub debris, unverified verify-commands, superseded-but-cited decisions; slop-pattern pass on recent diffs (gstack).
- **Propose:** each kill candidate becomes a backlog item with pain, predicted impact, and risk lane — presented for approval (grooming never deletes on its own initiative).
- **Execute:** approved kills run as tiny/small cells through the normal worker loop (reservation, verify, cap).
- **Close the loop:** record actual outcome against the prediction; feed durable lessons to compounding.

### Writing bee skills (comb building — maintainer guide)

Carried from khuym/superpowers nearly verbatim, and re-homed out of the product into [handbook/writing-skills.md](handbook/writing-skills.md) — see also [04-skills-spec.md](04-skills-spec.md#skill-writing-discipline). The Iron Law: **no skill (or skill edit) without a failing pressure test first.**

## Red flags (chain-wide)

- jumping from exploring to swarming · code before CONTEXT.md exists · skipping the reality check (SMALLER PATH) · ignoring locked decisions · workers self-selecting cells · capping without verification · commits without cell ids · continuing past open P1s · reservation leaks · stale state.json after a phase transition · resuming without surfacing HANDOFF.json · plausibility language ("should work") accepted as evidence · a tiny fix wearing epic ceremony · a hard-gate change routed below high-risk · session history pasted into a worker dispatch
