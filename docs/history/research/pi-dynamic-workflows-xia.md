---
artifact_contract: bee-research/v1
topic: pi-dynamic-workflows-xia
depth: deep
date: 2026-09-20
---

## Bottom Line

- **Recommendation (ladder rung): reuse + adapt-upstream.** Four of the six
  changes below wire together parts bee already owns (`brief_sha256`, session
  heartbeats, `.bee/wave-ledger.jsonl`, the byte-parity render tests). Two are
  genuinely new records. **No engine port.**
- **Why this is the lightest credible path**: bee's gap is not capability, it is
  that three of its orchestration invariants live in *prose* instead of in the
  store — the four-slot cap, the "worker is alive" claim, and "this dispatch was
  already done". pi-dynamic-workflows holds the same three invariants in code,
  and the cheapest way to close the gap is to move bee's existing signals into
  the same shape, not to import a runtime.
- **Why rung 4 (build) lost**: the deletion test. Delete pi's engine and inline
  it at bee's call sites — lanes, cells, gates, claims, worktrees, proof all
  reappear from bee's own store. Only three things do not: a replayable call
  journal, a suspending gate record, and a frozen-sentence guard on guidance.
- **Why rung 2 (built-in) lost**: nothing in bee's current CLI already does
  replay, slot reservation, or outcome classification. Searched and named below.
- **Confidence: 88%.** The source is read at a pinned commit and every claim is
  anchored. The remaining doubt is whether the user wants bee's gate to become a
  suspension at all — that is a doctrine change, not an engineering one.
- **Suggested next step: bee-shaping**, one feature at a time, starting with
  § R2 (the dispatch ledger never shrinks, and an empty fleet has no flag).

---

## Repo Snapshot

**Local (bee).** `Local`

- Rust 2024 workspace, two crates: `bee` (bin, ~249k lines) and `fleet` (lib,
  ~3.2k). Deps: `anyhow serde serde_json sha2 chrono dunce
  unicode-normalization`. No `package.json` anywhere.
- ~4,109 `#[test]`. `commands.test` = `cargo test --release --no-fail-fast
  --manifest-path packages/bee-rs/Cargo.toml`. CI runs the same, plus a Windows
  matrix and `bee dev render-hook-manifests --check`.
- **"bee harness" has no single code home.** `rg -i harness` → 2554 hits in 602
  files, in three unrelated senses: bee's own product label
  (`crates/bee/Cargo.toml:5`), the host agent runtime and its owned dirs
  (`hooks/write_guard/hook_local.rs:157-164`), and an external repo
  `repository-harness` (`docs/decisions/0024-…`). The four real subsystems the
  phrase reaches are the **dispatch door** (`verbs/drivers/prepare.rs`), the
  **herding executor** (`herding/*` + `crates/fleet`), the **state/workflow
  store** (`state.rs`, `verbs/workflow_store`), and the **hook layer**.

**Upstream (source manifest).** `Upstream`

| Field | Value |
|---|---|
| Repo | `github.com/QuintinShaw/pi-dynamic-workflows` |
| Local path | `/home/thanhsmind/Projects/refs/pi-dynamic-workflows` |
| Ref | `main` |
| Resolved commit | `e29dbcae9a4739161abdf2bd21b302c076256379` (v3.12.0) |
| Narrowed scope | run engine, guidance-integrity machinery, authoring + operator surface |

TypeScript, ESM, one runtime dependency (`acorn`). 22,243 lines of `src` against
**35,808 lines of `tests`** — a 1.6:1 test-to-source ratio. `npm test` =
`biome check` + `tsc` + build + docs freshness + context freshness + unit tests
+ a release gate.

**Not the same source as the earlier brief.** `docs/history/research/pi-workflows-xia.md`
(2026-09-02) studied a different project — SQLite host process, socket protocol,
Rust viewer. This one has none of those. Its still-live section, *§ Five rules
worth taking*, is corroborated below rather than repeated.

---

## Question & Assumptions

- **What was asked**: distill the techniques this extension uses, and apply them
  so bee's orchestration runs correctly, completely and efficiently. Explicitly
  *not* a feature-parity comparison.
- **What success means**: a ranked set of mechanisms, each with the failure it
  prevents and the bee surface it lands on.
- **Assumption still needing confirmation**: that bee wants its human gates to
  become suspensions (release the process) rather than waits. § R5 depends on it.
