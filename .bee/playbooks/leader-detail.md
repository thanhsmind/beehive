# Leader detail

The full text of blocks that `.bee/LEADER.md` states by their boundary sentence. Each heading names the rule or section it extends; the text under it is the original block, unchanged.

## Bee workflow

Read when: you route a task through the lifecycle, or log an agreement.

Use bee to build with bee. For any non-trivial code, docs, or behavior
change, load the `bee-hive` skill and follow the bee lifecycle:
explore, gate, plan, execute, scribe, compound. That lifecycle is the
Main flow, idea to ship; a separate Discovery flow carries a
fog-state ask from an open question to a locked decision before
handing into it. The skill routes by size and risk — a typo fix takes
one cell and one merged question; an auth change takes the full
chain.

- Modify bee state only through the CLI (`.bee/bin/bee …`),
  never by hand-editing `.bee/*.json(l)`. Log agreements with
  `bee decisions log --relation supersedes:<id>|touches:<id>|none`
  (the relation is required); `docs/history/<feature>/CONTEXT.md` holds
  the locked ones — cite it for a locked decision and its reason, never
  reinterpret it. Current behavior is cited from its `docs/knowledge`
  concept.

## agents-gates-never-self-approved

Read when: a gate, a decision answer or a bee command is in question.

- Never approve a gate yourself (the opt-in `gate_bypass` switch is
  the one recorded exception; bee-herding's unattended control loop is
  that exception in use — it requires `gate_bypass` at `full` or
  `total`). Gates, decision answers, and privacy
  approvals belong to the user; every bee command belongs to the
  agent, run the moment the workflow calls for it — never printed for
  the user to run.

## agents-worktree-first

Read when: you place feature work in a worktree or on main, or set the uat door.

- Code-touching feature work lives in its feature worktree from the
  start (`bee worktree new --feature <slug>`); the main checkout takes
  integration and release work, plus docs-lane and a solo `tiny` fix
  when no other session is live — land through `bee worktree merge`.
  That exemption list is the whole list: with another live session
  present, docs-lane and `tiny` take a worktree like any feature.
  The user's acceptance is the `uat` door; config `uat_stop` places it.
  Default `"close"` (absent means this): the agent merges on green
  without asking, keeps the worktree, and hands the user main to test —
  `bee close` holds the stop. `"merge"`: the door blocks
  `bee worktree merge` until approved. Staging (`bee staging add`) is
  the opt-in test ground via `staging_before_merge: true`.

## Speaking a rule id

Read when: the user names a rule id mid-run.

Every rule id here can be spoken: the user may say "apply
never-build-on-red" mid-run. Resolve the id in the rule index
(`docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`,
§ AGENTS.md rule homes), apply the rule to the work in hand, and name in
the reply the decision or step it changed — or say plainly that it
changed nothing. A bare acknowledgement is not an answer.

## Start a session

Read when: you start or resume a session.

Read the injected preamble instead of re-fetching state. One ritual, one
verb: when you are routing, starting, or resuming work — and only then —
run `bee orient`; it names the phase, the blockers, and the next skill.
A plain question needs neither.

## Route each task

Read when: you classify a new task that writes something and open its situation.

The class belongs to the task in front of you, never to the session. A
session the preamble has not bound to a record holds no route and no
principles until you classify. For each new task that writes something:
classify it into one of the nine classes (bugfix, feature, refactor, perf,
docs, research, spike, content, release); run `bee orient --class <c>`,
which prints that class's situations and principles and writes nothing.
Pick the situation whose trigger phrases match the task from the
situation lines it prints, open that situation's file, and open the
session todo list with its numbered steps before any other work. A step
you skip stays on the list, marked `skip:` with its reason. The list is a
projection of the situation file, which stays the one home of the steps.
A class with no situation lines has one playbook file; open that.

A session situation — pick up, pause, run until done, orchestrate,
babysit, ship — comes from `skills/bee-planning/playbooks/session/index.md`
and binds no class. When no situation fits, or the work is large and
cross-cutting, use `bee-figure-it-out`.

## Triage proposed PBIs

Read when: a proposed PBI arrives and needs a verdict.

The project leader triages every proposed PBI itself — build (it stays
`proposed`), park or drop — and records the verdict with
`bee backlog pbi status` and in the owner brief's "Your call" paragraph.
Only a money, publishing or irreversible call goes to the owner,
decision-ready: the question, two options at most, and your
recommendation. Never leave an open choice for the owner on any other
item (`docs/knowledge/areas/decision-memory/decision-memory-the-leader-triages-proposed-pbis-the-owner-gets-only-owner-only-calls.md`).

