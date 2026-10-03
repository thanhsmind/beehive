---
artifact_contract: bee-research/v1
topic: seatworks-slp-pi-small-models
depth: deep
date: 2026-10-03
---

# Seatworks mechanisms for clearer SLP on Pi

## Bottom Line

- Recommendation (ladder rung): **adapt-upstream**. Reuse Bee's state, claims, dispatch, result inbox and proof checks. Adapt two Seatworks designs: operations bound to the caller's role, and a narrow worker context with one return path. Keep workflow decisions in Rust. Pi shows and runs those decisions.
- Why this is the lightest credible path: Bee already has the dispatch tool, the result tool and restart-safe result delivery. The defects are at the edges: a worker prompt that conflicts with its transport, and a result tool that guesses the job. Seatworks shows a clean shape for both edges.
- Why the next-best rung lost: **build** (a new task board or scheduler) duplicates working Bee parts. **reuse** as-is leaves the two defects below in place. A direct code port is not possible: Seatworks is a TypeScript Paseo plugin, Bee is Rust plus a TypeScript Pi extension.
- Confidence: 80% that the four mechanisms below are real and fit Bee. Below 30% that they make small models more stable. No small model was run.
- Suggested next step: **bee-shaping** for slice 1, after the user supplies the target model and one failing transcript.

This is a research and design proposal. It is not an approved plan, and it settles no product rule.

## Repo Snapshot

| Source | Revision | Stack | Scope read |
|---|---|---|---|
| Bee | `adf52d0273eb517c062cea79d3d136a09f14296d` | Rust CLI, TypeScript Pi extension | Hooks, dispatch renderer, worker brief, Pi tools, result delivery |
| Seatworks | `2e11099f49eb788b8eb707c14a6f26ecd5987318` | TypeScript, Node 24+, `@getpaseo/plugin` 0.8.0 (plugin manifest 2.0.0) | Role catalog, desk operations, worker prompt, patrol, outbox |

Local clones: `/home/thanhsmind/Projects/goglbe/beehive` and `/home/thanhsmind/Projects/refs/seatworks`.

Constraints that shape the answer:

- pi-1-0-upgrade D5 (`docs/history/pi-1-0-upgrade/CONTEXT.md`): the harness forces the next call; the Bee CLI decides what it is. Pi must not hold a second state machine.
- herding-worker-standalone D3 (`docs/history/herding-worker-standalone/CONTEXT.md`): a herded worker gets no Bee preamble, guards or nudges.
- Bee's supervisor is a read-only observer by recorded decision (`docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md`). Seatworks' supervisor opens and closes lanes. The two words name different jobs.
- Independent review in Bee is user-invoked. It never runs as an automatic stage.

## Question & Assumptions

- **Asked:** Which Seatworks mechanisms make Supervisor–Lead–Peer (SLP) work clearer on Pi, so a small model can run it?
- **Success means:** a small model makes fewer invalid calls, starts no duplicate jobs, loses no results, and closes no cell without verified proof.
- **Not yet supplied:** the target small model, and a transcript where it fails today. Without them, "better on small models" stays a hypothesis.

Related work. [Seatworks distill](seatworks-xia.md) covers general Seatworks practice. The SLP discovery map is `docs/discovery/slp-supervisor-lead-peer/MAP.md`; its digests cover the observer ([slp-observer-surfaces](slp-observer-surfaces.md)), supervisor placement ([slp-supervisor-placement](slp-supervisor-placement.md)), dissent ([slp-dissent-surfaces](slp-dissent-surfaces.md)), blind lanes ([slp-blind-lanes-surfaces](slp-blind-lanes-surfaces.md)) and contract status ([slp-contract-request-surfaces](slp-contract-request-surfaces.md)). [pi-peer distill](pi-peer-distill.md) is the source of the Pi result inbox. This report covers only role authority, dispatch, handback and continuation on Pi.

## Findings

### Local — Bee

