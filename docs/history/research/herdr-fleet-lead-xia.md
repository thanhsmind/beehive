---
artifact_contract: bee-research/v1
topic: herdr-fleet-lead-xia
depth: standard
date: 2026-09-08
mode: xia
---

# Xia: the herdr "25-agent fleet" lead, versus bee herding

## Bottom Line

- **Recommendation (ladder rung): reuse (rung 1) for every mechanism, plus
  adapt-upstream (rung 3) for one discipline — the routing rules.** No code
  can be ported anyway: the fleet layer itself (`fleet-watch`, `fleet-done`,
  `fleet-track`, `fleet status|assign|review|followup|land`) is the author's
  own unpublished shell scripts. Neither public repo in the ask contains it.
  `Upstream`
- **What the two named repos actually are.** `hmans/beans` is an *issue
  tracker* — markdown beans in `.beans/`, a Go CLI, a GraphQL API, a Svelte UI
  with worktrees and Claude Code chat panes. It is not the orchestrator.
  `herdrdev/herdr` is the *pane runtime* — and bee already runs on it
  (`herding.transport: herdr`, `bee herding pane …`). `Upstream` + `Local`
- **The one real gap: bee has an inbox, and no router.** bee's asynchronous
  inbox already exists — `bee herding run --inbox-session <token>` drops
  `.bee/result-inbox/<token>/<job-id>.json` before the pane splits, and a
  drain injects the finished header into a session. But it is **Pi-only**
  (`.pi/extensions/bee-guard.ts`), **advisory**, and it delivers to *a human's
  session*, never to a rule. Every other path is synchronous: `bee herding
  run` blocks the leader, and the dispatch role starts one worker per
  iteration and never speaks to it again. The trick of the source is the
  second half — a **routing watcher**: DONE routes to review, CHANGES routes
  to a followup lane, an unrouted DONE goes to a retry lane. bee's ledgers
  (`.bee/wave-ledger.jsonl`, `.bee/logs/dispatch.jsonl`) have no consumer that
  routes off them. `Local`
- **Everything else the thread claims, bee already has — usually stronger.**
  Work-is-a-file, brief-as-a-file, guards before dispatch, occupancy from a
  ledger not a status column, "status is a lie", one human lands, mined
  retro metrics with pass/fail bands. See § Dependency Matrix: 13 rows
  `EXISTS` (several stronger than the source), 1 `PARTIAL`, 5 `NEW`, 2
  deliberate `CONFLICT`.
- **Why the next-best rung lost.** Rung 4 (build a fleet engine) would
  duplicate `bee herding occupancy`, `control-loop`, `wave`, `supervisor` and
  the cell store. Rung 2 (built-in) is closed: no herdr verb routes outcomes —
  herdr moves panes, it does not hold a work queue.
- **Confidence: 85%.** Local claims are anchored to files and to live `--help`
  output from the installed binary. The 15% is the fleet layer itself: it is
  described only by the author's thread and two slides, never read as source.
- **Suggested next step: `bee-shaping`** — item 1 below is a feature; item 2
  contradicts a locked rule and is the user's call, not mine.

## Source Manifest

| Field | Value |
|---|---|
| Repo | `github.com/hmans/beans` |
| Ref | `main` |
| Resolved commit SHA | `99260bf1a6bec3e395b629406b737f6418653a17` ("fix: be more explicit about including modified beans in commits") |
| Narrowed scope | `README.md`, `CLAUDE.md`, `.beans.yml`, `.beans/*.md` (the bean file format), package layout |
| Second source | `herdrdev/herdr` — public, Rust, 2 public repos, "the runtime your coding agents live on". Metadata only; bee already depends on it. |
| Third source | The user's 13-point thread + 2 slides. **Unpublished** — the `fleet-*` scripts exist in no repository I can read. Treated as `Upstream` testimony, not as code. |

Mode **`xia`** — port-protocol steps 1–4, no challenge pass, nothing built.

