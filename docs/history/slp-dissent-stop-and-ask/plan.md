---
artifact_contract: bee-plan/v1
mode: high-risk
---

# Plan: SLP Dissent / Stop-and-Ask

Mode: `high-risk` — 4 risk flags: data-model, public-contracts,
covered-contract-change, multi-domain
Why this is the least workflow that protects the work: the obligation this
feature adds is only real at two refusal doors and one worker contract, so a
wrong shape does not fail loudly — it silently makes dissent decorative, which
is the exact outcome 4b7aa303 exists to prevent.

Revised once after an advisor consult
(`reports/advisor-digest.md`) and an independent coverage review. Both
found the first draft wrong in ways that would have shipped a toothless
feature; what changed is recorded in "What the review wave changed" below.

## Requirements (from CONTEXT.md)

- 787a9eb0 — bee's locked rules win; dissent adds an obligation, never relaxes a gate.
- a020319d — cluster 2 of four; blind lanes and contract/original-request stay out.
- 4b7aa303 — a blocker-severity dissent pauses the RELATED work only, and the
  orchestrator owes one of three answers (accept and log / reject with
  reasoning / escalate a rung), **recorded in the decision log**, before that
  work resumes.
- a2affcba — a cells-level record `{target, claim, alternative, severity}` through
  the CLI; `bee close` AND `bee worktree merge` refuse while a dissent lacks its
  verdict (the judge-debt enforcement shape); blocker severity rides the blocked
  status so related work stays unclaimable; StopAndAsk is the round-mailbox shape
  with `options[]` and `leaning` on the `[BLOCKED]` form; no live mid-flight Q&A;
  the new verb never reuses `bee cells escalate`, which means model tier.

## Discovery

Inspected every surface the mechanism names. Four findings changed the shape.
The first two are recorded as decision e29918f7:

- `bee worktree merge` has **no** judge-debt door. Its only cell-debt
  precondition is proof debt (`packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs:154-172`).
  Verified: `rg -n judge packages/bee-rs/crates/bee/src/verbs/worktree/` hits
  tests and one unrelated comment only.
- A blocked cell does **not** refuse `bee close`; it only suppresses cell
  retirement. The unclaimable tooth is the scheduler's dependency check
  (`packages/bee-rs/crates/bee/src/verbs/cells/schedule.rs:98-111`).
- The judge-debt arm this feature copies is **route-gated** to standard and
  high-risk (`packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1462`). a2affcba is
  unconditional, so a verbatim copy would switch dissent off in every lane below
  standard — including `small`, which dispatches workers.
- Cell-trace free text is **not** secret-scanned. `find_secret_pattern`
  (`packages/bee-rs/crates/bee/src/verbs/cells/audit.rs:60`) reaches the trace through
  nothing; `run_block` writes its reason straight through
  (`packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1136`).

Full anchor list is in CONTEXT.md "Existing Code Context", corrected against the
source in this same pass.

## Approach

Build the record and its teeth first, the doors second, the worker contract
last (a2affcba, 4b7aa303). The record is the thing every other part reads, and
the build order the map cites puts validators and schemas before wiring.

The two doors are separate phases because they are different code: close gains
one more arm in an existing door builder, while merge gains a precondition it
has never had. Both must read the **same** debt helper and the **same**
`dissent-deferral` escape — an escape that worked at one door and not the other
would be neither the judge-debt shape a2affcba names nor the proof-debt shape
merge already has.

Named implementation choices (CONTEXT.md left these to planning):

| Choice | Value | Why |
|---|---|---|
| Record verb | `bee cells dissent` | inside the cells group where the record lives; does not touch the banned `cells escalate` name |
| Verdict verb | `bee cells dissent-verdict` | mirrors the `judge` / `judge-record` pairing already in this group |
| Severity set | closed: `blocker`, `consider` | a2affcba names blocker; consider is the grade the research found has no carrier today |
| Verdict input | flags `--verdict accept\|reject\|escalate --reason "..."` | a judge verdict takes a `--file` schema because it is FOREIGN-model output; a dissent verdict is session-authored, so the schema module buys nothing. Deviation from the established `--file` pattern, recorded here |
| Debt helper | one shared function in `verbs/cells/`, read by both doors | the placement `feature_proof_check` already uses for exactly this reason |
| Lane gate | **none** — the door exists in every lane | a2affcba is unconditional, and a dissent record only exists because a worker wrote one, so its existence is the gate |
| Claim on the record verb | the record releases the exiting worker's claim | a worker that dissents is exiting; leaving the claim would make the orchestrator's verdict trip an ownership guard and route `--force-ownership` as the normal path |
| Claim on the verdict verb | no worker-shaped ownership guard | the verdict is the orchestrator's by 4b7aa303 |

