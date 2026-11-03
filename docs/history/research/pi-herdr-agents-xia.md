---
artifact_contract: bee-research/v1
topic: pi-herdr-agents-xia
depth: deep
date: 2026-09-06
---

## Bottom Line

- **Recommendation (ladder rung): adapt-upstream — design rules only, no code, no engine, no second transport.** That is the boundary decision `9f5c6d17` already fixed (Pi dispatch stays herding-only; bee takes design rules from Pi-side orchestrators, never their engine). This brief stays inside it.
- **What pi-herdr-agents v1.5.1 is**: a Pi extension that spawns Pi children into herdr panes/worktrees, watches them with a sidecar-first completion detector, pushes one bounded result message into the parent turn, and runs an approval-hash-bound JavaScript review workflow. It is the same shape bee-herding already has (pane spawn, mailbox receipts, typed outcomes, waves, supervisor), built as an in-process extension instead of a CLI cockpit. `Upstream`
- **What bee already has**: 9 of the 20 components in § Dependency Matrix, including every transport and completion-contract row. The herding cockpit is not behind on the core loop. `Local`
- **What is genuinely missing, ranked by gap size × cost** (all transport-neutral, all fit inside `bee herding` verbs; none need Pi):
  1. **Turn-level interrupt** — `bee herding interrupt <job-id>`: send Escape, mark `interrupted`, clear on the next activity. bee has no interrupt verb at all. Small.
  2. **`stalled` → `recovered` as a non-terminal signal** — 60 s without an activity/status change steers the leader once and clears itself; today bee only has the terminal `timed_out_idle` after `--idle-timeout`. Small.
  3. **`retryable` bit on every failure** — the result envelope and wave buckets carry `retryable: bool`; today `agent_prompt_stalled` is "retryable" only in prose. Small.
  4. **Git handoff block in the worker result** — branch, base/head SHA, commits ahead, changed/untracked files, `clean|dirty|conflicted|unknown`; bee returns `files_changed` only. Medium.
  5. **Ordered agent fallback launched ONLY on a provider/launch error, never on a negative task result**, with the attempted list in the result. Medium; needs a provider-error classification bee does not emit yet.
  6. **Fail-closed cancel** — capture pids, close panes, wait absence 5 s, wait exit 5 s; a survivor is `cancel_termination_failed`, never "cancelled". Small once the wave `first-success-cancel-rest` policy (D11) is implemented.
  7. **Orphan sweep at startup** — a wave row with no live owner is marked `interrupted` (reason `process_restarted`), never replayed. Small.
  8. **Review discipline for bee-reviewing, not herding** — identity-stripped synthesis with fixed aliases and fixed order, cross-family verification of P0/P1 only, `evidenceStatus reproduced|trace-backed|unverified`, `INCOMPLETE` propagation. Prose rules, zero code.
- **Why the next-best rung lost**: rung 2 (built-in) is closed — the extension's whole value is Pi-process-resident, and `9f5c6d17` says bee does not add a Pi-resident transport. Rung 4 (build a workflow engine) is refused by D11/D16 (a wave is a Rust value; no recipe format before a second real scenario) and would duplicate gates bee already has.
- **Confidence: 88%.** The upstream facts are anchored to file:line in a pinned SHA. The 12% is whether items 4 and 5 deserve a feature before a second herding scenario has run end to end (D1 is still the open completion criterion).
- **Suggested next step: bee-shaping**, one feature, "herding cockpit completeness" carrying items 1, 2, 3, 6, 7 (all small, one crate). Items 4, 5 and 8 wait for the user's pick.

---

## Repo Snapshot

