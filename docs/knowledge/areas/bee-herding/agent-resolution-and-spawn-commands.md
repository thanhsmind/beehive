---
type: bee.area
title: "Bee Herding — which agent a pane runs as, and how its command is built"
description: "The config tier route that sends a whole purpose through a pane, the named-agent registry, the four-step precedence a bare run obeys, per-agent pane environment, and why bee keeps no list of agent kinds."
timestamp: 2026-08-20
bee:
  id: bee-herding-agent-resolution-and-spawn-commands
  lifecycle: active
  areas: [bee-herding]
  required_context: [areas/bee-herding/overview.md]
  decisions: ["herding-tier D1-D6 (the config tier route)", "herding-review-slots D1 (every purpose on a herding slot)", "herding-review-slots D3 (optional per-slot fallback to default)", "herd-registry D1-D2 (the named-agent registry)", "herding-bare-agent D1-D5 (four-step bare-run agent resolution order: --agent > tier slot > string agent_command > array fallback)", "defaults-and-agent-env D3 (built-in claude-sonnet and agy-flash registry entries)", "defaults-and-agent-env D4 (registry entry carries validated pane env)", "herding-orchestration D12 (starting a worker is two acts)", "herding-orchestration D14 (command tokens are never re-joined into a shell string)", i54-closeout D4, "herding-prompt-stall D1 (retires herding-pointer-delivery D1's hand-rolled receipt)", "herding-prompt-stall D2 (narrows herding-run-ready-wait D1 — done counts as ready)", "herding-prompt-stall D3 (blocked is a fast, loud failure at every wait point)", "herding-prompt-stall D5 (corrects D3's reach: blocked does not cover a trust dialog; a herd entry may declare and pre-seed the foreign tool's own trust store instead)"]
  sources: [docs/history/herding-bare-agent/CONTEXT.md, docs/history/herd-registry/CONTEXT.md, docs/history/defaults-and-agent-env/CONTEXT.md, docs/history/herding-tier/CONTEXT.md, docs/history/herding-prompt-stall/CONTEXT.md, "herding-review-slots, herd-registry, herding-tier and defaults-and-agent-env promote proposals (reviewed 2026-08-20)"]
  authoritative_for: "bee-herding: agent resolution, the named-agent registry, and spawn-command construction"
---

# Bee Herding — which agent a pane runs as, and how its command is built

A herded pane runs some external coding agent. Two questions decide which one
and with what command: the ROUTE that sends work to a pane at all, and the
RESOLUTION that names the agent once the work is going there.

## The route: a configured slot sends a whole purpose through a pane

Setting `{"kind": "herding"}` on a `models.<runtime>.generation` slot (or any
configurable slot) routes EVERY purpose dispatched against it — cell, gather,
reviewer, advisor, extraction — through `bee herding run` automatically, with no
per-purpose request needed. The old gather/review/advisor default-model fallback
is gone, so the operator who sets the slot owns the pane cost for every purpose
it serves (herding-tier D1-D6, widened by herding-review-slots D1).

An optional `"fallback": "default"` on the same shape lets a failed herding run
(spawn failure, timeout, invalid result) re-dispatch through the runtime's own
default model path for that slot instead. Absent the field, a failed run stays
loud and keeps its pane open as forensics (herding-review-slots D3).

Widening the slot to every purpose needed no change in the model guard, because
the guard routes on the slot's KIND alone and never on which purpose asked — a
new purpose inherits the routing for free, and no guard rule has to learn its
name. The routing is enforced, not merely offered: against a herding-kind slot
the guard DENIES a native subagent dispatch outright, so the pane payload is the
only legal way through and no caller can quietly take the cheaper in-process
path the operator opted out of.

## The resolution: four steps, in strict order

The working-agent and control-pane spawn commands are config-driven templates,
byte-equivalent to the hardcoded default (i54-closeout D4). `bee herding
control-loop` reads an optional `.bee/config.json` `herding.control_command` — a
JSON array of argv-token strings — and, when present, substitutes `{PROMPT}` /
`{MODEL}` / `{MAX_TURNS}` / `{ALLOWED_TOOLS}` per token and runs the result
verbatim: tokens are never joined into one string and re-split or shell-`eval`'d,
so a config-supplied command cannot smuggle shell injection through a
placeholder value (herding-orchestration D14). The working agent's spawn tail has
the matching `herding.agent_command` seam.

`.bee/config.json`'s `herding.agents` (herd-registry D1/D2) names several agents
once — a map of name → argv tokens with the same validation. When picking which
external agent a herded pane runs as (`bee herding run` and `bee herding wave`
use the same resolver, herding-bare-agent D1-D5), resolution follows a strict
four-step precedence:

