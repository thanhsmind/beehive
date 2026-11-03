# Herding caps on a clean report — Context

**Feature slug:** herding-caps-on-clean-report
**Date:** 2026-09-19
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN

## Feature Boundary

When a `bee herding run` job carried a cell id and its worker returns a report
with a valid proof line, herding records the cap itself — with the worker's own
bytes — and says so in one visible line. Anything less than a valid proof line
caps nothing and is reported loudly. It ends there: no change to what a cap
means, what `cells finish` validates, or how a leader checks a worker's work.

## Why now

Measured, not suspected. All five dispatched workers in `pi-native-stage-driver`
committed, ran their tests, and returned a populated report carrying a valid
proof line — and every one left its cell `claimed`, with no outcome and no
`capped_at`. The leader capped all five by hand. PBI `p-6f2623c9` already tracks
the behavior; the recurrence rate here was total, not intermittent. A leader who
trusts the returned `outcome=done` without checking cell status ends a feature
with uncapped cells and no recorded proof — and `bee close` then refuses.

<!-- bee:not-a-deferral: This table records D1-D7, which describe a runner auto-cap the plan-step hat wave ruled out on evidence before any of it was built. They are kept as the record of a rejected shape, not as work postponed; decision 974f2a70 routes them as feature-local. -->
## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

Decision log: `9d2347e4` (the contract).

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | When a job carried a cell id AND the worker's report carries a VALID proof line, herding records the cap itself, passing the worker's bytes through unchanged. | `9d2347e4`. The cap is byte-identical to the one the worker should have made. |
| D2 | A valid proof line is the existing documented shape: non-empty command, a result from the existing closed vocabulary, non-empty scope reason. Herding does NOT define this — it calls the existing parser and the existing vocabulary constant. | Single source of truth. `parse_tests_proof` already parses the shape and `PROOF_RESULT_VALUES` already holds the vocabulary; a second copy is how a closed vocabulary decays into free text, which that constant's own doc comment says outright. |
| D3 | No proof line, a malformed one, or a result outside the vocabulary caps NOTHING and is reported loudly, naming the cell and what was wrong. Silence is never the outcome. | The defect being fixed is a silent omission; replacing it with a different silent omission fixes nothing. |
| D4 | A job that carried no cell id is untouched — same bytes out as today. | Gathers, advisors, hat seats and reviewers all run cell-less; none of them may grow a cap path. |
| D5 | No new trust is extended. `bee cells finish` records the proof line it is handed and runs nothing; this feature changes only WHO calls it, never what is checked. | The owner accepted the behavior on that basis. If a later change makes the cap validate more or less than `cells finish` does, it has left this decision. |
| D6 | The cap reuses the existing callable entry point rather than shelling out to the bee binary or duplicating cap logic. | `cap_cell_from_flags` exists, and herding already calls into the cells module directly for dissent — this is the same road, not a new one. |
| D7 | One visible line per cap, naming the cell and the result segment. A refusal line is never silenced. | A leader must be able to see, in the run output, that a cap happened and on what proof. |

<!-- /bee:not-a-deferral -->

### Agent's Discretion

Planning picks the call site inside the end-of-run path, the exact wording of the
capped and refused lines, and how the outcome is surfaced on the result envelope.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| valid proof line | The three-segment string `<command> — <result> — <scope reason>` whose result is one of the existing closed vocabulary values. |
| clean report | A worker report that carries a valid proof line. Nothing else about the report is inspected. |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:169` — `cap_cell_from_flags(root, &CapFlags, finish: bool)`, the callable cap.
- `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:110` — `parse_tests_proof`, the shape parser (deliberately blind to the vocabulary).
- `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:81` — `PROOF_RESULT_VALUES`, the closed vocabulary, with its meanings written there and nowhere else.
- `packages/bee-rs/crates/bee/src/herding/run.rs:3525` — `crate::verbs::cells::record_dissent(...)`, the precedent for herding calling into cells.

### Integration Points

- `packages/bee-rs/crates/bee/src/herding/run.rs:180` and `:439` — the job's `cell_id`, already carried.
- `packages/bee-rs/crates/bee/src/herding/run.rs:3066` and `:3698` — the worker's `proof` on the result envelope, already carried.

## Canonical References

- `docs/history/proof-strength-and-expiry/CONTEXT.md` — D1/D2/D3, which own the proof vocabulary and why the read path stays blind to it.
- `AGENTS.md` § "Prove, then say so" — the cap's proof line contract.

## Outstanding Questions

<!-- bee:not-a-deferral: This section records what planning must settle from evidence it can read in the code, not work postponed to a future session. Both items are answered inside this feature's own cells. -->
### Deferred To Planning

- [ ] Where exactly in the end-of-run path the cap belongs, so it fires once per job and never on a dry run.
- [ ] Whether the cap outcome belongs on the result envelope as a field, or only in the printed line.
<!-- /bee:not-a-deferral -->

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