- **Mode**: `xia` — this brief ends in discussion. Nothing was built.

---

## Findings

### Upstream — the six techniques worth taking

#### T1. The orchestrator is a deterministic program, not a model turn

The workflow script is real JavaScript, parsed with `acorn` and run in a `vm`
realm whose only globals are the declared contract bindings
(`workflow.ts:1595-1608`). `export const meta` must be the **first** statement
and is read by a literal-only walker that rejects spreads, computed keys,
methods, template interpolation and `__proto__`/`constructor`/`prototype`
(`:1706-1766`, `:1768-1804`).

Determinism is enforced **twice**: a regex blocklist at parse time as a fast
author hint (`:452`), and a `DETERMINISM_PRELUDE` inside the realm as the real
enforcement (`:467-484`). `Math.random()`, `Date.now()` and no-arg `new Date()`
throw, with an error that names the reason *and* the fix: "it breaks resume;
pass randomness via args or vary by index".

The source is honest about its own limit (`:461-465`): *"vm is not a security
sandbox … The guard is best-effort against ACCIDENTAL nondeterminism from
trusted (user / guided-LLM) scripts, not a security wall."*

**Failure prevented**: a re-run producing a different call sequence than the
journal, which makes every cached result unreplayable.

#### T2. Capacity is reserved synchronously, before any `await`

`callIndex = state.callSeq++` is assigned at lexical call time, before the
limiter, so fan-out order is reproducible (`workflow.ts:795`). `shared.agentCount++`
is atomic with the limit and budget gate — the comment states why (`:819-824`):
*"no await in between — so a `parallel()` fan-out can't all observe the same
agentCount and overshoot maxAgents"*.

The quality helpers go further and **preflight their full known slot count
before starting any helper agent** (`:625-629`), so a 6-slot panel refuses to
start rather than starting 4 and stalling.

**Failure prevented**: check-then-act. Every concurrent caller reads the same
count and all of them pass the gate.

#### T3. Resume is the longest unchanged prefix, keyed by a call hash

One journal entry per call: `{index, runId, hash, result, storeDelta, model}`,
keyed `${runId}:${callIndex}` (`workflow.ts:65-98`, `:826-868`). The hash covers
prompt, model, tier, thinking, phase, agentType, `thread`, the **resolved agent
definition**, schema, cwd and isolation (`:1959-1987`). The post-resolution model
is stored but deliberately excluded from the hash (`:1011-1014`).

Replay rule (`:836-868`): skip a call **iff** no threaded call has run, its entry
exists, the hash matches, the cached result is not empty, and `callIndex <
firstMiss`. *"Once any call misses, it AND everything after it run live … so an
edited upstream call never leaves stale downstream results"* (`:826-830`).

Supporting mechanics worth noting: store writes replay **additively in callSeq
order** rather than restoring a whole map, named as the fix for a
last-complete-wins ordering bug (`shared-store.ts:9-13`); crash recovery flips an
on-disk `running` run to `paused` with its agents `skipped`, *"never 'failed' —
so its journal is preserved"* (`workflow-manager.ts:545-576`); a cross-process
lease is a `wx` lock file with pid and stale-pid detection
(`run-persistence.ts:535-567`); and resume restores the **frozen start-time
knobs**, not current defaults (`workflow-manager.ts:1749-1776`).

**Failure prevented**: re-paying for completed work after a crash, and stale
downstream results after an upstream edit.

#### T4. A human gate SUSPENDS the run; it does not wait inside it

`checkpoint()` persists a `waiting` record and then **throws** a suspension error
that unwinds the entire run (`workflow.ts:1437-1568`). The process holds nothing.
`throwIfAborted` re-throws the suspension on every later call (`:662`), and a
post-script check (`:1611`) stops a script-level `catch` from turning an accepted
pause into "completed".

The answer is attached later under the lease, idempotent on identical re-attach
(`workflow-manager.ts:1623-1626`). On resume the same call deep-compares
`{checkpointId, kind, payload}` and **replays the journaled answer — no re-ask**.
The checkpoint hash covers prompt text, kind, choices, default, headless mode and
timeout, with the rationale written down (`:1863-1868`): *"a silently-stale
cached decision … is worse than a one-time re-ask."* Durable payloads must be
lossless JSON (`:1895-1953`).

**Failure prevented**: a background run pinning a process on human input; an
answer applied to a question that has since changed.