Rejected alternatives:

- One phase for both doors — rejected: merge has no door to extend, so the
  "copy the judge-debt arm" estimate is wrong for half the work.
- Copying the proof-debt style literally at merge — rejected: it has no escape,
  which would strand a logged `dissent-deferral`.
- Dissent stored in its own file store — rejected: the record is per-cell and
  a2affcba says cells-level; the cell trace already carries `semantic_judge` this way.
- Reusing `bee cells escalate` — refused by a2affcba: it means model tier.

Risk map:

| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| Dissent record + verbs | MEDIUM | new record type and two new verbs; the cells namespace has NO served-but-undeclared guard, so a missing registry row passes green | unit tests plus a hand-written declaration probe; `--test registry_dispatch --test registry_contracts` |
| Blocker tooth | MEDIUM | touches the blocked status and the scheduler dependency check, which existing tests assert | scheduler and cells tests; prove the release path, not only the block |
| Close door | LOW | one more arm in an existing builder; the pattern and its escape both exist | close-driver tests, red first |
| Merge precondition | MEDIUM-HIGH | genuinely new, and a wrong refusal blocks landing work | worktree merge tests, both refuse and pass paths |
| Worker contract | MEDIUM | `MailboxResult` is parsed from foreign-agent output; three surfaces must agree, one of them asserted byte-for-byte | mailbox parse and brief tests plus the prompt-projection tests |

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1. The record, its teeth, and its answer | `bee cells dissent` writes a validated `{target, claim, alternative, severity}` record onto the cell trace, secret-scans its free text, releases the writer's claim, and — at `blocker` severity only — blocks the target cell so dependents stay unschedulable. `bee cells dissent-verdict` takes `--verdict accept\|reject\|escalate --reason`, appends the answer, **writes it to the decision log**, and does its own release rather than shelling into the claim-guarded reopen path. Both verbs declared in the handler module and its group `mod.rs`, the cells dispatch table, the registry payload, the router coverage line, and the catalog flag ratchet | Everything downstream reads this record, and the tooth is one call inside the same verb — splitting it would ship a severity that means nothing and then rewrite its tests | Record a blocker dissent: the cell goes blocked and its dependent stops being schedulable. Answer it: the decision log carries the answer and the cell releases. An unknown severity, an empty claim, and a verdict outside the closed set each refuse by name and write nothing | Phases 2–4 |
| 2. The close door | A dissent-debt door joins `build_close_report_doors`, counting **archive-inclusive**, blocking in EVERY lane while any dissent lacks a verdict, cleared by a logged `dissent-deferral` decision naming the feature, with a remedy line that names `unarchive` first | The obligation is not real until a door refuses, and close is where the pattern already exists | Record a dissent, `bee close` refuses naming the cell and the remedy; record the verdict, close goes green. A `tiny`-lane feature refuses exactly the same way | Phase 3 |
| 3. The merge door | `bee worktree merge` gains its own dissent precondition beside the proof check, reading the SAME shared debt helper and the SAME `dissent-deferral` escape the close door reads, refusing with its own typed code, and proceeding ungated when the merge identity carries no feature | The branch must not land while a worker's blocker question is unanswered; this is the half that is new code | An unanswered dissent makes `bee worktree merge` refuse; the verdict clears it, and a logged `dissent-deferral` clears it at BOTH doors identically | Phase 4 |
| 4. StopAndAsk on the worker form | `options[]` and `leaning` join the `[BLOCKED]` result contract in all THREE surfaces — the herding mailbox brief, the parsed result struct, and the JSON the run verb re-emits to the orchestrator — plus the swarming worker contract, which also gains one line naming the dissent verb and one line in the orchestrator's per-result list. The boundary signals (contract or API change, trading data quality or user experience for a technical target, a new dependency) land in the worker instructions | The worker half: a worker that can only say "blocked, here is prose" cannot hand the orchestrator a choice | A worker result carrying two options and a leaning reaches the orchestrator as structure, not prose; a result with neither parses exactly as before | — |

Current slice = Phase 1.

## Open question for the gate

