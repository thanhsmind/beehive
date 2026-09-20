# Idempotent Herding Receipts — Context

**Feature slug:** idempotent-herding-receipts
**Date:** 2026-09-20
**Shaping session:** complete
**Scope:** Standard
**Domain types:** CALL | RUN

## Feature Boundary

A `bee herding run` dispatch is safe to replay: the job mailbox it already
wrote is the receipt, so the same `job_id` + round with the same brief returns
that stored result instead of spawning a second worker, a changed brief refuses
loudly, and the Pi drain's injected header names `job_id + round` as the dedupe
key so a genuine later round is never read as a replay. It ends at the dispatch
door, the mailbox directory and the drain header, plus a prune verb for the
mailbox tree. Delivery itself is unchanged and still at-least-once; cells,
gates, proof and worktrees are untouched.

## Why now

`docs/history/research/pi-workflows-xia.md` (2026-09-02, confidence 92) closed
with five design rules worth taking from pi-workflows and no code. Rule 3 —
idempotent command receipts — is the one that names a hole bee already has:
the mailbox advertises at-least-once delivery and pushes dedupe onto the model
as prose. Scouting that prose on 2026-09-20 found the prose is also wrong for
multi-round jobs (see D2's evidence).

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

Store provenance: D1–D4 were logged at the first shaping pass on 2026-09-20.
D1 and D2 carry `touches:a05636f8-d308-466c-95a6-80ca3490c585` (the
pi-result-mailbox design settlement that named `job_id` the dedupe key and
forbade any exactly-once claim). That decision is NOT superseded — see D1.

**Second shaping pass, same day.** Slice 1 (D2) shipped and merged at `aa918c8`.
The plan-step hat wave then found that D1, D3 and D4 could not be planned: four
questions their spec could not answer, and one delete with no undo. D5–D8 below
answer all four and retire one. **D4 is partly superseded** — its receipt-home
half stands, its prune half is dropped (D8) and its digest-input clause is
replaced (D6). The live restatement of all three is store decision
`11b68a1b-0bb6-4d83-800b-fe58a9079097`.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Receipts reach **every** `bee herding run` dispatch, not only the Pi drain. A re-run whose `job_id` + round already has a stored receipt **returns that receipt** instead of spawning a second worker. Delivery stays **at-least-once** — `a05636f8` stands, and no exactly-once claim is added anywhere in code, tests or docs. The receipt is what makes a replay provably safe, never what stops the replay. Store decision `2eb61b7b`. | A drain-only ledger helps Pi alone; every runtime shares `bee herding run`. Keeping at-least-once is what lets this ship without superseding `a05636f8`. |
| D2 | The replay key is **`job_id` + round N**, never `job_id` alone. The Pi drain's injected header gains the round and names `job_id + round` as the dedupe key. Store decision `2f189c37`. | Evidence, reasoned from code and **not yet reproduced**: one job legitimately writes `result-1.json` then `result-2.json` through `--continue` rounds; `latestResultFile` (`.pi/extensions/bee-guard.ts:720`) always reads the highest round; the marker is named `<job_id>.json` with no round (`packages/bee-rs/crates/bee/src/herding/run.rs:2287`). So a requeued claim or a round-2 marker injects a genuinely new result under an already-seen `job_id`, and the header's own line (`bee-guard.ts:802`) then tells the model to drop it. |
| D3 | A conflict **refuses loudly**: a dispatch whose `job_id` + round already has a stored receipt but whose **brief digest differs** errors, names the stored receipt path and both digests, and spawns nothing. The caller must pass a fresh `job_id`. A matching digest returns the stored receipt (D1's idempotent half). Store decision `42a59eea`. | Rule 3's own shape. A silent extra worker on a mistaken id reuse is the failure this feature exists to prevent. |
| D4 | ~~The receipt's home is the **existing** job mailbox directory `.bee/mailbox/<job-id>/`, never a new store~~ — **this half STANDS**. ~~adds only a **brief digest** beside them~~ — replaced by D6. ~~Retention is in scope: a `bee herding prune` verb~~ — **dropped by D8**. Store decision `adfa3f1c`, restated live as `11b68a1b`. | The deletion test and single-source-of-truth still hold for the receipt's home: a second receipt store would be a pass-through over the mailbox dir. The retention half rested on `ls .bee/mailbox \| wc -l` = 341 with no checkout named; the real figure is 344 dirs / **14 MB**, gitignored — see D8. |
| D5 | A recognised replay returns the **same shape a normal run returns**: `bee herding run` prints its ordinary JSON envelope carrying the stored result, exits 0, and **still writes the result-inbox marker**. Store decision `5d0692b5`. | The marker is not optional. Without it a detached Pi dispatch receives a success envelope and then waits forever for an injection that never comes — the opposite of returning the receipt. Named by the `hat-facts-gaps` seat as the sharpest hole in D1. |
| D6 | The digest covers the **task and files** of the dispatch, **never** the rendered `brief-N.txt`. Amends D4's Terms table. Store decision `7b3a93d4`, restated live as `11b68a1b`. | `render_brief` embeds the absolute worktree root and the configured proof command (`mailbox.rs:277,294,312-316`), so the same task in a moved or re-created worktree digests differently and D3 would refuse genuine work — a refusal that reads as a bee bug. Reached independently by `hat-risks` and `hat-alternatives`. |
| D7 | The receipt pre-flight runs in the **parent process, before it re-launches itself detached** — above the `spawn_detached_runner` branch at `run.rs:3944-3958`, **not** inside `fn execute`. Store decision `10e20c92`. | On an `--inbox-session` run the parent prints `detached_envelope` and exits SUCCESS before the child ever reaches `execute()`, so a refusal there goes to a stderr nobody reads, with no marker written. This placement also fixes the funnel error: `run()` branches on `no_pane` itself, so `execute()` is not above every dispatch. |
| D8 | **`bee herding prune` is dropped.** This feature adds no prune verb and deletes nothing. Collecting the mailbox tree is a backlog row. Store decision `11b68a1b`. | 344 dirs / 14 MB, gitignored, causing no problem. `hat-value` scored it a materiality FAIL; `hat-risks` showed the "no terminal result" safety test is wrong for three live shapes (finished-but-undelivered, multi-round in flight, usage-limit paused) and that `.bee/mailbox` has no git history, so a wrong delete has no undo. It was also the single reason this feature routed high-risk. The user chose this knowing the 14 MB figure they did not have when they first asked for prune. |

### Implementation rule (agent-decided, not a user decision)

**Round resolution for the pre-flight.** The pre-flight resolves the round
exactly as the callee does: **1** for a fresh run (`execute_no_pane` hard-codes
`round: 1` at `run.rs:2904`; `execute_new` writes `brief-1.txt` at
`run.rs:2635`), and `latest_result_round + 1` for `--continue`
(`run.rs:3323-3327`). It is forced by the code, not a choice — recorded here so
no implementer re-derives it. A pre-flight that assumes round 1 for a
`--continue` would refuse **every** continue on every runtime (`hat-risks`).

### Agent's Discretion

Digest algorithm and its encoding; where the stored digest lives inside the job
mailbox and under what filename — it must fall outside both the `brief-N.txt`
and `result-N.json` parser families (`run.rs:3127-3133`,
`.pi/extensions/bee-guard.ts:729`), so something like `digest-N.sha256`; the
exact refusal wording for D3 (the named parts are fixed: stored receipt path,
both digests); test file placement.

### Fail-open rules the pre-flight must honor (from `hat-risks`)

- A **readable** digest that **differs** refuses. A **missing** digest beside a
  present result **returns the receipt**. An **unreadable** digest **proceeds
  to spawn** with a note on stderr — otherwise all 344 legacy mailboxes and any
  ENOSPC fault would refuse every dispatch.
- The receipt test keys on **`result-N.json` presence**, never on the digest or
  the brief: a worker that crashed after writing `brief-1.txt` and before its
  result must stay re-runnable under its own id.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| receipt | The stored result of one dispatch round — `result-N.json` in the job mailbox, plus the brief digest that says which payload produced it |
| round | The N in `brief-N.txt` / `result-N.json`; one job_id carries several rounds through `--continue` |
| brief digest | A fixed-length fingerprint of `brief-N.txt`, the value D3 compares to tell a replay from a conflict |
| replay | A second arrival of a `job_id` + round that already has a receipt with the **same** brief digest |
| conflict | A second arrival of a `job_id` + round whose brief digest **differs** from the stored receipt's |

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/herding/mailbox.rs` — `mailbox_dir`
  (`.bee/mailbox/<job-id>/`) and the job.json / brief-N / ack-N / result-N
  contract. The digest joins this family; the prune verb reads this tree.
- `packages/bee-rs/crates/bee/src/herding/run.rs` — the poll-for-`result-N.json`
  loop, the `--continue` round logic, `write_inbox_marker` (line 2266) and
  `build_child_argv` (line 2720).
- `packages/bee-rs/crates/bee/src/herding/wave_ledger.rs` — the append-only
  `.bee/wave-ledger.jsonl` pattern (compact JSON + `\n`, folded at read time,
  never rewritten). The shape to copy if any index is ever needed; **not** a
  place to duplicate the receipt (D4).
- `.pi/extensions/bee-guard.ts` — `latestResultFile` (720),
  `renderResultInjection` (781) and the drain loop (894). D2 changes the header
  rows and the dedupe sentence here.
- `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` — the
  stub-under-node harness that already drives the result-inbox drain
  (property 7). D2's failing test belongs in this suite.

### Established Patterns

- Atomic write temp + rename (`crate::fsutil::write_json_atomic`) — the digest
  must land the same way.
- Advisory surfaces never throw (pi-support D3). The drain stays advisory; the
  dispatch-door refusal of D3 is **not** advisory and must fail closed.
- A named refusal carries its own FIX line — every `bee` refusal in this repo
  names the remedy.

### Integration Points

- `bee herding run`'s argument parse and pre-spawn path (`run.rs:304`, `:377`)
  — where D1's receipt lookup and D3's refusal sit, **before** any spawn.
- `write_inbox_marker` (`run.rs:2266`) — the marker gains the round (D2).
- `renderResultInjection` (`bee-guard.ts:781`) — the header gains the round and
  the corrected dedupe sentence (D2).
- The `bee` CLI verb table — `bee herding prune` is a new verb (D4).

## Canonical References

- `docs/history/research/pi-workflows-xia.md` § Five rules worth taking, rule 3
  — the source of this feature; the other four rules stay unshaped.
- `docs/history/pi-result-mailbox/CONTEXT.md` D1–D6 — the mailbox contract this
  extends; D6's one-delivery-path-per-job split is unchanged.
- Store decision `a05636f8-d308-466c-95a6-80ca3490c585` — at-least-once and the
  no-exactly-once rule, both kept (D1).

## Outstanding Questions

### Resolve Before Planning

None.

### Deferred To Planning

- [x] ~~Can a round-2 marker and a requeued round-1 claim coexist on disk?~~ —
      **answered**: one header change was enough. Both transports rewrite the
      marker every round (`run.rs:2296-2302`, `:2892-2896`) and the drain parses
      the round out of the result filename, so no marker change was needed.
      Shipped in slice 1, `aa918c8`.
- [ ] Does any non-Pi caller reuse a `job_id` today? — decides whether D3's
      refusal needs a migration note. `--job-id` is a real flag (`run.rs:325`)
      and bee's own detached launcher re-execs with it (`run.rs:3895`), so the
      answer is at least "bee itself does". Confirm the shape before slice 2's
      gate.

## Deferred Ideas

- Rules 1, 2, 4 and 5 of `pi-workflows-xia.md` § Five rules worth taking —
  crash semantics per effect, the waiting-work invariant, the machine-readable
  gate subject, and store-enforced human provenance. Each is a separate change
  to bee's doctrine or store; none is a docs edit. Not in this feature.
- Making async injection exactly-once — would supersede `a05636f8`. D1
  deliberately keeps at-least-once instead.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.

D2 is reasoned from code and not reproduced. `bee-principle-red-before-green`
applies: the first cell drives a round-2 injection and watches it fail for the
reported reason before anything is fixed.
