# Hat wave — mailbox-broker plan step

Seats: hat-facts-gaps (Opus, native), hat-alternatives (Opus, native),
hat-user-impact (herding, report
`/home/thanhsmind/Projects/goglbe/beehive/.bee/mailbox/job-1791026711370-4073025-1/report-1.md`).
Brief: the open problems of CONTEXT.md "Deferred To Planning", as a LaneBrief.

## Where the seats agreed

- The question lives in the existing `result-N.json` as a new status
  `question`; the answer lives beside it as `answer-N.json`. No second mailbox.
- The worker classifies the question (`technical`, `product`, `gate`); code
  never classifies free text. A `gate` question never reaches the advisor.
- "Not sure" is the advisor's own `blocked` result. No new confidence field.
- The broker is a code-only control-loop role whose child is the bee binary.
  It is not a step of the supervisor (observer-only) or the route role (LLM).
- `--continue` needs a live pane, so the next round is a new job that carries
  the question, the answer and the files the earlier round changed.
- The Pi drain must skip a `question` result and keep the marker, or the
  leader gets the question as the final result and never sees round 2.
- `job.json` lacks agent, seat, cell, inbox session and leader session; the
  re-dispatch needs all of them.
- A human answer needs a verb; the supervisor store has no "answered" event.
- Product and gate questions carry signal `big-decision` so consent-sweep
  never proceeds without the human.

## Findings taken into the plan

| Finding | Seat | Where it landed |
|---|---|---|
| Native Agent workers and a native advisor are out of reach of code (BLOCKER) | facts-gaps, alternatives | D6 (store `969f077a`) |
| Drain race on Pi | all three | mb-2 |
| job.json re-dispatch facts and leader session | facts-gaps, alternatives | mb-1 |
| Rust requires proof on every result; Pi only for done | facts-gaps | mb-1 (proof optional for question) |
| No broker running leaves a question unanswered | facts-gaps | mb-1 `broker_running`, mb-3 heartbeat |
| Round limit, default 2, enforced by the brief | alternatives, facts-gaps | mb-1 `--no-question`, mb-3 |
| Round 2 must know the files round 1 changed | user-impact | mb-3 child brief |
| Escalated question must carry the advisor's notes | user-impact | mb-3 (advisor report path in the intervention) |
| Consent on a technical question uses the worker's leaning | alternatives, facts-gaps | mb-3 |

## Findings not taken

- A 5-second broker interval and running the tick inside every loop pass
  (user-impact): a separate role with a 30 s default keeps one role, one job.
- Writing question attempts into the cell trace (user-impact, facts-gaps):
  seats have no cell record, and the broker must not touch the leader's claim;
  the child brief carries the history instead.
- A Pi workers-widget glyph for a waiting worker (user-impact): a display nicety,
  not part of the loop.