## WORKSPACE_PROTOCOL.md

Read when: `WORKSPACE_PROTOCOL.md` exists at the repo root.

When `WORKSPACE_PROTOCOL.md` exists at the repo root, read it in full before
you route work. Copy the boundaries a cell touches into that cell's brief,
and name each listed boundary the plan touches in the Gate 2 message (under
bypass, in the auto-approval reason and the closing report). A boundary adds
no new stop: the `gate_bypass` level decides, as for every gate. Workers
never read the file, and `bee protocol check` reports its state.

## agents-knowledge-before-code

Read when: you search the knowledge store before reading source.

- Search the knowledge store before you read code. Once per turn, before
  your first read of source under `packages/` or `src/` (or any source
  file), run `.bee/bin/bee knowledge search --text "<what you need>"`;
  when the feature has knowledge under `docs/knowledge/`, read its
  context (`bee knowledge context`) before planning or executing. A
  search answers in milliseconds what a code hunt answers in dozens of
  tool calls, and knowledge records the reasons code cannot show. The
  same search comes first when a turn answers how something works,
  before `docs/history` is opened for that answer.

## agents-proof-at-cap

Read when: you pick a cell proof, record a proof line, or run the full suite.

- The agent owns test scope: pick the proof your change type needs
  (code → related tests green; docs → parity/pointer checks; behavior
  → judge verdict; a user-facing surface → its mapped feature driven,
  evidence attached — `green:live`), run it yourself, and record it on
  the cap as a proof line `<command> — <result> — <scope reason>`.
  `bee close` and `bee worktree merge` check that recorded proof; they
  run nothing themselves — CI runs the full declared command on every
  push, the one deterministic net.
  A cell proves itself with its related tests only. The full declared
  suite (`commands.test`) runs once per feature, at its end: `bee test`
  in the worktree, then `bee worktree merge`, which refuses a
  code-touching feature without a fresh green record. Never run the
  full suite per cell, and never re-run a worker's recorded proof;
  `scripts/release.sh` and CI keep their own full runs.

## agents-never-build-on-red

Read when: a proof is red at claim, review or cap.

- A red proof refuses the cap — never build on a red base, in every
  scope the rule reaches: at claim time (a known-red base is its own
  fix-first cell), at review (a P1 finding blocks), and at cap. It
  holds as a principle, never as a pre-claim full-suite order. A
  scoped-green cap whose CI later goes red is a fix-first cell plus a
  captured learning on why the scope missed.

## agents-verify-before-agreeing

Read when: you write a reply that makes a technical claim, or the user pushes a direction that conflicts with the code.

- A task opens with one line before its first tool call: `Triage:
  <lane> — <reason in 10 words or fewer>`. A route flag never takes a
  fast lane; the user may override; a plain question needs none.
- A technical claim about code or behavior that this session checked
  carries a numbered citation `[1]`, `[2]` in the text. End the reply
  with a `Sources:` (or `Nguồn:`) block, one line per number: `[n]
  path:line`, or `[n]` the command run with its result. One source
  keeps one number on reuse. A claim from general knowledge or
  inference keeps the inline `[unverified]` marker, gets no number,
  and is never stated as fact.
  Source order: a claim about how something works now cites the
  `docs/knowledge` concept that holds it, then code `file:line`.
  `docs/history` is cited only for why a decision was made. A decision
  hit leads to the concept that cites its D-number first.
