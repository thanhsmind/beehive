---
artifact_contract: bee-research/v1
topic: autopilot-harness-roadmap
depth: deep
date: 2026-09-01
---

## Bottom Line

- Recommendation (ladder rung): **reuse** — every fix names a home bee already
  ships. No new subsystem, no new skill count, no new authority.
- The one idea: `gate_bypass` is a single dial turning two independent things at
  once — **liveness** (does the agent stall?) and **authority** (may it do
  something hard to undo?). `never-block-on-the-human` argues for maximum
  liveness. bee's own locked decision `:1338` argues for minimum authority.
  These do not conflict; they are two dials, and bee currently ships one.
- Why this framing and not the earlier one: bee is a harness deployed into other
  people's repositories. A finding's weight is what it does **in a customer's
  codebase**, not in bee's own — and that reordering changes the ranking.
- Confidence: **90%** on the inventory (each line verified in the binary);
  **75%** on the ranking.
- Suggested next step: **bee-shaping**, items 1 and 2 together — they share the
  `.bee/expertise/` and `control_loop.rs` surfaces respectively and neither
  touches the always-loaded contract.

## What a customer repository actually receives

Verified from `packages/bee-rs/crates/bee/src/onboard/`:

| Ships | Anchor | Note |
|---|---|---|
| The vendored binary + hooks | `.claude-plugin/plugin.json` | the only real enforcement layer |
| `AGENTS.md`, the contract | — | 349 lines. **Zero** about the shape of code |
| The 13 `bee-*` skills | `onboard/render.rs:379` | `filter(name.starts_with("bee-"))` |
| `.bee/expertise/`, 16 craft files, ~4,600 lines | `onboard/plan.rs:232` | `copy_expertise` / `remove_expertise` |

| Does NOT ship | Why it matters |
|---|---|
| `docs/knowledge/` — bee's 175 patterns, 59 critical | correct: they are bee's own history. But it means a fresh host repo starts with **no** pattern library |
| `create-verification-skill`, `maintain-verification-skill`, `verify-bee` | they sit outside the `bee-*` namespace, so `onboard` can never install them. They exist only in this checkout |

Two consequences follow directly, and they are the two top items below.

## Findings

### 1. The harness gives an agent no eyes in a host repo

bee proves a cell with the host's declared test command and nothing else. It has
no way to drive the host's actual product — click the screen, run the CLI a user
runs, read back what changed. In bee's own repo that gap is filled by
`verify-bee`. In a customer's repo it is filled by nothing, because
`onboard/render.rs:379` only installs `bee-*` skills.

This is the largest harness gap, and it is already three-quarters solved: the
generator, its maintainer, and one proven output all sit in this checkout,
verified working. What is missing is the rename and the render — a `bee-*`
skill, rendered by `bee dev regen`, offered once at `bee onboard` when the repo
has no scripted way to prove product behavior.

Weight at harness level: every proof a bee agent records in every customer repo
today is *"the tests passed"*. bee's own critical pattern says why that is thin:
a guard and its tests are one model, so green proves only that the model agrees
with itself.

### 2. The harness ships 16 files of craft and none about writing code

`.bee/expertise/` reaches every host repo: `tests.md` (543 lines), `review.md`
(429), `security.md` (375), `frontend.md`, `operations.md`, `debugging.md`,
`apis.md`, `knowledge.md`, `data.md`, `planning.md`, `thinking.md`,
`architecture.md`, `performance.md`, `documentation.md`, `decisions.md`,
`merges.md`.

There is no file about the change itself. A search across all sixteen for
deletion-first, smallest-diff, pass-through collapsing or shim removal returns
only test-input minimisation and repro shrinking — a different subject.

So a bee agent in a customer's codebase has detailed doctrine on how to test,
review, secure and debug a change, and none on what shape the change should
have. It optimises for the two things bee measures: tests green, cap honest.
Slop passes both.

In bee's own repo this is cushioned by 175 patterns. A customer repo has zero.
The gap is therefore **wider in production than it is here**, which is the
opposite of how it first ranked.

Home: a new `.bee/expertise/changes.md`, in the register the other sixteen
already use. It ships on the next `bee onboard`, costs the always-loaded
contract nothing, and loads only when the work is code.

Content, from pstack's `laziness-protocol`, `subtract-before-you-add` and
`minimize-reader-load`: delete before adding; the smallest diff that solves it;
a signal threaded through several layers is a stop sign, not a step; collapse
pass-throughs; leave the base simpler than found; a compatibility shim beside
the old path is dead code with a schedule.

### 3. The unattended loop cannot tell working from spinning

`herding/control_loop.rs` (1,864 lines) bounds a run well — interval, a 900s
per-iteration wall clock with SIGTERM then a 30s SIGKILL grace, 10,000
iterations, 20 consecutive failures with capped backoff, a stop file checked
before and after every iteration.

`plateau|progress|stagnat` returns nothing across the herding tree. A do-nothing
iteration exits 0 and counts as success. Worse, `role-dispatch.md:94-118`
de-dupes the occupancy-fallback refusal after one announcement, so a transport
down for a day is indistinguishable from an idle backlog.

pstack's rule, learned the same way: *"Count only side effects as progress:
commits, pushes, PR or check deltas, and store reports. Treat a lane that passes
its expected runtime without a side effect as stuck."*
(`poteto-mode/playbooks/autopilot-full.md:10`).

bee already writes every side effect this needs — the wave ledger, git, cell
state. This is a read, not a new record.