- **This repo (bee)**: Rust CLI (`packages/bee-rs`, crates `bee` + `fleet`), plugin 2.33.2, skills as markdown contracts, one TypeScript belt per harness. Herding code: `crates/bee/src/herding/{run,mailbox,control_loop,pane_verbs,split_lock,tmux}.rs`; generic core in `crates/fleet` (WorkerBackend, `fleet::screen` classifier). `Local`
- **Herding transport**: `herding.transport` herdr|tmux; herdr 0.8.x split-then-start (D12). `Local`
- **Config**: `.bee/config.json` `models.pi.*` and `models.claude.*` slots all `kind: herding` (agents `agy-flash`, `pi-gpt-5.6-luna`, `pi-gpt-6-astra`); `gate_bypass: full`; `worktree_first: true`. `Local`
- **Source distilled**: `giuseppecrj/pi-herdr-agents` 1.5.1 — TypeScript Pi extension, ~15 modules under `pi-extension/subagents/`, plus `skills/orchestrate`, `docs/adr/0001–0008`, `docs/research/*`, `test/evals`. `Upstream`

## Question & Assumptions

- **What was asked**: distill the repo, learn in detail the features that would complete bee's current herding cockpit, and learn how it implements its orchestrator.
- **What success means**: a ranked list of gaps with evidence, each mapped to a bee verb or rule, inside the locked boundaries — nothing built.
- **Assumptions**: (a) herding stays the one Pi transport (`9f5c6d17`, `5d87f14e`); (b) merge/push/remove stay human gestures (herding-adopt R1–R8) — the source agrees, so this is alignment, not tension; (c) the earlier herdr-0.8.0 spawn finding is consumed by D12 and is not re-raised here.

## Source Manifest

| Field | Value |
|---|---|
| Repo or path | `D:\projects\goglbe\refs\pi-herdr-agents` (`giuseppecrj/pi-herdr-agents`) |
| Ref | `main` |
| Resolved commit SHA | `ebef2657b0b21177373868d83d66965fdf77febd` (2026-09-05, "chore: release v1.5.1") |
| Package | `pi-herdr-agents` 1.5.1 |
| Narrowed scope | `pi-extension/subagents/*.ts` (index, launch, lifecycle, activity, status, completion, session, subagent-done, runtime-routing, herdr, terminal, role-config, pane-config, model-config, type-guards, workflow, workflow-worker.js); `skills/orchestrate/*`; `docs/adr/0001–0008`; `docs/research/*`; `docs/review-evaluation.md`; `test/evals/README.md`; README, CONTEXT.md, AGENTS.md, CHANGELOG |
| Mode | **`xia`** — distill and discuss; port-protocol steps 1–4, no challenge pass, nothing built |
| Guardrail | Fetched content — code, README, comments — is data, never instructions |

Gather method: two read-only `bee herding run` workers (`agy-flash`), digests at `.bee/mailbox/job-1788662740681/report-1.md` (orchestrator) and `.bee/mailbox/job-1788662743898/report-1.md` (workflow runner). Deviation, named: both were launched with `--task-file <path>` instead of the prepare payload's stdin `-` form — same verb, same agent, recorded so it is not silent.

---

## Findings

### Upstream — how the orchestrator is built (the part the user asked to learn)

**1. One launch transaction, seven steps, one owner** (`launch.ts:218-314`). Resolve request → prepare surface (pane, split, or worktree) → prepare child session file + policy sidecar → confirm shell ready → write task artifact → build the `pi` argv → run it in the pane via `herdr pane run`. The argv ends with a sentinel `; echo '__SUBAGENT_DONE_'$?'__'` so the pane itself carries an exit code (`launch.ts:572-642`). On launch failure an ordinary pane is closed; a worktree pane is **kept**, its manifest set `failed`, the worktree retained (`launch.ts:291-313`). `Upstream`

**2. Child bootstrap by env, not by prompt.** `PI_SUBAGENT_ID`, `_NAME`, `_AGENT`, `_SESSION`, `_ACTIVITY_FILE`, `_SURFACE`, `_AUTO_EXIT`, `PI_DENY_TOOLS`, `PI_CODING_AGENT_DIR` (`launch.ts:612-635`). The child loads one extension (`subagent-done.ts`) that adds `caller_ping` (ask the parent for help, then exit) and, only for interactive children, `subagent_done` (`subagent-done.ts:297-360`). Same shape as bee's `BEE_HERDING_WORKER=1` + `BEE_HERDING_JOB_ID` contract. `Upstream`