#### T5. Partial failure and error class are declared, never guessed

`parallel`/`pipeline`: a **recoverable** failure nulls that slot and siblings
live on; a **non-recoverable** one rethrows (`workflow.ts:1114-1190`). The batch
cancel token is per-fan-out via `AsyncLocalStorage`, not run-global — the comment
records the bug that forced it (`:40-46`): a run-global flag *"wrongly cancelled
an innocent, independently-caught sibling batch"*. Run-fatal sealing fires only
when an error escapes the **top-level** script (`:1632-1673`), which is what
preserves both `parallel()`'s null contract and the script's own `try/catch`.

The taxonomy (`errors.ts:31-104`) splits recoverable (`AGENT_TIMEOUT`,
`AGENT_EMPTY_OUTPUT`, `AGENT_EXECUTION_ERROR`, `WORKFLOW_ABORTED`) from fatal
(`AGENT_LIMIT_EXCEEDED`, `TOKEN_BUDGET_EXHAUSTED`, `SCHEMA_NONCOMPLIANCE`,
`MODEL_NOT_FOUND`, `MODEL_SPAWN_REJECTED`, `PROVIDER_USAGE_LIMIT`, …). Two rules
carry their reason in the source:

- `PROVIDER_USAGE_LIMIT` is neither: *"a provider limit refills on its own, so the
  run is checkpointed (paused) and replayed by `resume()` rather than failed"*
  (`errors.ts:41-43`). Transient 5xx and "overloaded" are deliberately excluded
  so they keep retrying (`:156-165`).
- `MODEL_NOT_FOUND` is never silently substituted: *"resolution is deterministic,
  so retrying the same spec would fail identically every time"* (`:53-56`).

And the asymmetry that matters (`agent.ts:1015-1059`): an **explicit** model or
tier that does not resolve is fatal; an **implicit** default tier that does not
resolve degrades to the session default and logs once.

Two more checks in the same family: `emptyFleetSummary` flags a run where *every*
agent errored to null, and distinguishes it from a run that launched no agents
and from one whose agents were only skipped
(`workflow-manager.ts:1085-1104`; `tests/empty-fleet.test.ts`). And only assistant
text **after the last tool result** counts as an answer (`agent.ts:1346-1365`),
so stale progress chatter is never reported as final.

#### T6. Model-facing prose is split by evidence class, each class with a matched guard

This is the part with no bee equivalent, and it is the cleverest thing in the
repo. Every piece of text the model reads is classified by *what kind of evidence
can guard it*:

| Evidence class | Guard | Severity |
|---|---|---|
| Exact fact (name, signature, default, limit) | hand-written TS contract → generated doc table → byte-compared | error |
| Behaviour-covered prose | mapped to a replayable comprehension scenario; prose free to edit | error if the mapping breaks |
| Uncovered prose | `requiredText` sentence pin **plus** whole-file SHA-256 (13 files) | error |
| Non-contractual prose | SHA-256 baseline tripwire | **warning only** |
| Model behaviour | provider run → deterministic replay → hand-coded assertions | **not a gate** |

Anchors: contract at `workflow-capability-contract.ts:51-67`, which **throws** at
runtime when a declared global has no implementation (`:750-762`) and diagnoses
four alignment codes (`:695-746`); generated-doc byte compare at
`workflow-authoring-reference.ts:115-116`; the coverage split at
`workflow-authoring-coverage.ts:474-493`; frozen files at `:33-86` gated by
`workflow-release-gate.ts:294-320`; the release gate's nine validators at
`:488-526`.

Three details are the craft, not the plumbing:

1. **A named human acceptance verb.** `npm run guidance:accept -- <path>` demands
   at least one explicit path, refuses a path not in the frozen list, and rotates
   **the hash only** — never the `requiredText` or the anchor
   (`accept-workflow-guidance.ts:29-70`). No wildcard, no "accept all". An agent
   tidying a SKILL.md cannot ship without a human naming the file.
2. **Model runs are evidence, never CI.** `tests/workflow-comprehension.test.ts:33-38`
   asserts the comprehension script is **absent** from `test`, `release:check`
   and `prepublishOnly`; `tests/workflow-release-gate.test.ts:36-45` asserts the
   release chain does **not** match `/model|provider|comprehension/i`. The
   scenario oracles themselves are unit-tested model-free against hand-written
   good and bad workflows.
