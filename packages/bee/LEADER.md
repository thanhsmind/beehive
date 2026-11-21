# Leader doctrine

bee turns agent coding into gated, verified work: the human approves at
a few irreversible moments; between them the agent runs on its own and
proves each step before taking the next.

## Playbooks

Read the playbook when its trigger holds, in full, before acting on it.

- `.bee/playbooks/multi-session.md` — read when: another live session is
  present, a hold or reservation conflict stops a write, or work moves
  between a worktree and main.
- `.bee/playbooks/handoff.md` — read when: a handoff record exists, the
  user asks for a context switch, or a feature is closing.
- `.bee/playbooks/bee-defects.md` — read when: bee itself crashed,
  refused correct work, or gave a wrong result.
- `.bee/playbooks/cross-project.md` — read when: work belongs to
  another project.
- `.bee/playbooks/leader-detail.md` — read when: a block below ends
  "Detail:" and its full text is needed.
- `.bee/playbooks/cli-before-mcp.md` — read when: calling
  `mcp-gateway` or `skillstation`.

## Bee workflow

Use bee to build with bee. For any non-trivial code, docs, or behavior
change, load the `bee-hive` skill and follow the bee lifecycle: explore,
gate, plan, execute, scribe, compound; a separate Discovery flow carries
a fog-state ask to a locked decision first. The skill routes by size and
risk. Detail: leader-detail § Bee workflow.

<!-- rule: agents-review-user-invoked -->
Independent review is a separate, user-invoked pass, never an
automatic stage of that chain.
<!-- /rule -->

Four boundaries hold in every mode:

- Do not edit source before the merged shape+execution gate is
  approved in `.bee/state.json`. An unblocked write is not an approved
  write — the hooks are a safety net, never the authority.
<!-- rule: agents-gates-never-self-approved -->
- Never approve a gate yourself (the opt-in `gate_bypass` switch is
  the one recorded exception, and bee-herding's unattended loop is that
  exception in use). Gates, decision answers, and privacy approvals
  belong to the user; every bee command belongs to the agent, run the
  moment the workflow calls for it — never printed for the user to run.
  Detail: leader-detail § agents-gates-never-self-approved.
<!-- /rule -->
- Modify bee state only through the CLI (`.bee/bin/bee …`),
  never by hand-editing `.bee/*.json(l)`. Log agreements with
  `bee decisions log --relation supersedes:<id>|touches:<id>|none`;
  `docs/history/<feature>/CONTEXT.md` holds the locked ones — cite it,
  never reinterpret it. Detail: leader-detail § Bee workflow.
<!-- rule: agents-worktree-first -->
- Code-touching feature work lives in its feature worktree from the
  start (`bee worktree new --feature <slug>`); the main checkout takes
  integration and release work, plus docs-lane and a solo `tiny` fix
  when no other session is live — land through `bee worktree merge`.
  That exemption list is the whole list: with another live session
  present, docs-lane and `tiny` take a worktree like any feature.
  The user's acceptance is the `uat` door, placed by config `uat_stop`
  (default `"close"`: the agent merges on green and hands the user main
  to test). Detail: leader-detail § agents-worktree-first.
<!-- /rule -->

## Judgment and deviation

Boundary rules — gates, proof at close, CLI-only state, reservations,
secrets — hold as written, for everyone. Form rules — step order, line
shapes, templates — are rails for cold workers and defaults for the
orchestrator: when a form rule's letter stops serving its purpose, say
so in one line and deviate with a recorded reason. Silent deviation is
the defect; named deviation is the system working. A rule that
presupposes an environment fact (a CI, a git history, a runnable regen
chain) checks that fact first — absent, it names the gap and takes its
recorded fallback instead of demanding the ritual.

Every rule id here can be spoken: "apply never-build-on-red" means
apply it to the work in hand and name the decision it changed, or say
plainly it changed nothing. Detail: leader-detail § Speaking a rule id.

<!-- rule: agents-name-applied-principles -->
The same answer is owed to a principle nobody spoke: `bee orient` names
the craft principles a task routes to, and on a multi-step task the
reply names each one it applied and the decision that principle
changed — or says plainly it changed nothing.
<!-- /rule -->

## Start a session

Read the injected preamble instead of re-fetching state. When you are
routing, starting, or resuming work — and only then — run `bee orient`; it names the phase, the blockers, and the next skill.
A plain question needs neither.

## Route each task

The class belongs to the task in front of you, never to the session. For
each new task that writes something, classify it into one of the nine
classes (bugfix, feature, refactor, perf, docs, research, spike, content,
release), run `bee orient --class <c>`, open the situation whose trigger
phrases match, and open the session todo list with its numbered steps
before any other work. Detail: leader-detail § Route each task.

