# SLP supervisor–lead–peer — discovery map

## Destination

A locked decision set that hands FOUR feature clusters to shaping, in
build order: (1) supervisor heartbeat — an observer that watches bee
sessions, asks small open questions, and writes a wake report; (2)
dissent / stop-and-ask — a worker voice channel with an obligated
response; (3) blind lanes for hard decisions; (4) contract_status +
verbatim original_request pass-down. Source of ideas:
docs/specs/slp-supervisor-lead-peer/ — mined, not obeyed; bee's locked
rules win on conflict unless a point is re-decided explicitly.

Map closed 2026-08-26 — 7 tickets, 13 decisions, no fog left. Ready
for bee-shaping's Lock, which consumes the decisions below into each
feature's CONTEXT.md, in build order (a020319d):
1. slp-supervisor-heartbeat (322695d6, da7cb49b, c80debd7, 9f5cd250)
2. slp-dissent-stop-and-ask (4b7aa303, a2affcba)
3. slp-blind-lanes (9cffdfb5, 5981246b)
4. slp-contract-original-request (ca9960f5, 3899fa60, 9c0104e0)

## Notes

- The user confirmed all four clusters in scope and picked the
  supervisor heartbeat as the first build (2026-08-26, round 1+2).
- What bee already covers, so the map does not re-invent it:
  - SLP Lead ≈ the orchestrator session (gates, dispatch door,
    decide-altitude stays on the session model).
  - SLP Peer ≈ dispatched workers in worktrees (write scope isolated;
    one cell per worker).
  - SLP decision_log ≈ `bee decisions` — append-only, supersede/touch
    relations, revisit-conditions via `bee triggers`.
  - SLP Better SLP weekly retro ≈ `bee feedback` digest + bee-evolving.
  - SLP escalation ladder ≈ lanes + gates + advisor role (session-class
    consult) — partial; the "lane council" rung is missing.
  - SLP branch Lead / Handback ≈ feature worktrees + `bee worktree
    merge`.
- Genuinely missing (nearest neighbors named):
  - Supervisor as OBSERVER — bee-herding's control loop is a
    dispatcher, not a watcher; nothing today reads live sessions and
    intervenes with open questions.
  - Detector (cheap signal poller) — nearest: herding activity hook,
    waiting-on marks, session heartbeats, the wave ledger.
  - Blind multi-lane design — nearest: `bee dispatch wave`, but
    nothing enforces isolation-then-convergence.
  - contract_status labels — nearest: locked decisions in CONTEXT.md
    and docs/knowledge; no CHỐT/CHƯA-CHỐT surface a ticket can cite.
- Standing constraints inherited from
  docs/discovery/herding-orchestration/MAP.md and
  docs/knowledge/areas/bee-herding/overview.md: merge stays a human
  gesture (R2), dispatch stays behind the owner interlock (R3), the
  permission-posture split holds (R4). An observer layer adds beside
  those; it never relaxes them.
- docs/discovery/model-role-split/MAP.md (closed) makes model roles an
  open fall-through set — a cheap `supervisor` role plus a stronger
  escalation role is configuration, not new machinery.
- The spec's own chapter 9 endorses the merge-into-bee stance: separate
  mechanism (infrastructure) from policy (opinionated workflow).
- Post-close input (2026-08-26): the sibling harness spec distilled at
  docs/history/research/agent-harness-slap-distill.md feeds feature 1's
  shaping — two extra stuck signals, harness-measured budget telemetry,
  the confidence×door escalation predicate, and one flagged user
  question ("silence is consent"). Its "Supervisor" is a router, not
  this map's observer — mechanisms imported, name not.
- Deviation, recorded: the first gather worker's digest was lost in the
  herding transport (only its summary line returned); the mapping above
  was re-derived inline from the two discovery maps and the session
  preamble.

## Decisions so far

- 787a9eb0: the SLP spec is an idea source distilled into bee's
  existing skeleton, never a parallel six-agent layer; bee's locked
  rules win on conflict — session-1 interview, no ticket.
- a020319d: all four clusters in scope, each its own feature;
  supervisor heartbeat builds first — session-1 interview, no ticket.
- 322695d6: the supervisor runs as a `supervisor` role of the native
  herding control loop, cold per tick, cheap model role + advisor
  escalation, read-only tool surface — tickets/002-supervisor-placement.md.
- da7cb49b: the observation surface is bee's seven existing read
  surfaces; struggling-loop, big-decision and danger-op are observable
  day 1, the other four signals need new machinery —
  tickets/001-observer-surfaces.md.
