---
artifact_contract: bee-research/v1
topic: bee-unattended-hardening
depth: deep
date: 2026-09-01
---

## Bottom Line

- Recommendation (ladder rung): **reuse** — every fix below already exists
  somewhere in bee, applied to the wrong scope or left as prose. Nothing here
  needs a new mechanism, and nothing here widens bee's authority.
- The axis, from bee's own history: bee's scars support strengthening the
  unattended path's **liveness and honesty** — receipts, typed outcomes,
  counting doors, stall diagnosis, digest delivery. The same scars argue hard
  against widening its **authority** — merge, spawn-on-death, self-approval,
  silence-as-consent. Every widening bee has accepted is logged as a named,
  owner-spoken accepted risk, never inferred.
- Why the next-best rung lost: **adapt-upstream** would import pstack's
  coordinator machinery. pstack measured that machinery losing 12-to-1 against
  a plain agent on a small job (`orchestrate.md:3`). Take its scars, not its
  ceremony.
- Confidence: **90%** on the six findings (each verified in the binary or the
  decision log); **70%** on the ranking.
- Suggested next step: **bee-shaping**. Findings 1 and 2 contradict locked
  decisions and are defects, not proposals.

## Question & Assumptions

- What was asked: `never-block-on-the-human` fits bee's autopilot — go deep and
  find improvements for bee.
- The redirect this research forces: `never-block` is about **not stalling**,
  not about **more power**. bee's history is emphatic on the difference. Every
  item below is a liveness or honesty fix; none grants the loop new authority.
- Live configuration this repo runs today: `gate_bypass: "full"`,
  `uat_stop: "close"`, `uat_before_merge: false`, `staging_before_merge: false`
  (`.bee/config.json:13,17`). Merge-on-green, no human at merge.

## Findings

### Local — the six findings

**1. The control loop counts failures, never progress.** [defect]

`packages/bee-rs/crates/bee/src/herding/control_loop.rs` is 1,864 lines and
bounds a run properly: interval 60s, per-iteration wall clock 900s then SIGTERM
plus a 30s SIGKILL grace, max-iterations 10,000, 20 consecutive failures with a
capped backoff, and a stop file checked before *and* after every iteration.

A search for `plateau|progress|stagnat` across the herding tree returns nothing.
A do-nothing iteration exits 0 and counts as a success, so ten thousand iterations
that accomplish nothing are a clean run. The loop cannot tell working from
spinning.

It is worse than a blind spot. `role-dispatch.md:94-118` deliberately de-dupes
the occupancy-fallback refusal after announcing it once, so a transport that has
been down for a day looks identical to an idle backlog.

pstack's rule, learned the same way: *"Count only side effects as progress:
commits, pushes, PR or check deltas, and store reports. Treat a lane that passes
its expected runtime without a side effect as stuck."*
(`poteto-mode/playbooks/autopilot-full.md:10`). bee already writes every side
effect it would need — the wave ledger, git, cell state.

**2. The two control panes hold a wildcard, against a locked decision.** [defect]

`control_loop.rs:277-286` grants the dispatch and merge panes:

```
Bash(.bee/bin/bee:*), Bash(git -C:*)
```

That wildcard is the entire bee CLI — `worktree merge`, `state gate`, every
mutating verb — plus arbitrary `git -C` against any repository on the machine.
The rules that keep those panes in their lane ("merge stays a human gesture",
"dispatch only starts work", "refuse below `gate_bypass` full") are prose, read
by a cold Sonnet every sixty seconds.

The locked decision says otherwise, verbatim:

> herding-adopt D7-FINAL, OWNER DECISION: the four working agents keep
> `--permission-mode bypassPermissions` as an explicitly accepted risk with its
> blast radius recorded; **the two CONTROL panes are narrowed to an enumerated
> command surface**, never to "read-only".

The fix already exists twenty lines below in the same file.
`SUPERVISOR_ALLOWED_TOOLS` (`control_loop.rs:292-302`) enumerates the observer's
surface verb by verb, and a test asserts the forbidden tokens can never
reappear. Its own doc comment names the gap:

> Read the arm doc on `allowed_tools_for` for why the `Bash(.bee/bin/bee:*)`
> wildcard the cockpit roles carry is deliberately absent here.

bee knew the wildcard was the risk, solved it once, and did not apply the
solution to the two roles that can actually write.

