# Proof strength and expiry — Context

**Feature slug:** proof-strength-and-expiry
**Date:** 2026-09-01
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN, CALL

## Feature Boundary

A bee cap records `<command> — <result> — <scope reason>`. Today the middle
segment is free text that only refuses the literal word `red`, so a live product
run and a type-check land as the same string; and the recorded commit is written
but never compared to anything, so nothing notices the tree moved between cap and
merge. This feature closes the result segment over a strength vocabulary and adds
a staleness advisory at merge. It ends there: the doors still CHECK recorded
proof and never re-run it, and nothing new refuses a merge.

## What was found

Verified in code, not taken from the prior study:

- `finish_support.rs:80-94` (`parse_tests_proof`) splits on the first two ` — `
  separators and accepts ANY non-empty middle segment. `parse_report_flag`'s doc
  at :103 confirms the only closed value: "A result segment reading `red` refuses
  the cap outright."
- The recorded commit is real — `handlers_close.rs:942` reads
  `report_line("commit")` — but it is used only to render the letter. No
  comparison against a merge base exists anywhere in
  `verbs/worktree/merge.rs`.
- 20 files outside `docs/history/` carry a `— green —` example, split across
  Rust sources, Rust tests, skills, `packages/bee/prompts/worker-cell.md`,
  `docs/product-description/`, and one Vietnamese guide page
  (`site/guide/vi/cell-lane.html`).
- 25 already-capped cells in `.bee/cells/` carry a bare `green`. They are
  historical records; a read-side refusal would make every past feature
  uncloseable.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | The result segment closes over exactly three values: `green:live`, `green:unit`, `green:static`. A bare `green` is REFUSED at the write path, naming the three. `red` keeps refusing as it does today. | The user's call, taken over the softer "accept both, name the unqualified ones" option. A vocabulary that stays optional is one people keep not using, and the gap this feature exists to close is precisely that a live run and a type-check are indistinguishable. |
| D2 | The WRITE path closes; the READ path stays tolerant. `parse_report_flag` / `parse_tests_proof` refuse a bare `green` on a NEW cap. `feature_proof_check` continues to accept a historical bare `green` on an already-capped cell as valid. | 25 capped cells already carry a bare `green`. Refusing them on read would make every existing feature uncloseable — a migration disguised as a validation change. The write/read split is how the vocabulary lands without a flag day. |
| D3 | Meanings, stated once where the vocabulary is defined: `green:live` — the real product or command was driven and its observable result inspected. `green:unit` — automated tests passed. `green:static` — it compiled, type-checked, linted, or a parity/pointer check passed; nothing was executed. | Without pinned meanings a closed vocabulary is three new free-text values. `green:static` is deliberately the weakest and deliberately legal — a docs cell has nothing stronger to offer honestly. |
| D4 | `bee worktree merge` compares each capped cell's recorded commit against the merge base and emits a NAMED `proof-stale` advisory listing the cells whose proof predates the tree being merged. It is an advisory: it never refuses the merge. | The doors' standing contract is that they CHECK recorded proof and never run tests themselves (`proof.rs:1-24`). An advisory keeps that contract while making "this proof is older than the tree" visible, which is the failure the prior study measured upstream as 21 stale verdicts with no signal. Refusing would be a new door, which this feature does not add. |
| D5 | Every doc, skill, prompt and test example moves to a qualified value. The worker brief (`packages/bee/prompts/worker-cell.md`) states the three and their meanings, because that is where a worker reads the contract at the moment it writes the proof line. | 20 files carry `— green —`. An example that still shows the refused form teaches the refused form. The brief is the one that must not just list the values but say what each means. |
| D6 | No change to the proof line's three-segment shape, to `commands.test`, to what any door RUNS, or to the `red` rule. | The blast radius is already 20+ files. Widening it to the proof line's structure would put a second contract change in the same feature. |

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Proof strength | Which of the three qualified values a cap recorded — how the change was actually shown to work, not whether it was. |
| Stale proof | A cap whose recorded commit is not an ancestor of the merge base, so the tree moved after the proof was taken. |

## Existing Code Context

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:80-94` and its
  `parse_report_flag` at :107 — the write path D1/D2 closes.
- `packages/bee-rs/crates/bee/src/verbs/cells/proof.rs:47` (`feature_proof_check`)
  — the read path D2 leaves tolerant.
- `packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs` — where D4's advisory
  lands; the merge base is already resolved there (`:204` names it).
- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:942` — proof that
  the recorded commit is already read and available.
- `packages/bee/prompts/worker-cell.md` — the worker brief D5 must carry.

### Established Patterns

- A closed vocabulary with a typed refusal naming the legal set —
  `ROUTE_CLASS_VALUES` in `state_group/workflows.rs` is the model, including its
  pinned refusal-message assertion.
- An advisory that names its rows and blocks nothing — the cap advisories the
  existing doors already print.

## Outstanding Questions

### Resolve Before Planning

None.

### Resolve During Execution

- Whether `site/guide/vi/cell-lane.html` is generated from a source elsewhere. If
  it is, edit the source; do not hand-edit a rendered page.
