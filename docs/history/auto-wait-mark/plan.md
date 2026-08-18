---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset until approval>
---

# Plan: Auto Wait Mark

Mode: `standard` — 3 risk flags: public-contracts, covered-contract-change, multi-domain
Why this is the least workflow that protects the work: the change is three files and
one new enum value, but it edits a documented closed vocabulary and narrows a
behavior two existing tests pin — so it earns a written shape and a gate, and
nothing more.

## Requirements (from CONTEXT.md)

- **D1** — the Stop hook sets a `waiting_on` mark on every turn end. No text
  heuristic, no phrase table, no language detection.
- **D2** — a hook-set mark carries no provenance marker; same setter, same record
  shape as an agent-declared mark.
- **D3** — `kind` grows from `gate | question` to `gate | question | turn-end`.
  A reader wanting real blocks filters `kind != turn-end`.
- **D4** — a surface rendering a live wait as a **blocker** must not list a
  `turn-end` mark. Display-only surfaces keep showing all three kinds.
- **D5** — the hook never overwrites a live mark; it only fills an empty slot.
- **D6** — a `turn-end` mark's `subject` is the last non-empty line of the turn's
  final assistant text block, trimmed and truncated.

## Discovery

- The vocabulary has exactly one enforced encoding:
  `WAITING_ON_KIND_VALUES: [&str; 2] = ["gate", "question"]`
  (`packages/bee-rs/crates/bee/src/verbs/workflow_store/record.rs:356`), with
  `build_waiting_on` (`:365-372`) the only validator. Every other `gate`/`question`
  pair in the tree is a test fixture, generated CLI help, or doc prose — no second
  closed list, and no per-runtime copy (`.codex/` has zero hits). Evidence:
  `rg -n '"gate"' packages/bee-rs/crates/bee/src skills .claude .codex`.
- Nothing refuses, skips, or reroutes on `run_state == "awaiting-approval"`. The
  only branch on it is a clear-path cleanup
  (`verbs/state_group/waiting_on.rs:204`). So D1's permanently-live mark blocks no
  command. Evidence: `rg -n 'run_state' packages/bee-rs/crates/bee/src`.
- The Stop hook already resolves and reads the session transcript on every turn for
  the perf rollup (`hooks/session_close/perf.rs:71` `resolve_transcript_for`), so
  D6's last line costs no new I/O.
- Two existing tests pin behavior this feature changes:
  `build_waiting_on_refuses_unknown_kind_empty_subject_and_empty_session`
  (`verbs/workflow_store/tests.rs:369`) asserts the literal message
  `"kind must be one of gate/question"`, and
  `orient_reports_a_live_wait_as_a_blocker_naming_the_subject`
  (`verbs/status_full/tests.rs:3763`) asserts a live mark *is* a blocker.

## Approach

**Recommended path.** Widen the one validator and narrow the one blocker surface
first (D3, D4), then hang the setter off the Stop branch that
`hooks/session_close/mod.rs` already has (D1, D5, D6). The setter reuses
`waiting_on.rs`'s existing target resolution rather than adding a second one, and
follows `prompt_context.rs:338`'s best-effort shape — a failed write logs and the
hook still emits its normal output, never a failed Stop.

**Rejected alternatives.**
- A new `waiting_on` writer inside the hook, bypassing the store functions — would
  duplicate D3-target resolution and projection rebuild, the exact split
  `awaiting-human` R128 exists to prevent.
- Reading `stop_hook_active` to skip re-entrant stops — no evidence yet that the
  bypass block net (`nudges.rs:349`) can re-enter; carried as a test probe, not a
  design commitment.
- A fourth `kind` for the AskUserQuestion path — that path never reaches Stop, so
  it is a separate mechanism; already filed to the backlog.

**Risk map.**

| Component | Risk | Proof needed |
|---|---|---|
| `record.rs` vocabulary widening | LOW — additive, no stored value invalidated | Refusal test still refuses, now naming three values |
| `orient.rs` blocker narrowing | MEDIUM — a wrong condition either hides a real block or floods orient with noise every turn | One test per kind: `gate`/`question` still blocker, `turn-end` not |
| Stop-hook setter | MEDIUM — runs on every turn; a panic or a slow path degrades every stop | Best-effort path proven by a failure-injection test; non-Stop events proven to write nothing |
| D5 no-overwrite | MEDIUM — getting it wrong silently destroys the high-value declared mark | Explicit test: declared `question` survives a Stop |

## Shape

