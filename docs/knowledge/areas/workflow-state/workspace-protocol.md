---
type: bee.area
title: "Workflow State — the workspace protocol file, dispatch receipts, and brief discipline"
description: "One optional root file names the human decision boundaries, the merge owner and the deploy owner; bee validates it and reports its state but never gates on it. Every dispatch prepare record carries a digest of the prompt and of that file, dispatch verify compares them after the dispatch, and two prompt lines plus a warn-only purpose lint keep menus and leaning out of briefs."
timestamp: 2026-11-07
bee:
  id: workflow-state-workspace-protocol
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: [workspace-protocol D1-D12 (`bee decisions search --text WORKSPACE_PROTOCOL`), protocol-init-from-project D1-D8 (init renders from the project; narrows D4), onboard-workspace-protocol a4c2c38d and 0687dd2a (onboard reports the protocol state and the procedure runs after it)]
  sources: [docs/history/workspace-protocol/CONTEXT.md (D1-D12), docs/history/workspace-protocol/plan.md, workspace-protocol cells wsp-1..wsp-4, docs/history/protocol-init-from-project/CONTEXT.md (D1-D8), onboard-workspace-protocol decisions a4c2c38d and 0687dd2a, cell owp-2]
  authoritative_for: "workflow-state: WORKSPACE_PROTOCOL.md, bee protocol, the prepare digests, bee dispatch verify, the purpose leaning lint"
  anchors: [packages/bee/prompts/worker-cell.md, packages/bee/prompts/advisor.md, packages/bee-rs/crates/bee/src/verbs/protocol.rs, packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs, packages/bee-rs/crates/bee/src/verbs/drivers/brief_lint.rs, packages/bee-rs/crates/bee/src/doctor.rs, packages/bee-rs/crates/bee/src/onboard/notices.rs, packages/bee-rs/crates/bee/src/onboard/mod.rs]
  verified_at: 2026-11-10
  verified_commit: 2f0495d24e30048da79fa3d5dcc287605cf6869c
---

# Workflow State — the workspace protocol file, dispatch receipts, and brief discipline

A leader must not guess three facts: what a human must decide, who merges, and
who deploys. A host project writes them once, in one root file. bee reads the
file and reports it. bee never blocks work because of it.

This concept also owns the receipt that `bee dispatch prepare` writes, and the
two brief rules that came with it. The ideas come from the slp projects
(paseo-pi-team, paseo-room). Cited as `workspace-protocol D1-D12`.

## Behaviors & Operations

**The root file and its key format (D1).** The file is
`<repo root>/WORKSPACE_PROTOCOL.md`. bee reads every line that matches
`^([A-Z][A-Z0-9_]*):[ \t]*(.*)$`, inside or outside a fenced block. A key with
an empty value takes the following `- <item>` lines as its list. The list ends
at the next key line or at the first blank line. Values are trimmed. A
trailing ` # comment` stays part of the value. An unknown key is legal and is
kept, so a prose line such as `NOTE: x` becomes a harmless key. A key seen
twice keeps its last value and adds the issue `duplicate key <KEY>`. The key
format is the same as paseo-pi-team's template, so one file serves bee and
Paseo.

**Four states, three required keys (D2).** The state is `missing` (no file),
`unreadable` (an I/O error, or the bytes are not UTF-8), `invalid` (at least
one issue) or `valid`. Three facts are required for `valid`: `MERGE_OWNER`
non-empty, `DEPLOY_OWNER` non-empty, and `HUMAN_DECISION_BOUNDARIES` with at
least one item. Each failure is one issue string that names its key. A scalar
`HUMAN_DECISION_BOUNDARIES: x` is an issue that says to use a `- item` list.
A value or list item that still holds a template placeholder is the issue
`<KEY> still holds a template placeholder`. A placeholder is text that
contains both `<` and `>`, or ` | ` between choices.

**Test commands keep one home (D3).** `.bee/config.json` `commands.test` is the
only place a test command lives. `FULL_TEST` is allowed in the protocol, but
when it holds a non-empty value that differs from `commands.test`, that is the
issue `FULL_TEST drifts from .bee/config.json commands.test — delete FULL_TEST
or make it equal`. An array in `commands.test` compares as its items joined
with ` && `. An equal value or an empty value is fine. The owner's first idea
was to gather test commands into the protocol. D3 narrows it on purpose, so
that one fact does not live in two files.

**`bee protocol check` and `bee protocol init` (D4).** `bee protocol check
[--json]` prints the state, the issues, the parsed keys and the sha256 of the
file. It exits 0 for every state, because it reports and never gates. `bee
protocol init` writes the starter template to the repo root only when no file
exists. An existing file gives a typed refusal (`protocol_exists`) that names
the file. `init` never overwrites. The template ships filled, so `init` then
`check` gives `valid`: version 1, criticality `medium`, both owners `human`, six
boundaries, and one protected area `none`. The template holds no placeholder.
`init` ends with one line: review and edit the file, because the defaults are
bee's guess and not the owner's choice.