## Question & Assumptions

- **What was asked:** read `hmans/beans`, see how a lead coordinates many
  agents in many roles, and say how bee can do the same through herding.
- **What success appears to mean:** bee's leader dispatches many agents in
  distinct standing roles, and the work routes itself between them —
  coder → reviewer → followup → land — without the human relaying each step.
- **Assumptions still needing confirmation:**
  - That the user wants routing *automation*, not just more seats. The thread's
    value is the watcher, not the head count.
  - That `agents-review-user-invoked` (a locked bee rule) may be revisited.
    I did not assume it may.

## Findings

### Local — bee's fleet is real, and it is deaf

**What exists and works.**

- 25 `bee herding` verbs are built, including `pane`, `agent-start`,
  `pane-id`, `result`, `run`, `wave`, `occupancy`, `record-worker`,
  `control-loop`, `interlock`, `classify-lane`, `interrupt`, `cancel`.
  (`.bee/bin/bee herding --help`) `Local`
- **A real dispatch door with 18 role slots per runtime.** `.bee/config.json`
  `models.<runtime>.<role>`: `code`, `read`, `test`, `docs`, `plan`,
  `extraction`, `generation`, `review`, `advisor`, `supervisor`, `lane-1..3`,
  and five `hat-*` seats. Each names a registry agent. `Local`
- **Five harnesses already registered.** `herding.agents`: `claude-sonnet`,
  `pi-opencode-free`, `pi-gpt-5.6-luna`, `pi-gpt-6-astra`, `agy-flash`.
  The multi-harness fleet the thread describes is already configured here.
  `Local`
- **Guards before dispatch, four of them, checked fresh every iteration** —
  ready (PBI carries a `feature` slug AND `docs/history/<slug>/CONTEXT.md`
  exists), `in-flight`, no worktree grant, zero cells — plus a two-key
  lane-safety filter where the classifier AND the agent's own reading must
  both say safe. (`skills/bee-herding/references/role-dispatch.md:230-320`)
  `Local`
- **Occupancy from a ledger, never from a status column**, and it distinguishes
  a real answer from a degraded one: `{count, source: "live"|"fallback"}`,
  cap 4, and `fallback` is a *refusal to dispatch*, not a guess.
  (`role-dispatch.md:100-140`) `Local` — this is the thread's point 6 already
  implemented, and implemented harder.
- **"Status is a lie" is already doctrine.** `agent_status`/`agent_session` are
  read "for exactly one purpose in this role — spotting an anomaly… They are
  never evidence that a working agent or its item has finished."
  (`role-dispatch.md:187-192`) `Local`
- **One human lands.** `bee worktree merge` refuses on proof debt before
  `git merge --no-ff` is attempted, refuses on an unapproved uat gate, runs
  through a shared integration queue, and cleans up on green. Merge is an owner
  gesture; the dispatch loop never merges. `Local`
- **Mined, counted retro metrics already exist** — `bee supervisor metrics`:
  seven derived counters, each with a two-sided band, sample count, and a
  `not-measurable` verdict that is never rendered as zero. Nothing is
  self-reported. `Local` — this is the thread's point 10, already built.

**What is deaf.**

- `bee herding run` **blocks**: it polls natively for `result-N.json` and
  returns to the caller. Good for a leader-driven wave; it cannot be the shape
  of 25 agents reporting in whenever they finish. `Local`
- **An asynchronous inbox does exist, and it stops one step short.**
  `--inbox-session <token>` writes a pending marker at
  `.bee/result-inbox/<token>/<job-id>.json` *before* the pane splits; the Pi
  extension drains it and injects the result header (`job_id`, `cell_id`,
  `status`, `summary`, `proof`, `report_path`) into that session. Three limits
  are written into its own contract: delivery is **at-least-once** with
  `job_id` as the dedupe key, the drain runs **only while that Pi session is
  live**, and the report body never rides the injection.
  (`docs/config-reference.md:184`, `.pi/extensions/bee-guard.ts:534-650`)
  It is a *notification* channel to one human's session — not a queue a rule
  can act on. `Local`
