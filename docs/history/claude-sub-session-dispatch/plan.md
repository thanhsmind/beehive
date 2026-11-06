---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Claude sub-session dispatch

## Summary

Today a bee leader on Claude starts native workers as hidden subagents.
After this change they start as Claude Code background sessions: each one
shows in `claude agents` and in `ListAgents`, the leader can steer it with
`SendMessage`, and it reports through the same mailbox herding panes use.
Herding panes stay as they are, and a leader may now steer a Claude pane
with `SendMessage` too. Codex, Pi and Grok do not change.

Mode: `standard` — 3 risk flags: external-systems (the `claude` CLI),
public-contracts (the `dispatch prepare` payload), multi-domain (dispatch,
herding, doctrine).
Why this is the least workflow that protects the work: the payload change
reaches every downstream project on refresh, so it needs a plan, a hat wave
and a rollback switch, but no data or auth risk lifts it to high-risk.

Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied).

## Requirements (from CONTEXT.md)

- D1 (7fe67d57): herding roles keep herding; a Claude worker can be steered
  with `SendMessage`. Native Claude roles start as Claude Code sub-sessions.
  Codex, Pi, Grok stay herding-only.
- D2 (16fb3feb): every kind (cell, gather, reviewer, advisor) on a native
  Claude role opens a sub-session, and its result comes back through the
  herding mailbox channel.
