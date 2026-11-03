---
artifact_contract: bee-plan/v1
mode: standard
---

# Plan: SLP Supervisor Heartbeat

Mode: `standard` — 1 risk flag: multi-domain (herding control loop +
hooks + new state records). Route note: `bee route --set` is refused
from a worktree-isolated session (control-plane lives in main); the
route (class=feature, lane=standard, flags=multi-domain, files≈10) is
recorded here as a named deviation and should be stamped from main at
merge time.

Why this is the least workflow that protects the work: a new
always-running role with its own state records spans three subsystems;
a plan with demoable phases keeps each tick of added machinery provable
before the next lands (CONTEXT "7-step build order").

## Requirements (from CONTEXT.md)

- 322695d6: `supervisor` role of `bee herding control-loop`, cold per
  tick, default interval 900s, read-only enumerated tool surface,
  cheap `supervisor` model role, `advisor` escalation.
- da7cb49b: reads the seven existing surfaces; day-1 signals
  struggling-loop / big-decision / danger-op; NO transcript detector.
- c80debd7: interventions = mailbox file records read at the target's
  next turn boundary; frequency-cap state on the record; report-only
  visibility, UrgentAlert immediate.
- 9f5cd250: away/back mark with exactly two effects; ONE WakeReport
  (≤10 lines, 4 sections) + one push notification on back.
- a8f4b8ab: extra signals (2×-estimate, same-region repeat) and
  harness-measured telemetry; confidence×door sort for the queue.
- c706053e: narrow opt-in silence-is-consent, deterministic timeout,
  logged + prominent in WakeReport.
- 66c4c251: health metrics with two-sided bands; earned autonomy;
  WakeReport sorted by impact-if-wrong.
- 787a9eb0 / R2 R3 R4: observer adds beside bee's locked rules — no
  write scope, no dispatch, no merge, no approvals.

## Discovery

Three advisor-tier digests already anchor every mechanism:
docs/history/research/slp-supervisor-placement.md (control-loop role is
the runner; `Role` enum control_loop.rs:54-75, allowed_tools_for
:220-235), slp-observer-surfaces.md (the seven surfaces with
can/cannot-see anchors), agent-harness-slap-distill.md (metrics,
build order, intervention hygiene). No further research needed —
precedent beats research.

## Approach

Extend, never invent: a new `Role::Supervisor` arm in the existing
control loop (per 322695d6); records under a new `.bee/supervisor/`
store written ONLY through new bee verbs (CLI-only state law); delivery
rides the existing hook path that already injects turn-boundary content
(per c80debd7). Rejected: a separate daemon (duplicates the loop); a
persistent pane session (context accumulation — rejected in ticket
002); transcript scanning (Detector, explicitly out — da7cb49b).

Risk map: control_loop.rs arm — LOW (pattern exists; tests exist) ·
supervisor store/verbs — MEDIUM (new record types; prove with unit
tests + dry-run tick) · hook delivery — MEDIUM (must not disturb
existing preamble; prove with hook tests) · notification transport —
LOW (file + best-effort notify; no external provider).

## Shape

Phase plan (milestones a user can demo in order):

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1. Skeleton tick | `Role::Supervisor` in the control loop: read-only tool surface, `supervisor-prompt.md`, `supervisor` model role resolution; a tick reads the surfaces and records ONE observation (or "reviewed, silence") via a new `bee supervisor` verb into `.bee/supervisor/observations.jsonl` | Everything else hangs off a running tick; validators/record types land first (7-step order) | `bee herding control-loop --role supervisor --once` → one observation record exists; silence is a logged outcome | Phases 2–4 |
| 2. Interventions | Intervention records with point-key frequency cap (second hit = escalate, never repeat); next-turn delivery through the existing hook injection; UrgentAlert immediate path | The observer becomes useful: questions reach live sessions | A planted struggling-loop signal produces one open question visible in the target session's next turn; a repeat produces an escalation, not a repeat | Phase 3 |
| 3. Night watch | `away`/`back` presence mark (two effects only); non-urgent asks queue; on back ONE WakeReport ≤10 lines, 4 sections, assumptions sorted by impact-if-wrong; one push notification | The autonomy payoff the user named first | Mark away, plant 3 events, mark back → exactly one report, actionable, ≤10 lines | Phase 4 |
| 4. Metrics + consent | Report-time health counters with two-sided bands; narrow opt-in silence-is-consent with deterministic timeout in the loop; earned-autonomy streak surfaced; 2×-estimate & same-region signals where inputs exist | Needs real records from 1–3 to compute anything honest | Reports carry the counter block; a queued non-gate ask with consent enabled auto-proceeds after timeout and shows prominently in the report | slp cluster 2 |

Current slice = Phase 1.

## Test matrix

Triad at smallest demonstrating size (writers judge existing coverage
first — control_loop.rs and models.rs carry extensive tests to extend,
never duplicate):

- Happy: `--role supervisor --once` runs, writes one valid observation
  record; role resolves configured model; unconfigured `supervisor`
  role falls through with a warning, never fails.
- Edge: nothing to observe → a "reviewed, silence" record (a legal
  outcome, A4-style); stop-file honored mid-tick; record store absent →
  created on first write.
- Error: malformed record refused by the verb (typed error); supervisor
  role given a write tool in config → allowed_tools stays enumerated
  read-only regardless.

## Out of scope

- The transcript Detector, sub-supervisors, voice reports,
  heterogeneous lane models (map Out of scope).
- Clusters 2–4 (dissent, blind lanes, contract/original-request).
- Any change to gate law, merge law, or permission posture (R2/R3/R4).
