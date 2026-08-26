# Glossary

The vocabulary used across these documents. When a document uses one of these words, it means exactly this.

## The actors

**The agent.** The LLM session that uses bee: it runs every `bee` command, edits files, and dispatches subagents. In these documents "the agent" is the user of the product. Commands belong to the agent; approvals belong to *the human*.

**The human.** The person the agent works for. The human approves gates, answers decision questions, and approves privacy releases. The human never runs bee commands in the intended flow; a command printed for the human to run is a defect in the flow, not a feature of it.

**Orchestrator.** The interactive session that routes work: it claims cells, dispatches workers, talks to the human, and writes state. One checkout has at most one write-capable orchestrator at a time in `isolated` write-policy mode.

**Worker.** A dispatched subagent that executes exactly one assigned unit of work and reports one *status token*. A worker inherits the operating-system working directory of the session that spawned it.

**Sibling session.** Another live bee session in the same repository — in the main checkout or another worktree — visible through the store: its session record, heartbeat, claims, and holds.

## The surfaces

**Host repo.** A repository onboarded with bee: it has `.bee/` (the store, the binary, config), hooks wired into the runtime, and the bee skills available. These documents describe a freshly onboarded host repo with default configuration.

**Main checkout.** The primary working copy of the host repo. Integration and release work happens here; feature work happens in a *feature worktree*.

**Feature worktree.** A git worktree created by `bee worktree new --feature <slug>` — a sibling directory `<repo>--wt--<feature>` on branch `wt/<feature>` — holding one feature's code changes until `bee worktree merge` lands them in main.

