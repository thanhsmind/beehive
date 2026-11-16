# Routing And Contracts Reference

Load when resuming a handoff (`HANDOFF.json`), checking lane ceremony or concurrency rules, running the leader completeness check, formatting a turn/question/catch-me-up/pushback, or checking chaining or file paths.

## Topology

Three checkouts, three jobs: main stays clean and integration-only, never where code
truth changes; a feature worktree is the only place code truth changes, one worktree
per independent feature; staging is the disposable mixing ground where finished
features sit together for the user to test at one place, rebuilt from main plus every
feature still awaiting approval. The `uat` gate is the door between staging and main —
a feature only lands on main once its uat gate is approved. See
`docs/knowledge/areas/worktree-parallelism/staging-mixing-ground.md` (staging-lane
D0/D0a) for the mechanics. A repo that sets `"staging_before_merge": false` skips
staging entirely — `bee staging add`/`rebuild` refuse `STAGING_DISABLED`, and the
feature worktree itself stands in for the mixing ground; the `uat` gate is unaffected.

## Onboarding Protocol

Lives in `onboarding.md` — status contract, `blocked_*` branches, forced-apply
transparency, greenfield init lane. Load it only when onboarding is in
question; an `up_to_date` session never does.

## State Bootstrap

`bee orient` is the session-start packet — phase, gates, blockers (pending
handoff, debts, stale reservations), and the next action/skill in one call;
its output supersedes any manual read-these-files order. A pending
`.bee/HANDOFF.json` it surfaces follows Resume Logic below. Critical
patterns come from the preamble digest; the full source is
`docs/knowledge/index.md`'s `## Critical patterns` section with a bundle,
else `docs/history/learnings/critical-patterns.md` when present.

## Resume Logic

If `.bee/HANDOFF.json` exists, read its `kind` (`bee state handoff show --json`; a missing/unknown kind normalizes to `pause`, fail-safe) and branch:

**Pause** (or any kindless record):

1. Read `HANDOFF.json` and `.bee/state.json`.
2. Extract phase, feature, mode, cells in flight, done/remaining, and next action.
3. Present the pause point to the user in plain language.
4. Continue only after explicit confirmation. If the user's first message is an unrelated request, still surface the handoff first, then ask which to pursue.

Do not auto-resume. Ever.

**Planned-next** — the previous cell was capped (its proof line recorded, checked — never re-run — at the boundary) and the next cell was already claimed for this handoff. Adoption fires ONLY at a fresh-session boundary (a cleared or newly started session — never a resumed or memory-compacted one, which follows the pause path above):

1. `bee state handoff adopt` transfers the carried claim to this session and clears the handoff record.
2. On success, present the adopted cell, its verify command, and its lane as a start-now instruction — no wait, no confirmation prompt.
3. On a failed adoption (claim lost the race, handoff already cleared), fall back to the pause presentation above — never fabricate a start-now instruction.

## Phase-Boundary Moves

At a phase boundary — never mid-phase — weigh five moves in order; the
first yes wins:

1. **Continue in-session** when the next phase needs this one as a
   primary source, or budget remains. Costs nothing, loses nothing —
   rule it out first.
2. **Fresh start** when everything in-session is disposable. Cheapest
   move on the board — and an irreversible discard.
3. **Handoff** only for a genuinely new place or person: a new session
   boundary, a new checkout, or another agent. That list is the whole
   clause.
4. **Subagent** when the piece is scoped enough to run
   away-from-keyboard.
5. **Compact last.** Every move but continue turns a primary source
   into a lossy summary, and compaction's failure mode is a fresh
   session confidently wrong about a decision the summary flattened.

This is the procedure behind the handoff rule (rule: agents-context-handoff-65):
a handoff happens only on the user's word or at feature close, never at a
context mark alone; that rule says when, this tree says which move to make.

## Scout Contract (just-enough reading)

Lives in `scout-and-ticks.md` — the scout matrix, the route record, the
re-lane checkpoint, crash recovery, ship visibility, the progress-tick worked
examples, and the session scout in full.

## Lane ceremony in full

`bee-planning/SKILL.md`'s Route section keeps the classification rule and the scaling law; this section
carries the full per-lane ceremony detail.