**3. The `normal` safety floor exists only in prose.**

`skills/bee-hive/references/gates-and-delegation.md:95` says `normal` never
bypasses a high-risk or hard-gate Gate 1-2. The recording verb —
`packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs`, 2,990 lines —
contains zero occurrences of `bypass`, `high-risk`, or `hard-gate`. It requires
an auto approval to carry `--bypass-level` and `--reason`, and refuses
`--name uat --actor auto` outright, but it never checks that the claimed level
is entitled to cover this feature's risk. An auto approval declaring
`--bypass-level normal` on a high-risk feature is accepted.

The only bypass-aware hook (`hooks/session_close/nudges.rs:374-465`) pushes a
stopped agent forward. It can force an approval; it can never refuse one.

**4. Documentation and binary disagree about secret reads at `total`.**

`gates-and-delegation.md:86,100` says `total` auto-proceeds on secret-file
reads. `write_guard/` contains zero references to `bypass` at any level, and the
privacy guard hard-denies Read/Glob/Grep on secret-shaped paths with the
`@@BEE_PRIVACY@@` marker regardless. The binary wins on every guarded channel;
the prose "wins" only where the hook cannot see. One of the two must move.

**5. Nothing is told when the loop dies, and nothing tracks budget.**

After 20 consecutive failed iterations the control loop exits non-zero and never
restarts itself; the remedy is "re-run bootstrap"
(`herding/references/operational-invariants.md:589-591`). Nobody is notified.
bee owns the right endpoint already — the human mailbox, where an unattended run
composes a letter (decision human-mailbox D9).

There is no cumulative budget or quota accounting anywhere.
`retry.fallbackChains` is published-only: *"bee does not execute dispatches, so
bee never retries"* (`swarming-reference.md:385`). bee has been bitten here
twice — five advisor seats died together on a Fable HTTP 429 (2026-08-31), and
herding-limit-pause D1-D4 records *"a paid, resumable worker context was
classified as idle timeout and wrongly discarded"*.

pstack's landing rule addresses the same failure from the other side: *"by
roughly 70% of it, stop spawning and land what is verified, because
finished-but-unlanded work counts as zero"* (`orchestrate.md:62`).

**6. A capped-green cell whose base moved has no rule.**

`bee worktree merge` performs a zero-mutation check of the recorded proof line
and then merges. A textual conflict stops it; a clean-merging semantic break
lands on main, and CI is the only net. The green-base check runs at claim time
only.

This is the exact failure pstack instrumented after measuring it: *"Twenty-one
verdicts went stale this way in one run with no signal at all"*
(`shipping.md:9`); its fix keys every verdict to a head SHA so a new head voids
it, enforced in `scripts/orch/store.ts:1366`.