**Granted worktree.** A worktree registered in main's grant ledger, with the split store: its control plane (sessions, claims, workers, lanes, handoffs) stays in main's `.bee/`, its data plane (its own decisions, its granted feature's cells) is local. Control-plane commands refuse inside it and name main as the place to run them.

**Control root.** The checkout whose `.bee/` answers for shared coordination state in the current invocation — its own root in an ordinary checkout, the main checkout's root from inside a worktree. Claims, sessions, and lease files live there; a worktree's data plane stays local.

**Staging.** A disposable integration ground between feature worktrees and main — worktree `<repo>--wt--staging`, branch `staging`, opt-in via config — where the human tests before the UAT gate. Rebuilt from main at will; its history never lands anywhere.

**Workspace.** A checkout as the write policy sees it, recorded in the store. It has at most one write-owner session in `isolated` mode; later live sessions attach read-only or isolate into their own worktree.

**Skill.** One `SKILL.md` with a frontmatter name and trigger description, a short body, and references loaded on demand. Twelve ship with bee. Skills plus `AGENTS.md` are *the instruction layer*: text that tells the agent what to run and say, as opposed to the CLI and hooks — the only layer that changes state or stops an action.

## The store

**The store.** Everything bee remembers on disk under `.bee/`: `state.json`, lane files, cells, decisions, reservations, sessions, workflow records, config, logs. The store is written only through the CLI; hand-editing a CLI-owned file is denied by the *direct-edit guard*.

**Workflow record.** One durable unit of pipeline state per feature attempt, under `.bee/runtime/workflows/`. `state.json` and the lane files are rebuildable projections of it.

**Cell.** One bounded unit of executable work: a JSON record with an id, feature, title, action, verify command, lane, and role. Its states are `open`, `claimed`, `capped`, `blocked`, `dropped`; archiving is a file move, not a state. A cell is claimed, executed, and capped; its cap carries the *proof line*.

**Lane.** The size-and-risk class of a piece of work — the route vocabulary is `docs`, `tiny`, `small`, `spike`, `standard`, `high-risk` — which scales ceremony (workers, gates wording, knowledge budget), never memory.

**Decision.** A logged agreement with an id and a required *relation*, appended to `.bee/decisions.jsonl` and rendered under `docs/decisions/`. The *active set* is derived on every read: decide and supersede events, minus superseded ids, minus redacted ids, with the retro-tag overlay applied.

**Relation.** The declaration every `bee decisions log` call must carry about what the new decision does to what is already active: `supersedes:<id>` retires the named decisions, `touches:<id>` names related ones that stay active, `none` relates to nothing. A missing relation refuses the write.

**Locked decision.** A decision frozen in `docs/history/<feature>/CONTEXT.md` as `<feature> D<n>`. The log is where it was agreed; CONTEXT.md is where it is cited from and never reinterpreted.

**Capture stub.** A one-line note filed by `bee capture add` the moment something settles, queued in `.bee/capture-queue.jsonl` until a capture session merges it into the knowledge bundle and marks it flushed. Pending = stub events minus flush events.

**Knowledge bundle.** The recorded state layer under `docs/knowledge/`: *areas* (durable subjects, each with an authoritative `overview.md` and an ownership map), *concepts* (typed OKF files), and *critical patterns* (lessons marked universal, ranked into every context manifest). Agents read it before code. The legacy `docs/specs/` tree is its read-only *compatibility surface*.

**Context manifest.** The ordered reading list `bee knowledge context` returns — paths, sizes, and inclusion reasons, never content — cut at the lane's token budget with the top critical patterns reserved as a floor.

**Promote proposal.** The document `bee close` files at `docs/history/<feature>/promote-proposals.md`: candidate bundle additions mined from the feature's traces. Applying or declining it is a recorded decision; the debt shows in the preamble until one is.

**PBI (product backlog item).** One numbered backlog item — `p-` plus eight hex characters, a story, conditions of satisfaction, a status — computed by folding the `kind:"pbi"` events in `.bee/backlog.jsonl`. `docs/backlog.md` is its *generated view*: recomputed, byte-stable, CLI-owned.

**Generated view.** A file that carries no truth of its own — recomputed from a store record, byte-identical for the same input. A hand edit is denied, and the deny names both the data verb and the refresh verb.

**Feedback digest.** The privacy-safe snapshot of a repo's own friction at `.bee/feedback-digest.json`: entries of exactly six fields (`kind`, `layer`, `source`, `title`, `first_seen`, `pain`) and no free text. Regenerated whole, never appended.

**Letter.** One filed record covering exactly one unattended run: a single markdown file under `.bee/human-mailbox/` with typed frontmatter and a prose body for the human. Built only from *entries* the run appended at its clean stops — the end-of-run pass is a renderer, never a summarizer (the authorship ban).

**Scratch.** Ephemeral files bee writes for its own working purposes — probe scripts, judge payloads, digests. They live under `.bee/tmp/` (or `.bee/spikes/` for feasibility proofs) and nowhere else; `bee tmp sweep` removes them.

**Discovery map.** One fog-state effort's index at `docs/discovery/<effort>/MAP.md`, with its open questions as *tickets* (one file each). The *frontier* is the tickets a session may take right now: open, unclaimed, every `blocked-by` closed. A non-empty frontier is what makes orient recommend resuming the map.

**Trigger.** A durable record of a deferred condition under `.bee/triggers/`, so "revisit when X lands" cannot sink into prose. `bee decisions log` refuses deferral-shaped text that names no registered trigger.

**Review session.** One user-invoked independent-review pass at `.bee/reviews/<id>.json`: a frozen scope (baseline, head, included, excluded — immutable after create), findings with severities (`P1` blocks approval), user-acceptance items, and one decision. Review status is derived at read time against real git history; a commit landing after the session head makes the coverage `review stale`.

## Coordination

**Claim.** Leased ownership of one cell: a claim file published by exclusive create, TTL 3600 seconds, renewed by the owning session's heartbeat. A dead session's claim is swept — cell parked `blocked` with the dead session named — once TTL and heartbeat both lapse.

**Reservation.** One agent's claim on one path, keyed `(agent, cell)`, taken with `bee reservations reserve`. Truth lives in per-path *lease files* under the control root; `.bee/reservations.json` is a projection. Default kind `lease` hard-blocks overlapping writes by others; kind `intent` is a planning-time declaration that warns instead — except on its exact declared path.

**Hold.** A reservation mirrored into main's shared ledger so other checkouts can see it. Hard on an *exclusive path* (lockfiles, migrations, and the configured list), advisory otherwise.

**Budget.** A cell's attempt limits — max claims, failed attempts, same-signature failures (default 3/4/2, hard max 9/12/6). Exhaustion closes the claim door until an audited `reset-budget` with a named operator and reason.

## Phases and gates

**Phase.** Where a feature attempt stands: `idle`, the gated phases `exploring` and `planning`, `swarming`, `reviewing`, `scribing`, `compounding`, `grooming`, `compounding-complete`. `idle` and `compounding-complete` are terminal; an unrecognized phase refuses writes. The phase decides what the *write guard* allows.

**Gate.** A moment where the human approves before the agent proceeds. Five are recorded — `context`, `shape`, `execution`, `review`, `uat` — all false by default. The three doors in practice: Gate 1 (the decisions), Gate 2 (shape and execution merged — the door to editing source), Gate 3 (UAT — the door to main). Every approval stamps actor, time, reason, and bypass level on the workflow record.

**Gate bypass.** An opt-in config level — `off`, `normal`, `full`, `total` — that lets `bee state gate` self-approve specific gates with the actor recorded as `auto`. `normal` covers Gates 1–2 for tiny/small/standard lanes. It never patches a hook, and the UAT gate refuses `--actor auto` at every level.

**Idle intake gate.** The write-guard rule at idle or a terminal phase: source writes outside `.bee/`, `docs/`, `plans/`, and `AGENTS.md` are refused until work is routed through the workflow.

**Intent anchor.** The on-disk pin of the objective, written by `bee intent set` (`bee shape` is its alias): the human's verbatim request and the definition of done, immutable once set. It outlives compaction; workflow state serves it, never replaces it.

## The invocation

**Invocation.** One run of the `bee` binary, or one guarded action a hook intercepts. The unit of interaction in every document; its phases are *invoke*, *ends at once*, *first side effect*, *while running*, *finish*.

**Porcelain.** The command set shown by plain `bee --help` — the flow surface. Everything else is *plumbing*: still callable, listed by `bee --help --all`.

**Refusal.** The binary answering an invocation it will not serve, with a fixed wording contract: `bee: unknown command`, `bee: not built into this binary`, `bee: unexpected positional argument`, `bee: missing required argument`, `bee: unsupported argument shape`. Every argv shape gets an answer; silence is a defect.

**Unbuilt verb.** A command the registry advertises — full schema, example, listed in help — that the binary refuses with `bee: not built into this binary`. A served refusal, not an unknown command. The current set: the `config` group, the `perf` group, `recovery window`, `herding enable`/`disable`, and the `state compact-*` verbs.

**Deny.** A hook stopping a tool call before it runs, exit code 2. A deny names its *remedy* — the CLI verb to use, the path that is allowed, or the gate to route through. Following the remedy is the sanctioned way past a deny; working around it is not.

**Fail-open.** A hook that cannot decide exits 0 with a stderr line and lets the action pass — *undecidable* is visible, never silent, and never permission. bee's hooks fail open so a broken harness never silences the agent; the guards are a safety net, never the authority.

**Fail-closed.** The opposite, reserved for corrupt coordination state: a hold, reservation, workspace, or lane store that cannot be read denies the write rather than guessing.

**Repair.** The model guard's third answer, between allow and deny: it rewrites one field of a dispatch to what configuration says, lets the call proceed, and announces the rewrite to both the agent and the human. A repair carries no permission decision.

**Proof line.** The evidence recorded on a cap: `<command> — <result> — <scope reason>`, with a literal ` — ` separator. A `red` result refuses the cap. `bee close` and `bee worktree merge` check the recorded line; they run nothing themselves.

**Declared test command.** The repo's one test path, `commands.test` in config. `bee test` runs it and records the result; CI runs it on every push — the deterministic net. `commands.test: "none"` is the *no-test sentinel*, the one way a repo declares itself deliberately test-free.

**Red base.** A repo whose last recorded test run was red. `bee cells claim` refuses on it by name unless `--fix-first <reason>` records the fix-first intent on the claim.

## Delegation

**Dispatch envelope.** The JSON object `bee dispatch prepare` returns and the agent executes verbatim: `tool`, `payload` (including the rendered worker prompt), `dispatch_id`, and the *economics record*. The envelope is the whole product of the dispatch door; the agent never composes or edits one.

**Role.** The name of the job a dispatch is, used to select its model. Roles are an open set — any key `models.<runtime>` configures is legal; one nothing configures is refused by name, never silently resolved onto another model.

**Rendered agent.** One of the four subagent definitions bee writes into a host — `bee-gather`, `bee-extract`, `bee-build`, `bee-review` — each generated from a role's configured model and carrying its contract as its system prompt. Naming one in a dispatch is a role declaration.

**Status token.** The single word a worker returns at the head of its final message — `[DONE]`, `[BLOCKED]`, `[HANDOFF]`, or `[NOOP]` — with its result fields beside it. A `[DONE]` is goal-checked, not believed.

**Wave.** The set of cells the *schedule* places first — `bee cells schedule` orders open cells into dependency-and-file-overlap waves — claimed and prepared in one `bee dispatch wave` call, one feature per wave, forgiving per cell.

**Judge verdict.** The goal-check's recorded answer on a cell: `PASS` or `NEEDS_REVISION`, stored by `bee cells judge-record`; a standing `NEEDS_REVISION` blocks the cap unless overridden loudly. The judge tier scales with the lane — mechanical checks for tiny/small, a checklist judge for standard, model-independence preferred for high-risk. Distinct from the *doctor verdict* and from a review finding.

**Nudge.** A hook-injected reminder that proposes and never writes: the capture-queue nudge at Stop, the mid-phase hive-door warning, the intent-anchor nudge on a prompt, the chain nudge after a subagent stops. A nudge repeats on its own dedup clock until the state it names is fixed.

**Cockpit.** The pane layout herding's bootstrap builds: the human's chat pane, the dispatch and merge control panes, and up to four *working agent* panes. Dispatch runs one cold *control loop* iteration at a time; the *enable marker* (`touch`ed only by the human) arms it, the *stop marker* halts it at the next boundary; merge stays the owner's single-shot gesture.

## Events that end or interrupt

**Killed.** The invocation's process ends early — Ctrl+C, a kill signal, the terminal closing. Record writes are atomic (old file or new, never torn); a held store lock goes stale (30 s with a dead pid, 1 h unconditionally) rather than wedging siblings.

**Compaction.** The session's context is summarized mid-conversation. A compacted session gets the compact capsule — disk state overrides conversational recollection — and never auto-adopts a handoff.

**Handoff.** A written record (`planned-next` or `pause`) that carries work across a session boundary. `planned-next` is adopted by a fresh session (`startup`/`clear` only, never the writer itself); `pause` waits for the human's word. A kindless record reads as pause.

**Heartbeat expiry.** A session record goes stale after 900 seconds without a touch (the hook stamps at most once per 60 s); its waiting marks expire with it and siblings stop counting it as live. *Signal* is the separate 90-second "did the agent do anything just now" reading.

**Lease expiry.** A claim or reservation outlives its holder: the lease TTL is 3600 seconds, after which it can be swept or re-claimed.

**Sibling change.** Another session takes the claim, places a hold or reservation, or merges a worktree underneath the current flow. The affected invocation is refused or denied with the sibling named; the response is to take disjoint work and report, never to write through.

**Crash candidate.** A dead session `bee recovery scan` judges worth mining: heartbeat stale, transcript findable, no clean-end pattern at its tail, and a live work signal. Scan reports candidates; mining files `--source mined` capture stubs; nothing auto-resumes.

**Coverage gap.** A named event or input a hook could not decide on, logged to `.bee/logs/hooks.jsonl`. Never a deny and never an allow — the record that the guard did not run.

**Channel change.** The same invocation in a different mouth: output piped or `--json`, the Codex runtime (advisory events, no SessionEnd), or the command run from inside a hook. It changes what is printed and which events exist, never what is true in the store.

## Output

**Preamble.** The state summary injected at session start (after compaction, the compact capsule instead): version, phase, gates, standard commands, dispatch door, project map, pending queues, recent decisions. It is already read; re-fetching it is waste.

**Progress line.** The one-line-per-step narration contract: `▸` started, `✓` green, `⚡` auto-approved, `✗` red. A red or refusal line is never silenced.

**Timing line.** The stderr trailer every served run prints — `[bee] <cmd> <N>ms` — mirrored into `.bee/logs/timings.jsonl` as `{ts, cmd, ms, ok}` (the *self-timing log*: telemetry, not state).

**`--json`.** The machine mode nearly every command takes. With it, the payload — including errors, as `{"error": msg}` — goes to stdout. Without it, success text prints on stdout and refusal or error text on stderr; the timing line is stderr either way.

**Waiting-on mark.** The record that a turn ended waiting on the human — kind `gate`, `question`, or `turn-end`. The agent sets `gate` and `question`; the Stop hook alone sets `turn-end`; any human prompt clears a live mark. Expiry is dual-condition: mark age *and* the owning session's heartbeat both stale.

**Privacy marker.** The `@@BEE_PRIVACY@@{"file":…,"question":…}@@END@@` block a secret-guard deny emits. It is routed to the human, never acted on by the agent; no bypass level covers a secret read.

**Doctor verdict.** Doctor's three-state grade of a runtime's harness — `ready`, `degraded`, `blocked` — the one bee answer whose value decides its own exit code (`blocked` exits 1). Always evaluated, never assumed.

**Staleness warning.** One advisory sentence in `bee status` naming a way the repo drifted out of step with itself. Warnings never change an exit code and never block.