- 4b7aa303: blocker dissent pauses the related slice and obligates a
  logged one-of-three response — tickets/005-dissent-stop-and-ask.md.
- 9cffdfb5: the agent opens blind lanes on its own judgment with a
  logged reason; deadlock always hands the user the dossier —
  tickets/006-blind-lanes.md.
- ca9960f5: contract settled-status is a derived view over the
  decision log, never a hand-kept registry —
  tickets/007-contract-status-original-request.md.
- 3899fa60: the verbatim original request rides every cell/dispatch
  immutably; layers only add —
  tickets/007-contract-status-original-request.md.

- a2affcba: dissent mechanism — a cell-level dissent record, verdict
  enforced at the close/merge doors (judge-debt pattern), StopAndAsk
  via the herding round-mailbox, no live Q&A channel —
  tickets/005-dissent-stop-and-ask.md.
- 5981246b: blind-lane mechanism — a procedure over the dispatch door:
  linted brief, parallel advisor dispatches, second-round
  cross-critique, dossier + decision + trigger —
  tickets/006-blind-lanes.md.
- 9c0104e0: contract label = `contract:<name>` tag view over active
  decisions with a dispatch-time tripwire; original_request = bee
  intent's verbatim anchor injected at dispatch prepare —
  tickets/007-contract-status-original-request.md.
- c80debd7: interventions are mailbox records read at the next turn
  boundary, carrying the frequency-cap state; human sees them in
  reports only, UrgentAlert excepted —
  tickets/003-intervention-channel.md.
- 9f5cd250: an explicit lightweight away/back mark defines the report
  window and queues non-urgent asks; gates untouched; on back exactly
  one ≤10-line four-section WakeReport plus one push notification —
  tickets/004-night-watch-wakereport.md.
- a8f4b8ab (post-close, harness distill): the supervisor feature
  absorbs two extra stuck signals (over-2x-estimate, same-region
  repeat), harness-measured overrun telemetry, and the
  confidence×door queue-sort predicate.
- c706053e (post-close, user): a NARROW opt-in silence-is-consent
  mode — enabled explicitly, non-gate queued asks only, user-set
  timeout, every auto-proceed logged and prominent in the WakeReport;
  gates and one-way low-confidence asks always wait.
- 5144314c (post-close, harness 5-Hats detail): the 5-Layer rubric +
  Truth Table + CRUD Lifecycle join the reviewer/judge checklists;
  blind-lane dossiers validate citations against the verbatim
  proposals (anti-fabrication); pushback must name the missing
  context. Hats = review instrument, distinct from lanes.
- 66c4c251 (post-close, harness full-text tail): supervisor reports
  carry a small health-metric set with two-sided bands; raising
  silence-is-consent is EARNED (zero human-reversed one-way decisions
  across 40-60 tasks); WakeReport sorts assumptions by
  impact-if-wrong; forwarded-up content is verbatim with a mechanical
  length-compare guard.

Named deviation (2026-08-26): three with-user tickets (005/006/007)
were grilled in one session on the user's explicit "tiếp tục xây dựng
cho 3 phần còn lại" — the one-ticket cap yields to the user's say-so;
mechanism research for the same tickets ran in parallel at the
advisor tier per the user's "use the biggest model" instruction.

## Not yet specified

Nothing. Every open question this map could phrase has an answer.
Resolved on close: the 80%-budget-warning telemetry is named as new
machinery inside the slp-supervisor-heartbeat feature (da7cb49b) —
its exact emitter is a shaping question, not a map question; Better
SLP dissolved to "covered" — bee-evolving over the feedback digest
already is the weekly retro; wake-report medium answered by 9f5cd250.

## Out of scope

- Building SLP as a standalone layer with the spec's six agents and
  message names verbatim — the user chose merge-into-bee.
- Relaxing R2 (human merge), R3 (owner interlock), R4 (permission
  split) in the name of night-watch autonomy.
- Adopting the spec's MUST/invariant list as binding — it reads as
  source material only.
- Heterogeneous lane models (spec §10.b) — requires breaking the
  one-name advisor slot (decision 4faf1de9); returns only as a fresh
  effort.
- Many sub-supervisors fanning events to one main supervisor (spec
  allows 100+) — matters only after one supervisor works; returns as
  a fresh effort.
- Voice rendering of the WakeReport — a delivery upgrade on top of
  the text report; returns as a fresh effort.