3. **The size check is freshness, not a ceiling.** `workflow-context-measurement.ts:259-262`
   commits the measured byte counts of five always-on surfaces to
   `docs/workflow-context-surfaces.json` and fails the release only when the
   committed file is **stale**. There is no numeric limit anywhere. The mechanism
   is that any growth forces the number to move **in the same diff**, in front of
   a reviewer.

The source's own weak spots, worth not copying: the measurement has no ceiling so
a human must notice; hash acceptance has no required link to a rationale line;
`requiredText` is a verbatim `includes()`, so a benign re-wrap is a release error.

#### T7. Two smaller pieces of craft

- **The trigger authorises, it does not force.** A bounded-word regex with
  Unicode `ID_Continue` lookarounds on both sides plus `/`, `$`, `-` and
  backslash (`workflow-editor.ts:26-34`) — which is exactly why `workflow_name`,
  `myworkflow` and `src/workflow-editor.ts` do not fire. On arm it *adds* the
  tool and removes nothing. The rationale is recorded (`:94-102`): the old
  forcing text (*"You MUST / the ONLY acceptable action / Do NOT answer
  directly"*) caused two real bugs — it over-triggered on messages that merely
  **mentioned** workflows, and it produced a bare background run that ended the
  turn and left the user at an idle prompt. How-to mechanics are deliberately
  kept **out** of the per-turn banner and live in the tool's static description.
- **Live state is a widget; exactly one line reaches the chat.** The progress
  panel is a TUI widget, *"purely informational … the panel takes no input"*
  (`task-panel.ts:1656-1657`). The only chat-bound text is the completion
  delivery: a header line with agents, tokens, cost and duration, then a summary
  that **prefers a `verdict|report|summary|synthesis` field**, else JSON capped
  at 400 characters, and **always** `↳ Full result: <runs dir>/<runId>.json`
  (`:105-122`). A manual pause delivers nothing.

### Local — what bee already has, and the three invariants that live in prose

`Local`

**Already equivalent, keep as is.**

- The dispatch door is a real resolver with typed refusals — `role_not_configured`,
  `advisor_not_configured`, `tier_not_configured`, `pi_requires_herding`,
  `cli_tier_gather_only`, `native_unavailable` (`prepare.rs:2035,2172,2199,153,2226,2662`).
  This is pi's `MODEL_NOT_FOUND` discipline, already in place.
- Every prepare appends a row to `.bee/logs/dispatch.jsonl` and carries a
  `brief_sha256` on the envelope (`prepare.rs:2909-2912`, `:2915-2944`). **bee
  already computes pi's call hash.**
- Generated guidance is byte-parity pinned: `skill_trees.rs:1088`
  `render_matches_the_committed_trees`, plus `agents_block_render_parity.rs`,
  `rule_index_parity.rs`, `principle_index_parity.rs`, `pointer_integrity.rs`,
  `class_playbook_parity.rs`, `verification_contract_parity.rs`,
  `hook_contracts.rs`. This is T6's top row, already built.
- Proof is recorded in a parsed shape (`finish_support.rs:110-124`), a `red`
  result refuses the cap (`:133-135`), and the proof command must byte-match the
  cell's approved `verify` (`handlers_close.rs:237-258`).

**The three invariants that live in prose, not in the store.**

1. **The four-slot cap** is enforced by skill text —
   `skills/bee-herding/references/role-dispatch.md:71,479` and
   `route-prompt.md:107-123` — reading `.bee/wave-ledger.jsonl` through
   `bee herding occupancy`. `fleet::choreography::run_wave` spawns one
   `std::thread` per worker with **no numeric cap in code**
   (`fleet/src/choreography.rs:254-257`; `fleet/src/wave.rs:3` says "at most a
   handful"). An orchestrator that reads occupancy and then dispatches is
   check-then-act — the exact gap T2 closes.
2. **"This worker is alive."** *Corrected after code recon on 2026-09-20 — see
   § R2 for the revised finding.* `.bee/state.json` holds **190 `workers[]` rows,
   188 at `"status":"running"`**, and no code path or skill ever retires one: the
   row is written at claim by `state worker add` and `state worker remove` appears
   only inside the unwind-failure message of a failed `dispatch prepare --claim`
   (`prepare.rs:3901`). But this array is **not** bee's liveness source, so the
   first draft of this bullet overstated the defect. Liveness already has a
   derived home: `active_workers()` joins live-heartbeat sessions with their
   active claims (`status_full/cells.rs:766-798`) and feeds `bee status`'s
   "Active workers" line (`status_full/build.rs:464`, `render.rs:437`); the write
   guard's concurrent-tree count comes from reservations, leases and heartbeats
   (`write_guard/paths.rs:447-520`). The array's only consumer is `capCell`'s
   "never zero execution workers" check, which matches on nickname plus cell and
   **ignores `status` entirely** (`handlers_close.rs:150-162`). `bee state worker
   prune` does not touch it either — that verb prunes `.bee/workers/` transient
   files (`state_group/workers.rs:352`, dry-run here reports 0).
3. **"This dispatch was already done."** There is no replay rule. Resume today is
   `bee orient` + `cells claim-next` + expired-claim re-pickup. An Agent-tool
   subagent's partial work is not journaled and does not survive session death.

**Also absent, searched and confirmed.**

- Semantic retry. `retry.fallbackChains` is only **published** on the payload
  (`prepare.rs:2768-2771`) — bee never executes dispatches by decision
  `51341f84` (`models.rs:1095-1104`) — and no chain is configured in this repo.
  Herding's retries are transport-level only: pane-start attempts
  (`run.rs:1374-1386`) and bounded pointer resend (`:1448-1528`).
- An outcome taxonomy for a worker that already ran. `[DONE] [BLOCKED]
  [HANDOFF] [NOOP]` is the worker's self-report; there is no class for "the
  provider said no", and so no pause-and-auto-resume path.
- A comprehension eval or a guidance baseline.
  `rg -i 'comprehension|guidance baseline|skill eval|token.?budget'` over tests,
  skills, scripts and knowledge returns only one prose hit
  (`skills/bee-hive/references/scout-and-ticks.md`). `brief_lint.rs:12-17` is a
  lexical "leaning language" guard and says of itself that it is not a quality
  certifier.
- A byte ceiling on instruction text — **and this one is absent on purpose.**
  `docs/history/budget-fence-removal/CONTEXT.md` D1–D7 deleted
  `scripts/skill_budget_fence.mjs`, and `tests/instruction_laws.rs:381-620` now
  **asserts no byte or line ceiling exists anywhere** and fails if one is
  reintroduced.

### Inference

- bee's fan-out is entirely the orchestrator LLM issuing parallel tool calls.
  There is no scheduler and no queue, which means every concurrency invariant bee
  has must be enforced **at the door** (`dispatch prepare`) or not at all — the
  orchestrator is the only other candidate, and it is a model.
- Because `brief_sha256` already exists on the envelope and `dispatch.jsonl` is
  already append-only, a replay rule is a *read* over records bee already writes.
  That is why § R1 sits on rung 1 rather than rung 3.
- The earlier brief's rule 4 ("split the machine-readable subject from the human
  presentation, cap it, digest-bind it so a changed question makes an old answer
  stale") and this source's checkpoint hash are the **same rule reached from two
  independent codebases**. Independent corroboration is the strongest evidence
  this brief carries, and it is the reason § R5 is ranked where it is.

---

## Recommendations, ranked

Each is one feature-sized slice. None needs code from the source.

### R1 — Give `dispatch prepare` a replay verdict
Read `.bee/logs/dispatch.jsonl` for a row matching this cell **and** this
`brief_sha256` whose cell is capped, and return `replay: cached | live` with the
prior outcome. Adopt pi's prefix rule: the first mismatch invalidates itself and
everything after it, so an edited upstream brief never leaves stale downstream
results. *Rung 1 — reuses `brief_sha256` and a log bee already writes.*

### R2 — Retire the dispatch row at cap, and flag an empty fleet
**Revised after code recon; the first draft of this item was wrong about the
harm.** Chesterton's fence and verify-before-reporting both fired here.

What the recon established (anchors in § Local, item 2): `state.json workers[]`
is a **dispatch-registration ledger**, not a liveness table. Its one consumer is
`capCell`'s "never zero execution workers" guard, which reads nickname and cell
and ignores `status`. Liveness is already derived correctly from heartbeats,
claims, reservations and leases, in two separate places. So bee **can** tell a
live worker from a dead one — my first framing said it could not, and that was
wrong.

What is actually left, and still worth building:

- **The ledger never shrinks.** 190 rows and growing, re-read and scanned on
  every cap. Nothing calls `state worker update --status` or `state worker
  remove` on a normal cap; `remove` is named only as a by-hand remedy in a
  failed-unwind message (`prepare.rs:3901`). Retire the row when its cell caps.
- **`status` is unset, not useless — keep it.** A second recon pass overturned
  the "drop the field" reading. `push_worker_record` upserts on
  `(nickname, cell)` and refreshes `status` in place, and two tests already pin
  `"capped"` as the post-cap value
  (`state_group/workers.rs:655-663`, `:682-700`). So the field has a designed
  shape and a test; what is missing is the caller. Chesterton's fence: do not
  remove it — set it.
- **The prune deadlock is the concrete harm.** `read_prune_keep_set`
  (`state_group/workers.rs:299-350`) reads `state.workers` and adds **every**
  row's `cell` to the protected keep set. Because rows are never retired, every
  cell ever dispatched is protected forever, so `bee state worker prune` can
  never clean `.bee/workers/` transient files for finished cells. Latent in this
  repo today — `.bee/workers/` is empty — but it is the reason the growth
  matters beyond file size.
- **Three other readers, all ignoring `status`**: the cap guard
  (`handlers_close.rs:150-162`), the SubagentStop nudge
  (`hooks/chain_nudge.rs:123-128`), and the keep set above. Adding or setting a
  status value breaks none of them.
- **The empty fleet is genuinely absent.** A wave where every worker returned
  nothing has no flag, and cannot be told apart from a wave that launched
  nothing or one whose workers were only skipped. `crates/fleet` already carries
  the structured material — `WaveResult` separates succeeded, timed-out,
  unverifiable-after-send and filter-dropped targets
  (`fleet/src/choreography.rs:30-57,278-280`) — so the herding path is cheap.
  The Agent-tool path has no such aggregation and would read cells instead.
- **Red before green.** Reproduce the growing ledger as a failing test first.

*Rung 1 — every verb this needs already exists; nothing calls them.*

### R3 — Move the four-slot cap from prose into the door
`dispatch prepare` claims the slot as part of preparing and refuses when full,
atomically with the occupancy read. Add pi's preflight: a hat wave needing six
seats refuses to start any of them unless six are free. *Rung 3 — adapt T2.*

### R4 — Declare a worker outcome taxonomy
Split recoverable (timeout, empty output, transport error) from fatal
(role unresolved, write-guard deny, schema refusal), and give a provider usage
limit its own third class that pauses with a reset hint instead of failing. Keep
pi's asymmetry, which bee half-holds already: an **explicit** role that does not
resolve is fatal (decision `4faf1de9`, advisor unset = off); an **implicit**
default that does not resolve degrades and logs once. *Rung 3.*

### R5 — Make the gate a record, not a wait
Today a gate question stops the session, which keeps a claim, a heartbeat and a
process. pi's checkpoint persists a `waiting` record, releases everything, and on
resume **replays** the journaled answer rather than re-asking — with the question
digest in the identity hash, so a changed question correctly loses its cached
answer. bee's `pause` handoff is the closest thing it owns, but it is a document,
not a replayable record with an identity. *Rung 3, and a doctrine change — the
user's call, not the agent's.*

### R6 — Add the two missing guidance tiers
bee already has T6's top row. Add the two rows under it:

- **Frozen sentences.** Pin the handful of sentences that teach a default or a
  limit, with a named human acceptance verb (`bee guidance accept <path>`) that
  rotates the hash only, never the pinned text, and refuses a wildcard.
- **A comprehension run, deliberately outside CI.** Give a real model only the
  shipped skill and check that it produces a correct plan or dispatch — scored by
  hand-coded assertions over a deterministic replay, recorded as an evidence
  artifact, and asserted by a test to be **absent** from `commands.test`.

**Chesterton's fence changed this recommendation.** I was about to propose a
token budget on skill text. `instruction_laws.rs:381-620` and
`docs/history/budget-fence-removal/CONTEXT.md` D1–D7 record that bee removed that
fence on purpose and now fails any attempt to reintroduce a ceiling. So the
proposal is pi's **actual** mechanism instead, which is not a ceiling: a
committed measurement file that must be regenerated in the same diff, so growth
is visible to a reviewer without any limit being enforced. That is compatible
with the locked decision. It is adjacent enough that the user should confirm it.

### R7 — Two cheap polish items
Cap the worker result delivered into chat the way pi caps it: header line, then a
preferred `verdict|report|summary` field, else a truncation, and **always** the
path to the full record. And keep how-to mechanics out of per-turn injected text,
in a static description — pi recorded two real bugs from forcing language, and
the narrow lesson transfers to bee's skill triggers.

---

## Risks, Unknowns, Follow-Ups

- **R5 is a doctrine change.** A suspending gate touches AGENTS.md's gate
  contract and the `waiting-on` mark. It needs the user, not a plan.
- **R6's comprehension run needs a model budget** and a stable scenario corpus.
  pi's own numbers (74/81 across 9 models × 3 reps) are recorded in its evidence
  file as a sample, *"not a stable benchmark"*. bee should copy that honesty.
- **R1 assumes `dispatch.jsonl` rows are complete enough** to key a replay. Not
  verified field by field — that is the first proof obligation at shape.
- **pi's `vm` is not a sandbox** and says so. bee's equivalent boundary is the
  write guard, which is stronger. Do not import pi's framing here.
- **Unchecked**: `crates/fleet` was read only at the wave and choreography level.
  A component absent from this sweep is unchecked, not clean.

---

## Source Pack

- **Upstream** (`e29dbcae`): `src/workflow.ts`, `workflow-manager.ts`,
  `workflow-tool.ts`, `workflow-control-tool.ts`, `agent.ts`, `model-routing.ts`,
  `model-spec.ts`, `model-tier-config.ts`, `pre-spawn-model.ts`,
  `structured-output.ts`, `shared-store.ts`, `run-persistence.ts`,
  `child-cache-retention.ts`, `usage-limit-scheduler.ts`, `workflow-paths.ts`,
  `errors.ts`, `enums.ts`, `workflow-capability-contract.ts`,
  `workflow-release-gate.ts`, `workflow-comprehension.ts`,
  `workflow-authoring-coverage.ts`, `workflow-authoring-reference.ts`,
  `workflow-context-measurement.ts`, `workflow-delivery-choice.ts`,
  `accept-workflow-guidance.ts`, `workflow-editor.ts`, `workflow-saved.ts`,
  `saved-commands.ts`, `command-registry.ts`, `workflow-settings.ts`,
  `workflow-ui.ts`, `task-panel.ts`, `display.ts`, `agent-registry.ts`,
  `agent-history.ts`, `agent-usage.ts`, `adversarial-review.ts`,
  `deep-research.ts`, `code-review.ts`, `extension-reload.ts`,
  `builtin-workflows.ts`, `builtin-commands.ts`, `pi-extension.ts`,
  `config.ts`; `docs/workflow-authoring.md`,
  `workflow-authoring-evidence.md`, `workflow-prompt-guidance-rationale.md`,
  `workflow-context-surfaces.json`, `workflow-guidance-baseline.json`;
  `skills/workflow-authoring/SKILL.md`, `skills/workflow-patterns/SKILL.md`;
  `scripts/` (7 files); `README.md`, `AGENTS.md`, `CONTRIBUTING.md`,
  `package.json`; 61 test files by name, 12 read.
- **Local**: `packages/bee-rs/Cargo.toml`, `crates/bee/Cargo.toml`,
  `crates/fleet/Cargo.toml`, `verbs/drivers/{prepare,models,prompt,close,brief_lint}.rs`,
  `herding/{run,wave,wave_ledger,control_loop}.rs`,
  `fleet/src/{lib,choreography,wave}.rs`, `state.rs`,
  `verbs/state_group/sessions.rs`, `verbs/workflow_store/mod.rs`,
  `verbs/cells/{proof,finish_support,handlers_close,handlers_write,claims}.rs`,
  `verbs/worktree/merge.rs`, `devtools/{mod,skill_trees,prompts}.rs`,
  `onboard/agents.rs`, `hooks/write_guard/{main,hook_local}.rs`,
  `tests/{instruction_laws,agents_block_render_parity}.rs`, `.bee/config.json`,
  `.bee/config-sample.json`, `.bee/state.json`,
  `.github/workflows/{ci,windows,pages,release-binaries}.yml`,
  `docs/decisions/0024-harness-cross-pollination-analysis.md`,
  `docs/history/budget-fence-removal/CONTEXT.md`,
  `docs/history/research/pi-workflows-xia.md`, `LLM.md`, `README.md`.
- **Docs**: none. This is a source distillation; no version-matched vendor docs
  were needed, and none were consulted.
