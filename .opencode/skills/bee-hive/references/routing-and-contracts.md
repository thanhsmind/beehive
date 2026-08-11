# Routing And Contracts Reference

Open this when the compact bootstrap in `SKILL.md` is not enough.

## Skill Catalog

| # | Skill | One-line description | Load when... |
|---|-------|----------------------|--------------|
| 1 | `bee-hive` | Routing, go mode, gates and the bypass level, red flags. | Starting any session; setting or checking gate bypass |
| 2 | `bee-shaping` (Explore/Qualify/Lock) | Identify gray areas or triage a backlog item unattended; lock decisions into `CONTEXT.md`. | Feature request is vague or new; a backlog item needs its first triage pass |
| 3 | `bee-planning` | Research, mode gate, approach, unified plan, current-slice cells; the SMALLER PATH reality check and the review wave run inline before its merged Gate 2. | Decisions are locked, or scope is already clear |
| 4 | `bee-swarming` | Launch and tend bounded workers with reservations. | Gate 2 approved (merged shape+execution) |
| 5 | `bee-swarming` ("Execute") | Bounded worker loop for one cell. | Spawned by swarming |
| 6 | `bee-reviewing` | Parallel review gate with P1/P2/P3 findings, user-invoked over a scope the user chooses. | User explicitly requests review — never automatic after a final slice or feature close |
| 7 | `bee-capturing` | Knowledge capture: BA-grade area specs (sync, capture, harvest) plus durable learnings and decisions. | Execution done; documenting any area (UI/API/job); a settled outcome must be kept; work abandoned with lessons |
| 8 | `bee-grooming` | Entropy audit, debt hunt, approved kills. | Cleanup/audit requested; hive idle |
| 9 | `bee-researching` | Evidence-labeled research scout. | Research a topic/library/approach; planning discovery L2/L3 |
| 10 | `bee-herding` | Cockpit roles: bootstrap, dispatch, merge. | Human invokes the cockpit, or the control loop runs one iteration |
| 11 | `bee-shaping` (Brief) | Render the one human-readable implement plan per feature, and the post-Gate-3 walkthrough (consolidator, not planner). | Planning shaped `small`+ work; a feature's implement plan needs (re)generating; a `standard`/`high-risk` feature passed Gate 3 |

Gate bypass is set from `bee-hive` (Gates); developing bee itself
(authoring skills, the self-improvement loop) is maintainer territory in
the bee source repo's handbook, never product routing in a host repo.

## First-Skill Routing

| Request type | First skill | Notes |
|---|---|---|
| Vague/new feature | `bee-shaping` (Explore) | Always start here if gray areas exist |
| Research a topic/library/approach (no feature underway) | `bee-researching` | Standalone brief; suggests shaping or planning as next step |
| (Re)generate or read a feature's implement plan or walkthrough | `bee-shaping` (Brief) | Consolidates the truth artifacts into `docs/history/<feature>/implement-plan.md`, any phase; writes `walkthrough.md` post-Gate-3 for `standard`/`high-risk`; renders nothing for `tiny`/`spike` |
| Research inside a scoped feature | `bee-planning` | Discovery L2/L3 invokes `bee-researching` in-chain |
| "Just fix this" / small change | `bee-planning` | Route in tiny or small mode |
| Review code | `bee-reviewing` | Load directly — only on an explicit review request; never automatic after execution completes |
| Document a screen/API/job/area; keep a settled outcome (rule agreed, behavior confirmed, value tuned); spec a legacy area; capture learnings | `bee-capturing` | Load directly, any phase — capture never waits for feature close |
| Clean up / tech debt / audit | `bee-grooming` | Load directly |
| Drive the cockpit (bootstrap/dispatch/merge) | `bee-herding` | Load directly |
| `/go` / full pipeline | Go mode | See `go-mode.md` |
| Turn gate-bypass on/off, widen it, or check it | `bee-hive` (Gates) | Any phase; the agent sets `.bee/config.json` `gate_bypass` on the user's instruction |
| Resume session | Resume logic | Check `HANDOFF.json` first — kind-aware: pause waits, planned-next adopts only at a fresh-session boundary |
| Explicit request to run the automatic backlog-triage pass on a `docs/backlog.md` row (a human or an external caller invoking the pipeline path directly — no auto-trigger exists yet) | `bee-shaping` (Qualify) | Pipeline path, explicit invocation only |
| Docs/spec/README/sample-only change | docs lane | "Docs lane" under Lane ceremony in full — announce, write, format-check, capture or "nothing settled"; no pipeline |
| Merge/ship/release request while unreviewed or stale candidates exist | Report the candidate count + risk level, then ask ONE question: "Create a review session for this scope?" | Only an explicit yes dispatches `bee-reviewing` — never spawn a reviewer silently |

