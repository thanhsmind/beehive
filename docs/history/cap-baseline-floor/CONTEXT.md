# Cap Baseline Floor — Context

**Feature slug:** cap-baseline-floor
**Date:** 2026-09-20
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN

## Feature Boundary

A cap may record what the same verify command did on the BASE commit, in its
own structured trace field. That is all: the floor is recorded and readable,
never required, and no door refuses a cap for lacking one. The proof string,
its parser, and every historical cap are untouched.

## Why now

From `docs/history/research/openjev-verdict-2-xia.md`, practice 2. The source
commits a floors file and states every headline number beside it, with the
reason recorded verbatim:

> "The floors matter because a TF-IDF baseline on this benchmark already
> reaches about 0.65 accuracy … Any headline number has to be read against
> that." — `RUNBOOK.md` § 7

The brief's bee analog: a cap proof line reads "36 suites 0 failed" with no
floor, so **a reader cannot tell a real pass from a suite that passed before
the change too**. That is the whole gap.

The owner adopted the two one-line rules first (`proof-completeness-rules`,
decision `c03a77c0`) and this one second, firing trigger
`the-owner-decides-whether-to-adopt-the-r__c03a77c0`.

## Locked Decisions

Decision log: `e4ad60dd` (where it lives), `bccfb923` (optional).

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | The floor lives in its OWN structured trace field, beside `verify_command`, `verify_output`, `verify_passed` and `verification_evidence`. It is NOT a fourth segment of the proof string. | `e4ad60dd`. `parse_tests_proof` splits on the FIRST TWO separators only and the reason segment deliberately absorbs any further ones (`finish_support.rs:91-123`); the READ path runs that same parser over already-capped cells, and its comment records that tightening there would retroactively refuse ~200 historical caps carrying a bare `green`. A fourth segment is not available. |
| D2 | The floor is OPTIONAL and encouraged: recorded when present, never blocking a cap when absent. | `bccfb923`. The base may not build, the command may not exist on it, and requiring one doubles every verify. Recording first lets it earn enforcement. |
| D3 | `parse_tests_proof` is UNCHANGED — signature, split rule and blindness to the result vocabulary all stay. | Its own comment names the write/read split as deliberate and says not to "fix" it. This feature must not be the thing that breaks 200 caps. |
| D4 | No door — not `bee close`, not `bee worktree merge`, not the cap itself — gains a refusal for a missing floor. | D2. Recording is the whole scope; enforcement is a later, measured decision. |
| D5 | The floor records what the SAME command did on the base commit. A floor from a different command is not a floor. | Otherwise it is an unrelated number sitting beside the result, which is the metric gaming the source's rule exists to stop. |

### Agent's Discretion

- The field's name and its exact shape, provided it is structured and sits with
  the other four proof fields.
- Whether anything computes the floor automatically, or it is recorded by
  whoever caps — the boundary above requires only that the field exist and be
  readable.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Baseline floor | What the same verify command produced on the base commit, recorded so a result can be read against it rather than in isolation. |
| Base commit | The commit the cell's work started from, not an arbitrary earlier point. |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:91-123` —
  `parse_tests_proof` and the comment recording why it must not tighten. Read
  before touching anything near the proof string.
- The four structured trace proof fields added by `pi-harness-workflow-parity`
  D4 (`verify_command`, `verify_output`, `verify_passed`,
  `verification_evidence`) — the precedent and the neighbours for D1's field.

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/cells/proof.rs` — `feature_proof_check`,
  the read path over capped cells. D4 says it gains no refusal.
- The cap verb's `--report` parsing, where a new optional field would be
  accepted.

## Canonical References

- `docs/history/research/openjev-verdict-2-xia.md` — practice 2 and its
  landing-place line.
- `docs/history/proof-strength-and-expiry/` — D2 there is the write/read split
  this feature must preserve.

## Outstanding Questions

### Resolve Before Planning

None.

### Deferred To Planning

- [ ] Does the cap's `--report` JSON gain an optional key, or does the floor
  arrive by its own flag? Reading `parse_report_flag` decides it.
- [ ] Is the base commit already known at cap time, or must it be resolved?
  The trace already carries a commit; reading the cap path answers it.

## Deferred Ideas

- Enforcing the floor on some cell class, once there is data on how often one
  can actually be produced. Out of scope by D2 and D4.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