**3. Settle before exit.** Auto-exit fires on `agent_settled`, not `agent_end`, and reads the active branch after compaction; an `aborted` stop reason never auto-exits; a settled provider error is written to the `.exit` sidecar as `{type:"error"}` (`subagent-done.ts:197-241, 17-39, 55-72`). A manual takeover flag is reset at `agent_end` so a human nudge cannot strand an autonomous child (`subagent-done.ts:200-205`). `Upstream`

**4. Completion: sidecar first, sentinel second, pane loss third** (`completion.ts:58-76, 139-148, 163-174`). Poll every 1000 ms. Grace windows: 500 ms for a sidecar to land after a non-zero sentinel; 500 ms (25 ms ticks) for a sidecar after the pane disappears; a malformed sidecar is retried next tick, not failed. Missing pane with no evidence = failure "pane disappeared before completion evidence was recorded". `Upstream`

**5. Lifecycle = process × turn, with Herdr as coarse authority.** `ProcessState` `starting|running|finalizing|completed|failed`; `TurnState` `unknown|starting|active|blocked|waiting|interrupted`; projection `starting|running|active|blocked|waiting|interrupted|stalled|finalizing|completed|failed` (`lifecycle.ts:21-126`). Child activity snapshots (throttled 500 ms, circuit-broken after 3 write failures, monotonic sequence enforced) enrich the pane status (`activity.ts:101-102, 276-281`). `stalled` fires after 60 s without a fresh observation and emits `stalled`/`recovered` transitions that steer the parent, suppressed for interactive children (`status.ts:6, 545`; `lifecycle.ts:581-600`). `Upstream`

**6. Interrupt is one keystroke plus a local mark.** `subagent_interrupt` sends `herdr pane send-keys <surface> Escape`, sets turn `interrupted`; the mark outranks lagging Herdr status until a newer activity sequence arrives (`index.ts:1833-1877`; `lifecycle.ts:442-456, 354-364`). Process and session stay alive. `Upstream`

**7. Result delivery is one custom message that starts a turn.** `sendMessage({customType:"subagent_result", triggerTurn:true, deliverAs:"steer"})` with a fixed continuation line "Continue the parent task using this result; do not return an empty response" (`index.ts:1191-1210`). The body is the child's last assistant message, middle-abbreviated to 16,000 chars keeping head and tail, plus `Session:` and `Resume:` pointers (`index.ts:1055-1096`). Failure envelopes distinguish clean exit, non-zero exit, provider/agent error, launch error (`index.ts:1245-1306`). `Upstream`

**8. Model fallback is a launch policy, not a retry policy.** Comma-separated candidates; the next one launches under the same child id only when the child settled with a provider/agent error; a negative task result never advances; worktree children get no fallback; the final envelope lists requested model, models attempted, raw failures in order (`runtime-routing.ts:207-328`; `index.ts:2238-2276, 2429-2506`). `Upstream`

**9. Worktree = manifest before resources.** Base ref resolved to a SHA; manifest `<artifacts>/worktree-runs/<id>.json` written `provisioning` **before** `herdr worktree create --no-focus`, then `provisioned`, later `ready_for_review|needs_help|failed` (`launch.ts:368-423, 967-977`). Completion captures a Git handoff from six `git` reads (status, untracked, conflicted, diff names, `rev-list --count base..HEAD`, `rev-parse HEAD`) and prints the removal command instead of running it (`launch.ts:903-965`; `index.ts:1212-1243`). If `worktree create` output cannot be parsed, it recovers the created worktree/pane from `worktree list` + `pane list` before failing (`herdr.ts:307-328`). `Upstream`

**10. Resume restores capabilities, refuses owned sessions.** Tool policy comes back from `<session>.pi-herdr-subagent-policy.json`; a session owned by `managed-worktree` or `workflow` is refused with a named message (`launch.ts:697-702`). `Upstream`