**Surface-scope-earlier check** (runs before routing to `bee-shaping`): the request contains concrete acceptance criteria AND references to existing patterns → offer "Found clear requirements. Jump straight to planning, or explore alternatives first?" On approval, planning receives a one-paragraph scoping synthesis whose decisions still carry D-IDs.

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

**Planned-next** — the previous cell was finished with the declared tests green and the next cell was already claimed for this handoff. Adoption fires ONLY at a fresh-session boundary (a cleared or newly started session — never a resumed or memory-compacted one, which follows the pause path above):

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

This is the procedure behind the 65%-context handoff rule (AGENTS.md,
"Care for the session"): that rule says when to stop, this tree says
which move to make.

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
| `docs` | none — announce one line | format check (parse/lint if applicable) | direct, in-session | none | 0 |
| `tiny` | none — the cell is the micro-plan | SMALLER PATH check inline, 0 ceremony subagents (I/O-offload workers exempt — Delegation contract) | inline in the orchestrator session (cap discipline and done-report unchanged), or one dispatched execution worker at the orchestrator's option (when dispatched, the execution-worker contract applies: param-carrying dispatch, model param or pinned type, never a bare marker; standard worker prompt template, no reviewers/panels/waves) | orchestrator-authored done-report (worker's verbatim diff + commit; `bee finish` ran the declared tests — the result record is the evidence; orchestrator re-runs only on smell or hard-gate) — verification, not independent review | 1 — the merged shape+execution gate |
| `small` | logged scoping synthesis; plan.md is opt-in | SMALLER PATH check inline, 0 ceremony subagents (I/O-offload workers exempt — Delegation contract); spike only if a blocking assumption demands it | one dispatched execution worker (same contract as `tiny`'s Execute column), its 1-3 cells dispatched in PARALLEL when disjoint (see Concurrency law in full below) | orchestrator-authored done-report, self-checks only, no auto reviewer (the correctness reviewer moves inside an on-demand review session) | 2 — merged shape+execution gate, self-checks close-out |
| `standard` | full `plan.md` | SMALLER PATH check + merged reviewer; ≤5-file diff (0 hard-gate flags): inline self-review, no dispatch | swarm workers | on user request only: session panel scaled to scope risk (4 core reviewers) | 2 — Gate 1, Gate 2 (merged shape+execution) |
| `high-risk` | `plan.md` + brief | SMALLER PATH check + persona panel | swarm workers | on user request only: session panel scaled to scope risk (full wave + conditionals) | 2 — Gate 1, Gate 2 (merged shape+execution) |

**Gate 3 is additive, not counted above:** it is asked once, whenever a review session actually runs for that scope — never automatically at the end of a lane's default chain.

### Concurrency law in full

**THE LAW:** if pieces of work can run at the same time, open the threads and run them; serial only when forced. One rule, three tiers — gather work fans out to I/O workers (Delegation contract, `gates-and-delegation.md`), a slice's cells fan out to a wave whenever their product file sets are disjoint (reservations are the proof and the police, 3-4 live workers is the cap), and independent ready features fan out to lanes or worktrees (Lanes, first-class below). Undeclared-overlap concurrency for the same feature is a `standard`/`high-risk` wave shape wearing a `small` lane, the exact ceremony-mismatch red flag this lane scaling exists to catch.

**MANDATORY CONCURRENCY PLAN:** before dispatching anything, the orchestrator states in one line what runs concurrently and what is forced serial and why — computed, not guessed, never assumed by default. WAIVED when exactly one cell is being dispatched (a one-worker plan states nothing); owed again from two cells up. Cells: `bee cells schedule` names the disjoint sets from declared file overlap; a real product-file conflict named in the dispatch note is what makes a cell wait, nothing else does. Features: the declared `--paths` on `state start-feature --as-lane` are checked against every other live session's claims/reservations before the lane starts — a refusal names the holder and is itself the plan's proof that the paths were not disjoint.

**THE ONLY LEGAL REASONS FOR SERIAL, exhaustive:** a declared file-set overlap (including a shared generated artifact not deferred by a wave barrier), a true data dependency (`deps`), a single scarce external resource, or an explicit human instruction. Nothing else is a reason — anything else fans out.

**LANES, FIRST-CLASS:** before every feature start, check whether other ready feature work has disjoint declared paths — if so, the paved road is a lane, not a queue, whether or not another feature is already live — `bee state start-feature --feature <f> --mode <m> --as-lane --paths <declared>`; lane-scoped mutations take `--lane`. Lanes classify and coordinate; they no longer keep code in main — a code-touching feature branches into its own worktree at feature start regardless (worktree-first — `docs/knowledge/areas/worktree-parallelism/routing-and-visibility.md`), its declared paths still coordinating through the shared store; only docs-lane and solo tiny work runs directly in the main checkout. A lane refusal (holder + expiry) means the paths were not disjoint after all — pick other ready work or wait for the hold to lapse — never work around it.

**TICK:** the concurrency plan emits its own progress line per the Progress ticks catalog (`scout-and-ticks.md`, "Progress ticks — worked examples") — same silent-bookkeeping rule as every other tick, never suppressed by bypass.

Full doctrine for the cell/wave tier — the wave-barrier regen protocol and the execution-worker class relationship: `bee-swarming/references/swarming-reference.md` ("Single execution worker in full", "Default — parallel"), and the Delegation contract (`gates-and-delegation.md`).

### Docs lane

The change is knowledge upkeep, same class as capture — announce one line ("docs lane: writing X"), write it, run a format check when one exists (JSON parses, markdown lints), then close by logging a decision/capture stub when the content encodes a settled outcome, or stating "nothing settled" when it does not — a docs-lane close with neither is not a close. No cells, no gates, no reviewers. If the target path is outside the write-guard allowlist (`.bee/, docs/, plans/, AGENTS.md`) the hook will block the idle write — fall back to the tiny fast path instead of fighting the guard.

### Tiny/small fast path

The draft cell(s) are rendered as a **preview inside the gate message** — never persisted first — and the 2-minute reality check runs inline against that preview, before the shape and execution approvals are presented as **one merged question** — "Work shape + execution: I'm about to do X via Y, verified by Z. Approve?" — approval records both `shape` and `execution` and covers exactly the previewed work packet. `cells add` runs only **after** approval, and the cells are claimed only then — previewed before persist, never persist-then-preview. Implementation runs inline in-session for `tiny` (the merged gate, cap discipline, and done-report are unchanged; dispatching stays legal when the orchestrator prefers it), and through the one dispatched execution worker for `small`. After execution (worker return or inline finish): no separate merge gate — the orchestrator authors the done-report itself from the worker's verbatim diff plus its commit (`bee finish` ran the declared tests; a re-run only on smell or hard-gate) and that done-report (diff + commit + capture line) closes it, once `bee close` re-runs the declared tests green for the feature (`bee-swarming/references/swarming-reference.md`, "Tests at finish and close, in full"). A real problem found during the orchestrator's own review stops and asks, always.

### Capture discipline

Lanes scale ceremony, never memory — zero exceptions, the docs lane and non-cell quick work included: a feature whose capped cells include `behavior_change` owes ONE `bee-capturing` spec sync covering all of them — tiny lanes included — recorded as PENDING at close and run deferred, at the owner's pace, batching several closed features into one session when cheaper (decision c8e25271; `bee orient` carries the reminder until it runs). A settled discussion outcome (rule, behavior, tuned value; backend or frontend alike) is still captured the moment it settles — deferral applies to the close-time sync, never to same-turn settlement capture. Every task close carries either a decision-log/capture-stub line or an explicit "nothing settled" statement — a close with neither is not a close. **Settlement detection is the agent's duty, unprompted:** the routing row "user asks to document" is the fallback, not the norm — the norm is the agent noticing "this just settled", announcing it in one line, and capturing in the same turn without being asked. What same-turn capture costs is lane-scaled: high-risk = full spec sync inline; every other lane = decision log + a one-line capture stub (`bee capture add`), with the full merge at a flush point (wrap-up, PreCompact warning, or next session's offer). Capture writes only `docs/` + `.bee/` — no gate applies.

## Chaining Contract

| Skill | Reads | Writes |
|-------|-------|--------|
| hive | onboarding, state, HANDOFF, critical-patterns, decisions | state routing updates only |
| shaping (Explore/Qualify/Lock) | user conversation, backlog row, critical-patterns, quick scout | `docs/history/<feature>/CONTEXT.md` (lock or park), backlog row status, state update |
| planning | CONTEXT.md, critical-patterns, active decisions, bee_status | `approach.md`, `plan.md` (frozen at Gate 2 — approval stamp only after approval; none for `tiny`, opt-in for `small`), current-slice cells via `bee cells add` |
| shaping (Brief) | CONTEXT.md, approach.md, frozen plan.md + cells (drift re-render triggers on cell changes only, since the plan cannot drift after approval), test-result records (`.bee/logs/test-results.json`), state gates (render/refresh); capped cell traces, review findings, UAT (walkthrough) | `docs/history/<feature>/implement-plan.md` (projection; `high-risk` always, `standard` on-demand, `small` optional on request); `docs/history/<feature>/walkthrough.md` (post-Gate-3; `standard`/`high-risk`) |
| swarming (orchestrate) | Gate-2-approved cells, state, reservations | worker registry in state, HANDOFF at ~65%, wave results |
| swarming ("Execute") | assigned cell, CONTEXT.md, reservations | implementation commits (one per cell, cell id in message), finish (runs the declared tests; the result record is the evidence), report in `docs/history/<feature>/reports/` |
| reviewing | user-selected immutable scope (a `bee_reviews` session — never triggered by phase or cell completion) | session findings (P1/P2/P3) and the Gate 3 decision recorded on that session, backlog items, `residual-findings.md` fallback |
| capturing | `behavior_change` cells + test-result records, CONTEXT.md, active decisions, UAT/worker reports, feature history, traces, commits, code + user interview (harvest) | with a bundle: `docs/knowledge/areas/<area>/` concepts (BA-grade merge); with no bundle: `docs/specs/<area>.md` (BA-grade merge), `docs/specs/reading-map.md`; plus `docs/history/learnings/YYYYMMDD-<slug>.md`, critical-patterns promotions, decision log entries, backlog friction, state record |
| grooming | entropy inputs, backlog, traces, diffs | kill proposals, tiny/small cells, outcome records |

**Recommended-next after execution:** once a feature's execution work is done, the recommended next action is LANDING (`bee worktree merge` from main); capture is recorded as pending — `bee_status`/`bee orient` report the pending capture alongside the review-candidate count, and neither `bee-capturing` nor `bee-reviewing` is chained into automatically. Both run deferred, on the owner's call, over any scope the owner names. The feature closes truthfully `unreviewed` (and `uncaptured` until Compound runs); the reminders stand until each is settled.

Every skill ends with an explicit handoff: `[Outcome]. Invoke bee-<next-skill> skill.`

## Direction of Truth — Projection Rule

The repo artifacts are the single source of truth for what work exists and its state: **cells** (`.bee/cells/`) for in-flight execution and the **PBI rows** in `docs/backlog.md` for product intent. A session's todo list — `TaskCreate`, `TodoWrite`, and any equivalent scratch checklist — is an **ephemeral projection** of those durable records, never the reverse.

The mapping is one-way: cells and PBI rows generate the session todo list, and no edit to that list ever writes back to a cell or a backlog row. When the two disagree, the repo artifact wins and the session list is regenerated from it. A todo item with no cell or PBI behind it is a projection bug, not a new unit of work — file the cell or the backlog row first, then let the list re-derive. This keeps the durable layer authoritative and the chat/session state disposable.

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
- **Body** is the work itself. Prose narration stays within ~5 lines per turn.
  Progress ticks are not prose and do not count against that budget — they are one
  fixed-format line per step ("Progress ticks"). The complete record (reports,
  findings, matrices) lives in a linked file, never pasted into chat.
- **Close** with exactly one next action: the agent's own next move, or the one
  thing only the user can decide. Never a menu of maybes.

**Five rules.** These are the ones a message can actually violate:

1. **Pre-send check.** Reading only the first and last line must answer (a) what
   just happened and (b) what happens next.
2. **Evidence before claims.** "done", "green", "fixed" appear only beside fresh
   output in the same message.
3. **One question at a time**, formatted apart from progress text, phrased so the
   user can restate what they are deciding in their own words ("Question Format";
   the Gate Presentation Contract is the template). A question buried in a
   progress paragraph does not count as asked.
4. **A red or a refusal is never silenced** — not by a quiet switch, not by a
   bypass level, not by "I'll mention it at the end".
5. **Work language**, with the two exemptions below.

One standing exception: a destructive or irreversible action gets full explicit
clarity — safety beats brevity, always.

**Everything else is craft.** Open each unit with what is being done and why;
give concrete units for anything over a minute ("verify ~2 min", never "a
while"); make a win runnable by naming the command or path — name a doc by
its bare repo-relative path (`docs/...`), never a viewer URL; state a failure as
cause, fix, and who acts, quoting the shortest decisive line; file a tangent and
mention it once at the close; let the work be the subject of every line, with
ids and counts trailing as handles or standing beside a claim as evidence, never
leading and never as achievement statistics. Protocol and record surfaces —
worker status tokens, cap traces, decision logs, CONTEXT.md — keep their ids,
because that is where ids live.

One example teaches this better than the list does:

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

### Work language — a vocabulary rule, not a silence rule

Bee is bookkeeping, not the deliverable, so chat speaks the user's work language:
"fixing the login redirect", "tests pass" — never "capped cell auth-3" or "phase
is now swarming". What this constrains is the VOCABULARY, not whether a step is
mentioned at all. Every perceivable step still gets its progress tick ("Progress
ticks"); the tick names what happened to the WORK rather than what happened to the
record — `✓ capped: tick catalog rewritten`, not `✓ capped cell vt-3`.

Bee vocabulary may lead a line in exactly two cases:

1. the user asks about bee itself (state, cells, workflow) — answer plainly, in their language;
2. a gate genuinely needs their decision — and the Gate Presentation Contract already requires that question in work terms, not bee terms.

Litmus: strip every bee term out of a chat message; if nothing the user needs is
lost, those terms should not have been there.

### The agent runs the machinery, not the user

Every bee command (`bee orient`, `cells`, `reservations`, `decisions`, onboarding, cell verify
commands) is run by the agent itself the moment the workflow calls for it — never printed for the
user to execute, never "run this and tell me the output". The only human actions in bee are gate
approvals, decision answers, and privacy approvals. `AGENTS.md` states the same law inside its
workflow boundaries and defers here for the full form; `SKILL.md`'s Hard rules list carries the router-side pointer.

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

One question per message. Never bundle. Never answer your own question.

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

`.bee/bin/bee <group> <verb>` is the sole canonical form — the vendored
binary onboarding installs into the repo, invoked by its repo-relative path
from the session's cwd. Prose elsewhere writes it `bee <group> <verb>` for
readability; that always means this same binary, never a PATH lookup and
never a Node script. `bee --help` prints the porcelain flow surface;
`bee --help --all` prints the full registry (`bee --help --json` / `--names`
give the same two surfaces machine-readably) — the help output is the command
reference, not this file. Legacy `bee_*.mjs` shims do not ship;
`LEGACY_HELPER_RE` in the write-guard stays only as a transition guard for
hosts mid-upgrade.
