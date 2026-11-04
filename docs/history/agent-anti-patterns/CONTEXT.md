# Agent Anti-Patterns — Context

Source: review of https://vhlam.com/article/coding-agent-anti-pattern against
bee (2026-11-04). The article names three anti-patterns: solution before
outcome, too many intermediate phases, and a plan so detailed it is
implementation in markdown.

## Feature Boundary

In scope:

- Cell actions name the outcome and the contract, not internal structure.
- Phases, slices and expand-migrate-contract need a real dependency.
- `bee report outcomes` gains a plan departure row.

Out of scope:

- Anti-pattern 1 (solution before outcome): already covered, see D4.
- A length cap on cell actions: rejected in D1.

## Locked Decisions

- **D1 (831b3afa).** A cell action names the outcome and the contract the
  cell must keep (inputs, outputs, persisted state, error shape, the tests
  that prove it). It does not prescribe internal structure (helper names,
  call graph, variable placement, line-level edits) unless that choice
  crosses a shared contract. Read anchors that say where to look stay
  allowed. A worker may take a simpler route that keeps every truth and
  prohibition and records it as a found-a-better-route departure. The
  goal-check compares artifacts against `must_haves` and cited decisions,
  never against action wording.
- **D2 (eb394d8f).** Expand-migrate-contract and any multi-phase or
  multi-slice split need a real dependency: a persisted-data migration, a
  consumer outside this change (another live worktree, an external client,
  a published contract), or a compatibility window. Without one, the change
  lands as one coordinated change proved by the compiler and tests. The
  removal test is asked of every phase and slice: if we skip it and go
  straight to the final state, what breaks. A temporary bridge left in the
  tree is filed with `bee backlog add`, naming its removal condition, and
  cited from the owning concept.
- **D3 (4bac33de).** `bee report outcomes` gains a plan departure row: cells
  with recorded departures and counts by the four departure kinds plus
  followed-the-plan, read from `trace.deviations`. `not recorded` when no
  cell carries deviations; no advice line.
- **D4 (43ea5ec4).** Anti-pattern 1 needs no new change: shaping locks the
  outcome first, the original request travels verbatim, and the brief names
  goal, hard constraints and open choices apart (slp-article-gaps D1).

### Agent's Discretion

- The exact wording of the doctrine edits, inside D1 and D2.
- The text layout of the departure row, inside D3.

## Terms

- **Contract** — what other parts rely on: API promises, valid inputs,
  persisted state, what success means, error shape, the proving tests.
- **Implementation** — how the code is organized inside: names, helpers,
  loops, intermediate variables.
- **Real dependency** — a reason the final state cannot land at once:
  persisted data, an outside consumer, or a compatibility window.

## Specific Ideas And References

- Article: "Một brief quá chi tiết vô tình biến những quyết định chưa từng
  được kiểm chứng thành constraint."
- Article removal test: "nếu bỏ qua bước này và đi thẳng tới trạng thái
  cuối, điều tồi tệ gì xảy ra?"

## Existing Code Context

### Established Patterns

- Cell evidence over 1200 cells (parsed 2026-11-04): action median 1301
  chars, p90 2959, max 9952. 730 cells record departures: 162 hit an
  unforeseen obstacle, 159 found a better route, 144 the plan was wrong
  about a fact, 123 something else had to be fixed first.
- `expertise/planning.md` "Expand, migrate, contract" came from
  pocock-nuggets D1 with no dependency test.

### Integration Points

- Cell quality rules: `skills/bee-planning/references/planning-reference.md`.
- Planning craft: `expertise/planning.md` (source of `.bee/expertise/`).
- Worker prompt: `packages/bee/prompts/worker-cell.md`.
- Goal-check: `skills/bee-swarming/references/swarming-reference.md` step 7.
- Report: `packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs`.

## Outstanding Questions

None.

## Handoff Note

Two cells, disjoint files: doctrine text, and the report row.