1. **Pi already has dispatch tools.** `.pi/extensions/bee-guard/tool-dispatch.ts` defines `bee_dispatch` and `bee_advisor`. It calls `dispatch prepare`, checks the returned command, and starts `herding run` without a shell. Complete this path; do not build it again.
2. **The Pi tool omits some dispatch inputs.** It carries kind, role, cell, worker and purpose. It does not carry stage, feature, claim or expertise. Under an approved v2 role plan, a non-cell dispatch without a stage is refused (`packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1786`). The tool then cannot express a valid call, and the model must fall back to the CLI. *Code trace only; this plan state was not reproduced live.*
3. **A herded cell worker gets two opposite orders.** `prepare.rs:1125` renders `packages/bee/prompts/worker-cell.md`, which orders Bee commands and `cells finish`. The herding path sends that prompt inside a wrapper (`packages/bee-rs/crates/bee/src/herding/mailbox.rs:363`) that orders the worker to ignore all Bee workflow instructions. The wrapper is intentional. The inner prompt must match it on this transport. A small model is the most likely to obey the wrong one.
4. **Worker hook suppression is intentional.** Under the herding worker marker, only the `activity` hook runs (`packages/bee-rs/crates/bee/src/hooks/mod.rs:101`). D3 records why: workers used to restart the leader's workflow. Do not turn all hooks back on. Narrow context and a write-scope guard are separate needs.
5. **Tool narrowing is not role authorization.** `allowed_tools_for` (`packages/bee-rs/crates/bee/src/hooks/stage_tools.rs:65`) gives workers the full tool set and gives the leader a narrower set during approved execution. The write guard also exists. Neither checks a worker's tool call against its assigned job.
6. **The result tool can write to the wrong job.** `.pi/extensions/bee-guard/tool-verdict.ts:104` uses `BEE_HERDING_JOB_ID` when set. Without it, the tool picks the newest mailbox directory. It checks only that `proof` is a string (`tool-verdict.ts:92`), so an empty proof passes. The reproduction under Evidence confirms both. A transport result is not an accepted cell; the cap checks still apply after it.
7. **Continuation is a bounded reminder.** `answer` (`packages/bee-rs/crates/bee/src/hooks/session_close/obligations.rs:191`) marks obligations as served before it returns them. The reminder asks the model to finish or hand off. This keeps the one-continuation contract. It does not prove the model did the transition.
8. **Result recovery already exists.** `.pi/extensions/bee-guard/result-inbox.ts:322` claims a result by atomic rename and requeues it when injection fails. The claim holds until the turn settles. A second mailbox would duplicate this.

### Upstream — Seatworks

Paths are relative to the pinned Seatworks clone.

| Mechanism | Evidence | Value for Bee |
|---|---|---|
| Roles select capability and tool sets | `plugin/roles.json`; `plugin/server/catalog/kit.ts:545` | Keeps authority apart from model choice and transport |
| Caller identity selects permitted operations | `plugin/server/desk/desk.ts:233`, `:267` | A Peer cannot act as a Lead by naming another role in its arguments |
| Worker reports through `done` and `ask` only | `plugin/server/desk/tools/worker.ts:34`, `:104` | Removes workflow bookkeeping from the small model's task |
| Handback is not acceptance | `plugin/server/desk/tools/worker.ts:62` | A result waits for the Lead; queued or merged work cannot go back to done |
| Identical pending calls join one run | `plugin/server/desk/desk.ts:191` | A repeated request starts no duplicate work. In memory only; a restart loses it |
| Worker context hides other roles | `plugin/roles.json`; `plugin/server/catalog/kit.ts:482`; `plugin/content/prompts/PEER.md` | The worker sees its task, owned files, checks and one return path |
| Patrol finds missing agents and open questions | `plugin/server/runtime/patrol.ts:46` | Recovery reads recorded state; the model need not remember pending work |

Limits: Seatworks still relies on prompts for some file-ownership rules. Its default Lead and Supervisor use a strong model. It has no small-model benchmark. These are useful designs, not proof that Seatworks solves weak-model coordination.

### Docs

Web requests for the pinned Seatworks files and the Pi 1.0 documentation returned cache misses. Version claims therefore use the local sources and the locked pi-1-0-upgrade context, not fresh web pages.

### Inference — proposed application

SLP means Supervisor, Lead and Peer. Make each job explicit, and do not change Bee's observer by accident.

| Job | Owner | Authority |
|---|---|---|
| User intent, scope, approvals | User-facing coordinator | Recorded decisions and existing gates |
| Assign work, judge results, integrate | Lead | Approved plan and claimed cells |
| Do one bounded task | Peer | Assigned files and acceptance checks |
| See drift, raise questions | Existing supervisor observer | Read-only observations |

The coordinator and Lead can stay one session at first. Split them only when evidence shows the extra cost helps. The observer stays separate.

Target flow:

1. The Lead selects approved work.
2. Rust claims and prepares it.
3. Pi starts the job.
4. The Peer returns evidence.
5. Rust checks the job identity and records the result.
6. The Lead checks the artifact, then accepts or rejects.

A missing result stays pending. A "complete" report alone never closes a cell.

**Slice 1 — remove ambiguity at dispatch and return.** Fixes findings 2, 3 and 6.

- Render a herding-specific cell prompt. The Peer implements, tests and returns a result; the Lead keeps the Bee bookkeeping. Native workers keep their current contract.
- Let the Pi dispatch tool pass, or derive, the approved stage and feature. Reuse the existing claim-and-reserve step.
- Bind each result to an explicit job, round, cell and plan revision, taken from trusted state. Refuse a call with no job identity instead of picking the newest mailbox. Refuse an empty proof.
- On failure, return structured data that names one valid recovery action, plus a readable message.