Review is on demand: no lane auto-dispatches a reviewer wave or asks Gate 3 after execution. Every lane below closes through scribing/compounding as `unreviewed`; a review session — and its Gate 3 — happens only when the user asks, over whatever scope they choose. Separately, `standard`/`high-risk` goal-checks also run a semantic checklist judge once per slice over its capped `behavior_change` cells (table: "Goal-check judge tier", `gates-and-delegation.md`) — that is verification of the cells, not this on-demand review session.

**"Validate" below is ceremony, not a phase — it runs inline inside `planning`'s shape stage.**

| Lane | Plan | Validate (inline, inside planning) | Execute | Review | Human stops |
|---|---|---|---|---|---|
| `docs` | short brief (Lock writes a short `CONTEXT.md`, D1) — then announce one line | format check (parse/lint if applicable) | direct, in-session | none | 1 — Gate 1 (the brief's approval) |
| `tiny` | short brief (Lock writes a short `CONTEXT.md`, D1) — the cell is still the micro-plan | SMALLER PATH check inline, 0 ceremony subagents (I/O-offload workers exempt — Delegation contract) | a dispatched implementer, then a separate dispatched tester for a code cell (the execution-worker contract applies to both: dispatch through `bee dispatch prepare --runtime <rt> --kind cell --json` and run exactly the tool and payload it returns; standard worker prompt template, no reviewers/panels/waves); a docs-only cell takes one worker plus a second for its parity and pointer checks | orchestrator-authored done-report (worker's verbatim diff + commit; `bee finish` requires the recorded proof line, checked — never re-run — at the boundary; orchestrator re-runs only on smell or hard-gate) — verification, not independent review | 1 — Gate 1 folded into the merged shape+execution gate |
| `small` | short brief (Lock writes a short `CONTEXT.md`, D1) + logged scoping synthesis; plan.md is opt-in | SMALLER PATH check inline, 0 ceremony subagents (I/O-offload workers exempt — Delegation contract); spike only if a blocking assumption demands it | a dispatched implementer and separate tester per code cell (same contract as `tiny`'s Execute column), its 1-3 cells dispatched in PARALLEL when disjoint (see Concurrency law in full below) | orchestrator-authored done-report, self-checks only, no auto reviewer (the correctness reviewer moves inside an on-demand review session) | 2 — Gate 1 folded into the merged shape+execution gate, self-checks close-out |
| `standard` | full `CONTEXT.md` + `plan.md` | SMALLER PATH check + the plan-step hat wave as the plan check — 3 seats dispatched `--kind advisor`, no review-tier reviewer (home: `gates-and-delegation.md` ("Hat wave")); ≤5-file diff (0 hard-gate flags): inline self-review, no dispatch | swarm workers | on user request only: session panel scaled to scope risk (4 core reviewers) | 2 — Gate 1, Gate 2 (merged shape+execution) |
| `high-risk` | full `CONTEXT.md` + `plan.md` + brief | SMALLER PATH check + the plan-step hat wave, all 5 seats `--kind advisor`; its synthesis IS the gate's advisor consult, recorded with `bee state advisor-ref record` | swarm workers | on user request only: session panel scaled to scope risk (full wave + conditionals) | 2 — Gate 1, Gate 2 (merged shape+execution) |

The situation the task matches seeds the leader's session todo list as a projection: `bee orient --class <c>` prints the class's situation lines, the leader opens the matching situation file, its numbered steps open the list, and a skipped step stays with `skip:` and its reason. The situation file stays the one home of the steps; the list never replaces it.

No added plan ceremony rides along with any of this (D1): `tiny`/`small`/`docs` gain exactly a brief and a recorded approval, never a `plan.md` requirement they didn't already have.

**`gate_bypass` scopes the STOP, never the record (D2).** Every row above writes its brief and its Gate 1 approval regardless of bypass level; what bypass removes is the human pause, not the artifact — an auto-approved run stamps the approval record `actor: "auto"` plus the bypass level and reason instead of stopping for a human `actor: "user"` (`gates-and-delegation.md`, "Gate bypass mode"). **D6 boundary:** this whole table applies only to a request that writes at least one file, code or docs alike; a pure question that changes nothing on disk takes no row here — it gets no brief and no record.

**Gate 3 is additive, not counted above:** it is asked once, whenever a review session actually runs for that scope — never automatically at the end of a lane's default chain.

### Concurrency law in full

**THE LAW:** if pieces of work can run at the same time, open the threads and run them; serial only when forced. One rule, three tiers — gather work fans out to I/O workers (Delegation contract, `gates-and-delegation.md`), a slice's cells fan out to a wave whenever their product file sets are disjoint (reservations are the proof and the police, the machine-aware limit caps it (`herding.limits.concurrency`; unset, bee sizes it from the CPU count — `docs/knowledge/areas/bee-herding/waves-and-occupancy.md`), the same number for native workers, never a fixed count), and independent ready features fan out to lanes or worktrees (Lanes, first-class below). Undeclared-overlap concurrency for the same feature is a `standard`/`high-risk` wave shape wearing a `small` lane, the exact ceremony-mismatch red flag this lane scaling exists to catch.

**MANDATORY CONCURRENCY PLAN:** before dispatching anything, the orchestrator states in one line what runs concurrently and what is forced serial and why — computed, not guessed, never assumed by default. WAIVED when exactly one cell is being dispatched (a one-worker plan states nothing); owed again from two cells up. Cells: `bee cells schedule` names the disjoint sets from declared file overlap; a real product-file conflict named in the dispatch note is what makes a cell wait, nothing else does. Features: the declared `--paths` on `state start-feature --as-lane` are checked against every other live session's claims/reservations before the lane starts — a refusal names the holder and is itself the plan's proof that the paths were not disjoint.

**THE ONLY LEGAL REASONS FOR SERIAL, exhaustive:** a declared file-set overlap (including a shared generated artifact not deferred by a wave barrier), a true data dependency (`deps`), a single scarce external resource, or an explicit human instruction. Nothing else is a reason — anything else fans out.

**LANES, FIRST-CLASS:** before every feature start, check whether other ready feature work has disjoint declared paths — if so, the paved road is a lane, not a queue, whether or not another feature is already live — `bee state start-feature --feature <f> --mode <m> --as-lane --paths <declared>`; lane-scoped mutations take `--lane`. Lanes classify and coordinate; code lives in the feature's worktree — a code-touching feature branches into its own worktree at feature start regardless (rule: agents-worktree-first) — mechanism in `docs/knowledge/areas/worktree-parallelism/routing-and-visibility.md` — its declared paths still coordinating through the shared store. A lane refusal (holder + expiry) means the paths were not disjoint after all — pick other ready work or wait for the hold to lapse — never work around it.

**TICK:** the concurrency plan emits its own progress line per the Progress ticks catalog (`scout-and-ticks.md`, "Progress ticks — worked examples") — same silent-bookkeeping rule as every other tick, never suppressed by bypass.

Full doctrine for the cell/wave tier — the wave-barrier regen protocol and the execution-worker class relationship: `bee-swarming/references/swarming-reference.md` ("Single execution worker in full", "Default — parallel"), and the Delegation contract (`gates-and-delegation.md`).

### Docs lane

Before the write (D1, D6): Lock writes a short brief — `docs/history/<feature>/CONTEXT.md`, the same file full lanes use, in a short form — naming what was asked, what was found, and what will be written; then Gate 1 asks a one-line approval ("Brief: about to write X, because Y. Approve?"), recorded even when `gate_bypass` auto-approves it (D2). Only a request that writes no file skips this — a pure question is not a docs-lane change (D6).

Then the change is knowledge upkeep, same class as capture — announce one line ("docs lane: writing X"), write it, run a format check when one exists (JSON parses, markdown lints), then close with a capture line or an explicit "nothing settled" (rule: agents-capture-line-at-close). No cells, no separate execution gate, no reviewers — the brief and its Gate 1 approval are the only ceremony this lane gains. If the target path is outside the write-guard allowlist (rule: hook-runtime-docs-lane-allowlist) the hook will block the idle write — fall back to the tiny fast path instead of fighting the guard. And the docs lane holds main only while solo (rule: agents-worktree-first) — take the worktree it names, never wait out the peer.

### Tiny/small fast path

Before any of this (D1): Lock writes the brief first — a short `CONTEXT.md` naming what was asked, what was found, what will be done — no added plan ceremony, just the brief. Then the draft cell(s) are rendered as a **preview inside the gate message** — never persisted first, dry-run-validated first (`bee cells add --stdin --dry-run`; `bee-planning/references/cells.md`, "Pre-flight before cells add") — and the 2-minute reality check runs inline against that preview, before the shape and execution approvals are presented as **one merged question** — "Work shape + execution: I'm about to do X via Y, verified by Z. Approve?" — approval records both `shape` and `execution`, covers exactly the previewed work packet, and is the same call that approves the brief (Gate 1 and Gate 2 fold into this one stop at `tiny`/`small`). `cells add` runs only **after** approval, and the cells are claimed only then — previewed before persist, never persist-then-preview. Every code cell, `tiny` included, is dispatched through `bee dispatch prepare` because each role runs its own configured model: a dispatched implementer and a separate dispatched tester that shares no context with it; the merged gate, cap discipline, and done-report are unchanged. Inline execution of a code cell is refused at cap, and a `tiny` cap with more than one counted file (paths outside `.bee/` and `docs/history/`) is refused without `--inline-reason`. `bee route` refuses `--lane tiny` with `--files` above 1. After execution (the tester's return): no separate merge gate — the orchestrator authors the done-report itself from the worker's verbatim diff plus its commit after running the leader completeness check (worker reports are navigation aids, not completion evidence; compare approved requirements against actual artifacts before accepting). `bee finish` requires the recorded proof line; a smell or hard-gate still gets a spot `bee test` re-run, and that done-report (diff + commit + capture line) closes it. `bee close` and `bee worktree merge` check that recorded proof for the feature's capped cells — neither runs `commands.test` itself (`bee-swarming/references/swarming-reference.md`, "Proof at finish and close, in full"). A real problem found during the orchestrator's own review stops and asks, always.

### Leader completeness check

When a worker caps a cell, the leader must verify completion before accepting the result. Worker reports are navigation aids, not completion evidence.

- **Check artifacts, not prose**: For every worker completion, compare every approved requirement (from cell and plan) against actual artifacts, wiring, and execution evidence. Never accept a cell solely on the worker's report or status token.
- **Reuse existing requirements**: Reuse existing `must_haves`, acceptance criteria, and proof lines from the cell and plan. Do not invent new checklists.
- **No new report schema or mandatory full-suite rerun**: This check introduces no new report schema. It never re-runs the worker's recorded proof and never runs the full suite per cell (rule: agents-proof-at-cap).
- **Risk-based depth**: Verification depth stays risk-based. Deeper checks (reading full diffs, verifying call sites, inspecting runtime evidence) apply to high risk, missing evidence, or contradictions. Low-risk mechanical changes need only direct artifact spot-checks.
- **Recorded check**: Record the check with `bee cells leader-check --id <cell> --verdict ok|gap --file <answers.json>`. The payload answers every derived requirement with an artifact. The verb verifies that these artifacts exist. An artifact that looks like a repo path must exist on disk. At least one answer must name a path from the cell's `trace.files_changed`. Every `trace.files_changed` path must also appear in the real diff of a commit that carries the `cell: <id>` trailer (last `COMMIT_TRAILER_WINDOW` commits of the feature's history). A claimed path that no such commit touched refuses. No trailer commit refuses unless `trace.commit_pending` is set. Each entry records `commit_diff`: `checked`, `skipped: commit_pending`, or `skipped: git unavailable`.
- **Verdict and doors**: A `gap` verdict records the check and reopens the cell — status back to `open`, the gap answers as its reopen reason — so the next dispatch reworks it. Both `bee close` and `bee worktree merge` refuse a feature whose capped cells carry no `ok` mark. This rule applies at every lane. Cells capped before `LEADER_CHECK_DOOR_INTRODUCED_AT` are grandfathered. A `bee audit log --kind leader-check-deferral` row is the one named escape.
- **Due findings first**: Before recommending the next open review finding, read `bee triggers list --due` and handle each due one by `bee-reviewing`'s Finish rule.

### Capture discipline

Lanes scale ceremony, never memory — zero exceptions, the docs lane and non-cell quick work included: a feature whose capped cells include `behavior_change` owes ONE `bee-capturing` spec sync covering all of them — tiny lanes included — recorded as PENDING at close and run deferred, at the owner's pace, batching several closed features into one session when cheaper (decision c8e25271; `bee orient` carries the reminder until it runs). A settled discussion outcome (rule, behavior, tuned value; backend or frontend alike) is still captured the moment it settles — deferral applies to the close-time sync, never to same-turn settlement capture. Every task close carries a capture line or an explicit "nothing settled" (rule: agents-capture-line-at-close). **Settlement detection is the agent's duty, unprompted:** the routing row "user asks to document" is the fallback, not the norm — the norm is the agent noticing "this just settled", announcing it in one line, and capturing in the same turn without being asked. What same-turn capture costs is lane-scaled: high-risk = full spec sync inline; every other lane = decision log + a one-line capture stub (`bee capture add`), with the full merge at a flush point (wrap-up, PreCompact warning, or next session's offer). Capture writes only `docs/` + `.bee/` — no gate applies.

## Chaining Contract

| Skill | Reads | Writes |
|-------|-------|--------|
| hive | onboarding, state, HANDOFF, critical-patterns, decisions | state routing updates only |
| shaping (Explore/Qualify/Lock) | user conversation, backlog row, critical-patterns, quick scout | `docs/history/<feature>/CONTEXT.md` (lock or park), backlog row status, state update |
| planning | CONTEXT.md, critical-patterns, active decisions, bee status | `approach.md`, `plan.md` (frozen at Gate 2 — approval stamp only after approval; none for `tiny`, opt-in for `small`), current-slice cells via `bee cells add` |
| shaping (Brief) | CONTEXT.md, approach.md, frozen plan.md + cells (drift re-render triggers on cell changes only, since the plan cannot drift after approval), test-result records (`.bee/logs/test-results.json`), state gates (render/refresh); capped cell traces, review findings, UAT (walkthrough) | `docs/history/<feature>/implement-plan.md` (projection; `high-risk` always, `standard` on-demand, `small` optional on request); `docs/history/<feature>/walkthrough.md` (post-Gate-3; `standard`/`high-risk`) |
| swarming (orchestrate) | Gate-2-approved cells, state, reservations | worker registry in state, HANDOFF on the user's word or at feature close, wave results |
| swarming ("Execute") | assigned cell, CONTEXT.md, reservations | implementation commits (one per cell, cell id in message), finish (proof line recorded in `--report`, checked — never re-run — at close/merge, where the recorded proof is the evidence), report in `docs/history/<feature>/reports/` |
| reviewing | user-selected immutable scope (a `bee_reviews` session — never triggered by phase or cell completion) | session findings (P1/P2/P3) and the Gate 3 decision recorded on that session, backlog items, `residual-findings.md` fallback |
| capturing | `behavior_change` cells + test-result records, CONTEXT.md, active decisions, UAT/worker reports, feature history, traces, commits, code + user interview (harvest) | with a bundle: `docs/knowledge/areas/<area>/` concepts (BA-grade merge); with no bundle: `docs/specs/<area>.md` (BA-grade merge), `docs/specs/reading-map.md`; plus `docs/history/learnings/YYYYMMDD-<slug>.md`, critical-patterns promotions, decision log entries, backlog friction, state record |
| grooming | entropy inputs, backlog, traces, diffs | kill proposals, tiny/small cells, outcome records |

**Recommended-next after execution:** once a feature's execution work is done, the recommended next action is LANDING (`bee worktree merge` from main); capture is recorded as pending — `bee status`/`bee orient` report the pending capture alongside the review-candidate count, and neither `bee-capturing` nor `bee-reviewing` is chained into automatically. Both run deferred, on the owner's call, over any scope the owner names. The feature closes truthfully `unreviewed` (and `uncaptured` until Compound runs); the reminders stand until each is settled.

Every skill ends with an explicit handoff: `[Outcome]. Invoke bee-<next-skill> skill.`

## Direction of Truth — Projection Rule

The repo artifacts are the single source of truth for what work exists and its state: **cells** (`.bee/cells/`) for in-flight execution and the **PBI rows** in `docs/backlog.md` for product intent. A session's todo list — `TaskCreate`, `TodoWrite`, and any equivalent scratch checklist — is an **ephemeral projection** of those durable records, never the reverse.

The mapping is one-way: cells and PBI rows generate the session todo list, and no edit to that list ever writes back to a cell or a backlog row. When the two disagree, the repo artifact wins and the session list is regenerated from it. A todo item with no cell or PBI behind it is a projection bug, not a new unit of work — file the cell or the backlog row first, then let the list re-derive.

## Communication contract

One home — chat style is never governed from anywhere else. This section says
what reaches the user and in what shape; the vocabulary rule below says in whose
words.

**Who is reading.** They supervise; the agent executes. They drop in and out of
long multi-phase sessions, so state not restated is state lost. They think in
product terms — bee mechanics are noise. Their high-stakes moments are rare (a
gate, a decision, a privacy approval) and must be unmistakable from progress
chatter. They trust fresh output, not assurance.

**Turn shape** — every user-facing turn during bee work:

- **Open** with one line of state, in work language: what finished, what is running,
  what remains. Not "Step 3 of 5 (cell jr-2)" — "Rewrite landed and verified; now
  renumbering the references."
- **Body** is the work itself, sized for a supervisor who drops in and out: what they need to act on now, nothing a linked record already says.
  Progress ticks are not prose and do not count against that budget — they are one
  fixed-format line per step ("Progress ticks"). The complete record (reports,
  findings, matrices) lives in a linked file, never pasted into chat.
- **Close** with exactly one next action (rule: agents-one-next-action). A multi-step routed task's closing turn also carries one Principles line: each applied principle slug and the decision it changed, or a plain statement that none changed anything (rule: agents-name-applied-principles).

**Rules.**

1. **One question at a time**, formatted apart from progress text, phrased so the
   user can restate what they are deciding in their own words ("Question Format";
   the Gate Presentation Contract is the template). A question buried in a
   progress paragraph does not count as asked.
2. **A red or a refusal is never silenced** — not by a quiet switch, not by a
   bypass level, not by "I'll mention it at the end".
3. **Work language**, with the two exemptions below.

One standing exception: a destructive or irreversible action gets full explicit
clarity — safety beats brevity, always.

**Craft.** Give concrete units for anything over a minute ("verify ~2 min", never "a while"); state a failure as cause, fix, and who acts, quoting the shortest decisive line; file a tangent and mention it once at the close. Protocol and record surfaces — worker status tokens, cap traces, decision logs, CONTEXT.md — keep their ids, because that is where ids live. Write user-facing prose through `bee-unslop`'s rules, and saved documents — plan,
spec, CONTEXT.md, capture, PR body, commit body — through `bee-technical-writing`
as well. Neither ever rewrites a progress tick, a red or refusal line, command
output, quoted evidence, an id or code. When the project's own instructions set
tone, length, style or language, they win, and the reply stays in the user's
language.

Example:

```text
✗  Great question! I've now completed the analysis of the authentication
   module and fixed 12 issues across 47 files. Capped cell auth-3; phase is
   now scribing. Let me know if you'd like me to continue, or if you'd
   prefer I look at the session handling first, or something else!

✓  Login redirect is fixed — `npm run dev`, open /login: the loop is gone.
   The session-expiry check has the same off-by-one; filed as P2.
   Next: rerun the auth suite.
```

The bad one opens on filler, leads with counts as achievement, speaks bee
instead of work, and closes on a menu. The good one is the same turn with the
five rules applied.

**Catch-me-up reply.** When the user asks to be caught up, the reply has five
parts, in this order: one line naming the scope covered; a short capsule of
where the work stands; one line per feature or worktree, each with exactly one
status tag read from bee state and said in the user's words — merged, open
worktree, in progress (a cell in flight), done but not merged (capped, not
merged), parked; what was tried and reverted, taken from feature reports and
superseded decisions; one next action. The status comes from state, never from
memory of the session.

### Claim labels and pushback

Full form of rule: `agents-verify-before-agreeing`. `tiny`, `small`, `docs` are the fast lane; `standard`, `high-risk` are the deep lane ("Route record" in `scout-and-ticks.md`).

Triage examples:

```text
Triage: tiny — one-word typo in a README heading
Triage: small — one function, callers in the same file
Triage: docs — rewrites one reference section, no code
Triage: standard — new verb across three modules
Triage: high-risk — touches auth flag, data-model change
```

Claim-label examples:

```text
Onboarding offers a detected lint check as a member of `commands.test` [1]. It stays propose-only because bee never writes a detected command [2].

Sources:
[1] docs/knowledge/areas/onboarding/host-project-artifacts.md:322
[2] docs/history/trust-layers/CONTEXT.md D1
```

```text
The handler rejects an unknown lane [1], and the route test covers it [2].
Most CLIs retry a failed write once [unverified — checking before I rely on it].

Sources:
[1] src/route.rs:88
[2] cargo test route_rejects_unknown_lane — 1 passed
```

**Pushback.** When the user's direction conflicts with the code, never say "you are wrong". Open in plain language: "I understand the goal is X. Current code does Y because Z (file:line). To keep X, the options are..." Then four parts, in this order:

1. **Current state** — what the code does, with `file:line`.
2. **Gap** — where that falls short of the goal.
3. **Options and trade-offs** — each way to close the gap and what it costs.
4. **Recommendation** — the one option to pick, and why.

The user decides. If the check shows the user's direction is right and the code is wrong, say that instead, and name the fix.

### Work language — a vocabulary rule, not a silence rule

This constrains vocabulary, not whether a step is mentioned: every perceivable step still gets its progress tick ("Progress ticks"), naming what happened to the work rather than the record (`✓ capped: tick catalog rewritten`, not `✓ capped cell vt-3`).

Bee vocabulary may lead a line in exactly two cases:

1. the user asks about bee itself (state, cells, workflow) — answer plainly, in their language;
2. a gate genuinely needs their decision — and the Gate Presentation Contract already requires that question in work terms, not bee terms.

## Gate Presentation Contract

Lives in `gates-and-delegation.md` — gate presentation, AskUserQuestion
schema, gate bypass levels, headless mode, the green base check, the delegation
contract, the judgment contract, the goal-check judge tier, verify scope, and
native Codex subagent tending.

## Question Format

Used at all gates and Socratic steps:

```text
CONTEXT: <one or two sentences of relevant state, plain language>
QUESTION: <one outcome-framed question>
RECOMMENDATION: <the option the evidence favors, and why in one line>
  (a) <option> — <expected outcome>
  (b) <option> — <expected outcome>
  (c) <option> — <expected outcome>
```

One decision per question. Independent questions may share one `AskUserQuestion` call (up to 4 — `gates-and-delegation.md`, "AskUserQuestion"); dependent ones go one at a time. Never answer your own question.

## File Quick Reference

```text
.bee/
  onboarding.json  state.json  config.json  HANDOFF.json
  reservations.json  decisions.jsonl  backlog.jsonl
  capture-queue.jsonl                                 ← settlement stubs awaiting their flush
  cells/<id>.json  logs/hooks.jsonl  .inject-cache.json
  bin/  bin/lib/

docs/history/<feature>/
  CONTEXT.md  reports/                                ← always
  plan.md                                              ← frozen at Gate 2: standard/high-risk
                                                        always; small opt-in; tiny/spike none
  discovery.md  approach.md  implement-plan.md        ← conditional: separate files only for
                                                        L2+ discovery / high-risk; else folded
                                                        into plan.md sections
  walkthrough.md                                      ← standard/high-risk, post-Gate-3

docs/history/learnings/
  critical-patterns.md  YYYYMMDD-<slug>.md

docs/knowledge/                                       ← state layer when a bundle exists (bundleMode)
  areas/<area>/  index.md  <subject-slug>.md   patterns/  work/<id>/

docs/specs/
  <area>.md  reading-map.md                            ← read-only compat surface when a bundle exists;
                                                          state layer itself when no bundle

.bee/spikes/<feature>/
```

## Helper CLI Quick Reference

`bee` = `.bee/bin/bee` (run from the session's cwd, never via PATH or a Node script); `bee <cmd> --help` (`--all` / `--json` / `--names`) is the command reference.