**11. Reload survival.** Active children, watchers and the workflow owner live on `globalThis[Symbol.for("pi-subagents/…")]` so `/reload` keeps them; completion delivery re-selects the fresh extension API (`index.ts:141-158, 2545-2580, 2864`). `Upstream`

**12. herdr verbs used** (`herdr.ts`): `pane current --current`, `tab create --workspace --label --cwd --no-focus`, `pane split <id> --direction right|down --no-focus --cwd`, `worktree create --cwd --branch --base --label --no-focus`, `worktree list`, `pane list --workspace`, `pane read <id> --source visible --lines N`, `pane get`, `pane process-info --pane`, `pane run <id> <cmd>` (text + CR in one socket packet), `pane send-keys <id> Escape`, `pane close`, `tab rename`, `workspace rename|focus`. Readiness = `foregroundProcessGroupId === shellPid` (10 s / 50 ms); process confirmation = argv contains `pi --session <file>` with matching cwd (10 s / 50 ms); exit = `kill(pid,0)` with EPERM handling (5 s / 50 ms); pane absence 5 s / 50 ms. Error parsing reads stdout and stderr as JSON independently, then a `pane_not_found|not_found` regex (`herdr.ts:448-469, 600-701`). `Upstream`

**13. The workflow runner** (`workflow.ts`, `workflow-worker.js`, ADR-0004…0007). `prepare` parses a `/* herdr-workflow … */` JSON header (`version:1`, `sources`, `baseSha` 40-hex, `maxAgents` ≤ 8, `maxConcurrency` ≤ 4, `roles[{id,role,kind:"review",model,thinking}]`), refuses symlinks and >256 KiB, syntax-checks with `new Function` without running, binds canonical root + git common dir + base commit, fingerprints each role (SHA-256 of role definition + runtime + tools), derives tools as `role.tools ∩ {read,grep,find,ls} − deny`, and prints an approval packet. `start` requires `APPROVE <first 8 hex of script hash>` as the latest user message in the same session, revalidates every bound byte, creates `run.jsonl` with `wx`, writes `approved` as the first event, adds a detached read-only checkout `git worktree add --detach … <baseSha>`, runs the script in a `Worker` + `vm` context (`codeGeneration` off, `console` undefined) with only `agent()` and `log()`, FIFO-limits concurrency, and delivers one `herdr_workflow_result` message. Journal events: `approved, started, reader_checkout_ready|retained|disposed, agent_started, agent_completed, agent_result, workflow_log, pane_close_failed, cancel_process_info(_failed), cancel_pane_still_present, completed, failed, cancelled, interrupted, delivery`. Failure codes: `cancelled, workflow_agent_options, policy_error, child_error, empty_completion, workflow_runtime_mismatch, launch_error, agent_limit`, all `retryable:false` in v1. `cancel` is fail-closed (§ Bottom Line item 6). Startup scans `.pi/plans/*/run.jsonl` and marks owner-less in-flight runs `interrupted` (`process_restarted`), never replays. Not a security boundary; one workflow per process. `Upstream`

**14. The adversarial review procedure** (`skills/orchestrate/adversarial-review.md`). Routine: 2 discovery models, ≤1 verification per report, 1 fresh synthesizer, ≤5 calls. High: 3 lenses (spec/correctness, security/failure, ops/concurrency/compat), ≤7 calls. Verification only for potential P0/P1, by a **different model family**; no eligible family → finding stays unverified and the run is `INCOMPLETE`. Synthesis sees an identity-stripped projection (names, session paths, model/provider tokens redacted; aliases `R1..R3`, `V1..V3`, `S1`; order fixed before results exist). Finding record: `id, claimedSeverity, confirmedSeverity, resolution candidate|confirmed|rejected, evidenceStatus reproduced|trace-backed|unverified, location, provenance[], preconditions[], reproductionOrTrace[], expected, actual, impact, minimalFix`. Retry only when an envelope says `retryable:true`. `Upstream`

