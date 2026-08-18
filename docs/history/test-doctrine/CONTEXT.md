# Agent-Owned Test Scope — Context

**Feature slug:** test-doctrine
**Date:** 2026-08-18
**Shaping session:** complete (consumed from docs/discovery/test-doctrine/MAP.md — wayfinding D8 path)
**Scope:** Standard
**Domain types:** RUN | CALL

## Feature Boundary

Replace bee's fixed test cadence with agent-owned test scope: no door
auto-runs `commands.test`; every cap carries a proof line; skill text,
preamble, and AGENTS.md teach proof-per-change-type as principle. CI
stays untouched.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. Changing one requires the user, a new D-ID or an
explicit supersession note, never a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 (58ec9664) | The agent owns test scope end to end: it chooses which tests to run for a change at every point, including the close/merge boundary. | Current doctrine constrains agent judgment; full-project runs fire even for md-only edits. |
| D2 (58ec9664) | Docs-only diffs skip the test suite; cheap parity checks (regen parity, pointer integrity) are the right proof for them. | — |
| D3 (58ec9664) | The mandatory session-start full-suite red check is dropped. | — |
| D4 (58ec9664) | CI full suite on every push/PR stays exactly as is — the one deterministic net. | Local freedom is affordable only because CI is hermetic (pattern 20260721). |
| D5 (58ec9664) | DoD becomes proof-per-change-type taught as principle, not a fixed table: code → related tests green; docs → parity/pointer green; behavior → judge verdict. | — |
| D6 (58ec9664) | A scoped-green-but-CI-red miss is a fix-first cell PLUS a mandatory captured learning about why the chosen scope missed. | The learning loop is what keeps agent-owned scope safe over time. |
| D7 (1f534837) | No boundary auto-run remains: `bee close` and `bee worktree merge` stop running `commands.test` and instead require a recorded proof line. | — |
| D8 (1f534837) | The cap report `tests` field changes from the `boundary`/`undeclared` enum to a proof string `<command> — <result> — <scope reason>`. | Proof lives with the cell; no new record type. |
| D9 (1f534837) | One feature ships the whole package: CLI doors, preamble red-check removal, all skill-text refrains, and the stale AGENTS.md/AGENTS.block.md lines with their hash regen. | Text and machine must never disagree mid-rollout. |

### Agent's Discretion

Exact proof-string grammar, refusal wording, and how the doors verify
"a proof line exists" — within D7/D8. Which parity checks a docs diff
cites — within D2/D5 (principle, not a fixed list).

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| proof line | The cap report `tests` value: `<command> — <result> — <scope reason>`, written by the agent that ran it |
| boundary door | `bee close` and `bee worktree merge` — after this feature they check for proof, never run tests |
| proof-per-change-type | The DoD principle: the change's nature picks its evidence kind (tests / parity / judge verdict) |

## Existing Code Context

From docs/discovery/test-doctrine/research/002-findings.md — read it
before planning; anchors are file:line.

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:54,125-139` — REPORT_KEYS + the `tests` enum validator to replace (D8)
- `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:134,1657-1706` — close's test door to convert to proof check (D7)
- `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs:681-732` + `handlers.rs:530` — merge verify child to convert (D7)
- `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:597-617` — red-check line to remove/replace (D3); byte-pinned by `session_preamble/tests.rs:145,157`
- `packages/bee/prompts/worker-cell.md` — Result-form doc mirroring REPORT_KEYS (D8)
- ~16 skill-text refrain passages (list in 002-findings.md) + generated trees via regen (D9)
- `AGENTS.md:86-89`, `packages/bee/AGENTS.block.md:79-82` — stale cadence lines; block is hash-pinned, needs the onboard regen path (D9)

## Canonical References

- `docs/discovery/test-doctrine/MAP.md` — the discovery map this feature falls out of
- `docs/discovery/test-doctrine/research/002-findings.md` — full mechanism inventory
- decision 13ce1858 (test-cadence-boundary) — the cadence this feature supersedes in part; D7 must record the supersession relation when planning locks the door change

## Outstanding Questions

### Deferred To Planning

- [ ] What the door's "proof line exists" check reads exactly (last cap's report? all caps since last close?) — answered by reading close/merge record flow during planning discovery.
- [ ] Whether `bee test` (standalone runner) and `.bee/logs/test-results.json` stay unchanged — likely yes; confirm no coupling breaks.
- [ ] How D6's mandatory capture is enforced (chain-nudge text vs a CLI check) — planning picks the lightest honest mechanism.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning
reads locked decisions, code context, canonical references, and
deferred-to-planning questions. Planning's Gate 2 shape stage and
reviewing use locked decisions for coverage and UAT.
