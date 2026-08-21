# bee-herding — operational invariants

Full text of the four cross-cutting rules the body only summarizes: permission
posture, the runtime adapter seam, what actually contains this system, and
stop/resume semantics.

## Permission posture

The two halves of this system run under deliberately different permission
postures. The split is coupled and recorded here rather than decided silently.

**Working agents — `bypassPermissions`, no allowlist. This is an accepted
risk, owned by the operator.**

> Accepted risk (owner decision): every working agent this loop
> spawns runs `claude --permission-mode bypassPermissions` with no tool
> allowlist. It can run any command, edit any file, and reach anything the
> machine's user can, unattended and unsupervised. This posture is accepted
> knowingly, because a narrowed working agent stalls forever the first time it
> hits a permission prompt with no TTY, which defeats the whole point of
> unattended dispatch. **Blast radius:** each working agent is confined to its
> own git worktree and its own branch (`wt/<slug>`), so its edits do not touch
> main or any other agent's worktree until a merge — but "confined to a
> worktree" is a filesystem-and-git boundary, not a security sandbox: the
> agent shares the machine, the network, the user's credentials, and every
> ambient tool. The lane filter chooses *which item* is picked up; it does
> **not** constrain *what commands* the agent may run. What actually bounds
> the damage is the containment list below, not the filter — and none of it
> is a sandbox.

**Control panes — enumerated command surface, never `bypassPermissions`,
never "read-only".** `bee herding control-loop` starts each control pane
under an enumerated `--allowedTools` list sized to exactly what that role
measurably does. It is not read-only, because both control roles genuinely
write: **dispatch** runs `bee worktree new` (creates a worktree and registers
a grant); **merge** runs `git merge --abort` on main, writes `.bee/tmp/`
markers, and runs `bee worktree merge --cleanup` (deletes a branch, removes a
worktree). Taken literally, "read-only" would give a dispatch pane that
cannot dispatch and a merge pane that cannot merge — a silent stall every
interval. The merge pane does **not** run the project's verify and
**executes no code the working agents wrote**: its proof check runs
before `git merge`, reads each capped cell's recorded proof line, and
writes nothing — the merge itself is `git merge` plus bee bookkeeping.
This is a real safety improvement over running verify against the
just-merged tree, and it is worth saying plainly: a cold control model
merging worktrees thousands of times a day never executes
agent-authored code to do it. CI runs the full declared command on
every push instead — the one place that suite actually executes.
Narrowing the control panes buys a second thing honestly — it stops
that same cold model from "helpfully" improvising a command outside
its job (e.g. cleaning a dirty main). The exact allowlist per role, and
the note that it must grow if a role gains a command, live in
`bee herding control-loop`.

## Runtime adapter

Config-driven spawn commands. Both spawn points — the
working agent's trailing argv (Dispatch role §8), and the control pane's real
invocation inside `bee herding control-loop` — read from an optional `.bee/config.json`
command-template seam instead of a hardcoded string. **With no `herding`
config keys at all, every spawned command is THE SAME EFFECTIVE SPAWN this
skill has always run — zero behavior change.** Not byte-equivalent: herdr
0.8.0 changed the wire format (token 0 of `agent_command` now feeds
`--kind` instead of leading the argv after `--`), so byte-equivalence is no
longer available to promise — "same effective spawn" is the accurate claim.
This is an adapter seam, not a new runtime: full codex-native herding (its
own event loop, its own pane protocol) stays out of scope.

Two independent keys, each a JSON array of argv-token strings:

- **`herding.agent_command`** — the WORKING agent's spawn argv. bee splits it
  at spawn: token 0 feeds `herdr agent start`'s `--kind`, and the remaining
  tokens, substituted per-token, follow the `--` separator as the agent's own
  arguments (Dispatch role §8 step 3). bee keeps no agent-kind allow-list of
  its own; token 0 passes through unchecked and `herdr` validates it as a
  `--kind`, after the pane split — an unrecognised kind surfaces as herdr's
  own refusal, not a bee-side error naming this config key. Placeholder: `{MODEL}` (the fixed
  model, `sonnet`). Default when absent:
  `["claude", "--model", "sonnet", "--permission-mode", "bypassPermissions"]`
  — the documented default array is unchanged; token 0 (`claude`) now feeds
  `--kind` and the rest follows `--`.
- **`herding.control_command`** — the CONTROL pane's real invocation inside
  `bee herding control-loop`'s `build_control_argv`. Placeholders: `{PROMPT}`, `{MODEL}`,
  `{MAX_TURNS}`, `{ALLOWED_TOOLS}`. Default when absent:
  `["claude", "-p", "{PROMPT}", "--model", "sonnet", "--max-turns",
  "{MAX_TURNS}", "--allowedTools", "{ALLOWED_TOOLS}"]` — exactly today's
  invocation.