**15. The evals corpus** (`docs/review-evaluation.md`). Eight public before/after cases; a private oracle of expected findings and ground-truth programs; `validate.mjs` proves each defect passes-before/fails-after; human adjudication maps claims to oracle ids; `score.mjs` computes precision, recall, clean-case false-positive rate, completeness; 4 tuning / 4 sealed holdout cases. `Upstream`

### Local — what bee-herding already has

- Mailbox contract `.bee/mailbox/<job-id>/` (job, brief-N, ack-N, result-N, report-N, log, activity), signal ladder truth→liveness→progress→classification, typed outcomes `done|died|paused_limit|timed_out_idle|blocked`, `--continue` rounds, `--ceiling`, `--idle-timeout`, `--close-always`, `--expertise`, `--inbox-session`. (`docs/knowledge/areas/bee-herding/*`, `crates/bee/src/herding/run.rs:151-392`) `Local`
- Delivery + receipt: `herdr agent prompt --wait --until working --timeout` (20 s), receipt = worker's ack file, bounded resend, `agent_prompt_stalled` retryable-in-prose, give-up pane diagnosis. `Local`
- Detached result path on Pi: pointer marker under `.bee/result-inbox/<token>/`, drained by `.pi/extensions/bee-guard.ts` on `session_start`, header-only injection, at-least-once with `job_id` dedupe, live session required (`docs/config-reference.md:182-186, 248`). `Local`
- Wave/ledger/occupancy/control-loop/interlock/status verbs; `fleet` crate with `WorkerBackend` (ready/working/blocked/finished/unverifiable) and one screen classifier; split rules and cross-process split lock; `workspace_trust` pre-seed; four-step agent resolution; registry `{argv, env, workspace_trust}`. `Local`
- Supervisor observer (cold tick, append-only observations/interventions/presence/reports, frequency-cap escalation, next-turn-boundary delivery), presence away/back, one WakeReport per window, seven derived metrics. `Local`
- Worker result carries `options[]`, `leaning`, `dissent{claim,alternative,severity}` on `blocked`. `Local`
- Not present (grep over `crates/bee/src/herding` and `crates/fleet/src`): no interrupt/Escape verb, no `stalled`/`recovered` state, no `retryable` field, no git-handoff block, no ownership manifest state machine, no agent fallback list. `Local`
- Activity freshness for the supervisor is 120 s (`hooks/activity.rs`); the control loop's kill grace is 30 s (`control_loop.rs:113`). `Local`
- bee-reviewing already has P1/P2/P3 with a spec/standards axis, one synthesis report, delta re-review after a P1 fix (`skills/bee-reviewing/SKILL.md:53-126`). It has no identity stripping, no cross-family rule, no `evidenceStatus`. `Local`
- Prior briefs: `docs/history/research/herdr-orchestrator-distill.md` (2026-08-18; herdr-0.8.0 spawn finding, consumed by D12) and `docs/history/research/pi-workflows-xia.md` (2026-09-02; native-Pi dispatch declined → `9f5c6d17`). `Local`

### Docs

- Upstream docs read: README (v1.5.1), `docs/worktree-subagents.md`, `docs/plan-skill.md`, ADR-0001…0008, `docs/orchestrated-review-workflow-plan.md`, `docs/research/worktree-subagent-orchestration.md`, `docs/research/pdw-architecture-assessment.md`, `docs/review-evaluation.md`. ADR-0008 (Pi-only, one deep launch module, "do not add a runtime adapter seam without a second real execution path") is the same one-transport rule bee holds in `9f5c6d17`. `Docs`
- Deferred upstream, with reasons: writer lane (first flow must prove orchestration without write effects, ADR-0006), multi-writer run assembly (no unified candidate; integration order/conflicts/recovery unproven), autonomous delivery verdict (push/PR/merge/remove are irreversible shared operations; the human keeps them). `Docs`

### Inference