- The dispatch role "only STARTS work… never speaks to it again"
  (`skills/bee-herding/SKILL.md`). There is no second half. `Local`
- **The control loop has three roles, all of them already cold and
  stateless** — `dispatch`, `merge`, `supervisor`, each with its own interval
  (60 s / 60 s / 900 s) and its own enumerated `--allowedTools` surface
  (`packages/bee-rs/crates/bee/src/herding/control_loop.rs:72-94,277`). A
  fourth role is an addition to a working pattern, not a new subsystem.
  `Local`
- **Nothing consumes the ledgers.** `.bee/wave-ledger.jsonl` is read only by
  `bee herding occupancy`, and `wave-runs.md:100` records that "the ledger
  only grows. Nothing sweeps it and no row is marked resolved when its wave
  ends." `.bee/logs/dispatch.jsonl` is audit only. `Local`
- **No cross-model review rule.** I searched `skills/`, `AGENTS.md` and
  `docs/knowledge/` for any rule that a model must not review its own output.
  There is none. Worse than absent: when the `review` slot is unconfigured,
  `tier_role_list("review")` falls through to `generation` —
  *the same model that wrote the code*
  (`packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:666-671`). In
  **this** repo the two happen to differ (`claude.review` → `opus`,
  `claude.code` → `agy-flash`), but that is configuration luck, not a rule.
  The nearest real neighbours are `lane-model-diversity D1` (three blind
  design lanes may point at different models) and an *optional* cross-model
  second opinion at Gates 2 and 3. Review itself is locked to user invocation
  (`agents-review-user-invoked`, `AGENTS.md:23-26`). `Local`
- **The work queue and its refusals are already fleet-grade.**
  `bee cells claim-next` sweeps expired claims, walks other sessions' approved
  lanes by backlog rank, and publishes the claim atomically with an `O_EXCL`
  hard link. Eleven named refusals guard it, including `RED_BASE` (never build
  on red), `FOREIGN_HOLD` / `FILE_RESERVATION_FAILED` (another worktree owns a
  declared file), an unapproved execution gate, uncapped dependencies, and
  budget rations.
  (`packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs:1805-2005`)
  `Local`
- **An agent cannot push at all.** The write guard's intake gate: "git push is
  outward-facing and is never exempted from this gate, regardless of what it
  would push."
  (`packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:855-862`)
  The thread's "no agent ever pushes" is a convention there; here it is a
  hook that fires. `Local`
- **No pre-push hook.** `.git/hooks` carries no live hook and `core.hooksPath`
  is unset; the declared test command runs in CI on push, and
  `bee worktree merge` "never spawns commands.test itself". `Local`

### Upstream — what beans contributes, and what it does not

- **beans is an issue tracker, agent-first.** Beans are markdown files with
  YAML front matter (`title`, `status`, `type`, `priority`, `order`, `parent`)
  in `.beans/`, versioned with the code. The agent reads them through
  `beans prime` wired to a `SessionStart` hook. `Upstream`
- **Its real architectural idea is the same one bee already holds**: the
  control plane lives on main, and a worktree's view is a *dirty overlay*
  merged in at runtime — "`beans-serve` holds runtime state as the
  authoritative view… merges in changes from worktrees… status changes are
  runtime-only until the PR merges." (`beans/CLAUDE.md`) `Upstream`
- The thread's "23 bean files deleted on one branch, invisible for hours" is
  the failure that rule prevents. bee's equivalent guard already exists and is
  *stricter*: `bee dispatch prepare` refuses outright inside a granted feature
  worktree, naming the reason. `Local`
- **beans contributes nothing on roles or routing.** It has no reviewer, no
  judge, no lead, no watcher, no verdict grammar. Its `agent.default_mode`
  is `act|plan` and its worktree `integrate` is `pr`. `Upstream`