1. An explicit `--agent <name>`, resolved through `herding.agents`;
2. The cell-execution tier slot `models.<runtime>.generation`, but ONLY when it
   is an object with `kind: "herding"` and a non-empty `agent` string — that
   name resolves through `herding.agents`. This is the configured role-to-agent
   mapping that a bare `bee herding run` obeys;
3. `herding.agent_command` as a plain string, resolved through `herding.agents`;
4. `herding.agent_command` as an array (token 0 is the herdr `--kind`, rest
   are args), or the built-in default array when absent or malformed.

Any other slot shape (`kind: "herding"` with no agent, a plain model name like
`"sonnet"`, `{"kind":"cli",...}`, null, absent) is skipped and falls through.
A slot naming an agent not declared in `herding.agents` fails closed with a typed
`UnknownAgent` error listing every known key — never a silent fallback.
`<runtime>` resolves to `BEE_RUNTIME` when it names `claude`, `codex`, or
`opencode`, defaulting to `claude`.

When the key is absent, invalid, or empty, the command built is byte-equivalent
to the pre-existing hardcoded `claude -p ... --model sonnet --max-turns ...
--allowedTools ...` invocation — a project with no config change sees no
behavior change at all. A codex adapter example is documented purely as an
illustration of the seam; full codex-native herding (its own event loop and pane
protocol) stays out of scope (i54-closeout D4). None of enable/disable/status,
the dispatch interlock, or the merge owner-gesture change.

## bee keeps no list of agent kinds

The kind token passes straight through, and the pane manager is the one that
accepts or rejects it. A kind the pane manager learns tomorrow works here today
with no change on this side — the alternative, a second list to maintain, only
ever produces a refusal for something that would have worked. A herd name always
means the pane transport; the `cli` tier kind is unrelated.

## Built-in entries and per-agent pane environment

Since defaults-and-agent-env D3 (2026-08-20) the registry starts from two
BUILT-IN entries — `claude-sonnet` and `agy-flash` — so those names resolve on a
repo with no herding block at all. A same-name config entry overrides its
built-in, and the unknown-name listing includes the built-ins.