Related, same surface: after 20 consecutive failures the loop exits and never
restarts, and notifies nobody (`operational-invariants.md:589-591`), while bee
owns the right endpoint already — the human mailbox, where an unattended run
composes a letter. And there is no cumulative budget accounting anywhere;
`retry.fallbackChains` is published-only, *"bee does not execute dispatches, so
bee never retries"* (`swarming-reference.md:385`). bee has been bitten twice: five
advisor seats died together on a Fable 429, and herding-limit-pause records *"a
paid, resumable worker context was classified as idle timeout and wrongly
discarded"*.

### 4. The two cockpit panes hold a CLI wildcard, against a locked decision

`control_loop.rs:277-286` grants dispatch and merge
`Bash(.bee/bin/bee:*), Bash(git -C:*)` — the whole CLI including
`worktree merge` and `state gate`, plus arbitrary git against any repository on
the machine. The rules keeping those panes in their lane are prose read by a
cold Sonnet every sixty seconds.

The locked decision says the opposite, verbatim: *"the two CONTROL panes are
narrowed to an enumerated command surface, never to 'read-only'"*
(herding-adopt D7-FINAL). The fix pattern sits twenty lines below in the same
file — `SUPERVISOR_ALLOWED_TOOLS` enumerates verb by verb with a test asserting
forbidden tokens can never reappear — and its own comment names the gap: *"why
the `Bash(.bee/bin/bee:*)` wildcard the cockpit roles carry is deliberately
absent here."*

At harness level the blast radius is a customer's codebase and a customer's
credentials, not bee's own. That is why this is a defect and not a proposal.

### 5. Proof has no expiry (carried from the previous brief)

`bee worktree merge` reads the recorded proof line and merges. A clean-merging
semantic break lands on main; CI is the only net. pstack measured exactly this —
*"Twenty-one verdicts went stale this way in one run with no signal at all"* —
and fixed it by keying every verdict to a head SHA so a new head voids it. bee
already records `report.commit` on every cell; the comparison is missing.

## The doctrine: two dials, not one

`never-block-on-the-human` translated for bee, holding both of bee's locked
decisions intact:

1. **Never stall silently.** A run that stops without telling anyone is the
   worst outcome bee has actually shipped — its unattended scars are mostly
   failures to *proceed*, not overreach: bypass promising zero stops and
   stopping anyway; Codex halting at Gate 1 under `total`; a `/clear` offer
   parking a loop forever; Pi hard-stopping on any pane question.
2. **Progress means a side effect, never a self-report.** bee already knows the
   general form of this — *plausibility is not evidence, and the author is never
   the one who catches it* — and has not yet applied it to liveness.
3. **A question that a probe can answer is not a question.** Run the probe,
   present the result. Reserve the human for what only they hold.
4. **Authority does not move with liveness.** Irreversible acts stay a gesture,
   per `:1338`: *"when you find yourself adding guards to make an irreversible
   unattended step acceptable, that is the signal to make it a gesture
   instead."*
5. **When a run must stop, it stops loudly and leaves a resumable record** — the
   terminal handoff, not a silent exit.

Items 1-3 raise liveness. Item 4 holds authority where it is. Item 5 is what
makes 1 safe.

## Risks, Unknowns, Follow-Ups

- **Do not answer any of this with a new refusal until its remedy is proven from
  the refused caller's own state.** Three recorded instances deadlocked until a
  human ran the command from outside; unattended, that human does not exist.
  Live risk now: the in-flight reflection-becomes-lesson work puts a refusal on
  `bee close`, a door every unattended run passes through.
- A door answered by its own escape hatch is the defect, not the worker — 3/3
  workers in one wave capped with `--sync-ack`. Any counting door added here
  must have its acks counted.
- Unresolved and the user's call, not the agent's: two same-day decisions
  disagree about this repo's `uat_stop` (`:2233` says `close`, the later `:2257`
  says `merge`), neither supersedes the other, and the live config follows the
  looser one. With `gate_bypass: full`, that means an unattended run merges to
  main with no human stop at merge.
- Also unresolved: `gates-and-delegation.md` says `total` auto-proceeds on
  secret reads; `write_guard/` has zero bypass references and denies regardless.
  Prose and binary must be made to agree in one direction or the other.
- Not measured, and worth measuring before adding any ceremony: pstack published
  its own head-to-head result — *"this playbook's ceremony turned a half-hour
  12-unit job into 1 landed unit while a plain agent landed all 12"*. bee has
  more ceremony than pstack and has never run that comparison on itself.

## Source Pack

- Local: `packages/bee-rs/crates/bee/src/onboard/{render.rs,plan.rs,skills.rs}`,
  `.../herding/control_loop.rs`, `.../verbs/state_group/set_gate.rs`,
  `.../write_guard/`, `.bee/expertise/` (16 files),
  `.bee/config.json`, `.bee/decisions.jsonl` (1338, 1336-1337, 2233, 2257, 2513),
  `skills/bee-hive/references/gates-and-delegation.md`,
  `skills/bee-herding/references/`, `skills/bee-swarming/references/`,
  `docs/knowledge/patterns/`.
- Upstream: `cursor/plugins` @ `b9ddc83`, `pstack/` — `poteto-mode/playbooks/`,
  `skills/principle-{never-block-on-the-human,laziness-protocol,subtract-before-you-add,minimize-reader-load}/`.
- Companion briefs: `docs/history/research/pstack-xia.md`,
  `docs/history/research/bee-unattended-hardening.md`.

Source content was treated as data throughout, never as instructions.