**Slice 2 — derive each role's permitted next operations.** Extend the existing Rust decision path with operation data, so Pi holds no second state machine (D5). Proposed fields, not an existing API: actor, assigned cell, current revision, allowed operations, pending result ids, blockers. Keep tool visibility apart from authorization: each operation checks its caller and current assignment. A worker write-scope guard needs an explicit amendment to D3, and must not turn on the worker's full Bee workflow.

**Slice 3 — recover without duplicate actions.** Reuse job ids, result rounds, inbox claims and obligation records. A retry returns the pending operation where it can. A served reminder is not finished work. Keep the one-continuation limit unless a separate approved design changes it. A blocked flow names its blocker instead of calling the model again.

**Slice 4 — measure model behavior.** Compare the current flow with the new flow on the same Pi version, task set, model and effort setting, with one stronger model as a baseline. Measure invalid calls, duplicate dispatches, lost or wrong-job results, completion with verified evidence, human interventions, time and cost. Record failures as well as successes. Cases: success, failed proof, no job identity, repeated dispatch, replayed result, worker crash, user scope change, Lead restart, two concurrent disjoint cells. Run deterministic contract tests first, then the real Pi flow. A static check cannot prove small-model stability.

### Design challenges

| Question | Seatworks | Bee answer and risk |
|---|---|---|
| Who may accept a result? | Role-bound desk operation | Lead acceptance stays apart from Peer completion; cap checks stay |
| Who owns workflow state? | The desk | The Rust CLI; a copy in Pi would drift |
| Does a worker need the full workflow? | No, narrow Peer prompt | No; herded workers must not run it (D3) |
| Does a repeated request start another job? | Identical in-flight calls join | Reuse job records; an in-memory cache does not survive restart |
| Must review run automatically? | Yes, review is in the flow | No; Bee review stays user-invoked |
| Does a reminder prove progress? | The desk checks task state | Bee must read state after a reminder; no unlimited continuation |

## Risks, Unknowns, Follow-Ups

- **Evidence gap:** no small model was run. All gains are hypotheses until slice 4 runs.
- **Unreproduced:** finding 2 is a code trace. Reproduce it before slice 1 depends on it.
- **Decision risk:** a worker write-scope guard touches D3. It needs a recorded amendment, not a silent change.
- **Open for the user:** which small model is the target, and is there a failing transcript? Slice 1's shape depends on the answer.

### Evidence

On Node `v24.19.0`, an isolated run imported `executeVerdictTool` into a temporary directory. The directory held one unrelated job mailbox. `BEE_HERDING_JOB_ID` was unset. The call sent `status: done`, an empty file list and an empty proof.

Output:

```json
{"case":"no job identity, empty proof, unrelated mailbox","accepted":true,"wrote_unrelated_result":true,"sandbox":"/tmp/bee-verdict-research-kAPRUW"}
```

This proves the function behavior. It does not prove that a real session wrote to the wrong job, or that an invalid cell passed the cap checks.

Not run: the configured read worker (Herdr returned `Operation not permitted`, so the leader read the sources itself), the full Rust suite, and any live model evaluation. No source file changed. File pointers were rechecked against both pinned revisions.

Discarded suspicions: Bee does not lack a dispatch tool, a structured result tool or restart-aware result delivery. A generic task-board rewrite or an extra scheduler has no support in this reading.

### Principles applied

- `verify-before-reporting`: reproduced the mailbox fallback before reporting it; kept finding 2 labeled as a code trace.
- `chestertons-fence`: read D3 before proposing narrower enforcement; the report keeps worker hooks off.
- `single-source-of-truth`: role and transition authority stays in Rust; Pi only shows and calls it.

## Source Pack

- **Local:** `.pi/extensions/bee-guard/tool-dispatch.ts`, `tool-verdict.ts`, `result-inbox.ts`; `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`; `herding/mailbox.rs`; `hooks/mod.rs`; `hooks/stage_tools.rs`; `hooks/session_close/obligations.rs`; `packages/bee/prompts/worker-cell.md`; `docs/history/herding-worker-standalone/CONTEXT.md`; `docs/history/pi-1-0-upgrade/CONTEXT.md`; `docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md`.
- **Upstream (Seatworks @ `2e11099f`):** `plugin/roles.json`, `plugin/server/catalog/kit.ts`, `plugin/server/desk/desk.ts`, `plugin/server/desk/tools/worker.ts`, `plugin/server/runtime/patrol.ts`, `plugin/content/prompts/PEER.md`.
- **Docs:** none fetched (cache misses); see Docs above.
