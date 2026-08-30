# Hat-wave synthesis — existence-is-not-evidence (plan step)

Wave: 3 seats (standard lane), all returned within budget. Dispatched via
`bee dispatch prepare --kind advisor` (facts-gaps: opus; alternatives: opus;
user-impact: sonnet). Draft under critique: plan.md revision 1 (pre-wave).

## Seat returns

### hat-facts-gaps (BLOCKER/WARNING over structure + claims-table audit)

- B1 CONFIRMED, ACTED ON: existing preconditions guard on
  `exec_component = merge || name == "execution"` (set_gate.rs:824,830);
  test at :1922-1924 pins that `--name shape` is NOT covered. The draft's
  "beside the two existing preconditions" would never fire at the shape
  gate. → own guard `approved && (merge || name == "shape")`.
- B2 CONFIRMED, ACTED ON: plan.md freezes at shape approval
  (planning-reference.md:25-26); an execution-gate refusal would be
  undischargeable. → the check does not fire on plain `--name execution`.
- B3 CONFIRMED, ACTED ON: no membership rule. → converse rule in template +
  leader pre-flight + D4 membership sweep; residual named. The draft's own
  two unrowed load-bearing claims became rows 9-10, verified by count.
- Claims audit: 7 match / 1 partial mismatch (suite command quote dropped
  `PATH=` prefix) → fixed to exact bytes; match rule (verbatim substring,
  " / " join) now defined in the table header.
- W1→anchor-existence hardening adopted; W3→M1 feature selection adopted;
  W4→NotFound vs other-error split + directory fixture adopted; W5→single
  call site, named deviation; W6→full truth table adopted; W7→audit
  procedure homed in .bee/expertise/review.md, seat row points; W8→named;
  W9→count fixed; W10→named residual in risk map; W11→full-verb integration
  test adopted; W12→named operational note.

### hat-alternatives (SMALLER PATH at plan altitude)

- Verdict PASS with trims, all adopted: cell 3 (knowledge pattern) dropped —
  written at close capture instead (better content, one fewer commit);
  module boundary pinned (parser+rules+unit tests in plan_claims.rs,
  ≤15-line wrapper in set_gate.rs, integration tests in set_gate's module);
  risk MEDIUM→LOW on counted evidence (exactly one fixture writes plan.md
  then approves a gate: set_gate.rs:1587-1600).
- Inlining the parser rejected (set_gate.rs at 2762 lines; precedent splits
  logic from wrapper).

### hat-user-impact (gray-area probes over the planned behavior)

- U1 adopted: leader pre-flight self-check before presenting the gate — the
  binary refusal is the net, never an ambush after a human "yes".
- U2 adopted: `## Open Questions` section added to the plan.md template so
  the refusal remedy names a destination that exists.
- U3 adopted: refusal text carries the expected table shape, not just "table
  missing".
- U4 adopted: tiny/small evidence folds inline into the one gate question.
- U5 adopted: zero load-bearing claims → say so; never manufacture a row.

## Leader decisions the seats did not make

- No version-keying on `artifact_contract` (zero binary readers — verified;
  and the annotation-gate pattern names the trap). 193 legacy plans accepted
  as rare re-approval friction with a self-serve remedy.
- Check ordering: after existing preconditions in the merged path, so the
  claim-9 fixture's expected refusal stays stable.

Zero seats dropped; no all-fall-through. Findings filtered against the
request: all retained findings trace to D1-D5 or to defects in the draft
itself.