- D3 (0f6ebf63, leader's named guess): `dispatch.claude_sessions: false`
  keeps the old Agent payload; no `claude` on PATH falls back to the Agent
  payload with `fallback_reason: claude_cli_missing`; a session-model
  escalation keeps the Agent payload.

## Load-bearing claims

Labels are `read` (file opened at the anchor) or `ran` (command run, output
kept); a claim matches when its evidence bytes are at its anchor.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | `claude --bg` starts a named background session with a chosen model and prints its id | ran | `claude --bg --name cssd-probe --model haiku --permission-mode bypassPermissions --settings '{"worktree":{"bgIsolation":"none"}}' "..."` | `backgrounded · 4e1731f9 · cssd-probe` |
| 2 | With `bgIsolation: none` the session stays in the bee worktree | ran | `claude agents --json --all` | `"cwd": "/mnt/data/Projects/goglbe/beehive--wt--claude-sub-session-dispatch"` |
| 3 | The session is listed with `id`, `name`, `state`, `status` | ran | `claude agents --json --all` | `"status": "busy", "state": "working"` |
| 4 | The session is reachable by `SendMessage` and shows in ListAgents | ran | ListAgents, then SendMessage to `cssd-probe` | `cssd-probe [11465a]  ·  bg  ·  idle` |
| 5 | `claude stop <id>` stops the session | ran | `claude stop 4e1731f9` | `stopped 4e1731f9` |
| 6 | The Claude runtime's native arm emits the Agent payload | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2858` | `tool = "Agent".into();` |
| 7 | The herding arm builds `bee herding run --task-file - --json` with `--cwd`, `--cell-id`, `--read-only`, `--ceiling` | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2619` | `let mut command = ".bee/bin/bee herding run --task-file - --json".to_string();` |
| 8 | Paseo is a separate transport function the new one can copy | read | `packages/bee-rs/crates/bee/src/herding/run.rs:3497` | `pub(super) fn execute_paseo(opts: &Options, spec: &PaseoSpec, cli: &dyn PaseoCli) -> ExecResult {` |
| 9 | Read-only Claude workers are held by an env flag plus the bee claude worker guard | read | `packages/bee-rs/crates/bee/src/herding/run.rs:3510` | `if opts.read_only && provider_is_claude {` |
| 10 | The doctrine line on watching workers lives in the AGENTS block source | read | `packages/bee/AGENTS.block.md:183` | `- A leader watches a worker through \`bee herding status --job <id>\` (one line),` |
| 11 | A shell `claude` call inside a worktree is refused by the worker-outward guard, so the transport must spawn from inside `bee` | ran | `claude --bg ...` from the worktree session | `` `claude` starts another agent ... `bee` itself is never judged by this guard `` |

## Discovery

The leader ran one live probe (claims 1–5) and read the dispatch door and
the herding paseo transport (claims 6–10). `claude logs` gives raw terminal
bytes, so results must come through the mailbox, as D2 requires. Two bee
defects were filed on the way: issues #164 and #165.

## Approach

Recommended path: add a third herding transport, `claude-session`, beside
paseo and the pane transports (D1, D2). `bee herding run --session-model
<model> [--session-effort <e>]` spawns `claude --bg` from inside bee with
the herding brief, the herding child env, `--permission-mode
bypassPermissions` and `bgIsolation: none`; polls `claude agents --json
--all` and the mailbox exactly as the paseo loop does; and stops the session
when the result lands or the ceiling hits. Then the door's Claude native arm
emits that herding command instead of the Agent payload (D2, D3).
`bee herding status` and the run output name the session (`steer_to`) so a
leader can `SendMessage` it (D1). Doctrine and knowledge follow.

Rejected alternatives:
- A new payload `tool: "ClaudeSession"` the leader runs by hand — adds a
  second result path and loses the mailbox; D2 says one channel.
- Herding agent entries (`herding.agents.<n>.claude_session`) per model —
  forces every project to declare agents for roles already configured as
  native; the flag reads the model the role already names.

Risk map:

| Component | Risk | Lands in | Proof |
|---|---|---|---|
| Spawn + poll loop on an external CLI | MEDIUM — the CLI's JSON shape can change | cssd-1 | a fake `claude` CLI trait in unit tests (like `PaseoCli`), plus one live run at cap; claim 3 is the shape |
| Every native dispatch changes payload | MEDIUM — downstream projects get it on refresh | cssd-2 | D3 switch and PATH fallback tests; existing Agent-payload tests stay green with the switch off |
| Read-only gathers in a bg session | LOW — same env + guard as paseo claude | cssd-1 | test that `--read-only` sets `BEE_HERDING_READ_ONLY=1` |

Waves: cssd-1 first; cssd-2 depends on its `--session-model` flag (real
dependency). cssd-3 (docs) runs after both, because it documents the
shipped flag and field names.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"cssd-1 and cssd-2 write Rust and its tests."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Each code cell writes its own tests red-first."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"cssd-3 updates doctrine and knowledge."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"The leader read every anchor."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Standard lane hat wave."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Standard lane hat wave."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Standard lane hat wave."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"Standard lane hat wave."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"Standard lane hat wave."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader writes this plan."},
    {"stage":"deployment","classification":"required","role":"deploy","reason":"The owner asked for the full chain to release."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No extraction."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"Standard lane; the hat wave is the consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One clear design."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One clear design."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One clear design."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI."}
  ]
}
```

## Shape

One slice, three cells: transport, door, doctrine.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| cssd-1 | Add a claude-session transport to herding run | `herding/claude_session.rs` (new), `herding/run.rs`, `herding.rs` | — | `bee herding run --session-model haiku` starts a session that shows in `claude agents` and returns its mailbox result | scoped cargo tests + one live run |
| cssd-2 | Route native Claude dispatches to sub-sessions | `verbs/drivers/prepare.rs` | cssd-1 | `bee dispatch prepare --runtime claude --kind advisor` returns a Bash herding command with `--session-model fable` | scoped cargo tests |
| cssd-3 | Document sub-session dispatch and steering | `packages/bee/AGENTS.block.md`, `AGENTS.md`, `skills/bee-swarming/references/swarming-reference.md`, `docs/knowledge/areas/bee-herding/claude-session-transport.md` (new), `docs/knowledge/areas/bee-herding/index.md` | cssd-1, cssd-2 | AGENTS.md says a leader may steer a Claude worker with `SendMessage` | `bee dev regen` clean + release-manifest check |

```json
[
  {
    "id": "cssd-1",
    "feature": "claude-sub-session-dispatch",
    "title": "Add a claude-session transport to herding run",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["7fe67d57-462a-4090-a2bb-d0b937e31c0c", "16fb3feb-5856-4cbb-a7c2-65125d6650b7"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/claude_session.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding.rs"
    ],
    "read_first": [
      "docs/history/claude-sub-session-dispatch/CONTEXT.md",
      "docs/history/claude-sub-session-dispatch/plan.md",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D1 and D2: add a claude-session transport to `bee herding run`. New flags `--session-model <model>` and optional `--session-effort <low|medium|high|xhigh|max>`; with `--session-model`, run takes this transport instead of paseo or a pane (refuse `--session-model` together with `--agent` or `--no-pane`). Follow execute_paseo (run.rs:3497) for the shape: write the inbox marker, build the herding brief and child env the same way, set BEE_HERDING_READ_ONLY=1 and apply claude_worker_guard_missing on --read-only. Spawn from inside bee, never through a shell, behind a trait like PaseoCli so tests use a fake: argv `claude --bg --name <name> --model <model> [--effort <e>] --permission-mode bypassPermissions --settings {\"worktree\":{\"bgIsolation\":\"none\"}} <prompt>` with cwd = --cwd. Name = `bee-<cell id or kind>-<short job id>`. Parse the id from the `backgrounded · <id> · <name>` line. Poll `claude agents --json --all` for that id and the mailbox result for round 1 with the same decisions the paseo loop makes (result -> done; status idle twice with no result -> TimedOutIdle; state failed or stopped, or id gone -> died; ceiling -> stop and timed out). Run `claude stop <id>` after the result or a give-up. Write `session_id` and `steer_to` (the session name) into the job record and the run JSON output, and have `bee herding status --job` print `steer_to`. Per D1 for a paseo run whose provider is claude: after spawn, find the one session in `claude agents --json --all` whose cwd equals --cwd and whose startedAt is after the spawn time, and record its name as steer_to; when none or several match, omit steer_to. The claude binary is `claude` unless `herding.claude_command` names another.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee claude_session",
    "must_haves": {
      "truths": [
        "a run with --session-model spawns exactly one claude --bg process with bgIsolation none and the given model",
        "the run returns the worker's mailbox result as its JSON output, the same shape a paseo run returns",
        "a session that goes idle twice with no result, fails, or hits the ceiling is stopped and reported, never left running",
        "--read-only sets BEE_HERDING_READ_ONLY=1 in the session env"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/claude_session.rs", "substantive": "spawn argv builder, id parser, agents-json parser, poll decisions, unit tests on a fake CLI"}
      ],
      "key_links": [
        "run.rs takes the claude-session transport when --session-model is given",
        "job record and run output carry session_id and steer_to",
        "a paseo claude run records steer_to when exactly one matching claude session is listed"
      ],
      "prohibitions": [
        "No change to the paseo, herdr or tmux transports' behavior",
        "No code comments",
        "Never read results from claude logs"
      ],
      "open_choices": [
        "How the fake CLI trait is shaped, as long as no unit test runs the real claude binary"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "cssd-2",
    "feature": "claude-sub-session-dispatch",
    "title": "Route native Claude dispatches to sub-sessions",
    "lane": "standard",
    "role": "code",
    "deps": ["cssd-1"],
    "decisions": ["16fb3feb-5856-4cbb-a7c2-65125d6650b7", "0f6ebf63-a640-4b8f-80a3-e7fea16e857e"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
    ],
    "read_first": [
      "docs/history/claude-sub-session-dispatch/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D2 and D3: in the runtime-claude native arm of dispatch prepare (prepare.rs:2858, the `_ =>` arm that builds the Agent payload), when the slot resolves a configured model, `dispatch.claude_sessions` in the main checkout's config is not false, and `claude` (or herding.claude_command) resolves on PATH, emit tool Bash with the herding command built the same way as the herding arm (prepare.rs:2619: --task-file - --json, --cwd for the worktree, --cell-id for cells, --read-only for gather and advisor, --ceiling from config) plus `--session-model <model>` and `--session-effort <effort>` when an effort is chosen; stdin is the herding pane prompt; channel `claude-session`; transport_ready true with a reason naming the binary. When the CLI is missing, keep the Agent payload and add fallback_reason `claude_cli_missing`. When the switch is false, the payload is byte-identical to today. A session-model escalation keeps the Agent payload. Codex and Pi arms do not change.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee prepare",
    "must_haves": {
      "truths": [
        "on runtime claude a native-model slot for cell, gather, reviewer and advisor returns a Bash bee herding run command carrying --session-model",
        "dispatch.claude_sessions false returns the Agent payload unchanged",
        "no claude CLI on PATH returns the Agent payload with fallback_reason claude_cli_missing",
        "codex and pi payloads are unchanged"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "substantive": "claude-session arm and its tests"}
      ],
      "key_links": [
        "the emitted command uses the --session-model flag cssd-1 added"
      ],
      "prohibitions": [
        "No code comments",
        "No change to the herding arm for herding-configured slots"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "cssd-3",
    "feature": "claude-sub-session-dispatch",
    "title": "Document sub-session dispatch and steering",
    "lane": "standard",
    "role": "docs",
    "deps": ["cssd-1", "cssd-2"],
    "decisions": ["7fe67d57-462a-4090-a2bb-d0b937e31c0c", "16fb3feb-5856-4cbb-a7c2-65125d6650b7", "0f6ebf63-a640-4b8f-80a3-e7fea16e857e"],
    "files": [
      "packages/bee/AGENTS.block.md",
      "AGENTS.md",
      "skills/bee-swarming/references/swarming-reference.md",
      "docs/knowledge/areas/bee-herding/claude-session-transport.md",
      "docs/knowledge/areas/bee-herding/index.md"
    ],
    "read_first": [
      "docs/history/claude-sub-session-dispatch/CONTEXT.md",
      "packages/bee/AGENTS.block.md"
    ],
    "affects_skills": ["skills/bee-swarming/references/swarming-reference.md"],
    "affects_specs": ["docs/knowledge/areas/bee-herding/claude-session-transport.md"],
    "action": "Per D1-D3: in packages/bee/AGENTS.block.md, extend the 'A leader watches a worker' line: a leader may steer a Claude worker (a claude-session job or a Claude herding pane) with SendMessage to the steer_to name bee herding status prints, and still never reads its transcript. Write the knowledge concept claude-session-transport.md (what it is, the claude --bg argv, bgIsolation none, the poll decisions, steer_to, the dispatch.claude_sessions switch and the PATH fallback, the worker-outward guard reason it spawns inside bee, Pointers to claude_session.rs and prepare.rs) and link it from the area index. Add one line to swarming-reference.md on steering. Run .bee/bin/bee dev regen so AGENTS.md matches.",
    "verify": ".bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "AGENTS.md permits steering a Claude worker with SendMessage and keeps the no-transcript rule"
      ],
      "artifacts": [
        {"path": "docs/knowledge/areas/bee-herding/claude-session-transport.md", "substantive": "concept with Pointers to claude_session.rs and prepare.rs"}
      ],
      "key_links": [
        "AGENTS.md matches the block after bee dev regen"
      ],
      "prohibitions": [
        "Do not edit AGENTS.md by hand outside regen"
      ]
    },
    "behavior_change": false
  }
]
```

## Test matrix

| Row | Case | Pass when |
|---|---|---|
| 1 | happy: fake CLI prints the backgrounded line, agents-json shows the id, mailbox has a result | run outcome is the result; `claude stop <id>` was called once |
| 2 | edge: agents-json shows `status: idle` twice and no result | outcome TimedOutIdle and stop called |
| 3 | error: spawn output has no backgrounded line | outcome SpawnFailed naming the output |
| 4 | error: id leaves the agents list before a result | outcome died, stop not required |
| 5 | edge: --read-only | child env has `BEE_HERDING_READ_ONLY=1` |
| 6 | door happy: native advisor slot on runtime claude | tool Bash, command has `--session-model fable --read-only` |
| 7 | door switch: `dispatch.claude_sessions: false` | payload equals today's Agent payload |
| 8 | door fallback: no claude on PATH | tool Agent, `fallback_reason: claude_cli_missing` |
| 9 | same scenario main vs head: native cell dispatch | main returns tool Agent; head returns tool Bash with `--session-model` |
| 10 | live: `bee herding run --session-model haiku --read-only` with a one-line gather | JSON output carries the worker's result and `steer_to`; `claude agents` no longer lists it as running |

## Open Questions

(none)

## Out of scope

- Session-model escalation stays on the Agent tool (D3).
- Cross-machine and cloud sessions; only local `claude --bg`.
- Choosing models from measured cost data (the other idea from the owner's
  post) — a separate backlog item.
- Fixing issues #164 and #165.