### Upstream — the fleet layer, from testimony only

The routing rules, verdict grammar, credit guard and artifact-liveness checks
in the thread live in shell scripts the author has not published. They are
recorded here as *design claims*, each mapped in the matrix below. No line of
them was read as source. `Upstream` (testimony)

### Inference

- The head count is not the mechanism. 25 seats work because the lead never
  waits on any of them: an announcement is a file drop, and the tick that
  routes it is 60 seconds long and stateless. bee's cap of 4 is a *concurrency*
  limit on disjoint-file writes, and is orthogonal — bee could route 25 seats
  and still hold 4 writers. `Inference`
- bee's gap is therefore narrow and deep, not wide: **one inbox and one router
  role**, not a new fleet subsystem. `Inference`

## Dependency Matrix

| # | Component (source) | Local answer | Verdict | Evidence |
|---|---|---|---|---|
| 1 | Work is a file on main (`.beans/*.md`) | `.bee/cells/*.json`, backlog PBIs, `docs/history/<slug>/CONTEXT.md` — plus gates and lanes beans has none of | `EXISTS` (stronger) | `Local` |
| 2 | "A bean not on main doesn't exist" | `bee dispatch prepare` refuses by name inside a granted worktree; control plane is main-only | `EXISTS` (stronger) | `Local` |
| 3 | Brief is a file + a one-line pointer | `.bee/mailbox/<job-id>/brief-N.txt`, `--task-file`, `--expertise` | `EXISTS` (stronger) | `Local` |
| 4 | `fleet-done <agent> <bean> DONE\|BLOCKED\|CHANGES "<line>" <report>` | `result-N.json`, `bee cells finish --report` carry the same content | `EXISTS` as content | `Local` |
| 5 | An **inbox** the announcement lands in, that nobody blocks on | `.bee/result-inbox/<token>/<job-id>.json` via `--inbox-session` — but Pi-only, advisory, at-least-once, and it injects into a *human's session* | **PARTIAL** — the store exists, the consumer is a human | `Local` |
| 6 | A **watcher** that routes inbox rows every tick | `bee herding control-loop` has three roles — `dispatch`, `merge`, `supervisor` — that start, retire, and observe. None routes | **`NEW`** (a fourth role on a working pattern) | `Local` |
| 7 | coder DONE → review on a **different harness** | no rule, no code; the `review` slot even falls through to `generation` when unset | **`NEW`** + `CONFLICT` with `agents-review-user-invoked` | `Local` |
| 8 | reviewer CHANGES → followup lane, served before the queue | `bee cells reopen`, `dissent-verdict`, review findings exist; the priority lane does not | **`NEW`** (partial base) | `Local` |
| 9 | unrouted DONE → retry lane, never dropped | ledger rows are never resolved; `unverifiable_after_send` is terminal | **`NEW`** | `Local` |
| 10 | Guards: on main · status open · not in flight | the four §5 conditions, checked fresh each iteration | `EXISTS` (stronger) | `Local` |
| 11 | Guard: harness has credits | none — a spent agent is dispatched to like any other | **`NEW`** | `Local` |
| 12 | Announcement for a bean you're not on → STALE | no announcements exist to be stale | n/a until #5 | `Inference` |
| 13 | Free = no task file, never the status column | `bee herding occupancy`, ledger × live panes, `live`/`fallback` split | `EXISTS` (stronger) | `Local` |
| 14 | Judge by artifacts: commits since dispatch · last-commit age · report mtime · 15-min flag | §4 anomaly scan uses `foreground_cwd`, the four finished conditions, and refuses `agent_status` as evidence — but has no commit-age or mtime metric | `EXISTS` in doctrine, **`NEW`** in measurement | `Local` |
| 15 | No agent pushes; one lead lands | merge is an owner gesture; and the write guard refuses `git push` outright — "never exempted from this gate, regardless of what it would push" | `EXISTS` (stronger: a hook, not a convention) | `Local` |
| 15b | Refill a free coder from the head of the queue | `bee cells claim-next` — expired-claim sweep, cross-lane work-stealing by backlog rank, `O_EXCL` atomic claim, 11 named refusals (`RED_BASE`, `FOREIGN_HOLD`, unapproved execution gate, uncapped deps, budget rations) | `EXISTS` (much stronger) | `Local` |
| 16 | Lead cherry-picks, builds, `merge --no-ff`, pushes; pre-push hook builds again | `merge --no-ff` yes; proof is *recorded*, not re-run; CI runs the suite on push; no pre-push hook in this repo | `CONFLICT` (deliberate) | `Local` |
| 17 | Reviewer cherry-picks the named commit onto main and builds it | bee reviewers are read-only; they may run read-only commands, never rebuild on main | `CONFLICT` (deliberate) | `Local` |
| 18 | Verdict grammar `MET\|UNMET / LANDABLE / APPROVE\|CHANGES` | `bee cells dissent-verdict` is a closed 3-verdict set with a required reason; `bee reviews record --kind decision` | `EXISTS` (different shape) | `Local` |
| 19 | Retro mined from artifacts, each class gets a failable metric | `bee supervisor metrics` — 7 derived counters, two-sided bands | `EXISTS` (stronger) | `Local` |
| 20 | "If a rule can't fire, it isn't a rule" | write-guard and model-guard hooks, typed refusals, `bee mailbox reflect` | `EXISTS` (same doctrine) | `Local` |
| 21 | 25 seats across 5 harnesses | 5 harnesses registered, 18 role slots; concurrency cap 4 | `EXISTS` for the registry; cap is a separate, named decision | `Local` |