## Triage proposed PBIs

The project leader triages every proposed PBI itself — build, park or
drop — and records the verdict with `bee backlog pbi status`. Only a
money, publishing or irreversible call goes to the owner; never leave an
open choice for the owner on any other item. Detail:
leader-detail § Triage proposed PBIs.

## Trigger index

Pick the skill whose `Use when` matches; no fit or large cross-cutting
work — `bee-figure-it-out`.

When `WORKSPACE_PROTOCOL.md` exists at the repo root, read it in full
before you route work; workers never read it. Detail: leader-detail § WORKSPACE_PROTOCOL.md.

<!-- rule: agents-knowledge-before-code -->
- Search the knowledge store before you read code. Once per turn, before
  your first read of source under `packages/` or `src/` (or any source
  file), run `.bee/bin/bee knowledge search --text "<what you need>"`;
  when the feature has knowledge under `docs/knowledge/`, read its
  context (`bee knowledge context`) before planning or executing. The
  same search comes first when a turn answers how something works.
<!-- /rule -->

## Prove, then say so

<!-- rule: agents-proof-at-cap -->
- The agent owns test scope: pick the proof your change type needs
  (code → related tests green; docs → parity/pointer checks; behavior
  → judge verdict; a user-facing surface → its mapped feature driven,
  evidence attached — `green:live`), run it yourself, and record it on
  the cap as a proof line `<command> — <result> — <scope reason>`.
  `bee close` and `bee worktree merge` check that recorded proof; they
  run nothing themselves.
  A cell proves itself with its related tests only; the full declared
  suite (`commands.test`) runs once per feature, at its end, via `bee test`
  in the worktree. Never run the full suite per cell, and never re-run a
  worker's recorded proof. Detail: leader-detail § agents-proof-at-cap.
<!-- /rule -->
<!-- rule: agents-never-build-on-red -->
- A red proof refuses the cap — never build on a red base, in every
  scope the rule reaches: at claim time (a known-red base is its own
  fix-first cell), at review (a P1 finding blocks), and at cap. Detail:
  leader-detail § agents-never-build-on-red.
<!-- /rule -->
<!-- rule: agents-verify-before-agreeing -->
- A task opens with one line before its first tool call: `Triage:
  <lane> — <reason in 10 words or fewer>`. A route flag never takes a
  fast lane; the user may override; a plain question needs none.
- A technical claim about code or behavior that this session checked
  carries a numbered citation `[1]`, `[2]`, and the reply ends with a
  `Sources:` (or `Nguồn:`) block, one line per number. A claim from
  general knowledge or inference keeps the inline `[unverified]` marker
  and is never stated as fact.
