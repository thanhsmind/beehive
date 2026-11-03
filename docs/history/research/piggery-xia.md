---
artifact_contract: bee-research/v1
topic: piggery-xia
depth: standard
date: 2026-10-16
mode: xia
---

# Xia: what piggery v0.10.2 could still improve in bee (Claude Code and Paseo only)

## Bottom Line

- Recommendation (ladder rung): **reuse**. bee already has an equivalent for
  most of what piggery does, and a large part of it came from piggery itself:
  the feature `piggery-borrow` (2026-10-07, piggery v0.6.0) shipped the
  turn-scoped ack, the process-tree kill, and the code-enforced concurrency
  and depth limits. Do not run piggery as a runtime, a mailbox, or a third
  daemon.
- Two small gaps are real. They are filed as two separate proposed PBIs:
  1. The worker cap has two homes that disagree: prose says "~3-4 live" and
     "four slots", but `herding.limits.concurrency` is absent in this repo's
     own config, so the code cap enforces nothing here.
  2. The review severity rule counts corroboration "across different models",
     but bee-reviewing dispatches every lens with no `--role`, so every lens
     runs on the one review model. piggery's `dual-lens` puts two lenses on
     two model families.
- Why the next-best rung lost: "adapt-upstream" (bring in piggery's daemon
  for workers or mail) would add a second mail system beside `.bee/mailbox`,
  a third daemon beside Paseo and waggledance, and a runtime whose Paseo
  support piggery itself removed.
- Confidence: 85%. The open point is item 2: whether a cross-model lens finds
  more real defects than a second same-model lens. That needs a measured run,
  not this brief.
- Suggested next step: **bee-shaping** triage of the two PBIs; none of this
  needs a feature now.

## Source manifest

| Field | Value |
|---|---|
| Repo | `/mnt/data/Projects/refs/slp/piggery` (read only), upstream https://github.com/sting8k/piggery |
| Ref | v0.10.2 as asked (`CHANGELOG.md:13`, 2026-10-12). The local CHANGELOG already opens with v0.10.3 (`CHANGELOG.md:3`, 2026-10-15); no git tag was checked. |
| Read | `README.md`, `docs/guide.md`, `docs/reference.md`, `manifests/README.md`, `AGENTS.md` (Rules that keep the design intact), `internal/core/{deliver,mail,limits,cwd,wake,notify,taskforce,roles,agent,engine,api}.go`, `internal/driver/local/{runner,claude,proctree}.go`, `internal/store/schema.sql`, `manifests/{dual-lens,council}.yaml` |
| Narrowed scope | Claude Code and Paseo as bee runs them today. piggery's other harnesses (pi, Codex, omp, dsh, opencode) are out of scope (spec drop `sup-20261016-a506`; `docs/history/piggery-eval/CONTEXT.md` D1). |
| Prior art | `docs/history/piggery-borrow/CONTEXT.md` (seven items from piggery v0.6.0, all capped) |

Two meanings of "gate" stay apart in this brief. A **bee gate** is a human
approval recorded in `.bee/state.json`. A **piggery gate** is the routing
door of a team and the one session that speaks for that team. They are not
the same concept and no verdict below treats them as one.

## Verdict table