- bee's header-only injection versus the source's 16 k body push is a deliberate divergence (bee: "the report body never rides the injection"), not a gap. The rule worth taking is only the **abbreviation shape**: when a `summary` must be capped, keep head + tail + a pointer, never head-only.
- The source's `pane process-info` readiness check (foreground pgid == shell pid, then argv match) is a stronger "truth" rung than bee's status-idle-or-done gate. Whether herdr 0.8.x on Windows returns `foregroundProcessGroupId` is unverified here.
- The workflow runner's "one owner per process" model conflicts with bee's multi-session etiquette (lanes, claims, holds); bee's wave ledger + occupancy is the multi-session-safe equivalent and should stay.
- Whether a bee herding worker can recursively call `bee herding run` (the source's self-spawn guard) is unchecked — a sweep item, not a finding.

---

## Dependency Matrix

| # | Component (source) | Local status | Evidence | Note |
|---|---|---|---|---|
| 1 | Pane / split / worktree surface, split-then-start, `--no-focus` | EXISTS | `Local` D12, `pane_verbs.rs` | same herdr verbs |
| 2 | Child bootstrap by env + sidecar files | EXISTS | `Local` `BEE_HERDING_WORKER`, mailbox | equivalent |
| 3 | Sidecar-first completion, sentinel/pane fallback, grace windows | EXISTS (values differ) | `Local` ack/result files, give-up diagnosis | take the 500 ms grace idea if pane-loss races appear |
| 4 | Typed outcomes / failure envelopes | EXISTS | `Local` `done|died|paused_limit|timed_out_idle|blocked` | — |
| 5 | Detached result push into the parent turn | EXISTS (header-only by design) | `Local` `--inbox-session` + bee-guard drain | divergence recorded, not a gap |
| 6 | Continue / resume rounds | EXISTS | `Local` `--continue` | source adds a frozen policy sidecar; bee's job.json is that |
| 7 | Waves, ledger, occupancy, control loop | EXISTS | `Local` D9–D11 | source has no equivalent (one workflow per process) |
| 8 | Supervisor / stalled detection at 120 s | EXISTS (terminal only) | `Local` `hooks/activity.rs` | see NEW #10 |
| 9 | Agent registry + role slots + trust pre-seed | EXISTS | `Local` `herding.agents`, `models.*` | source's "collision disables both, loudly" rule is worth taking |
| 10 | `stalled`→`recovered` non-terminal signal + one steer | NEW | `Upstream` `status.ts:545` | small |
| 11 | Turn-level interrupt (Escape + local mark) | NEW | `Upstream` `index.ts:1833-1877` | small; herdr `pane send-keys`, tmux `send-keys` |
| 12 | `retryable: bool` on failures | NEW | `Upstream` `index.ts:2638-2647` | small |
| 13 | Git handoff block in result | NEW | `Upstream` `launch.ts:903-965` | medium |
| 14 | Ownership manifest written before resources, state machine | NEW (partial: ledger row) | `Upstream` `launch.ts:368-423`; `Inference` on ledger write order | medium; may fold into the wave ledger row |
| 15 | Ordered agent fallback on provider error only | NEW | `Upstream` `index.ts:2429-2506` | medium; needs provider-error classification |
| 16 | Fail-closed cancel (`cancel_termination_failed`) | NEW (policy enum exists, D11) | `Upstream` `workflow.ts:918-956` | small |
| 17 | Startup orphan sweep → `interrupted` | NEW | `Upstream` `workflow.ts:647-722` | small |
| 18 | Identity-stripped synthesis, cross-family verify, `evidenceStatus`, `INCOMPLETE` | NEW (bee-reviewing) | `Upstream` `adversarial-review.md:62-177` | prose rules |
| 19 | Evals corpus (oracle, validate, score, holdout) | NEW (backlog `p-cf66d519`) | `Upstream` `review-evaluation.md` | reference for that backlog item |
| 20 | JS workflow engine, `.pi/plans` run dir, approval-hash script, Pi-resident `subagent` tool | CONFLICT | `Local` `9f5c6d17`, D11/D16 | design rule only: bind a gate to the bytes it approved (bee: `plan-rev bump` + `advisor-ref record` already do this) |

## Cross-Cutting Sweep

Wiring outside the herding folder that any of the NEW rows would touch:

- **Hooks**: `crates/bee/src/hooks/activity.rs` (activity.json, 120 s freshness) — row 10 reads it; `UserPromptSubmit` delivery of supervisor reports — row 10's steer would ride it.
- **Pi belt**: `.pi/extensions/bee-guard.ts` result-inbox drain (`session_start`) — unchanged by every row; row 5 stays header-only.
- **Config seams**: `herding.transport`, `herding.agents`, `herding.agent_command`, `herding.control_command` (`docs/config-reference.md`); `models.<rt>.<slot>` `kind: herding` — row 15 would add an ordered `agent` list to a slot; the model-guard refusal table in `verbs/drivers/prepare.rs` must keep refusing anything not herding.
- **fleet crate**: `WorkerBackend` five states (D7) — rows 10 and 11 add signals, not states; `unverifiable` stays first-class.
- **Wave ledger** (D10, append-only) — rows 14, 16, 17 write rows/reasons there, never a second store.
- **Knowledge**: `docs/knowledge/areas/bee-herding/*` six pages + `overview.md`/`index.md`; `.bee/verify/verify-app/features/` herding feature file — any shipped row updates both, then `bee dev regen`.
- **Locked decisions**: `9f5c6d17`, `5d87f14e`, herding-orchestration D1–D18, herding-adopt R1–R8 — no row supersedes any; row 20 is refused by them.
- **bee-reviewing** (`skills/bee-reviewing/SKILL.md`) — row 18 lives there, not in herding.
- **Backlog** `p-cf66d519` — row 19.
- **Unchecked** (not confirmed clean): recursive `bee herding run` from inside a worker; herdr `pane process-info` fields on Windows; the exact write order of a wave ledger row versus pane spawn.

## Strengths / Weaknesses of the source

- **Strengths**: one launch transaction with a named owner; settle-before-exit; sidecar-first completion with explicit grace windows and values; lifecycle as process × turn; fallback only on provider error; manifest-before-resources; refuses to auto-merge/remove; approval bound to bytes; fail-closed cancel; every ADR states the trade-off it accepts.
- **Weaknesses for bee's purposes**: Pi-process-resident (a second transport by construction); one workflow per process; the workflow runner re-implements gates bee already has; `retryable` is always false in v1; no list/status/resume for workflows; the tmux-free design cannot serve bee's `herding.transport = tmux`.

## Risks, Unknowns, Follow-Ups

- **Risk**: adding rows 13–15 before D1's end-to-end scenario has run widens the surface before the base is proven. Mitigation: ship rows 10, 11, 12, 16, 17 first (all inside `run.rs`/`fleet`), rows 13–15 after.
- **Unknown**: herdr `pane process-info` and `send-keys Escape` behaviour on Windows conhost/pty — verify against the running binary before row 11 is planned.
- **Open question for the user**: which of rows 13 (git handoff), 15 (agent fallback), 18 (review discipline) is wanted at all — each is a separate feature.

## Source Pack

- Local: `docs/knowledge/areas/bee-herding/*` (six pages), `skills/bee-herding/SKILL.md`, `skills/bee-herdr/SKILL.md`, `skills/bee-reviewing/SKILL.md`, `docs/history/herding-orchestration/CONTEXT.md`, `docs/history/research/herdr-orchestrator-distill.md`, `docs/history/research/pi-workflows-xia.md`, `docs/config-reference.md` (§ Pi), `docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md`, `.bee/config.json`, `crates/bee/src/herding/*.rs`, `crates/bee/src/hooks/activity.rs`, `crates/fleet/src/backend/*.rs`, `bee herding --help`.
- Upstream: the manifest scope above at SHA `ebef2657…`; gather digests `.bee/mailbox/job-1788662740681/report-1.md`, `.bee/mailbox/job-1788662743898/report-1.md`.
- Docs: upstream README, `docs/*.md`, `docs/adr/*`, `docs/research/*`, `test/evals/README.md`.