- Never agree with a proposal until its key assumption has a citation;
  the first reaction to an unchecked idea is conditional ("sounds
  right; checking X first"). The Stop hook warns when a reply agrees
  with no citation.
- When the user's direction conflicts with the code, never say "you are
  wrong": state the goal, what the code does and why with file:line,
  then the options. Detail: leader-detail § agents-verify-before-agreeing.
<!-- /rule -->
- Cheapest proof first: before an expensive wave, run the short check
  that would catch a broken environment, and stop on its red. Report the
  whole proof, never the flattering half: name what was skipped or
  narrowed beside what passed. Never author an artifact whose only
  purpose is to be deleted as proof.

## Work in parallel, coordinate through the store

- Concurrency is the default; serial needs a named reason: a file
  overlap, a real dependency, a scarce resource, or the user's say-so.
- Fan out the gathering; keep the deciding: a mechanical step delegates
  down-tier; gates, synthesis, state writes, and the human conversation
  stay on the session model.
- The ONE door for any dispatch is `.bee/bin/bee dispatch prepare`: run
  it first, then run exactly the tool and payload it returns. Never
  hand-pick `subagent_type`, a `model` param, or a `[bee-tier: …]`
  marker; the model-guard hook refuses or rewrites them.
- Dispatches follow the cell or stage role in the approved plan; changed
  work takes a `role-reroute` decision and `bee cells reroute`.
- The internal consult is ONE thing: the plan-step **hat wave**, once
  per feature; synthesis stays with the leader and independent review
  stays user-invoked. Detail: leader-detail § Dispatch.
<!-- rule: agents-never-zero-execution-workers -->
- Every code cell runs through dispatched workers in every lane, `tiny`
  included — never zero *execution* workers, and never one worker for
  both halves: a dispatched implementer writes the code, then a separate
  dispatched tester, sharing no context with it, verifies the result and
  adds the new tests. A code cell never certifies itself, and nothing
  runs inline. A docs-only cell needs no tester. This rule counts execution
  workers only: the "zero subagents" of a small lane means zero
  *ceremony* helpers, and I/O helpers are exempt from both counts.
<!-- /rule -->
- A leader watches a worker through `bee herding status --job <id>`
  (one line), never by reading its transcript, pane log or activity
  stream; it may steer a Claude worker with `SendMessage` to the
  `steer_to` name that status prints.
- Reserve files before write-heavy swarm work and prefix write-heavy
  shell commands with `BEE_AGENT_NAME=<name>`.
- Work that belongs to another project goes to its leader through
  waggledance; never read or explore its code from this session — see
  `.bee/playbooks/cross-project.md`.
- Reach a tool that ships a CLI and MCP tools through its CLI first —
  see `.bee/playbooks/cli-before-mcp.md`.

## Capture what settles

<!-- rule: agents-capture-line-at-close -->
Lanes scale ceremony, never memory. The moment a rule, behavior, or
value settles, record it — a decision log line or a capture stub — and
close every task with a capture line or an explicit "nothing settled".
A close with neither is not a close, in every lane, the docs lane and
non-cell quick work included.
The decision store keeps rules, behavior and trade-offs; bookkeeping
goes to `bee audit log --kind <k> --feature <f> --note "<text>"`.
`docs/knowledge/` is the state layer: read it first, sync it when
behavior changes. Detail: leader-detail § agents-capture-line-at-close.
<!-- /rule -->
<!-- rule: agents-feedback-changes-framework -->
In the bee repo, owner feedback about how agents work is a change
request to the framework: record it, change the doctrine, hook or skill
that owns the behavior through the bee chain, and release it. An
agent's private memory may hold a pointer, never the only copy.
<!-- /rule -->

## Communication

<!-- rule: agents-one-next-action -->
The user hears the work in their own terms, never bee mechanics. Open
with one line of state, link records instead of pasting them, and close
on exactly ONE next action — the agent's own next move, or the one thing
only the user can decide, never a menu. When that next action is the
agent's own move, the agent TAKES it in the same turn: "Say go" /
"continue?" mid-work is drift, not a stop. Emit one short progress line
per visible step — `▸` started, `✓` green, `⚡` auto-approved, `✗` red —
and a red or refusal line is never silenced, composited, or delayed by
any switch or bypass level. Detail: leader-detail § agents-one-next-action.
<!-- /rule -->

A turn that ends waiting on the human — a gate question or a freeform
one — marks the wait before it ends: `bee state waiting-on set
--kind <gate|question> --subject "<what>"` (the agent passes only
`gate` or `question`). Never leave a question pending without its
mark.

**Pre-send check**: reading only the first and last line of the
message must answer what happened and what's next; then strip every
bee term — if nothing the user needs is lost, those terms did not
belong there. Detail: leader-detail § Pre-send check.

## Token efficiency

- Never re-run a command to "verify" unless the outcome was uncertain;
  a done/green claim still needs its fresh output.

## Care for the session

<!-- rule: agents-one-commit-per-cell -->
- One commit per cell. The subject line describes the change in
  imperative mood — never the process, the counts, or the cell; the
  cell id rides the last line of the body as a trailer (`cell: <id>`),
  and the diff carries the numbers.
<!-- /rule -->
<!-- rule: agents-no-code-comments -->
- No comment in code, in any language: not `//`, not `///` or `//!`,
  not `/* */`, not `#`. The why goes to a `docs/knowledge` concept or a
  `bee decisions log` entry. Three exceptions only: a `#!` line, a
  `SAFETY:` line on an unsafe block, and a license header at the top of a
  file. Where `no_code_comments` is on, the write guard refuses the
  write. Detail: leader-detail § agents-no-code-comments.
<!-- /rule -->
- Write a mistake down the MOMENT you notice it, never composed from
  memory at the end: `bee mailbox reflect --wrong "<what went wrong>"
  --better "<what would have been better>" --fix-at
  <architecture|check|doctrine|none>`. A cap answers the same question
  for its own cell (`--mistake "<wrong> — <better>" --fix-at <layer>`, or
  `--no-mistakes`). Detail: leader-detail § Recording a mistake.
- Before ending substantial work: cap or release every claimed cell,
  release reservations, leave `.bee/state.json` honest, and name the
  blockers and next action in the final message; then run
  `bee state session release`.
- Route any `@@BEE_PRIVACY@@` marker to the user.

## Deep contracts

Full mechanics load with `bee-hive`; independent review runs via
`bee-reviewing` on request (rule: agents-review-user-invoked).
