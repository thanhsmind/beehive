# Glossary

The vocabulary used across these documents. When a document uses one of these words, it means exactly this.

## The actors

**The agent.** The LLM session that uses bee: it runs every `bee` command, edits files, and dispatches subagents. In these documents "the agent" is the user of the product. Commands belong to the agent; approvals belong to *the human*.

**The human.** The person the agent works for. The human approves gates, answers decision questions, and approves privacy releases. The human never runs bee commands in the intended flow; a command printed for the human to run is a defect in the flow, not a feature of it.

**Orchestrator.** The interactive session that routes work: it claims cells, dispatches workers, talks to the human, and writes state. One checkout has at most one write-capable orchestrator at a time in `isolated` write-policy mode.

**Worker.** A dispatched subagent that executes exactly one assigned unit of work and reports one status token. A worker inherits the operating-system working directory of the session that spawned it.

**Sibling session.** Another live bee session in the same repository — in the main checkout or another worktree — visible through the store: its session record, heartbeat, claims, and holds.

## The surfaces

**Host repo.** A repository onboarded with bee: it has `.bee/` (the store, the binary, config), hooks wired into the runtime, and the bee skills available. These documents describe a freshly onboarded host repo with default configuration.

**Main checkout.** The primary working copy of the host repo. Integration and release work happens here; feature work happens in a *feature worktree*.

**Feature worktree.** A git worktree created by `bee worktree new --feature <slug>`, holding one feature's code changes until `bee worktree merge` lands them in main.

**Granted worktree.** A worktree whose control-plane reads are redirected to the main checkout's store, so a session inside it sees the same claims, holds, and state as everyone else.

**Staging.** A disposable integration ground between feature worktrees and main, built by `bee staging add`, where the human tests before the UAT gate.

## The store

**The store.** Everything bee remembers on disk under `.bee/`: `state.json`, lane files, cells, decisions, reservations, sessions, workflow records, config, logs. The store is written only through the CLI; hand-editing it is denied by the *direct-edit guard*.

**Workflow record.** One durable unit of pipeline state per feature attempt, under `.bee/runtime/workflows/`. `state.json` and the lane files are rebuildable projections of it.

**Cell.** One bounded unit of executable work with an id, a state, and a budget, stored under `.bee/cells/`. A cell is claimed, executed, and capped; its cap carries the proof line.

**Lane.** The size-and-risk class of a piece of work — `tiny`, `small`, `standard`, `high-risk`, `spike` — which scales ceremony (workers, gates wording, knowledge budget), never memory.

**Decision.** A logged agreement with an id and a required relation (`supersedes`, `touches`, or `none`), appended to `.bee/decisions.jsonl` and rendered under `docs/decisions/`. Locked product decisions live in `docs/history/<feature>/CONTEXT.md`.

**Capture stub.** A one-line note filed by `bee capture add` the moment something settles, queued until a capture session merges it into the knowledge bundle.

**Knowledge bundle.** The recorded state layer under `docs/knowledge/`: areas, patterns, and work notes, with an index. Agents read it before code.

## Phases and gates

**Phase.** Where a feature attempt stands: `idle` (no active work), the gated phases `exploring` and `planning` (execution not yet approved), execution-approved, and the terminal states. The phase decides what the *write guard* allows.

**Gate.** A moment where the human approves before the agent proceeds. Five are recorded — `context`, `shape`, `execution`, `review`, `uat` — all false by default. The load-bearing ones in the flow: Gate 1 (the decisions), Gate 2 (shape and execution together — the door to editing source), Gate 3 (UAT — the door to main).

**Gate bypass.** An opt-in level in config — `off`, `normal`, `full`, `total` — that lets `bee state gate` self-approve specific gates with the actor recorded as `auto`. It never patches the hooks and never bypasses the UAT gate.

**Idle intake gate.** The write-guard rule at idle or a terminal phase: source writes outside `.bee/`, `docs/`, `plans/`, and `AGENTS.md` are refused until work is routed through the workflow.

## The invocation

**Invocation.** One run of the `bee` binary, or one guarded action a hook intercepts. The unit of interaction in every document; its phases are *invoke*, *ends at once*, *first side effect*, *while running*, *finish*.

**Porcelain.** The command set shown by plain `bee --help` — the flow surface. Everything else is *plumbing*: still callable, listed by `bee --help --all`.

**Refusal.** The binary answering an invocation it will not serve, with a fixed wording contract: `bee: unknown command`, `bee: not built into this binary`, `bee: unexpected positional argument`, `bee: missing required argument`, `bee: unsupported argument shape`. Every argv shape gets an answer; silence is a defect.

**Deny.** A hook stopping a tool call before it runs. A deny names its remedy — the CLI verb to use, the path that is allowed, or the gate to route through.

**Remedy.** The fix a deny names. Following the remedy is the sanctioned way past a deny; working around it is not.

**Fail-open.** A hook that cannot decide exits 0 with a stderr line and lets the action pass. bee's hooks fail open so a broken harness never silences the agent; the guards are a safety net, never the authority.

**Fail-closed.** The opposite, reserved for corrupt coordination state: a hold or reservation store that cannot be read denies the write rather than guessing.

**Proof line.** The evidence recorded on a cap: `<command> — <result> — <scope reason>`. `bee close` and `bee worktree merge` check that it exists; they run nothing themselves.

## Events that end or interrupt

**Killed.** The invocation's process ends early — Ctrl+C, a kill signal, the terminal closing. What is on disk afterward depends on whether the *first side effect* had happened; the store lock has a stale timeout so a killed holder does not wedge siblings.

**Compaction.** The session's context is summarized mid-conversation. A compacted session re-reads its footing from the compact capsule; it never auto-adopts a handoff.

**Handoff.** A written record (`planned-next` or `pause`) that carries work across a session boundary. `planned-next` is adopted by a fresh session; `pause` waits for the human's word.

**Heartbeat expiry.** A session record goes stale after 900 seconds without a touch; its waiting marks expire and siblings stop counting it as live.

**Lease expiry.** A claim or reservation outlives its holder: the lease TTL is 3600 seconds, after which it can be swept or re-claimed.

**Sibling change.** Another session takes the claim, places a hold or reservation, or merges a worktree underneath the current flow. The affected invocation is refused or denied with the sibling named; the response is to take disjoint work and report, never to write through.

**Channel change.** The same invocation in a different mouth: output piped or `--json`, the Codex runtime instead of Claude, or the command run from inside a hook. It changes what is printed and which events exist, never what is true in the store.

## Output

**Preamble.** The state summary injected at session start (and, after compaction, the compact capsule): version, phase, gates, standard commands, project map, pending queues. It is already read; re-fetching it is waste.

**Progress line.** The one-line-per-step narration contract: `▸` started, `✓` green, `⚡` auto-approved, `✗` red. A red or refusal line is never silenced.

**Timing line.** The stderr trailer every direct run prints — `[bee] <cmd> <N>ms` — mirrored into `.bee/logs/timings.jsonl` as `{ts, cmd, ms, ok}`.

**`--json`.** The machine mode nearly every command takes. With it, the payload — including errors, as `{"error": msg}` — goes to stdout; without it, the human message goes to stderr.

**Waiting-on mark.** The record that a turn ended waiting on the human — kind `gate`, `question`, or `turn-end` — set so a dashboard or sibling reads "waiting on you" instead of "idle". The agent sets `gate` and `question`; the Stop hook sets `turn-end`; the human's next message clears it.

**Privacy marker.** The `@@BEE_PRIVACY@@ … @@END@@` block a secret-guard deny emits. It is routed to the human, never acted on by the agent.