**A herding-lane worker cannot record a dissent at all.** The herding brief
orders "Never run any `bee` command"
(`packages/bee-rs/crates/bee/src/herding/mailbox.rs:310-315`), and the run verb does
nothing with a blocked result but label it
(`packages/bee-rs/crates/bee/src/herding/run.rs:2573-2593`). a2affcba says the record is
written "through the CLI", which only a swarming worker can do. So StopAndAsk
reaches herding workers and dissent-with-teeth does not.

Recommendation: scope herding-lane dissent OUT of this feature and file it as a
backlog item. Carrying dissent fields on the mailbox result and having the
control loop transcribe them is a second mechanism for the same record, and
a2affcba named one. The alternative is to add it to Phase 4 and grow the phase.

## Test matrix

High-risk: probes per applicable dimension. Each cell's writer judges existing
coverage first and authors only the gap.

| Dimension | Applicable | Probe |
|---|---|---|
| 1. User types | YES | A worker records a dissent against a cell it owns; a worker records against a cell it does not own; the orchestrator records a verdict WITHOUT needing an ownership override |
| 2. Input extremes | YES | Empty claim, empty alternative, a severity outside the closed set, a verdict outside the closed set, a verdict with no reason — each refuses by name and leaves the cell file byte-identical |
| 3. Timing | YES | A verdict before any dissent exists; a second verdict on an already-answered dissent; a dissent on an already-capped cell; a dissent on an archived cell |
| 4. Scale | YES | Several dissents on one cell, and dissents across several cells of one feature — the door names every unanswered one, never just the first |
| 5. State transitions | YES | open → dissent(blocker) → blocked → verdict → released, for all three verdict kinds. A `consider` dissent changes no status. The release is the verdict verb's own write, never a call into reopen |
| 6. Environment | NO | No new environment input; the store follows the cell store's own root resolution, and session identity resolution is unchanged and covered by dimension 8 |
| 7. Error cascades | YES | The verdict's decision-log write fails → the dissent stays UNANSWERED rather than half-answered, and the doors still refuse (the fail-closed precedent from sup-11) |
| 8. Authorization | YES | The record verb releases the writer's claim; the verdict verb needs no override on a cell a worker still held; the audited override path behaves as it does elsewhere |
| 9. Data integrity | YES | The record is append-only; an existing trace key is never overwritten; a refused call leaves the cell file byte-identical; the claim and alternative text is secret-scanned before it is written |
| 10. Integration | YES | The verb is served AND declared. Because the cells namespace has no served-but-undeclared guard (`tests/registry_dispatch.rs:360` scans only devtools and herding), Phase 1 authors that missing half by hand; plus `registry_contracts` green and the catalog ratchet bumped on purpose with its reason |
| 11. Compliance | NO | No regulated data. The reason the first draft gave was FALSE — cell-trace text is unscanned today — so Phase 1 adds the scan rather than claiming an existing one |
| 12. Business logic | YES | The verdict set is closed and exhaustive. The blocked reach is dependency-only, so the probe pins BOTH known limits: a sibling sharing files but carrying no dependency edge stays claimable, and an already-claimed in-flight dependent is untouched |

## What the review wave changed

- The verdict's decision-log write became a named Phase-1 deliverable. It was a
  requirement of 4b7aa303 that survived only as a failure probe.
- The close door is explicitly ungated by lane. The arm it copies is gated to
  standard and above, which would have voided a2affcba in every smaller lane.
- The blocker tooth moved from Phase 3 into Phase 1, so the severity means
  something the day it ships and its tests are written once.
- The merge precondition reads the same escape as the close door, through a
  shared helper, instead of copying an escape-less pattern.
- Archive-inclusive counting and an `unarchive`-first remedy were added.
- The claim-ownership shape was decided in both directions rather than left as
  "matching the existing guard".
- Dimension 11's reason was replaced; the secret scan it assumed does not exist,
  so Phase 1 adds it.
- Phase 4 gained its third surface — the JSON the run verb re-emits — without
  which the orchestrator never receives the options a worker offered.
- Both verbs are named, so every declaration surface has something to declare.

## Out of scope

- A live mid-flight question-and-answer channel between orchestrator and worker (a2affcba).
- Any change to `bee cells escalate`, which keeps its model-tier meaning.
- Clusters 3 and 4 — blind lanes, and contract status with the verbatim original request (a020319d).
- Any change to gate law, merge law, or permission posture (787a9eb0).
- Herding-lane dissent — pending the gate's answer to the open question above.
