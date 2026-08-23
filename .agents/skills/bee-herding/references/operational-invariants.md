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

**On tmux the allowlist substitutes exactly ONE entry**
(tmux-herding-cockpit D1). `allowed_tools_for` takes the transport beside
the role: the herdr arms are the byte-identical strings that verb
documents, and the tmux arms are those same strings with `Bash(herdr:*)`
rewritten to `Bash(tmux:*)` — nothing else added, nothing else removed,
nothing else reordered. The surface stays enumerated and stays the same
width. A control pane driving tmux needs the tmux client for exactly the
pane work it used herdr for, and gains no other tool; with no
`herding.transport` key the control argv is the byte-identical pre-tmux
one, and a typo'd key is `transport_kind`'s typed refusal that stops the
loop rather than quietly arming the other multiplexer.

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

### Which multiplexer — `herding.transport` and `herding.tmux.*`

The adapter seam above says WHAT bee spawns. One more key says WHERE:
which terminal multiplexer bee reaches a worker pane through
(tmux-herding-transport D1-D4). That key now governs the WHOLE cockpit,
not the run verb alone (tmux-herding-cockpit D1) — see "The cockpit on
tmux" below.

- **`herding.transport`** — the string `"herdr"` or the string `"tmux"`.
  **Absent = `herdr`**, the unchanged default; a missing or unparseable
  `.bee/config.json` reads as `herdr` too. bee **never auto-detects** the
  transport from `$TMUX` or `$HERDR_ENV` — a session nested in both tools
  must not pick by accident (D1). Any other value — a typo, a number — is a
  typed refusal naming both legal spellings, and `bee herding run` refuses
  on it **before** the job file, the mailbox, or any pane split, so a
  typo'd transport can never half-start a worker.
- The transport a run picked is reported back: `bee herding status --json`
  gains `transport.kind`, and its readiness probe reads the pane variables
  of the CONFIGURED transport only (`HERDR_ENV` + `HERDR_PANE_ID` for
  herdr, `TMUX` + `TMUX_PANE` for tmux).
- **Binaries.** `herdr` on `PATH` is required for the herdr transport (the
  default); the tmux transport needs `tmux` on `PATH` instead. Neither is
  needed for the other.
- **What does NOT change on tmux.** Workers are panes split inside the
  CALLER's current tmux window, under the same one-column rule and the same
  cross-process split lock (D2) — never a detached session per worker. The
  mailbox, the control loop, the merge gesture and every safety boundary
  are the herdr ones, untouched — with exactly one recorded exception, the
  control pane's allowlist, which substitutes its single multiplexer entry
  and nothing else (see "Permission posture" above).
- **The screen read is advisory (D4).** tmux has no agent API, so worker
  status is a classifier over a bounded `capture-pane` read: content
  stability plus two marker lists. `result-N.json` and `ack-N.json` stay
  the ONLY truth for done and delivered. A pane showing a trust /
  permission / auth dialog ends the wait as `blocked`, the pane STAYS OPEN,
  and bee types nothing into it (D3) — a key sent into a dialog would
  answer it on the human's behalf.

The marker lists and the poll shape are config **data**, not code, because
marker strings are another tool's UI chrome and rot with its releases. All
five keys are optional and fail open — a malformed value leaves the default
in place (the one typed refusal is `herding.transport` itself). Defaults
come from upstream (D5: `https://github.com/luongnv89/skills` @
`ab46724e`, scope `skills/tmux-agent-comms/`):

| Key | Default | What it does |
|---|---|---|
| `herding.tmux.busy_markers` | `["esc to interrupt", "esc to cancel", "ctrl+c to interrupt", "press esc to"]` | Present in the last **2** non-empty screen lines → the agent is mid-turn. The narrow window keeps a stale mention scrolled up in the transcript from reading as "still working". |
| `herding.tmux.blocked_markers` | `["do you trust", "trust the files", "paste your api key", "press enter to submit"]` | Present in the last **12** non-empty lines → a human must answer a dialog (D3). A dialog is a multi-row box, so its marker can sit rows above the cursor. |
| `herding.tmux.scrollback` | `40` | Lines each `capture-pane -p -S -<n>` read pulls. |
| `herding.tmux.quiet_cycles` | `3` | Consecutive identical reads that count as a settled screen (clamped to at least 1 — zero would make every first read "settled"). |
| `herding.tmux.interval_ms` | `2000` | Delay between polls, in milliseconds. |

A list override **replaces** the default list, it never extends it: a repo
correcting a rotted marker needs the stale one GONE. The cost is that an
override restates the markers it still wants, and an explicit empty array
is a legal override meaning "never classify on this list".