- Never agree with a proposal until its key assumption has a citation;
  the first reaction to an unchecked idea is conditional ("sounds
  right; checking X first").
  The Stop hook warns when a reply agrees with no citation.
- When the user's direction conflicts with the code, never say "you are
  wrong": state the goal, what the code does and why with file:line,
  then the options.
  Full forms: `routing-and-contracts.md` ("Claim labels and pushback"),
  `scout-and-ticks.md` ("Fast-lane read budget").

## Proof habits

Read when: you plan an expensive wave or report a proof.

- Cheapest proof first: before an expensive wave or a long run, run the
  short check that would catch a broken environment, and stop on its
  red rather than paying for the long one. Report the whole proof,
  never the flattering half: name what was skipped, filtered, or
  narrowed beside what passed — a narrowed check is never quoted as
  the whole one.
- Evidence is what the build already emits — red test output, a diff,
  a stack trace. Never author an artifact whose only purpose is to be
  deleted as proof.

## Dispatch

Read when: you dispatch work, run the plan-step hat wave, or watch a worker.

- Concurrency is the default; serial needs a named reason: a file
  overlap, a real dependency, a scarce resource, or the user's say-so.
- Fan out the gathering; keep the deciding. A mechanical step (read,
  render, mine) delegates down-tier when its content is needed as a
  digest, not verbatim — in every phase and lane, including plain
  turns where no skill is running. Decide-altitude never delegates:
  gates, synthesis, state writes, and the human conversation stay on
  the session model.
- The ONE door for any dispatch is `.bee/bin/bee dispatch prepare`
  (use `--role` to name the job outright, e.g. `--kind gather --role
  extraction`) — run it first, then run exactly the tool and payload
  it returns (an Agent call naming a rendered bee agent, or a Bash
  `bee herding run` / cli call). Pick the team member by job — the
  role's description says what it is good at; whether it runs as a
  native model, a herding pane or a cli is config, never the leader's
  choice. Never hand-pick `subagent_type`, a `model` param, or a
  leading `[bee-tier: …]` marker: those are what prepare RETURNS, and
  the model-guard hook refuses or rewrites anything else.
- Dispatches follow the cell or stage role in the approved plan. Changed work
  requires a decision tagged `role-reroute` and `bee cells reroute`; never
  select a different role ad hoc during execution.
- The internal consult is ONE thing: the plan-step **hat wave**, fixed
  perspectives dispatched `--kind advisor` at the plan step, once per
  feature. Its synthesis is the plan check AND the high-risk gate's
  advisor consult (recorded with `bee state advisor-ref record` after
  plan.md reaches its gate-ready bytes) — synthesis stays with the
  leader, and independent review is untouched and still user-invoked.
  Procedure home: `bee-hive/references/gates-and-delegation.md`
  ("Hat wave").

- A leader watches a worker through `bee herding status --job <id>` (one line),
  never by reading its transcript, pane log or activity stream; workers write the notes.
  A leader may steer a Claude worker — a claude-session job or a Claude
  herding pane — with `SendMessage` to the `steer_to` name that status
  prints; steering is not a transcript read.
- Reserve files before write-heavy swarm work and prefix write-heavy
  shell commands with `BEE_AGENT_NAME=<name>`.
- Work that belongs to another project goes to its leader through
  waggledance; never read, search or explore its code from this
  session (the write-guard refuses it) — see
  `.bee/playbooks/cross-project.md`.
- Reach any tool that ships a CLI and MCP tools through its CLI
  first, with MCP as the fallback, because the CLI runs the same
  handler and puts far less text in context — see
  `.bee/playbooks/cli-before-mcp.md`.

## agents-never-zero-execution-workers

Read when: you plan which workers run a code cell or a docs-only cell.

- Every code cell runs through dispatched workers in every lane, `tiny`
  included — never zero *execution* workers, and never one worker for
  both halves: a dispatched implementer writes the code, then a separate
  dispatched tester, sharing no context with it, verifies the result and
  adds the new tests. A code cell never certifies itself, and nothing
  runs inline. A docs-only cell needs no tester; a second worker runs its
  parity and pointer checks. The owner prefers team delegation because
  each role runs its own configured model. This rule counts execution
  workers only: the "zero subagents" of a small lane means zero
  *ceremony* helpers, and I/O helpers are exempt from both counts.

## agents-capture-line-at-close

Read when: a task closes, or the verify-app feature map is read at shaping or planning.

Lanes scale ceremony, never memory. The moment a rule, behavior, or
value settles, record it — a decision log line or a capture stub — and
close every task with a capture line or an explicit "nothing settled".
A close with neither is not a close, in every lane, the docs lane and
non-cell quick work included.
The decision store keeps rules, behavior and trade-offs. Gate audit,
close/merge/release/scribing/promote/handoff bookkeeping and door
deferrals go to `bee audit log --kind <k> --feature <f> --note "<text>"`.
`docs/knowledge/` is the state layer: read it first, sync it when
behavior changes. `.bee/verify/verify-app/features/` is the second — a
README index plus one file per user-facing feature: what it is, how a
user reaches it, how to drive it, its gotchas. Read the index at
shaping, the ONE matching feature file at planning, and carry that file
into the execution worker's brief with its last-modified date — a stale
map is judged, never trusted blind. Absent or empty path is the
environment-fact check of § Judgment and deviation: proceed silently,
treating the ask as new.

## agents-feedback-changes-framework

Read when: the owner gives feedback about how agents work.

In the bee repo, owner feedback about how agents work is a change
request to the framework: record it in the bee store, the backlog or the
doctrine, change the doctrine, hook or skill that owns the behavior
through the bee chain, and release it. An agent's private memory may
hold a pointer, never the only copy.

## agents-one-next-action

Read when: you write the closing message of a turn or a progress line.

The user hears the work in their own terms, never bee mechanics. Open
with one line of state; keep narration to what the user needs to act on;
link records
instead of pasting them — name a doc by its bare repo-relative path
(`docs/...`) — a file in an unmerged feature worktree by its absolute path,
since a relative path resolves against main — never a viewer URL; close on exactly ONE next action — the
agent's own next move, or the one thing only the user can decide,
never a menu. When that next action is the agent's own move, the agent
TAKES it in the same turn: between the merged gate and the uat stop, a
finished slice with approved work remaining continues without asking —
"Say go" / "continue?" mid-work is drift, not a stop; the only asks are
the gates, the uat door, and questions shaping needs before the plan.
Emit one short progress line per visible step, on by
default — `▸` started, `✓` green, `⚡` auto-approved, `✗` red — and a
red or refusal line is never silenced, composited, or delayed by any
switch or bypass level. The work is the subject of every line; ids and
counts trail it, never lead.

## Pre-send check

Read when: you send a long message or write a README, guide or handbook page.

**Pre-send check**: reading only the first and last line of the
message must answer what happened and what's next; then strip every
bee term — if nothing the user needs is lost, those terms did not
belong there. The full turn shape and rules load with the `bee-hive`
skill ("Communication contract").

A saved document with no bee template (a README, a guide, a handbook
page) is written through the `bee-technical-writing` skill: type first,
one type per file, linked where types meet.

## Token efficiency

Read when: you consider re-running a command.

- Never re-run a command to "verify" unless the outcome was uncertain
  ("Prove, then say so" still holds: a done/green claim still needs its
  fresh output — test and verify runs ARE the uncertain outcome).

## agents-no-code-comments

Read when: a comment seems needed in code or the write guard refuses a comment.

- No comment in code, in any language: not `//`, not `///` or `//!`,
  not `/* */`, not `#`. The why a comment used to carry has two homes
  and no third — a `docs/knowledge` concept whose Pointers name the
  file, or a `bee decisions log` entry; a public item's description
  lives in the owning concept; a workaround is fixed, or filed with
  `bee backlog add` and cited from the concept, never from the code.
  Three exceptions only: a `#!` line, a `SAFETY:` line on an unsafe
  block, and a license header at the top of a file. Where
  `no_code_comments` is on, the write guard refuses the write, and the
  comment baseline reds the suite on any count that rises.

## Recording a mistake

Read when: you notice a mistake, cap a cell, or close a session.

- Write a mistake down the MOMENT you notice it, never composed from
  memory at the end: `bee mailbox reflect --wrong "<what went wrong>"
  --better "<what would have been better>" --fix-at
  <architecture|check|doctrine|none>`. Name a concrete thing in
  the first part — a file, a command, or something you saw — never
  "the approach". Pick the strongest layer someone would actually
  build now (see `bee mailbox reflect --help` for the layer
  meanings). A cap answers the same question for its own cell
  (`--mistake "<wrong> — <better>" --fix-at <layer>`, or
  `--no-mistakes` when it hit none), and `bee close` refuses a
  feature whose capped cells never answered: silence and a clean run
  must not read alike. A closing session answers for named cells with
  `bee mailbox reflect --cell <ids>` (or `--no-mistakes --cell <ids>`),
  and the door clears only the cells named. A user's correction of how
  you worked is a mistake too: record it at once with `--source user`,
  their own words in `--wrong`; an accepted P1 or P2 review finding
  takes `--source review`; a key seen twice is a fix-first candidate.

## Ending substantial work

Read when: you end substantial work.

- Before ending substantial work: cap or release every claimed cell,
  release reservations, leave `.bee/state.json` honest, and name the
  blockers and next action in the final message; the proof to run is
  the one in `agents-proof-at-cap`. Then run
  `bee state session release` — sibling sessions stop counting this
  one as a live worker, and the next user message re-engages it.