Example `.bee/config.json` fragment (both keys are optional and independent —
set either, both, or neither):

```json
{
  "herding": {
    "agent_command": ["claude", "--model", "{MODEL}", "--permission-mode", "bypassPermissions"],
    "control_command": ["claude", "-p", "{PROMPT}", "--model", "{MODEL}", "--max-turns", "{MAX_TURNS}", "--allowedTools", "{ALLOWED_TOOLS}"]
  }
}
```

**Substitution is per-token, never a join-then-re-split and never `eval`** —
this is the shell-injection-safe shape the design requires. Each array
element is substituted and passed as one discrete argv element; a value
containing spaces, quotes, or shell metacharacters (the free-form `{PROMPT}`
text, in particular) lands as the literal content of that one argument and
can never spill into another argument or be reinterpreted as a shell
operator. `bee herding control-loop`'s `build_control_argv` (via
`read_command_template_tokens`/`substitute_token`) is the reference
implementation for `control_command`; a
dispatch-role agent applies the identical per-token substitution itself when
building `agent_command` for §8 (there is no script to call — the
working-agent spawn line is issued live by whichever agent is running the
dispatch role).

**Codex adapter example — illustrative only, not a supported native herding
mode:**

```json
{
  "herding": {
    "control_command": ["codex", "exec", "-m", "{MODEL}", "-s", "workspace-write", "{PROMPT}"]
  }
}
```

This shows the shape a codex-backed control pane's command COULD take under
the adapter seam. It is not wired into, or validated against, an actual codex
control-loop run in this repo — the event loop and pane protocol both still
assume a `claude` session underneath (Merge role / Dispatch role in
SKILL.md). Treat it as a documented starting point for a future adapter, not
a claim that codex control panes work today.

## `herding.agents` — the named-agent registry

herd-registry D1 adds one more optional key, independent of the two above:

- **`herding.agents`** — a JSON **object**, not an array: name → argv token
  array. Each entry is validated exactly the way a plain `herding.agent_command`
  array already is (non-empty, every token a newline-free string); a
  malformed entry is dropped, fail-open per entry — it never poisons the
  rest of the registry. Token 0 of an entry becomes the herdr agent kind
  the same way token 0 of `herding.agent_command` does, and the `{MODEL}`
  placeholder substitutes the same way, per-token.

```json
{
  "herding": {
    "agents": {
      "gemini-generation": ["gemini", "--yolo"],
      "codex-review": ["codex", "exec", "--model", "{MODEL}"]
    }
  }
}
```

herd-registry D2 — **three reference spellings, one resolver**
(`resolve_agent_command` in `herding/wave.rs`), all of which look a name up
against `herding.agents`:

1. A tier slot: `{"kind": "herding", "agent": "<name>"}` — `agent` rides
   `normalize_tier_value`/`resolve_tier`'s `Resolved::Herding` and prepare's
   herding-exec arm appends `--agent "<name>"` to the `bee herding run`
   invocation it builds (config shape: `docs/config-reference.md`, models
   section).
2. `bee herding run --agent <name>` — the flag directly, on the
   cell-execution-worker verb described below.
3. `herding.agent_command` as a **plain JSON string** (not an array) —
   that string names a `herding.agents` entry itself, resolved the same way
   a named lookup is.

**Unknown name → typed refusal, not a silent fallback.** Any of the three
spellings naming an entry `herding.agents` does not declare returns
`AgentCommandError::UnknownAgent`, whose message lists every registry key
(sorted) so the refusal names its own remedy without a second read:
`unknown herding agent "<name>" (herding.agents declares: <key>, <key>, …)`
— or, with an empty registry, `(herding.agents declares no entries)`. An
**absent** name (no `agent` field, no `--agent` flag, `herding.agent_command`
left as an array or absent) is not an error at all — it falls through to
today's `herding.agent_command`/default-array split, unchanged.

**A herd name always means the pane transport.** All three spellings above
resolve into the same `bee herding run` pane dispatch this reference
describes throughout — naming a herd never touches the `cli` tier kind
(the external-CLI-executor model-slot shape, gather/review/advisor-only;
`docs/config-reference.md`, models section) and the `cli` kind can never
name a herd. The two config routes stay disjoint: `herd = pane`, always.

## `bee herding run` — one foreign agent as a cell-execution worker