```json
{
  "herding": {
    "transport": "tmux",
    "tmux": {
      "busy_markers": ["esc to interrupt"],
      "blocked_markers": ["do you trust", "paste your api key"],
      "scrollback": 40,
      "quiet_cycles": 3,
      "interval_ms": 2000
    }
  }
}
```

Implementation: `packages/bee-rs/crates/bee/src/herding/tmux.rs`
(`TmuxSettings::from_config`, the `PaneTransport` impl, and a re-export of
the shared classifier that now lives in `fleet::screen`), selected at one
construction site by `herding.rs`'s `transport_kind`.

### The cockpit on tmux — one key, one vocabulary

`herding.transport` picks the multiplexer for the WHOLE cockpit, not for
`bee herding run` alone (tmux-herding-cockpit D1). Occupancy, waves, the
control-pane allowlist, the bootstrap script, and the dispatch and merge
roles all read that same key, and **no new config key was added for any of
them** — a repo with no `herding` block behaves exactly as it did.

- **Occupancy** crosses the wave ledger's unresolved pane ids against
  `tmux list-panes -a -F '#{pane_id}'` instead of `herdr pane list`, under
  the same fail-closed contract: any trouble at all (no `tmux` on `PATH`,
  no server running, a non-zero exit) means "no live list available", and
  the ledger answers through its degraded `Occupancy::Fallback` path —
  never through the other multiplexer's pane list. A refused key resolves
  the same way.
- **Waves** build `fleet::backend::tmux::TmuxBackend`, a peer of
  `HerdrBackend`, at the one wave construction site (D1/D4).
- **ONE screen classifier serves both crates (D4).** It lives in
  `fleet::screen` — the two marker lists, both tail windows, and the
  stability knobs — and bee's `RealTmux` reuses exactly it. `bee` depends
  on `fleet` and never the reverse, so the shared half moved DOWN; two
  copies would drift the moment one crate's marker list was corrected and
  the other's was not. `fleet` still never reads `.bee/config.json`: bee's
  `TmuxSettings::from_config` resolves `herding.tmux.*` and hands a
  `ScreenSettings` over already decided.

**The roles speak only bee verbs (D2).** A cockpit role document, a wave
brief, and `bootstrap-cockpit.sh` act on panes ONLY through
transport-neutral bee verbs — never a raw `herdr` or `tmux` line:

```
bee herding pane current|list|split|run|send-text|read|rename|close|layout
                 |tab-create|tab-list|tab-focus
bee herding agent-start <job_id> --kind <kind> --pane <pane_id> -- <args…>
bee herding pane-id --label <label>
bee herding result <dotted.path>
```

Every verb prints exactly one envelope, identical in shape on both
transports — `{"ok":true,"transport":"herdr|tmux","result":{…}}` at exit 0,
`{"ok":false,"transport":…,"error":{"code":…,"message":…}}` at exit 1 — and
`bee herding result <dotted.path>` reads such an envelope on stdin and
prints one field of it. A cold control agent therefore learns ONE
vocabulary, whatever the key names. `pane list --with-status` is the only
branch that costs a call per pane: it fills a row the transport did not
already answer for itself, which on herdr is no row at all (its own
`pane list` body carries `agent_status` server-side) and on tmux is every
row, one bounded `capture-pane` each through the shared classifier. That
answer stays ADVISORY (transport D4) — the mailbox files remain the only
truth for done and delivered.

**The tmux mapping (D3).** tmux has no workspace object and no pane-label
object, so the cockpit's nouns land on carriers that survive a reattach:

| Cockpit noun | herdr | tmux |
|---|---|---|
| workspace | workspace | the caller's current session |
| tab | tab | a window (`cockpit`, `runtime`) |
| pane label | pane label | the pane TITLE (`select-pane -T`) |
| chat pane | the pane bootstrap ran from | the pane bootstrap ran from |

`bee herding pane-id --label <label>` looks that label up in `list-panes`'
`pane_title` on tmux and in the pane label on herdr; a miss is the typed
`not_found` at exit 1 on both, so a role branches on one shape.

**The pre-send guard fails OPEN on an unreadable screen (D5).** Before
`pane send-text` types anything on tmux, it captures the pane and refuses
when the shared classifier says `Blocked`: a dialog is answered by a human,
never by whatever character the text happens to begin with (transport D3,
unchanged). A capture that cannot be READ does not block the send — the
guard takes the same posture `RealTmux::agent_prompt` already takes for its
own preflight, because a transport hiccup must not silently stop the
cockpit from typing.

Implementation: `packages/bee-rs/crates/bee/src/herding/pane_verbs.rs`
(the `CockpitTransport` seam over phase 1's `PaneTransport`, both
production impls, and every verb above),
`packages/bee-rs/crates/fleet/src/backend/tmux.rs` (the wave backend), and
`packages/bee-rs/crates/fleet/src/screen.rs` (the one classifier).

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