## Cross-Cutting Sweep

Wiring outside the herding folder that any of the `NEW` rows would touch:

- **`gate_bypass`** — dispatch refuses below `full`. An inbox router that
  reopens cells or starts reviewers inherits the same refusal. `Local`
- **The enable interlock** (`.bee/tmp/bee-herding.enable`) — the owner's
  durable "yes, run". A router must sit behind it too, or it becomes a second
  door with no owner. `Local`
- **`agents-review-user-invoked`** in `AGENTS.md` and `skills/bee-reviewing` —
  row 7 contradicts it head-on. `Local`
- **The 4-slot occupancy cap** — a router that spawns reviewers competes for
  the same slots as coders. `Local`
- **Ledger growth** — `wave-runs.md:100` already flags unbounded growth; an
  inbox that is *read* every tick makes that a real cost, not a note. `Local`
- **The uat gate and the integration queue** — untouched by any row here;
  landing stays where it is. `Local`

## Risks, Unknowns, Follow-Ups

- **Row 7 contradicts a locked rule.** Automatic review after a worker DONE is
  exactly what `agents-review-user-invoked` forbids. Recorded here with its
  evidence; superseding it is the user's move, not this brief's.
- **Rows 16 and 17 are deliberate bee positions, not gaps.** bee trusts a
  *recorded proof line* plus CI; the source builds twice locally because it has
  no CI in the loop. Copying it would add a second, slower net for the same
  fact. Chesterton's fence applies: the fence has a reason.
- **The cap.** Nothing here argues for 25 concurrent writers. The cap of 4
  guards disjoint-file writes; the thread's 25 seats are mostly idle readers.
  Raising the cap is a different question and needs its own evidence.
- **Unread source.** The fleet scripts were never seen. Every routing-rule
  detail above is testimony, and its shape may differ in ways that matter.
- **Two dispatched gathers returned and are folded in** (jobs
  `job-1788875649935`, `job-1788875651588`; reports under
  `.bee/mailbox/<job>/report-1.md`). They corrected two claims in the first
  draft: the result inbox exists (Pi-only), and the control loop already has
  three roles, not two.

