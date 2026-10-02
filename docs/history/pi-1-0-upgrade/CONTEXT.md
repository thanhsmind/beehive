# Context: pi-1-0-upgrade

The user installed Pi 1.0.0 (mise, 2026-10-01) and asked for a plan to bring
bee's Pi extension (`.pi/extensions/bee-guard.ts`) up to it. The last version a
bee workflow ran end to end on is 0.85.1. Eight Pi releases sit between:
0.86.0, 0.86.1, 0.87.0, 0.87.1, 0.99.0, 0.99.1, 0.99.2, 1.0.0.

## Locked decisions

- **D1 — The ceiling moves only on live evidence.** The belt header and the
  capability audit name 1.0.0 as proven only after a bee workflow is driven
  end to end on Pi 1.0.0. The floor stays 0.84.4. Rule home:
  `docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md`
  ("Business Rules").
- **D2 (amended after the hat wave) — `codemode` and `tool_search` pass the
  outer guard by name and survive stage narrowing.** Pi runs every nested
  call of a codemode script through the `tool_call` handler (Pi
  `docs/extensions.md:148`), so bee-guard checks each real read and write
  inside the script. The outer call carries only script text, which no bee
  hook can judge. `tool_search` only declares tools. Both names join the
  stage-tools full set. MCP tools and every other unknown name keep the
  write-capable fail-safe.
- **D3 — The per-turn context feed keeps its `systemPrompt` carrier for this
  slice (named guess).** Pi 0.86 stores mid-conversation system prompt
  changes in the transcript and sends a forced prompt as the leading system
  prompt. The live run measures what this costs. A move to another carrier
  opens only on that evidence.
- **D4 (amended 2026-10-02) — Out of scope.** MCP server export,
  `context_with_system`, `appendContextEdit`, exported hook types and a
  `user_bash` guard are backlog rows.

## The redirect (user, 2026-10-02)

The user: Pi's harness can steer any model through the right workflow. With
small models today the bee workflow does not take — team dispatch and
herding rarely fire, advisor calls are rare. Moving these into the
mechanism underneath works better, the way omp does it.

- **D5 — The harness forces the next call; the bee CLI decides what it is.**
  The belt holds no workflow state and no state machine
  (pi-native-stage-driver D6 stands). Every new belt behavior reads one bee
  verb and enforces its answer.
- **D6 — A `bee_dispatch` tool.** It runs `bee dispatch prepare` and then the
  exact command it returns — only the bee binary, in argv form, never through
  a shell — detached, returning a job handle; the result drain delivers the
  outcome. The spawner stays in Rust (`bee herding run`). This tool is the
  herding path on Pi; the herding cockpit itself is out of scope. Supersedes
  only the "NO dispatch tool" clause of pi-native-stage-driver D11.
- **D7 — A settle obligation check.** At `agent_before_settle` the belt asks
  the binary what is owed, using the obligations `session-close` already
  finds (claimed uncapped cells, high-risk advisor debt), not a new engine.
  When something is owed, it appends one message naming the required call
  and requests one continuation — at most once per obligation key (feature +
  cell), recorded by bee. No record written means no continuation. Never in
  a worker session. The close stays warn-only.
- **D8 (amended after the hat wave) — The leader may not write in the
  execute phase on lanes `small` and up.** The rule is ENFORCED in
  `bee hook write-guard` by session role, lane and phase, because codemode
  scripts call tools whatever the active set is (Pi `docs/mcp.md:204`) and
  write-guard sees those nested calls. The tool loadout from
  `bee hook stage-tools` is the visible signal. Worker sessions keep every
  tool; a session whose role is unknown keeps every tool; tiny keeps inline
  writes. Contract f3e80214 holds: bee answers the set, the belt holds no
  rule, and the narrowing announces itself to the user and the model.
- **D9 — The advisor from the harness.** `bee_advisor` is a thin second name
  over `bee_dispatch` with `--kind advisor`, shipped with `bee_dispatch`. The
  obligation check fires the plan-step hat wave once per feature when the
  plan is gate-ready, after a cost line, behind a config off-switch. Gate
  approval never moves into the harness.
- **D10 (amended after the hat wave) — No virtual-model router in this
  feature.** It is a backlog row. Model choice does not make dispatch or the
  advisor fire.
- **D11 — One kill switch.** One bee config key turns the D6-D9 behavior on
  or off without a release, read through the bee verbs. It ships on.

## Mid-run input (user, 2026-10-02)

The user: omp can take context typed while a run is going and add it to the
running flow; bring part of that into bee. Research: omp steers the main
loop at turn boundaries and steers a subagent only on an explicit parent call
(`steer_subagent`); Pi 1.0 steers the main loop the same way; bee's no-pane
Pi worker has no input channel after it starts (`herding/run.rs:2984`).

- **D12 — Leader input stays Pi's steer.** A new belt `input` handler only
  TAGS steered text that reads as a scope change. It never edits or drops it.
- **D13 — Worker steer inbox.** `bee herding steer <job> "<text>"` (a
  `bee_steer` tool on the leader) writes the user's words, verbatim, to that
  job's steer inbox; the worker's belt injects them with `deliverAs: "steer"`
  at its next turn boundary. The user picks the job, or there is exactly one
  running worker for the current cell. A capped cell's job refuses it.
  `bee herding interrupt` stays the hard stop.
- **D14 — Scope change is an obligation.** A tagged scope-change input with
  no decision record is a slice-3 obligation: one continuation asks the
  leader to log it or ask the user. A change to an approved plan reopens the
  gate; only the user answers it.