defaults-and-agent-env D4 (same day) adds a second entry shape beside the argv
array: `{"argv": [...], "env": {"KEY": "value"}}`. The env map is exported into
the freshly split pane as one `export K='v'` line BEFORE `agent start` (keys
`[A-Za-z_][A-Za-z0-9_]*`, values newline-free; any violation drops that entry
only — the registry's standing fail-open-per-entry rule), and a failed env send
is a typed spawn failure that closes the pane.

Only the `bee herding run` spawn path applies env; the wave/control-loop caller
resolves it but cannot apply it (its `agent start` lives in
`fleet::backend::herdr`, another crate — noted at the call site).

## A herd entry may declare the foreign tool's own trust store

A herd agent may gate on a per-workspace trust question that its own
auto-approve flag never reaches — an agent's "skip tool permissions" flag
skips tool permissions, not workspace trust, so the question surfaces anyway.
bee mints a fresh worktree directory for every feature, so a first run into
that directory meets the question every single time, not just once.

A `herding.agents` object-shape entry may therefore carry an optional
workspace-trust declaration: which file holds the foreign tool's own
trust-store data, and which field inside it names the array of trusted
absolute paths. Before the pane is created — the same moment the entry's
pane environment is exported — bee seeds the current working directory into
that array, so the question the dialog would have asked is already answered
by the time the agent starts. bee carries no knowledge of what the file or
its contents mean beyond that one array; the declaration names the shape,
nothing more.

The pre-flight FAILS OPEN and LOUD on every error: the file missing or
unreadable, its contents not valid data, the declared field missing or not
an array, or the rewritten file failing to write back all produce a warning
naming the file and what went wrong, and the run proceeds regardless — a
foreign tool's own config being unreadable or unwritable must never fail a
bee run, but the warning means the gap is never silent either
(herding-prompt-stall D5).

## Edge Cases Settled

- **Starting a worker is two acts, not one.** The pane is created first, and the
  agent is started INTO that pane; a single call that both creates and starts no
  longer exists (herding-orchestration D12). What the agent itself is — which
  runtime, and the arguments it gets — is configuration, read as separate tokens
  and never re-joined into a string a shell could reinterpret
  (herding-orchestration D14).
- **A worker's agent name is derived from its pane, and the multiplexer will not
  take it raw.** Panes are numbered 1 to 9 and then A, B, C…, so most panes in a
  busy workspace carry an uppercase letter — and an agent name may only be
  lowercase letters, digits, dash and underscore, must begin with a lowercase
  letter, and may not exceed 32 characters. The derived name is therefore made
  legal by construction before it is used; the cost is that two panes whose ids
  differ only by case would collapse onto one name, which is accepted because no
  such pair exists. This was found by the first live run, not by any test: before
  the repair, every pane with an uppercase letter was refused and the whole wave
  aborted before sending anything.

## After spawn: herdr's agent lifecycle contract

Resolving and spawning the right agent is only half the seam; bee also has
to read that agent back honestly once it is running. From `herdr --skill`,
verbatim (quoted, not paraphrased — herding-prompt-stall D1-D3):

> `idle` means the agent is ready for input and its tab has been seen in the
> focused Herdr UI. `done` is the same underlying idle state after unseen
> background work finishes. Focusing the tab or targeting the pane or agent
> with a focus command marks it seen. CLI reads do not mark it seen.
> `blocked` means Herdr recognized an approval or question UI. `unknown`
> means an agent is present but Herdr cannot classify it confidently.

Four states in that quote, and what each means to bee:

- **`idle`** — ready for input AND the tab has been seen in the focused
  Herdr UI.
- **`done`** — the SAME underlying ready state, for a tab nobody has looked
  at. bee splits every worker pane with `--no-focus` and reads it only via
  CLI reads — which never mark a tab seen — so `done`, not `idle`, is the
  NORMAL resting state of a bee worker pane (herding-prompt-stall D2 narrows
  herding-run-ready-wait D1: the ready gate accepts `idle` OR `done`, not
  `idle` alone).
- **`blocked`** — Herdr recognized an approval or question UI. This is
  bee's fast, loud failure at every wait point — the ready gate, pointer
  delivery, and the round poll (herding-prompt-stall D3). A blocked pane
  ends the wait immediately with a typed error naming the pane id, the tail
  of its text, and the remedy. `blocked` does NOT reliably cover a
  per-workspace trust dialog, though — that reach was retired
  (herding-prompt-stall D5, corrects D3): proven live, three concurrent
  runs into a genuinely untrusted workspace all sat at a trust dialog while
  Herdr reported the agent `idle`, never `blocked`. A trust dialog is
  covered two other ways instead: the declared trust-store pre-flight
  above, and a give-up diagnosis that reads the pane for a confirmation cue
  once a wait has already failed (`handing-a-foreign-agent-its-brief.md`).
- **`unknown`** — an agent is present but Herdr cannot classify it
  confidently. `unknown` does not prove completion.

`working` is not part of that quoted enumeration — herdr's own contract
mentions it only in passing, as "a non-working state" in the definition of
`agent_prompt_stalled` (a submission from a non-working state that produces
no observed lifecycle change within the timeout window). bee treats an
observed `working` status as the HEALTHY in-progress path once a pointer has
been sent: it is polled, never resent (herding-prompt-stall D4).

**A sample taken inside the agent's boot window is not trustworthy.** An
agy pane flaps through several of these states — unknown, working, idle,
done — while its TUI initializes. bee's earlier hand-rolled poll sampled
right after `agent start`, in that window, so a boot flap could satisfy its
old transition test and receipt a pointer the booting TUI had actually
discarded (herding-prompt-stall D1, supersedes herding-pointer-delivery
D1). bee now defers to herdr's own settle-aware verbs instead of polling
raw samples: `herdr agent prompt <job> <text> --wait --until working
--timeout <ms>` for delivery, and `herdr agent wait <job> --until idle
--until done --timeout <ms>` for the ready gate. herdr's own
`agent_prompt_stalled` is bee's delivery signal instead of an inference from a
sampled state — but it is a RETRYABLE one, not an immediate failure
(herding-prompt-stall D6, narrowing D1); see
`handing-a-foreign-agent-its-brief.md` for the bounded retry it feeds.

## Open Gaps

- **The dependency on the multiplexer's JSON shapes is still unpinned** — there
  is no capability or version probe anywhere on the path. What changed is the
  failure DIRECTION, not the gap: an unrecognised status string now maps to
  unverifiable, and a live-pane list that cannot be read now returns the tagged
  fallback that makes dispatch refuse. So an upstream shape change degrades to a
  loud refusal rather than to a silent stall — but it is still not detected, and
  nothing names the version this cockpit was proven against.

## Pointers (implementation)

- Resolution and the registry live with the `herding` command group in
  `packages/bee-rs/crates/bee/src/herding.rs`; the spawn path that applies env is
  `packages/bee-rs/crates/bee/src/herding/run.rs`.
- The trust-store declaration parses as `herding.agents`' optional
  `workspace_trust` field (`{"file": ..., "key": ...}`,
  `wave::parse_workspace_trust`); the pre-flight that seeds it is
  `run::preflight_workspace_trust`, fail-open by returning a `Warning`
  variant the caller logs and proceeds past (hps-8, herding-prompt-stall
  D5).
- The wave caller that resolves env without applying it reaches
  `fleet::backend::herdr` in `packages/bee-rs/crates/fleet/src/backend/herdr.rs`.
- Operating detail for operators:
  `skills/bee-herding/references/operational-invariants.md`.