| # | Hypothesis | Verdict | piggery evidence | bee evidence |
|---|---|---|---|---|
| 1 | piggery as the runtime for bee-swarming workers: cell = task mail, handback = mail to the lead, a crashed worker returns the cell | **Wrong premise, and bee has the equivalent.** bee does not depend only on the Agent tool: one door picks a native subagent or a herding worker from config, and pi already runs herding-only. bee manages the herding process itself (poll, typed outcomes, cancel). A worker that dies mid-cell does not count as done: the claim expires and the cell is parked `blocked`, on purpose not reopened. | Claude Code driven as `claude -p --input-format stream-json --output-format stream-json` (`internal/driver/local/claude.go:162-176`); no first-party Paseo support, `piggery setup paseo` removed (`CHANGELOG.md:113-116`) | Dispatch door `slot_for_kind` (`packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:175-184`); pi herding-only refusal `pi_requires_herding` (`prepare.rs:206`); result is `result-N.json` in the job mailbox (`packages/bee-rs/crates/bee/src/herding/mailbox.rs:20-24`); typed outcomes `Died`, `TimedOutIdle`, `Interrupted` (`packages/bee-rs/crates/bee/src/herding/run.rs:2043-2075`); claim TTL 3600 s and heartbeat stale 900 s (`packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:34-36`); sweep parks `claimed -> blocked` "rather than reopened, which would invite the next agent to redo half-finished work blind" (`packages/bee-rs/crates/bee/src/verbs/cells/handlers_select.rs:166-169`) |
| 2 | piggery mail is more durable: ack per turn, hold over a cap, no ack on idle | **bee has the equivalent where it needs one; the rest has no use in bee.** The turn-scoped ack and hold already shipped for the pi leader (piggery-borrow D1). The job mailbox has round-numbered receipts and results. Supervisor rows are delivered at the next turn boundary and marked delivered, never deleted. "Hold over a cap" guards peer-to-peer mail, and bee has no peer mail: workers talk only to the leader, and the broker re-dispatches a question as a new round. | ack only on a normal end of that batch's turn (`internal/core/deliver.go:1-6`, `internal/core/mail.go:284-320`); `policy_hold` over a rate cap (`internal/core/limits.go:14-77`); mail held during a permission prompt (`internal/core/deliver.go:34`) | `ack-N.json` / `result-N.json` per round (`packages/bee-rs/crates/bee/src/herding/mailbox.rs:13-24`); pi result-inbox requeue and hold on `aborted`/`error`, drain skipped while a prompt is open (`docs/knowledge/work/pi-result-mailbox/delivery.md`, piggery-borrow D1); broker claims one question round by atomic rename and re-dispatches it (`docs/knowledge/areas/bee-herding/the-mailbox-broker.md`); `bee supervisor pending` + `mark-delivered` (`packages/bee-rs/crates/bee/src/verbs/supervisor.rs`); human-mailbox letters with a read flag (`docs/knowledge/areas/human-mailbox/overview.md`) |
| 3 | Limits in code: parallel workers, depth, rate | **Mostly done in bee; one real gap.** Concurrency and depth are enforced in code at both doors since piggery-borrow D3. Rate limits are not worth it (no peer mail to rate). The gap: the operative cap in this repo is still prose, because `herding.limits` is absent from `.bee/config.json`, and the prose number lives in several files. → **PBI 1**. | `limits.concurrency`, `limits.depth`, `limits.messages_per_participant_per_minute`, `max_respawn_per_hour` (`internal/core/agent.go:675-716`, `internal/core/limits.go:26-38`); refusal `core.Error{code, rule_id, layer}` (`internal/core/api.go:400-416`) | `limits_refusal` with `limits.depth` / `limits.unverifiable` / `limits.concurrency` (`packages/bee-rs/crates/bee/src/herding/wave.rs:1114-1149`); checked in `bee dispatch prepare` (`prepare.rs:2376`); absent ⇒ no limit (`docs/config-reference.md:391`); `.bee/config.json` `herding` has no `limits` key (`python3 -c ... print(cfg['herding'])`, 2026-10-16); prose cap "~3-4 live" (`skills/bee-swarming/references/swarming-reference.md:33`), "four slots" (`skills/bee-herding/README.md:90`) |
| 4 | Worktree per cell (`can_set_cwd` + `spawn.allowed_roots`) fits bee's worktree model and `paseo-worktree-workspace` | **Wrong premise: piggery never makes a worktree.** It only allows a worker's cwd inside the team root, inside an existing git worktree of the repo, or inside an allowed root. bee already makes one worktree per feature and, inside Paseo, a Paseo workspace for it. The one hazard of many workers in one worktree (a shared git index) already has a code guard with a temp-index remedy. No overlap worth acting on. | `worktrees()` only lists `git worktree list --porcelain`; no `git worktree add` in the repo (`internal/core/cwd.go:1-65`); "a git worktree per lane, made by the agent" (`docs/guide.md`, Where workers run) | one worktree per feature (`AGENTS.md`, Bee workflow); Paseo workspace per worktree, `--workspace` passed to `bee herding run` (`docs/history/paseo-worktree-workspace/CONTEXT.md` D2-D4); shared-index pattern (`docs/knowledge/areas/worktree-parallelism/overview.md:89`); guard remedy `CONCURRENT_TREE_TEMP_INDEX_REMEDY` (`packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs:365-371`) |
| 5 | Independent review: `dual-lens` / `council` taskforce vs `bee-reviewing` | **bee is stronger on isolation; one gap on model diversity.** bee lenses already run in parallel on a frozen scope and never see each other, with a head check. piggery's chair is read-only "by prompt, not one piggery enforces". The gap: bee's severity rule names "different models", but the dispatch rule sends every lens with no `--role`, so all lenses share one review model. → **PBI 2**. | lenses `lens-a`/`lens-b` with their own `spawn.model`, routing keeps them apart (`manifests/dual-lens.yaml:16-30`, `manifests/README.md#dual-lens`); chair read-only only by prompt (`docs/guide.md`) | core four plus conditional, capped at six, `--kind reviewer` and no `--role` (`skills/bee-reviewing/SKILL.md:33-40`); "independent only across different models or disjoint lenses" (`skills/bee-reviewing/SKILL.md:84`); `REVIEW_HEAD_MISMATCH` frozen-head check (`skills/bee-reviewing/SKILL.md:39-48`) |
| 6 | Worker lifecycle: abort, kill the tree after 2 s, resume with unacked mail, wake on death | **bee has the equivalent; auto-wake is a deliberate no.** The tree kill with a 2 s grace shipped as piggery-borrow D2. `bee herding run --continue` resumes a job. bee's scope decision forbids auto-retry and auto-relaunch: every signal is a word the leader reads. piggery's wake-on-death is exactly the auto-relaunch that decision rejects. | SIGTERM, `KillWait` 2 s, SIGKILL to the tree (`internal/driver/local/runner.go:411-450`); crashed worker woken once per unfinished mail (`internal/core/agent.go:552-585`) | `TREE_KILL_GRACE` 2000 ms (`packages/bee-rs/crates/bee/src/herding/job_verbs.rs:279`); `--continue` and `paused_limit_at` resume (`run.rs:4656-4700`); decision `9615be76` "No agent fallback on provider error, no auto-retry, no auto-relaunch of orphans"; supervisor observer polls and records only (`docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md`) |