One slice. It is already a walking skeleton: a real Stop writes a real mark that a
real `bee status` shows and a real `bee orient` declines to call a blocker.

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 — the mark exists and reads correctly | `kind` accepts `turn-end`; `bee orient` stops calling it a blocker; docs and skill prose name three values | The setter cannot write a value the validator refuses, and a mark that reads as a permanent blocker is worse than no mark | `bee state waiting-on set --kind turn-end --subject x`, then `bee orient --json` shows it under `where.waiting_on` and NOT in `blockers` | The setter |
| 2 — the hook writes it | Stop branch in `hooks/session_close/mod.rs` sets a `turn-end` mark from the transcript's last line, never overwriting a live mark | The whole point of the feature | End a turn; `bee status --json` carries `waiting_on.kind == "turn-end"` with the turn's last line as subject | Nothing further — feature complete |

**Cells (current slice).**

- `awm-1` — Phase 1. Files: `verbs/workflow_store/record.rs`,
  `verbs/status_full/orient.rs`, their tests,
  `docs/knowledge/areas/workflow-state/workflow-records-and-projections.md` (R125),
  and **both** skill-prose spellings of the vocabulary —
  `skills/bee-hive/references/routing-and-contracts.md:318-319` and
  `skills/bee-hive/references/gates-and-delegation.md:165-167`. The `.claude/skills/`
  copies are generated: edit `skills/`, then run `bee dev regen` so the mirrors and
  `generated/registry_payload.json` (which carries the `--kind` CLI help text) both
  re-render. A hand-edited mirror is the defect to avoid here.
- `awm-2` — Phase 2. Files: `hooks/session_close/mod.rs` and its tests.
  Depends on `awm-1` (cannot write a kind the validator still refuses).

Sequential, not parallel: `awm-2`'s proof needs `awm-1`'s value to be legal.

**Review wave (inline — `standard`, 3 product files, zero hard-gate flags).**

*Structure.* WARNING, fixed above: the first draft named only one of the two skill
files that spell the vocabulary, and never named the regen that re-renders the
`.claude/` mirrors and the generated CLI help — a cold worker would have shipped a
half-updated contract. WARNING, accepted: D2 is a "do not add a field" decision, so
no cell proves it directly; the existing record-shape tests are what would catch a
stray provenance field.

*Cells, cold pickup.* MINOR, recorded not fixed: under D1 the session preamble and
the compact capsule will render a `turn-end` line whenever a stale mark survives
into a resumed session. Real but small — `UserPromptSubmit` clears the mark on the
user's first message, so the line appears at most once per resume. If it proves
noisy in use, the fix is the same one-line kind check D4 already applies to
`orient`, not a new decision.

**Smaller path check.** Asked: is there a cheaper shape honoring every locked
decision? Yes, and it was taken — the first draft split Phase 1 into two cells
(vocabulary, then orient). Merged into one: the two files never overlap, both are
the same workflow-state domain, and the split bought a third wave for no
protection. Dropping D3's enum value or D4's narrowing was considered and rejected:
both are locked, and the first would leave the feature with no signal at all.

## Test matrix

The triad, at its smallest demonstrating size. Each cell's writer judges existing
coverage first (`.bee/expertise/tests.md`) and authors only the gap — the two tests
named in Discovery are amendments, not new cases.

**Happy path**
- `build_waiting_on` accepts `turn-end` and round-trips it through the record.
- A Stop event with no live mark writes `kind == "turn-end"`, `subject ==` the
  transcript's last non-empty assistant text line.
- `bee orient` surfaces a `turn-end` mark under `where.waiting_on` (display) while
  omitting it from `blockers`.

**Edge cases**
- A live `question` or `gate` mark survives the Stop untouched (D5).
- `gate` and `question` marks still appear in `orient`'s `blockers` (D4's other half).
- A final text block that is empty or whitespace-only still yields a non-empty
  `subject` — R125 refuses an empty one.
- A very long last line is truncated and still renders on one line.
- `SessionEnd`, `PreCompact`, and a non-Stop event write no mark.
- Whether a re-entrant Stop (`stop_hook_active`) doubles the mark — probe first,
  guard only if it reproduces.

**Error paths**
- `build_waiting_on` still refuses an unknown kind, with a message naming all three
  legal values.
- A missing, unreadable, or malformed transcript leaves the Stop hook's normal
  output intact and writes no mark (best-effort, per `prompt_context.rs:338`).
- A failing store write is logged and does not fail the Stop hook — failure
  injection, the same shape `prompt_context.rs` already proves for the clear path.

Proof recorded on each cap: `cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
scoped to the touched modules, plus the whole-suite run before the merge.

## Out of scope

- Any dashboard or UI. `awaiting-human` D5 ships the data source only; unchanged.
- The `AskUserQuestion` path. That turn never reaches Stop, so this mechanism
  cannot cover it; filed to the backlog as a `proposal` (P3, layer `hooks`) for
  re-triage after this merges.
- Changing how the agent is instructed to ask questions. The user ruled this out
  explicitly — this feature changes bookkeeping only.
- `bee status --brief`. Deliberately excluded from wait reporting by
  `ah-3`'s recorded deviation; that decision stands.