An entry may also be an **object**, `{"argv": [...], "env": {...},
"workspace_trust": {...}}` — `argv` validated exactly like the plain array
shape; `env` (optional) is a per-agent environment map exported into the
freshly split pane before the agent starts; `workspace_trust` (optional,
herding-prompt-stall D5) is covered just below. A malformed object entry —
a bad `env` key, a bad `workspace_trust` shape — drops the WHOLE entry,
fail-open per entry, same as a malformed argv array.

**`workspace_trust` (herding-prompt-stall D5)** pre-seeds a foreign agent's
OWN per-workspace trust store so it never meets a first-time-workspace
trust dialog in a freshly minted `bee worktree new` directory — proven
live 2026-08-21: three concurrent `agy` runs into one fresh worktree all
sat at "Do you trust this folder?", and the herd entry's auto-approve flag
does not cover it (`agy --dangerously-skip-permissions` gates TOOL
permissions only; `agy --help` has no trust flag or subcommand). The
declaration names the file and the array key inside it, config-driven —
bee's source carries no hard-coded path for any specific tool:

```json
{
  "herding": {
    "agents": {
      "agy-flash": {
        "argv": ["agy", "--dangerously-skip-permissions"],
        "workspace_trust": {
          "file": "~/.gemini/antigravity-cli/settings.json",
          "key": "trustedWorkspaces"
        }
      }
    }
  }
}
```

`file`'s leading `~` is expanded to `$HOME` once, at config-parse time.
Before the pane split and `agent start` (`bee herding run`'s `execute_new`),
if the entry declares `workspace_trust`, bee reads `file`, and — unless the
run's absolute `--cwd` is already present in the array named by `key` — 
appends it and writes the file back atomically. This is FAIL-OPEN and
loud: a missing file, unparsable JSON, a missing or non-array `key`, or an
unwritable file all emit one warning line naming the file and what was
wrong, then let the run proceed unchanged — a foreign tool's config being
unreadable or unwritable must never fail a bee run. Nothing in that file is
ever rewritten beyond appending one absolute path to the named array.

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
- **Readiness defers to herdr's own lifecycle contract; delivery's receipt is
  the worker's own ack file, and herdr lifecycle state is a FAILURE detector
  only, never the success signal (herding-prompt-stall D1-D4).** The ready
  gate accepts `idle` OR `done` — not `idle` alone (herding-prompt-stall D2
  narrows herding-run-ready-wait D1, retiring the earlier idle-alone rule):
  `done` is the SAME underlying ready-for-input state for a pane nobody has
  focused, and since bee splits every worker pane with `--no-focus` and reads
  it only via CLI reads — which never mark a tab seen — `done` is the NORMAL
  resting state of a bee worker pane. Delivery no longer counts a pointer
  received by watching the agent's own state move to `working`
  (herding-prompt-stall D1 supersedes herding-pointer-delivery D1, retiring
  that receipt rule): a lifecycle sample taken inside the agent's boot
  window is not trustworthy — an agy pane flaps through
  unknown/working/idle/done while its TUI initializes, so a boot flap could
  have satisfied the old test and receipted a pointer the booting TUI had
  actually discarded. The send itself is still herdr's own atomic
  submit-and-observe, `herdr agent prompt <job> <text> --wait --until working
  --timeout <ms>`; herdr's `agent_prompt_stalled` is one of two things that
  can end the wait early as a typed failure — never as a success signal. The
  RECEIPT is the worker's own ack file, written as the brief's first
  instruction, or the round's result file for an ultra-fast round that
  finishes before an ack is ever observed (herding-prompt-stall D4). Once a
  send has gone out, an observed `working` status is the HEALTHY path — bee
  keeps polling for the ack, it never resends into a pane that is actively
  working; a resend fires only once the agent has returned to `idle` or
  `done` with still no ack, bounded by a fixed resend count and a separate
  wall-clock ack-wait budget. At every one of these wait points — the ready
  gate, pointer delivery, and the round poll — `blocked` ends the wait
  immediately as a typed, fast, loud failure naming the pane id, the tail of
  its text, and the remedy (herding-prompt-stall D3): this is how a
  per-workspace trust or approval prompt is covered without bee carrying an
  agent-specific pattern table. The pointer stays idempotent, so a duplicate
  send is harmless. Do NOT check whether the pane echoes the brief-file name:
  a booting pane echoes the keystrokes of the send itself, so that check
  passes exactly when delivery failed (two live smokes lost their brief this
  way). If the ready wait is exhausted without a `blocked` verdict, that is a
  typed spawn failure that KEEPS the pane for forensics — unlike a pre-start
  spawn failure, which closes it.
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