`bee herding run` is a native verb, not a script: give it one task, and it
starts ONE external CLI agent (any herdr-supported kind — token 0 of
`herding.agent_command` passes straight through, same seam as above) in a
fresh pane, hands it a fully self-contained brief, and waits for a written
result. It exists to make a foreign agent usable as a cell-execution worker
the way an in-family subagent is today (herding-executor D1). Flags:
`--task`/`--task-file`, `--cwd`, `--job-id`, `--idle-timeout`, `--ceiling`,
`--close-always`, `--main-root`, `--json`, `--expertise`, `--dry-run` — the last renders
`job.json` and the brief and spawns nothing, the seam this verb's own tests
drive instead of a real `herdr`.

**Completion travels through a file mailbox, never a screen (D3).**
`.bee/mailbox/<job-id>/` holds `job.json`, round-numbered `result-N.json`,
and `log.txt`, every write staged tmp-then-rename. A result file's
appearance under its final name IS the done signal for that round — no
screen-scraping, one exact schema-checkable shape across all herdr-supported
kinds.

**The worker stays bee-ignorant; the orchestrator owns bee's own
bookkeeping (D4).** The dispatch prompt this verb writes into the brief is
fully self-contained — task, absolute paths, file constraints, the result
schema, the tmp-rename write gesture — so a worker that has never seen bee
can complete it. Everything bee-shaped that follows (`cells finish`, the
proof line, reservations, the dispatch-log row for the CALLER's bookkeeping)
is done by the orchestrator after it reads the result file back, never by
the worker. The one exception the verb itself owns: it appends the
`dispatch.jsonl` row and a wave-ledger `record-worker` row for every run it
starts (D9), so occupancy counts these workers too, mechanically, without
relying on the bee-ignorant worker to say anything back.

**Liveness is health-check based, native, at zero token cost (D5).** The
poll loop watches `result-N.json` presence, `log.txt` mtime, worktree diff
activity, and `herdr agent list` status — no LLM call anywhere on the wait
path. A stale heartbeat past `--idle-timeout` ends the wait; an absolute
`--ceiling` caps it regardless of activity, the busy-loop backstop for the
infinite fix-test-fix case a heartbeat alone would miss. There is no fixed
short wall-clock timeout — wall-clock cannot tell a long cell from a stuck
agent.

**Pane lifecycle mirrors the result, not the clock (D6).** A valid result
closes the pane (`herdr pane close`); a failure or a timed-out wait leaves
it open as forensics — a dead foreign agent's pane is the only remaining
trace. `--close-always` closes the pane on every outcome, overriding both.

**Spawn resilience — every delivery step verifies, none trusts a flag
(live-proven 2026-08-20, smokes 1-8 + the hee-1/hsr-1 dogfoods).** Four
hardenings, each one bought by a real failure:

- **Start retries a booting shell (herding-start-retry D1).** A freshly
  split pane's shell may not have reached its prompt when `agent start`
  fires; herdr refuses with `agent_pane_busy`. The verb retries the start
  up to 10 times about a second apart; any other start error still fails
  immediately, and exhaustion keeps the close-the-pane failure shape.
- **The brief never rides argv or the prompt channel raw.** A multi-line
  text cannot be encoded as a start argument, and at least one agent kind
  silently drops a multi-line injected prompt even when idle. The brief is
  written to `<mailbox>/brief-N.txt` and the agent receives a ONE-LINE
  pointer at that absolute path.
- **Readiness is observed, then delivery is verified — by a state change,
  never by pane text.** After start, the verb waits (up to 60s) for the
  agent to report ready; then it sends the pointer and counts it delivered
  only when the AGENT'S OWN STATE moves (working or done) or the round's
  result file appears — resending up to 30 times about a second apart,
  because herdr's ready flags can fire before the agent's input loop
  accepts text. The pointer is idempotent, so a duplicate delivery is
  harmless. Do NOT check whether the pane echoes the brief-file name: a
  booting pane echoes the keystrokes of the send itself, so that check
  passes exactly when delivery failed (two live smokes lost their brief
  this way). If the ready wait is exhausted, that is a typed spawn failure
  that KEEPS the pane for forensics — unlike a pre-start spawn failure,
  which closes it.
- **Operator rules.** Put each kind's auto-approve flag in its herd entry
  (`claude … --permission-mode bypassPermissions`, `agy
  --dangerously-skip-permissions`) — an agent that stops to ask permission
  mid-run stalls until the idle-timeout. And NEVER type into a live worker
  pane: a human keystroke takes the agent's turn and the brief goes
  unanswered (observed live; the run then idles out with the pane kept).

**This verb is cell-execution-only (D7) for its own manual invocation** —
`bee herding run`/`--continue`, called directly (scope A), always executes one
cell; it never runs a gather, review, or advisor request itself. The `cli`
tier kind stays the true mirror of that split at the SLOT level: a
`cli`-shaped model slot is gather/review/advisor-only and always refuses cell
execution (`gates-and-delegation.md`'s cli gather branch). The tier-slot
route onto a `{kind:"herding"}` slot no longer mirrors that split —
herding-review-slots D1 widens herding-tier D1-D5 so EVERY purpose against
that slot (cell, gather, reviewer, advisor, extraction) resolves to the same
`bee herding run` payload; see the config-route section below. The two kinds
now overlap everywhere except cli's own cell refusal: a `cli`-shaped slot
never serves cell execution, while a `herding`-shaped slot serves every
purpose, cell included.

**The config route now covers every purpose (herding-tier D1-D6, widened by
herding-review-slots D1):** `models.<runtime>.generation` (or any configurable slot) accepts
`{"kind": "herding"}` as a value — ANY purpose dispatched against that slot (cell, gather,
reviewer, advisor, extraction) resolves automatically to the `bee herding run` payload this
section describes, no per-purpose request needed. The old gather/review/advisor
default-model fallback is gone: every purpose now pays the same pane cost, and the operator
who sets `{"kind": "herding"}` on a slot owns that cost for every purpose the slot serves.
The agent that runs is the single global `herding.agent_command` above by default;
herd-registry D2 lets the tier value carry a per-slot override too — `{"kind": "herding",
"agent": "<name>"}` resolves that name through `herding.agents` instead ("`herding.agents` —
the named-agent registry" above), refusing on an unknown name.

**Optional `"fallback": "default"` degrades a failed run instead of leaving it loud
(herding-review-slots D3).** Add it to the same shape — `{"kind": "herding", "agent":
"<name>", "fallback": "default"}` — and a failed herding run (spawn failure, timeout,
invalid result) re-dispatches through the runtime's own default model path for that slot.
`"default"` is the only value the field accepts; any other value is dropped. The payload
`bee herding run`'s prepare step writes carries the fallback through so the caller can act
on it; the actual re-dispatch move belongs to the orchestrator's own doctrine, not this
verb. Absent the field — the default state — a failed run stays loud: the pane is kept open
as forensics (see "Pane lifecycle mirrors the result, not the clock" above) and nothing
re-dispatches automatically.

Manual, scope-A `bee herding run`/`--continue`
invocations — everything this reference otherwise describes — are
unchanged; the config route is one more way to reach the same verb, not a
replacement for it. Config shape and samples: `docs/config-reference.md`
(models section), `.bee/config-sample.json`,
`.bee/config-sample-cli-executors.json`.

## What actually contains this

Do not assume the loop "will not pick up hard-gate work" —
that was measured false: the lane-safety classifier
passed **8 of 8** real backlog rows in an adversarial review, including one
whose story was "delete the entire JS runtime," because it matches an English
keyword list against a row that judges work by its title, and most real rows
are not in English. Do not rely on the lane filter as a containment. What
actually contains this system, in descending order of load:

1. **The enable interlock (Dispatch role §5)** — dispatch builds nothing
   at all without the owner's `bee-herding.enable` marker. This is the gate
   that decides whether the loop does anything; everything else only matters
   once it is running.
2. **Merge is an owner gesture, not a loop** — nothing lands in main
   unattended. The single highest-authority action in the system requires a
   human present.
3. **Worktree isolation** — each working agent's edits are confined to its
   own worktree and branch until a merge (a git boundary, not a security
   sandbox — see "Permission posture" above).
4. **The four-slot cap and the stop file** — bound concurrency and give the
   human an off switch (which stops the *control* loop, not agents already
   running — see "Stop and resume" below).
5. **Key-2 reading (Dispatch role §6)** — the dispatcher's own reading of
   each row, refusing when unsure. Advisory, fail-closed on refusal, but not
   a mechanical guarantee.

The lane classifier script is one advisory input to item 5, not a
containment in its own right. Treat every `lane_safe:true` from it as "no
obvious English keyword hit," never as "safe."

## Stop and resume

`touch <main-root>/.bee/tmp/bee-herding.stop` stops the **control loop**
(dispatch) at the next iteration boundary — `bee herding control-loop` checks the file
both before and after every iteration, so a stop created mid-iteration takes
effect at that boundary rather than a full interval later. Removing the file
lets the loop be started again (it does not restart on its own — re-run
bootstrap).

**The stop file does not stop working agents already running.** Each working
agent is an independent `claude` session in its own runtime pane and
worktree; the stop file is never read by them. To stop those, close their
panes (`herdr pane close <pane_id>`) or open a pane and talk to the agent
directly. Its worktree survives either way (`bee worktree list` shows it).
Stopping the dispatch loop only guarantees no *new* agents are spawned — not
that in-flight ones halt.
