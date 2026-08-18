---
artifact_contract: bee-plan/v1
mode: high-risk
# approved_gate2: <unset>
---

# Plan: uat-approval-reaches-the-door

Mode: `high-risk` — 4 risk flags: public-contracts, covered-contract-change,
multi-domain, audit-security
Why this is the least workflow that protects the work: `uat` is the human
acceptance door that no `gate_bypass` level may auto-approve. Widening the path
by which an approval is *found* is a validation-boundary change, so the shape
gets written down and gated before any line moves.

## The defect

A human approves the `uat` gate. `bee gate` prints success. `bee worktree merge`
then refuses `WORKTREE_MERGE_UAT_PENDING` anyway, while the lane file plainly
reads `uat: true`.

Observed live three times on 2026-08-18 (`merge-closes-the-lane`,
`merge-commits-the-lane`, `test-doctrine-text-sweep`), and independently by a
second session within twenty minutes — recorded as the pattern
`20260818-...-strands-an-approval-written-to-the-projection.md`. In every case
the only exit was `--skip-uat`, so a genuine owner approval was recorded as a
skip.

**Write and read land in different files.**

| | Where |
|---|---|
| Write, `bee gate --name uat --approved true --lane <f>` | `write_through_projection` finds no live workflow (the record is `closed`), so it takes the direct-write branch and writes `approved_gates.uat = true` into `.bee/lanes/<f>.json`. The block that stamps the workflow record's own `gates.uat.approved` is guarded by the same live lookup and is **silently skipped** — no error, no warning; the CLI still prints `Gate "uat" set to true.` |
| Read, merge and close | live workflow's `gates.uat.approved`, else fall back to the **default** `.bee/state.json`'s `approved_gates.uat`, filtered on `state.feature == feature`. **Neither door ever reads the lane file.** |

So the fallback read consults a file the lane-scoped write never touches.

The record goes `closed` through ordinary housekeeping: any session may run
`bee state workflows close --all-but-active`, which closes every live record
whose feature is not its own. Nothing reopens one, so waiting never helps.

## Requirements

- **R1** — An owner's `uat` approval recorded against a feature is visible to the
  door that blocks on it, whether or not that feature's workflow record is still
  live.
- **R2** — An unapproved gate still reads as unapproved. This fix widens where an
  approval is *found*; it must never widen what counts *as* one. The refusal
  path, the lane-classification rule, and `--actor auto`'s refusal on `uat` are
  untouched.
- **R3** — The merge-time door and the close-time door resolve the approval
  through **one** function. Today they carry byte-identical copies of the
  resolution in two modules — the shape the active pattern
  `20260728-one-membership-hand-copied-six-times-has-no-owner-and-no-alarm`
  warns about, and the reason a fix applied to one door would silently miss the
  other.
- **R4** — When `bee gate` cannot reach the durable record — the write lands only
  on a projection — it says so on its own line. A success line that hides a
  half-write is what let this ship.

## Approach

**Recommended.** Hoist one resolver into `uat.rs`, the policy module
`uat-stop-placement` already created for exactly this question, and give it a
third source in a fixed precedence:

1. the live workflow record's `gates.uat.approved` — unchanged, still first;
2. failing that, the **lane record** `.bee/lanes/<feature>.json`'s
   `approved_gates.uat` — the new step, and the file the lane-scoped write
   actually lands in;
3. failing that, the default `.bee/state.json`, still filtered on
   `state.feature == feature` — unchanged, for the unbound default-record case.

Both doors call it. Nothing else about either door moves.

**Why the lane file is a legitimate source.** It is CLI-only state, written by
the same gate command under the same `--actor` rules, and it is the file `bee
gate --lane <f>` was told to target. Reading it is not a new trust assumption; it
is reading the answer where the operator was told to put it.

**Rejected — teach `bee gate` to refuse when the record is closed.** It makes
the failure loud, which R4 does anyway, but leaves the owner with no way to
approve at all until someone reopens the record. It converts a silent block into
a hard one.

**Rejected — have `bee gate` reopen the closed workflow record.** Resurrecting a
closed record as a side effect of an approval is a much larger blast radius than
reading one more file, and it rewrites history a sibling session deliberately
closed.

**Rejected — fix only the merge door.** It is the one observed live, but the
close door carries the identical copy and would keep the defect under
`uat_stop: "close"`. R3 exists to stop exactly that.

Risk map:

| Component | Risk | Proof needed |
|---|---|---|
| The new lane fallback | **HIGH** — a wrong read here lets an unapproved merge through | explicit negative tests: lane file absent, lane file with `uat` absent, `uat: false`, and a non-boolean `uat` must all read unapproved |
| Collapsing two copies into one resolver | MEDIUM — both doors change at once | every existing merge-side and close-side uat test stays green, unmodified |
| `bee gate`'s new warning line | LOW — additive text | one test that the warning appears on the closed-record path and is absent on the live path |

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | One `uat_gate_approved` resolver in `uat.rs` with the three-source precedence; the merge door and the close door both call it, their own copies deleted (R1, R2, R3) | the defect and the duplication are the same fix | approve `uat` on a feature whose workflow is closed, then merge: it proceeds | a recorded approval reaches whichever door is armed |
| 2 | `bee gate` warns on its own line when the durable stamp was skipped (R4) | the silent half-write is what hid this for three merges | approve on a closed-record feature: the output names where it landed and what it could not reach | the next operator sees the split instead of discovering it at the door |

Single slice — both phases are the current slice. They touch disjoint files and
run in parallel.

## Test matrix

High-risk, so the negative cases carry the weight. Each writer judges existing
coverage first and authors only the gap.

- **Happy path** — a closed workflow plus a lane file reading `uat: true` merges;
  the same shape passes the close-time door under `uat_stop: "close"`.
- **Negative, the load-bearing set** — with the workflow closed and no live
  record, each of these must still read UNAPPROVED and refuse: no lane file at
  all; a lane file with no `approved_gates`; `approved_gates` with no `uat`;
  `uat: false`; `uat` present but not a boolean.
- **Precedence** — a live workflow saying `false` beats a lane file saying
  `true`; the live record is still consulted first and its answer stands.
- **Unchanged** — every existing merge-side and close-side uat test passes
  without modification, including the lane-classification exemptions and the
  `uat_stop` placement matrix.
- **Warning** — `bee gate` on a closed-record feature emits the warning; on a
  live-record feature it does not.

## Out of scope

- `bee state workflows close --all-but-active` closing other sessions'
  unfinished features. That is the upstream cause and the pattern's own closing
  rule names it, but it is a separate change to housekeeping semantics with its
  own blast radius.
- Reopening closed workflow records, by any command.
- The `uat_stop` placement policy itself, shipped by `uat-stop-placement` D1-D5
  and untouched here.
