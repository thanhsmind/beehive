# nudge-consult — CONTEXT

## Why this feature exists

bee's supervisor writes an `advisor-nudge` record when it reads a poor-work
signal. The record tells the target session's **lead** to run an advisor
consult or log a reasoned decline, and until it answers, the nudge is a
response debt that refuses the cell cap, `bee close`, and `bee worktree merge`.

Two things were verified on 2026-09-04 and both are broken:

1. **The consult the lead is told to run is not wired.** P27 shipped a
   worker-owned, on-failure advisor consult (`skills/bee-swarming/references/worker-details.md`,
   § "Advisor consult in full"). Its trigger is an `Advisor` line in the
   dispatch prompt. Nothing renders that line: `packages/bee/prompts/worker-cell.md`
   has zero advisor mentions, and `skills/bee-swarming/references/swarming-reference.md`
   still instructs the orchestrator to call `resolveAdvisor(root, runtime)` —
   a retired JS-era name with no CLI surface today. The loop is live by
   contract and dead by wiring.
2. **No consult is defined at execution altitude for a nudge.** `worker-details.md`
   defines two bundle shapes — on-failure (red) and gate-time pre-cap. A nudge
   arrives mid-execution on green-but-wrong-direction evidence, and fits
   neither.

## Locked decisions

- **D1 — the `Advisor` line is rendered, not hand-added.** `bee dispatch prepare
  --kind cell` resolves the advisor slot with the existing `resolve_advisor` and
  renders the line, applying the one honest no-op inside the verb. A hand step
  that depends on the orchestrator remembering is exactly the defect found.
  `prepare` is the ONE dispatch door and already resolves the worker's model.
- **D2 — decision `4faf1de9` is untouched.** It governs the advisor's
  RESOLUTION (one name, no fall-through, unconfigured means no advisor). It
  never governed whether a resolved advisor may be NAMED in a payload. Cite it;
  do not reinterpret it.
- **D3 — the no-op is stated, not invented.** `swarming-reference.md:186-196`
  already fixes it: no advisor configured → no line; advisor resolves to
  literally the same model name as the worker's resolved model → no line; a
  cli-shaped advisor is never the same model, so it is always consulted;
  otherwise always add the line, escalated workers included.
- **D4 — the nudge consult is a third BUNDLE SHAPE, not a new construct.** No
  new seat, kind, verb, flag, hook, or schema. It reuses the configured
  `advisor` slot through `bee dispatch prepare --kind advisor`, and it carries
  the gate-time compact-digest bundle (nothing failed, so the on-failure bundle
  cannot be filled) plus the nudge row verbatim and the in-flight diff.
- **D5 — its budget is one consult per nudge row.** No counter is added: the
  mailbox point key already forbids a second nudge on the same point, and
  escalates instead.
- **D6 — the debt door is unchanged.** `advisor_nudge_is_cleared` stays as it
  is (tag + row id in the decision text). Only a documented text form is added,
  so a clearing decision carries a verdict instead of clearing empty.

## Rejected alternatives

- Fix only the prose to name the real CLI read — rejected: it leaves the hand
  step nobody performs.
- Add the advisor to the tier fall-through list — rejected: `4faf1de9` forbids
  it, and it is a different mechanism.
- A new `hat-execution` seat — rejected: a mid-execution `advisor-ref` goes
  stale on write against its own `plan_sha256` anchor.
- A second hat wave window — rejected: 3–5 dispatches per nudge, the named
  ceremony-capture failure (`a52c854d` D2).

## Out of scope

- `p-1126e3ec` — the `budget-overrun` signal has no input (`estimate_minutes`
  is never written), so one of the three poor-work signals cannot fire today.
  Filed separately; NOT this feature.
- Giving the supervisor richer perception (the worker transcript tail) and a
  `SubagentStop` firing point. That is the follow-on slice, deliberately not
  started here.