## Recommendation — ranked, with the smallest honest shape first

1. **A fourth control-loop role, `route`, over the inbox bee already has.**
   Make `.bee/result-inbox/` runtime-neutral (today only the Pi extension
   drains it) and give the control loop a cold `route` iteration: read the
   unrouted markers, act on exactly one, exit — the same shape `dispatch`,
   `merge` and `supervisor` already run in. This is the missing half, and
   every item below depends on it. Reuse plus one role; not a new subsystem.
2. **Cross-harness review routing** — DONE routes to `--role review` on a
   registry agent different from the coder's. Small in code. **Blocked on the
   user**: it contradicts `agents-review-user-invoked`.
3. **Followup lane** — a reviewer's CHANGES becomes a cell its own worktree
   takes before anything from the queue. Small-medium; cells and lanes already
   carry the machinery.
4. **Harness-health guard** — probe a registry entry before dispatching to it,
   so a spent-credit agent is skipped instead of idling a seat. Small, and it
   plugs the one failure the thread paid 40 minutes for.
5. **Artifact-liveness measurement** — extend the §4 anomaly scan with commits
   since dispatch, last-commit age, and report mtime, with a flag threshold.
   Small; the doctrine is already written, only the numbers are missing.

**Not recommended:** porting the `fleet-*` scripts (unpublished, and they
would duplicate `occupancy`, `control-loop` and `wave`); adopting beans as an
issue store (bee's cells and PBIs already live on main, with gates beans has
none of); raising the concurrency cap toward 25.

## Source Pack

- **Local files read:** `.bee/config.json` (`models`, `herding`);
  `skills/bee-herding/SKILL.md`;
  `skills/bee-herding/references/role-dispatch.md`;
  `skills/bee-herding/references/wave-runs.md`;
  `skills/bee-herding/references/supervisor-prompt.md`;
  `AGENTS.md`; `docs/history/research/` (existing briefs, for overlap).
- **Via two dispatched read-tier gathers** (reports in
  `.bee/mailbox/job-1788875649935/report-1.md` and
  `.bee/mailbox/job-1788875651588/report-1.md`):
  `packages/bee-rs/crates/bee/src/herding.rs`,
  `herding/control_loop.rs`, `herding/mailbox.rs`, `herding/wave_ledger.rs`,
  `verbs/drivers/models.rs`, `verbs/drivers/prepare.rs`,
  `verbs/worktree/merge.rs`, `verbs/worktree/phases.rs`,
  `verbs/cells/claims.rs`, `verbs/cells/handlers_write.rs`,
  `hooks/write_guard/checks.rs`, `docs/config-reference.md`,
  `.pi/extensions/bee-guard.ts`,
  `docs/knowledge/areas/bee-herding/` (7 area files).
- **Live command output:** `bee herding --help`, `bee herding run --help`,
  `bee reviews --help`, `bee supervisor --help`, `bee worktree merge --help`,
  `bee dispatch prepare --runtime claude --kind gather --role read --json`.
- **Upstream repos checked:** `hmans/beans` @ `99260bf1` (cloned, read);
  `herdrdev/herdr` (metadata via `gh api`; already a bee dependency).
- **Testimony:** the user's 13-point thread and two slides, transcribed into
  the matrix above.

## Prior Art In This Repo — Read Before Extending

This brief deliberately does **not** repeat what these already settle:

- `docs/history/research/herdr-orchestrator-distill.md` — the herdr transport
  surface and the two-planes finding.
- `docs/history/research/pi-herdr-agents-xia.md` — cockpit completeness:
  interrupt, stalled/recovered, retryable, cancel, orphan sweep.
- `docs/history/research/autopilot-harness-roadmap.md` and
  `bee-unattended-hardening.md` — the liveness-versus-authority dials.
- `docs/history/research/agent-orchestrator-mailbox-distill.md` — the mailbox
  return channel.