## Improvements worth doing (value over cost, highest first)

| Rank | Item | Value | Cost | PBI |
|---|---|---|---|---|
| 1 | Give the herding worker cap one home. The number the docs promise ("~3-4", "four slots") and the number the code enforces (`herding.limits.concurrency`) are two facts today, and in this repo the code side is empty. Pick the owner (the config key, set by onboarding or a code default) and make the prose cite it. | Medium: the cap the docs promise becomes real; one place to change it | Small: one default plus a prose sweep; the refusal code already exists | `p-7cc2ebe1` |
| 2 | Let a review lens run on a second model family. Allow `bee-reviewing` to name a second review role (for example a `review-alt` role in `team.<runtime>`) for one lens, so the "different models" corroboration in the severity rule can happen. | Medium: cross-model disagreement is the one thing a same-model lens cannot give | Small to medium: a skill rule change plus one optional config role; the door already accepts `--role` | `p-165ef6d7` |

## What NOT to do, and why

- **Do not run piggery as the worker runtime.** bee's door already resolves a
  native subagent or a herding worker; piggery would replace a working file
  contract with a daemon, and it has no first-party Paseo support
  (`CHANGELOG.md:113-116`).
- **Do not add piggery mail beside `.bee/mailbox`.** Two mail systems would
  route the same cell through two stores, and a result could land in one
  while the leader reads the other. bee's broker design already refused a
  second mailbox (`docs/history/mailbox-broker/CONTEXT.md`).
- **Do not add rate limits or respawn caps.** They guard peer mail and
  automatic respawn. bee has neither.
- **Do not add wake-on-death or auto-resume.** Decision `9615be76` rejects
  auto-relaunch on purpose (Chesterton's fence: the reason is still true).
- **Do not add per-cell worktrees for this reason.** piggery does not make
  them either; the shared-index risk already has a guard.
- **Do not port YAML team templates or a taskforce chair.** piggery-borrow
  already skipped YAML templates, and bee's frozen-scope review is stronger
  than a chair that is read-only only by prompt.
- **Do not add multi-harness support.** Out of scope by the owner's word.

## Risks

- **Two parallel mail systems.** If piggery ever ran beside bee, a cell's
  handback could go to piggery while the claim, cap and proof stay in
  `.bee/`. The leader would see a stale claim, and the sweep would park a
  finished cell `blocked`.
- **Single machine only.** piggery is one Go binary, one SQLite file and one
  unix socket under `~/.piggery`, with nothing in the cloud
  (`README.md:27`; `internal/server/server.go:54-57`). Any bee flow on cloud
  sessions or a second machine would see none of it.
- **A third daemon.** bee already leans on two long-lived daemons: Paseo
  (`docs/knowledge/areas/bee-herding/the-paseo-channel.md`) and waggledance
  (`docs/discovery/slp-human-up/tickets/004-waggledance-supervisor-feasibility.md`).
  A third adds one more process to start, upgrade and debug, for features bee
  already has.

## Findings

### Local

- piggery-borrow already took the parts of piggery that fit:
  turn-scoped ack (D1), tree kill (D2), limits (D3), belt contract (D4),
  belt state (D5), worker ctx and turns (D6), runtime doctor rows (D7)
  (`docs/history/piggery-borrow/CONTEXT.md`).
- Notify when a human is needed: bee already has waggledance's notify
  watcher and `supervisor.notify` for urgent rows
  (`packages/bee-rs/crates/bee/src/verbs/supervisor.rs:73`). piggery's
  notify hook adds nothing new here.

### Upstream

- piggery's design rule "mail is acked only when its turn ends normally" is
  the one bee borrowed. Its other rules (routing table, cross-team gate,
  taskforce lifetime) serve peer-to-peer teams, which bee does not run.

### Inference

- A herded worker started by hand with a `--cwd` outside the repo's
  worktrees is not refused (no cwd bound found by `rg` in
  `packages/bee-rs/crates/bee/src/herding/run.rs`). It is not filed: `bee
  dispatch prepare` always fills `--cwd` with the worktree root, and the
  write guard is the net. Re-open only if a hand-run herding job ever wrote
  outside its tree.

## Risks, Unknowns, Follow-Ups

- No measured comparison of a cross-model lens against a same-model lens
  exists. PBI 2's triage should ask for one review run with both.
- The rate-limit and four-slot negatives come from `rg` over
  `packages/bee-rs/crates/bee/src`; the JS runtime under `packages/bee/` was
  not searched.

## Source Pack

- Local files read: every bee path cited in the verdict table.
- Upstream repo checked: `/mnt/data/Projects/refs/slp/piggery` at v0.10.2.
- Docs pages checked: piggery `README.md`, `docs/guide.md`,
  `docs/reference.md`, `manifests/README.md`, `AGENTS.md`.