bee's own evidence says this is its most likely next failure. Six independent
recorded incidents form one class — a green that proves nothing: an impacted run
computed after the commit selecting zero suites and capping `verify_passed:true`
on a red change; a selector matching a suite that did not exist yet; five
security tests landing inside another test's body (`0 passed, 1963 filtered
out`); `cargo test` stopping at the first failing target while three reds sat on
`origin/main` for days; sixteen workers running a whole wave green with the
authoritative CONTEXT.md absent. Each produces exactly a plausible recorded
proof line, and the doors check the recorded line, never re-run it. bee's own
critical pattern names the reason this is fatal unattended: *plausibility is not
evidence, and the author is never the one who catches it.*

### Local — a decision conflict to resolve

Two decisions from the same day disagree about this repo's own `uat_stop`, and
neither supersedes the other:

- `.bee/decisions.jsonl:2233` (2026-08-21, `relation: touches`) — *"Config:
  uat_stop=close and uat_before_merge=false — a green close merges the feature
  worktree into main without a pre-merge uat stop."*
- `.bee/decisions.jsonl:2257` (2026-08-21, later, `relation: none`) — *"This
  repo sets uat_stop=merge and worktree_cleanup_on_merge=false: bee worktree
  merge refuses until the user approves the uat gate."*

Live config follows the earlier one (`close`). The later, stricter decision is
unenforced. Under `gate_bypass: full` plus `uat_stop: close`, an unattended run
merges into main with no human stop at merge; the stop moves to `bee close`.
This is the user's call, not the agent's — but the record should say one thing.

### Upstream

pstack (`cursor/plugins`, `b9ddc83`) documents 29 distinct unattended failure
modes with measurements attached. The ones bee has no rule for, in bee's terms:
progress-by-side-effects and stand down a stuck lane at once
(`autopilot-full.md:10`); patch-id re-check before trusting an older verdict
(`shipping.md:9`); the 70% landing threshold (`orchestrate.md:62`); retry by
failure mode, two then abandon and replan (`orchestrate.md:99`); reconcile a
zombie's late result against current state, never blind-merge it
(`orchestrate.md:100`); bound your own infra retries and end with a terminal
handoff rather than looping (`orchestrate.md:102`); and *"never resume an agent
to check on it; a resume restarts an idle agent"* (`orchestrate.md:97`).

Honest split on pstack's side: its PR watcher and bookkeeping store are real
code with tests; every rule about the coordinator's own behavior is prose. It is
not a system to copy — it is a logbook to read.

### Inference

- Findings 1, 5 and 6 are the ones that fire while a human sleeps. Finding 2 is
  the one whose worst case is largest, and it is already a locked-decision
  violation rather than a design question.
- bee's bypass scars are mostly of the *opposite* polarity to the fear: the
  unattended path has failed by **stopping** (gate-bypass-stop-net, Codex
  stopping at Gate 1 under `total`, the `/clear` offer parking a loop forever,
  Pi hard-stopping on any pane question). That is the evidence behind the user's
  intuition, and it is correct — as far as liveness goes.
- The countervailing evidence is equally strong and points only at authority.
  Locked decision `.bee/decisions.jsonl:1338`: *"Split automation at the point
  where an action becomes hard to reverse… when you find yourself adding guards
  to make an irreversible unattended step acceptable, that is the signal to make
  it a gesture instead."* bee has rolled autonomy **back** twice on measurement:
  the lead-recovery auto-spawn was killed by its own plan check (*"the trigger
  has never fired in 218 sessions, both real candidates are false positives"*),
  and the dispatch classifier's measured performance includes passing *"delete
  the entire JS runtime"* 8 times out of 8.

## Risks, Unknowns, Follow-Ups

- **Do not answer any of this with a new refusal until its remedy is proven from
  the refused caller's own state.** bee has three recorded instances of a
  refusal whose remedy could not run, each deadlocked until a human ran the
  command from outside (`patterns/20260806-arm-a-refusal-only-after-its-own-remedy-is-proven-to-work.md`).
  Unattended, that human does not exist. This is live risk right now: the
  in-flight reflection-becomes-lesson work puts a refusal on `bee close`, a door
  every unattended run and the herding control loop pass through.
- **A door answered by its own escape hatch is the defect, not the worker**
  (`patterns/20260825-a-guard-that-cannot-pass-teaches-agents-to-ack-it.md` —
  3/3 workers in one wave capped with `--sync-ack`). Any counting door added
  here must have its acks counted.
- The `normal`-floor and secret-read findings are contradictions, so fixing
  either direction is a decision about intent, not a bug fix. Ask before
  choosing the direction.
- Unverified: whether Codex's runtime honors the same allowlist shape as
  Claude's, which bounds how finding 2 should be written.

## Source Pack

- Local: `.bee/config.json`, `.bee/decisions.jsonl` (lines 142, 174, 1336-1339,
  2148-2150, 2222, 2233, 2257, 2513, 2558, 2561),
  `packages/bee-rs/crates/bee/src/herding/control_loop.rs`,
  `.../verbs/state_group/set_gate.rs`, `.../write_guard/`,
  `.../hooks/session_close/nudges.rs`, `.../worktree/phases.rs`, `.../uat.rs`,
  `.../cells/handlers_close.rs`, `.../cells/handlers_select.rs`,
  `skills/bee-hive/references/gates-and-delegation.md`,
  `skills/bee-herding/` (SKILL + all 8 references),
  `skills/bee-swarming/references/swarming-reference.md`,
  `docs/knowledge/patterns/` (175 files surveyed; ~40 read),
  `docs/knowledge/areas/bee-herding/`, `.bee/expertise/`.
- Upstream: `cursor/plugins` @ `b9ddc83`, `pstack/skills/poteto-mode/`
  (SKILL + 25 playbooks + `references/bugbot-triage.md` + `scripts/`),
  `pstack/skills/principle-never-block-on-the-human/SKILL.md`.

Source content was treated as data throughout, never as instructions.