**`bee protocol init` renders from the project (protocol-init-from-project
D1-D7).** protocol-init-from-project D1-D8 narrow the fixed template of D4: `init` now
renders from the control root. `PROJECT_ID`, `DEFAULT_BRANCH` and
`REPOSITORY_REMOTE` come from git. `MERGE_OWNER` comes from `.bee/config.json`
`uat_stop`. `DEPLOY_OWNER` names the first release entry found. The boundaries
start from bee's base set, and `PROTECTED_AREAS` lists the vendored skill trees
that exist. The prose above the block holds a `## Sources` list, and `--json`
adds `facts` (key, value, source) and `review` (the project's rule files).
`--print` renders to stdout and writes nothing, also when a file exists. An
existing file is still the `protocol_exists` refusal without `--print`. The
render still holds no placeholder and no test command, so `check` gives
`valid`. The agent finishes the file by reading the `review` files, as the
"Workspace protocol" section of bee-hive `references/onboarding.md` says (D8).

**`bee onboard` reports the protocol state (a4c2c38d, 0687dd2a).** The onboard
JSON carries a `workspace_protocol` object. It holds the check state and
`action_needed`. `action_needed` is true when the state is `missing`,
`invalid` or `unreadable`, and also when the file is valid but still bee's
unedited render. The unedited test is byte equality of the file with the fresh
`bee protocol init --print` render. When `action_needed` is true, onboard adds
one notice: `workspace protocol: WORKSPACE_PROTOCOL.md is <what> — finish it
now with the Workspace protocol procedure in bee-hive references/onboarding.md`.
Onboard never writes the file. The agent runs the procedure, then runs onboard
again to see `action_needed` false. bee-hive `SKILL.md` makes that a required
step of onboarding, and the procedure says where to look for each key, what a
good item looks like and what to check before the commit.

**The `doctor` row (D5).** `bee doctor` always prints the row
`workspace-protocol`. States `missing` and `valid` have status `ok`; the
missing detail says the file is optional and that `bee protocol init` writes
one. States `invalid` and `unreadable` have status `advisory`, with the issues
and the fix `bee protocol check`. The row never makes `overall_status`
blocked, so a host without a protocol does not go red.

**The leader reads, the worker gets an excerpt (D6).** When the file exists,
the leader reads it in full before routing work. The leader copies the
boundaries a cell touches into that cell's brief. The leader names each listed
boundary the plan touches in the Gate 2 message. Under gate bypass, the
auto-approval reason and the closing report carry the same names. A boundary
adds no new stop. The `gate_bypass` level decides what stops, as for every
gate. A worker never reads the file. The rule lives in the leader doctrine
(`.bee/LEADER.md` and its vendored copies).

**Prepare records a digest of the prompt and of the protocol (D7).** Every
record that `bee dispatch prepare` appends to `.bee/logs/dispatch.jsonl`, of
every kind, has two more keys. `prompt_sha256` is the sha256 hex of the exact
text field that the returned payload carries to the worker: `prompt` for an
Agent or spawn payload, `stdin` for a Bash herding or cli payload, `message`
for a codex payload. `protocol_sha256` is the sha256 hex of the protocol file
bytes at prepare time, or `null` when the file is missing or unreadable. The
prepare JSON output carries both keys beside `dispatch_id`. Prepare reads the
protocol from the control root (the main checkout that holds the bee store),
never from a worktree copy. The keys `brief_sha256` (advisor blind lanes) and
all older keys keep their meaning. bee writes the receipt at dispatch time,
not the agent.

**`bee dispatch verify` (D8).** `bee dispatch verify --dispatch-id <id>
[--prompt-file <path>] [--json]` finds the newest prepare record with that id
and reports two fields. `protocol` is `same`, `changed`, `appeared`, `removed`
or `absent`, from the recorded digest against the current file. `prompt` is
`match` or `mismatch` when `--prompt-file` is given, `not_checked` when it is
not, and `unrecorded` when the record has no `prompt_sha256`. A prompt file
that is not UTF-8 is `mismatch`, so the one `&str` hasher stays the only
hasher. The leader saves the carried field byte-exact, for example with `jq -j
'.payload.prompt // .payload.stdin'` over the prepare output. Verify reads the
log and the file from the control root, from main or from a worktree alike.
An unknown id is the typed refusal `dispatch_not_found`, and its fix names the
control root that was searched. Verify exits 0 whenever it finds the record.
A leader runs it after a resume to prove that the brief and the protocol are
the same.

**No menu in a worker brief (D9).** `packages/bee/prompts/worker-cell.md` has
one contract line: a worker never stops to offer the leader a menu of
implementation options. The worker chooses inside the cell's scope and records
the choice as a departure. A decision outside the scope is one short yes/no
question, returned through `[PARKED]` or `[BLOCKED]`.

**No leaning in an advisor brief (D10).** `packages/bee/prompts/advisor.md`
has the line the reviewer prompt already had: the brief never carries the
orchestrator's own verdict, leaning or expected findings. Both read-only
consult kinds now follow one rule.

**The purpose lint warns and cannot refuse (D11).** For `--kind reviewer` and
`--kind advisor`, prepare runs one arm of the existing brief lint over
`--purpose`: the leaning-stem arm. A `--role` that starts with `hat-` is
exempt, because a hat purpose carries its perspective on purpose. The
four-section shape arm never runs on a purpose. A hit never refuses. The
output gains `purpose_leaning` (the lint's reason and fix), text mode prints
one warning line on stderr, the dispatch record gets `purpose_leaning: true`,
and the payload is built as before. The reason it must not refuse: a false
fire on `--purpose` would refuse the Gate 3 advisor consult and deadlock
high-risk work. Warning keeps that fence. The refusal on `--brief-file` is
unchanged. The stem list has one home in `brief_lint.rs`; the purpose lint
reuses it and keeps no copy.

**One rule was not taken (D12).** The paseo-room rule "do not narrow the review
dimensions" is not part of bee. The bee reviewer is lens-scoped by design: each
specialist reads one lens (`reviewer.md`).

## Business Rules

- R1 — The protocol file is optional. A missing file is `ok` in `bee doctor`.
  No gate, no cell claim and no dispatch refuses because of its state
  (D4, D5).
- R2 — `valid` needs `MERGE_OWNER`, `DEPLOY_OWNER` and a
  `HUMAN_DECISION_BOUNDARIES` list with one item or more. Anything else is
  `invalid` with one issue per failure (D2).
- R3 — `commands.test` in `.bee/config.json` is the one home of test commands.
  A differing `FULL_TEST` is an issue (D3).
- R4 — `bee protocol init` never overwrites an existing file (D4).
- R5 — The leader reads the file. The worker never does. A boundary adds no
  stop beyond what `gate_bypass` already decides (D6).
- R6 — Every prepare record carries `prompt_sha256` and `protocol_sha256`.
  Both are read from the control root (D7).
- R7 — `bee dispatch verify` reports and never gates. It exits 0 for every
  record it finds (D8).
- R8 — The purpose lint warns and never refuses. It does not run for `hat-`
  roles or for other kinds (D11).
- R9 — Onboard reports and never writes the protocol file. A true
  `action_needed` asks the agent to run the Workspace protocol procedure
  before feature work, and gates nothing in the binary (a4c2c38d, 0687dd2a).

## Edge Cases Settled

- A file written as `- KEY: value` dash lines (the paseo doctrine style) reads
  no keys and gives three issues. It is `invalid`, not an error.
- A copied, unfilled paseo template is `invalid`, because of its placeholders.
- A dispatch record written before the digests existed has no
  `prompt_sha256`. Verify reports `prompt: unrecorded` for it, and
  `protocol: appeared` or `absent` from the missing recorded digest.
- An unedited render stops matching the fresh render when the git facts
  changed since `init` (a new remote or default branch). Onboard then reads the
  file as edited and `action_needed` is false.
- Prepare and verify read the protocol from the control root, so a worktree
  copy of the file never changes either result.

## Pointers (implementation)

- Parser, states, `check`, `init`, template, `protocol_sha256`:
  `packages/bee-rs/crates/bee/src/verbs/protocol.rs`.
- Prepare digests, `purpose_leaning`, `verify_dispatch` and
  `run_dispatch_verify`:
  `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`. The lint arm it
  reuses: `lint_leaning_stems` in
  `packages/bee-rs/crates/bee/src/verbs/drivers/brief_lint.rs`.
- Doctor row: `workspace_protocol_row` in
  `packages/bee-rs/crates/bee/src/doctor.rs`.
- Onboard report and notice: `packages/bee-rs/crates/bee/src/onboard/notices.rs`.
  Procedure: `skills/bee-hive/references/onboarding.md`, "Workspace protocol".
- Prompt lines: `packages/bee/prompts/worker-cell.md` (D9) and
  `packages/bee/prompts/advisor.md` (D10). Leader doctrine (D6):
  `.bee/LEADER.md`.
- A new verb flag needs its catalog entries: see the `--run` note in
  [`dispatch.md`](dispatch.md).
