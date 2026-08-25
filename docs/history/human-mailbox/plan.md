---
artifact_contract: bee-plan/v1
mode: standard
---

# Plan: Human Mailbox

Mode: `standard` — 3 risk flags: data-model, public-contracts, multi-domain
Why this is the least workflow that protects the work: a new store shape and a
contract another project consumes both need a reviewable shape, but nothing here
touches auth, data loss, an external provider or existing proof — D10 keeps the
current flagless finish path byte-identical, so no hard gate is earned.

## Requirements (from CONTEXT.md)

D1–D14 and D17, `docs/history/human-mailbox/CONTEXT.md`. In short: bee appends a
plain-language entry at every clean stop, composes one letter per unattended run
at the run's end, files it as one markdown file with typed frontmatter, and
offers the single command that flips a letter's read state. The inbox that shows
letters is waggledance's, already handed over under D17 and out of this plan.

## Discovery

- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:64` parses
  `--deviation` (and `--deviations-file`) and validates the Result form onto the
  trace; `trace.rs:32` seeds `deviations` empty. This is the one path D5 and D10
  extend — there is no second place a departure is recorded.
- `handlers_close.rs:13` names `deviation_text` in `verbs/knowledge/` as "the ONE
  rendering of a deviation entry", already shared with the promote miner. The
  letter renders through it; a second renderer would be the defect.
- `verbs/triggers/` is a working file-backed verb group over a store directory
  (add / list / resolve) — the closest precedent for the mailbox's own group,
  so no new storage pattern has to be invented.
- `.bee/config.json`'s `herding` block already distinguishes an unattended run,
  which is what D9's arming reads.

## Approach

Recommended path: build the record first and the ceremony around it second. The
letter file is the contract (D3), so phase 1 makes one real letter exist end to
end — appended, composed, filed — before anything tightens the inputs or adds a
second letter kind. The departure contract (D5, D10) lands second because it
changes a published flag's accepted shape and is worth reviewing against a letter
that already exists. The read-flip command and the dead-run path (D6, D12) come
third, once there is something to flip and something to recover. The feature-close
letter (D14) is last: same shape, more sections, no new mechanism.

Rejected alternatives:
- Tighten the departure contract first — leaves the flag changed with no letter to
  show what the change bought, and reviews the cost without the benefit.
- Compose letters on read instead of at run end — rejected by D4; there would be
  no pinned artifact to point at.
- A JSON record plus a rendered markdown twin — rejected by D3; two artifacts drift.

Risk map:

| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| `handlers_close.rs` deviation path | MEDIUM | A published flag's accepted value shape changes; D10 must keep the flagless path byte-identical | Existing finish tests green unchanged, plus a new case per arming state |
| New mailbox store + verb group | LOW | Precedent exists in `verbs/triggers/`; no migration, no existing data | Unit tests over file layout and frontmatter validity |
| Entry appending at every clean stop | MEDIUM | Cap, feature close and blocker are three code paths; missing one silently truncates a letter | One test per stop kind proving its entry reaches the store |
| Dead-run recovery at session start (D12) | MEDIUM | Runs on every session start; a full store scan would tax an unrelated path | Test the detection, and assert the read stays bounded |
| Record shape as a cross-project contract | LOW | waggledance holds a copy of the contract in its own backlog; drift is a doc problem, not a runtime one | Frontmatter validity test named against CONTEXT.md's field list |

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 — A letter exists | Entry appended at a cap while armed; composed and filed at run end to `.bee/human-mailbox/<UTC-ts>-<run-slug>.md`; frontmatter carries subject, run, project, filed_at, status, items[], needs_you[] with ids; five body sections, empty ones dropped (D2, D3, D4, D7, D9, D11, D13) | The record is the contract every later phase and the waggledance inbox build against; nothing can be judged until one real letter exists | Cap a cell in an armed run, end the run, open the filed letter | Everything below, and the waggledance side's first real sample |
| 2 — The departure contract | `--deviation` takes the three required parts and a closed kind set; a cap in an armed run must state a departure or its absence; unarmed caps stay byte-identical; worker prompt and swarming reference re-worded to match; letter renders departures through `deviation_text` (D5, D8, D10) | It changes a published flag, so it reviews best against a letter that already shows what the change buys | One cap with a departure, one without, in both armed and unarmed runs — four letters/traces read correctly | The letter's most-read section becomes trustworthy |
| 3 — The mailbox's own surface | The command that flips a letter's read state (D6); a session start that files an unfinished letter for a run that went silent, naming the moment (D12) | Both need a filed letter to act on, and both are what makes the inbox usable rather than decorative | Kill a run mid-way; the next session files an unfinished letter; flip it read through the command | waggledance's inbox can mark read and can show a dead run honestly |
| 4 — The feature-close letter | The same record carrying the extra architecture, behaviour and usage sections (D14) | Same shape, more sections — cheapest last, and D7 already promises it | Close a feature; its letter carries the extra sections | D7's promise stops being empty |

Slice queue: 1 → 2 → 3 → 4, strictly ordered; each depends on the one before.
Current slice to prepare: **phase 1**.

## Test matrix

The triad, at its smallest demonstrating size. Each cell's writer judges existing
coverage first and authors only the gap.

- Happy path: an armed run appends entries at two caps, composes one letter at run
  end, files it under the D11 name, and the frontmatter carries every D3 field
  with a non-empty subject.
- Edge cases: a run with zero entries; a section with nothing to report is absent
  rather than empty; an attended run records entries and files no letter (D9); two
  runs in one night produce two letters, not one (D11).
- Error paths: a composed letter with an empty subject is refused (D2); a cap in
  an armed run that states neither a departure nor its absence is refused, while
  the same cap in an unarmed run is accepted byte-identically (D5, D10).

## Out of scope

- The inbox UI, listing and display — waggledance's, handed over under D17.
- Pushing the subject line through waggledance's notification outbox.
- Answering a Needs-your-call item from the inbox (D13 keeps the ids only).
- A weekly digest (D14).
