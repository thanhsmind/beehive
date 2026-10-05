---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: paseo-pi

## Summary

Bee can start a herded worker as a Paseo agent instead of a herdr pane. A
team config entry with a `paseo` block names the provider and model, and
`bee herding run` then starts the worker through the Paseo CLI, waits on the
same file mailbox, and archives the agent when the run ends. Herdr and tmux
stay exactly as they are. This slice carries workers only; the leader's
heartbeat tick (slice 2) and answer steering (slice 3) come after it.

Mode: `high-risk` — 3 risk flags: external systems (the Paseo daemon, a hard-gate flag), multi-domain (Rust and the Pi extension), public contracts (a new `herding.agents` config shape)
Why this is the least workflow that protects the work: an outside daemon owns agent state that bee must track and clean up, so the hat wave and a live proof guard the one slice that creates agents.

Playbook: `skills/bee-planning/playbooks/feature.md`. Step 1 (data shape first) is the `paseo` agent block below. Step 2 (walking skeleton) is slice 1. Step 3 (current slice only) holds: slices 2 and 3 are headlines. Step 4 (prove the user path) is the live `bee herding run` proof in pp-2's cap. Step 5 (sync knowledge) is pp-4.

## Requirements (from CONTEXT.md)

- D1: Paseo is a herding channel beside herdr and tmux, chosen per team config. Herdr keeps working unchanged.
- D4: role → team config → herding agent → Paseo provider and model. The dispatch door stays the only model picker.
- D5: an answer steers into a running worker only when its provider can steer; otherwise stop and next round; never interrupt-and-replace. (Slice 3.)
- D6: a Paseo heartbeat wakes the Pi leader; the extension runs the code tick and turns the heartbeat into a news prompt or one short turn; never swallows it. (Slice 2.)

## Load-bearing claims

Labels are `read` (the file opened at that line), `ran` (the command run, output kept) or `guessed`. Evidence is a verbatim byte substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | A worker path with no pane already exists to copy | read | packages/bee-rs/crates/bee/src/herding/run.rs:3005 | `pub(super) fn execute_no_pane(opts: &Options) -> ExecResult {` |
| 2 | `run` chooses the no-pane path before it builds a pane transport | read | packages/bee-rs/crates/bee/src/herding/run.rs:4286 | `let (transport, transport_name) = if opts.no_pane {` |
| 3 | `run` calls the chosen executor in one place | read | packages/bee-rs/crates/bee/src/herding/run.rs:4315 | `let result = if opts.no_pane {` |
| 4 | One rule decides when a worker is closed | read | packages/bee-rs/crates/bee/src/herding/run.rs:1226 | `fn should_close_pane(valid_result: bool, close_always: bool) -> bool {` |
| 5 | The child env builder is shared | read | packages/bee-rs/crates/bee/src/herding/run.rs:2843 | `pub(crate) fn build_child_env(` |
| 6 | `herding.transport` is one repo-wide key, so per-agent choice needs its own field | read | packages/bee-rs/crates/bee/src/herding.rs:635 | `h.get("transport")) {` |
| 7 | `dispatch prepare` decides transport readiness from that repo-wide key | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2573 | `let (transport_ready, transport_reason, _) = match transport_kind_at(root) {` |
| 8 | `paseo run --json` returns the new agent id | ran | `paseo run -d --provider pi --title spike-hb --cwd <sbx> --env SPIKE_LOG=<log> --json "Reply with the single word READY."` (npm @getpaseo/cli 0.10.3) | `"agentId": "f236f8a4-43c2-4411-b2a4-3a6b2607c32e",` |
| 9 | `--env` reaches the agent process, and Paseo sets `PASEO_AGENT_ID` | ran | same run; the extension wrote to the `SPIKE_LOG` path | `{"ev":"session_start","agent":"f236f8a4-43c2-4411-b2a4-3a6b2607c32e"}` |
| 10 | `paseo inspect --json` uses capitalized keys and a `Status` field | ran | `paseo inspect 3317e669-9340-4afe-a7a3-cc9f40c81822 --json` | `"Status": "closed",` |

## Discovery

Two live spikes on 2026-10-05 (decisions tagged `paseo-pi`, `spike`) and a
5-seat hat wave. Spike 2 used the npm `@getpaseo/cli` 0.10.3 because the
AppImage CLI cannot start the daemon. The scout found that `PaneTransport`
(run.rs:508) is pane geometry with no Paseo meaning for half its methods,
while `execute_no_pane` (claim 1) already spawns a process, polls the file
mailbox, and reads the result.

## Approach

**Recommended path.** A separate Paseo executor beside `execute_no_pane`, not
a third `PaneTransport` (D1). A `herding.agents.<name>` entry that is an
object with a `paseo` block selects it, so the choice rides the team config
(D1, D4) and the repo-wide `herding.transport` key is untouched. The block:

```json
{"paseo": {"provider": "pi", "model": "openrouter/~deepseek/deepseek-flash-latest", "thinking": "high", "mode": null}}
```

`provider` is required; `model`, `thinking` and `mode` are optional. An
optional `herding.paseo.command` names the CLI (default `paseo`). The
executor runs `paseo run -d --json` with `--provider <provider>[/<model>]`,
`--thinking`, `--mode`, `--cwd`, `--title <job id>`, `--label bee_job=<job id>`,
one `--env` per child-env entry plus `BEE_HERDING_WORKER=1` and
`BEE_HERDING_JOB_ID`, and the pointer prompt as the initial prompt. It writes
the returned agent id into `job.json` (`paseo_agent_id`) at once, then waits
on the file mailbox exactly like the no-pane path. Liveness comes from
`paseo inspect --json`: `running` is working, `idle` is idle, a non-empty
`PendingPermissions` is blocked, `error` and `closed` are dead, and a failed
inspect is unknown and never counts as alive. The result comes only from
`result-N.json`. On exit it runs `paseo archive --force <id>` only when
`should_close_pane` says so, never on an id equal to its own
`PASEO_AGENT_ID`, and keeps the agent on failure for inspection. Before the
run it checks `paseo --version` is 0.10.3 or newer and refuses with a FIX line
otherwise; a failed `paseo run` refuses with a FIX line that names
`paseo daemon start` from the npm `@getpaseo/cli`. The executor never calls
`paseo send` in this slice.

**Rejected.**
- A third `PaneTransport` implementer: about eight fake methods and the split-lock baggage, and fakes hide bugs.
- Spawning from the Pi extension: works only with a Pi leader and splits the job.json writer into two homes.
- A Paseo plugin (the Seatworks way): a second runtime and install for nothing the CLI cannot do.
- A Rust WebSocket client: a pre-1.0 protocol in a new dependency; slice 3 uses a small Node helper only where it must.

**Risk map.**

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Config parse and argv build | LOW | pp-1 | unit tests over the parse and the argv |
| Liveness mapping fails closed | MEDIUM | pp-1 | unit tests: inspect error and unknown status are not alive |
| Orphan agents | MEDIUM | pp-2 | job.json carries `paseo_agent_id`; every run carries `--label bee_job=<id>` |
| Archive deletes the wrong agent | HIGH | pp-2 | test: never archives its own `PASEO_AGENT_ID`; archives only under `should_close_pane` |
| Herdr and tmux drift | MEDIUM | pp-2 | the existing herding tests stay green unchanged |
| Dispatch says "not ready" in a Paseo session | MEDIUM | pp-3 | test: a paseo agent is ready when the CLI resolves, whatever `herding.transport` says |
| Daemon down or old | MEDIUM | pp-2 | test: version below 0.10.3 and a failed run both refuse with a FIX line |

Waves: wave 1 runs pp-1 and pp-4 in parallel (disjoint files). Wave 2 runs
pp-2 and pp-3 in parallel after pp-1, because both call its functions and
touch disjoint files.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"pp-1 to pp-3 add the Paseo module, the executor and the readiness probe."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Each code cell writes its own tests red-first; the leader drives the live run."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"pp-4 writes the Paseo channel concept."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"required","role":"review","reason":"The slice judge for the behavior_change cell pp-2 dispatches the review role; the user-invoked review session is untouched."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the advisor consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape, chosen by the hat wave."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape, chosen by the hat wave."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape, chosen by the hat wave."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Plan-step hat wave, high-risk five seats."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"Plan-step hat wave, high-risk five seats."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"Plan-step hat wave, high-risk five seats."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Plan-step hat wave, high-risk five seats."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Plan-step hat wave, high-risk five seats."}
  ]
}
```

## Shape

Epic map. Outcome: bee runs on Paseo with a Pi leader. Repo basis: the
no-pane executor, the file mailbox, the broker, and the bee-guard extension.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Worker on Paseo | start, watch, collect, archive a worker as a Paseo agent | D1, D4 | slice 1 (current) | live `bee herding run` with a paseo agent returns `outcome done` and the agent is archived |
| Leader heartbeat | the bee extension turns each Paseo heartbeat into a news prompt or one short turn | D6 | slice 2 | live heartbeat fires twice and Paseo returns the leader to idle each time |
| Answers | an answer reaches the same Paseo worker with no interrupt | D5 | slice 3 | live question round on a Pi and a Claude worker |

Slice queue: slice 1 → slice 2 → slice 3. Slice 2 and slice 3 do not depend
on each other once slice 1 lands.

- Slice 2 headline: in `.pi/extensions/bee-guard/events.ts` `input`, match a `<paseo-system>` heartbeat, run the bee code tick, return a `transform` to the news or to one minimal turn; run only in the leader, never in a worker; exempt that turn from the continuation nudge.
- Slice 3 headline: a question ends the round, so the answer goes to the same idle Paseo agent through `paseo wait` then `paseo send`; Pi workers keep the existing `steer-N.json` drain; a Node helper with the daemon WebSocket only for mid-turn steering of a non-Pi worker; never fall back to an interrupting send.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pp-1 | Add the Paseo agent block, CLI argv and status reading | packages/bee-rs/crates/bee/src/herding/paseo.rs (new); packages/bee-rs/crates/bee/src/herding.rs | — | nothing yet; unblocks pp-2 and pp-3 | herding::paseo unit tests green |
| pp-2 | Run a herded worker as a Paseo agent | packages/bee-rs/crates/bee/src/herding/run.rs | pp-1 | `bee herding run --agent <paseo agent>` starts a Paseo agent, returns its result, and archives it | herding tests green, then a live run |
| pp-3 | Report a Paseo agent as ready in dispatch prepare | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs | pp-1 | `bee dispatch prepare` for a paseo-bound role shows `transport_ready: true` when `paseo` resolves | prepare tests green |
| pp-4 | Write the Paseo channel concept | docs/knowledge/areas/bee-herding/the-paseo-channel.md (new); docs/knowledge/areas/bee-herding/index.md | — | the knowledge bundle explains the paseo block, its states and its FIX lines | knowledge check green |

```json
[
  {
    "id": "pp-1",
    "feature": "paseo-pi",
    "lane": "high-risk",
    "role": "code",
    "change_class": "api",
    "title": "Add the Paseo agent block, CLI argv and status reading",
    "deps": [],
    "decisions": ["D1", "D4"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding.rs"
    ],
    "read_first": [
      "docs/history/paseo-pi/CONTEXT.md",
      "docs/history/paseo-pi/plan.md",
      "packages/bee-rs/crates/bee/src/herding.rs",
      "packages/bee-rs/crates/bee/src/herding/wave.rs"
    ],
    "action": "Red first. Create packages/bee-rs/crates/bee/src/herding/paseo.rs and declare it with `pub(crate) mod paseo;` in herding.rs beside the other herding modules. Pure functions plus one injectable runner, all unit-tested in a #[cfg(test)] mod tests inside paseo.rs: (1) `PaseoSpec::from_config(cfg: &Value, agent: &str) -> Option<Result<PaseoSpec, String>>` reads `herding.agents.<agent>.paseo` when the entry is an object carrying a `paseo` object: `provider` required non-empty string; `model`, `thinking`, `mode` optional strings; None when the entry has no `paseo` block; Err naming the agent when `provider` is missing or not a string. (2) `paseo_command(cfg) -> String` reads `herding.paseo.command`, default `paseo`. (3) `run_argv(spec, job_id, cwd, env: &[(String,String)], prompt) -> Vec<String>` builds `run -d --json --provider <provider>[/<model>] [--thinking t] [--mode m] --cwd <cwd> --title <job_id> --label bee_job=<job_id> --env K=V ... <prompt>` in that order, one --env per pair. (4) `parse_run_agent_id(stdout) -> Result<String,String>` reads `agentId` from the first JSON object in stdout (Paseo prints lines before the JSON). (5) `PaseoState { Working, Idle, Blocked, Dead }` and `parse_inspect(stdout) -> Option<PaseoState>`: key `Status` (capitalized): running → Working unless `PendingPermissions` is a non-empty array → Blocked; idle → Idle; error and closed → Dead; any other value or unparseable body → None. (6) `version_at_least(output, (0,10,3)) -> bool` parses the last semver-looking token of `paseo --version` output; false when none parses. (7) `inspect_argv(id)`, `archive_argv(id)` = `archive --force <id>`, `logs_argv(id)`. (8) `trait PaseoCli { fn call(&self, args: &[String]) -> Result<String, String>; }` and `RealPaseoCli { command: String }` that runs the command with stdin null and returns stdout on exit 0, else Err with stderr. No code comments.",
    "must_haves": {
      "truths": [
        "an agent entry with a paseo block parses into PaseoSpec with provider, model, thinking and mode",
        "an agent entry with no paseo block returns None",
        "a paseo block without provider returns Err naming the agent",
        "run_argv joins provider and model with a slash and emits one --env per pair and the bee_job label",
        "parse_run_agent_id reads agentId after leading non-JSON lines",
        "parse_inspect maps running, idle, error and closed, maps non-empty PendingPermissions to Blocked, and returns None for an unknown status or bad body",
        "version_at_least accepts 0.10.3 and newer and refuses 0.6.1 and unparseable output"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/paseo.rs", "substantive": "PaseoSpec, argv builders, parse_inspect, version_at_least, PaseoCli"},
        {"path": "packages/bee-rs/crates/bee/src/herding.rs", "substantive": "the paseo module declaration"}
      ],
      "key_links": [
        "PaseoSpec::from_config reads herding.agents.<name>.paseo"
      ],
      "prohibitions": [
        "No code comments",
        "No change to TransportKind or herding.transport",
        "No paseo send argv anywhere"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding::paseo",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "pp-2",
    "feature": "paseo-pi",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior_change",
    "title": "Run a herded worker as a Paseo agent",
    "deps": ["pp-1"],
    "decisions": ["D1", "D4"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "read_first": [
      "docs/history/paseo-pi/plan.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "action": "Red first. In run.rs add `execute_paseo(opts, spec, cli: &dyn PaseoCli) -> ExecResult`, shaped like `execute_no_pane` (run.rs:3005): write the inbox marker and job.json the same way plus `transport: \"paseo\"`; render the same brief; build env from `build_child_env` plus BEE_HERDING_WORKER=1 and BEE_HERDING_JOB_ID; before spawning, call `--version` and refuse SpawnFailed with a message containing `FIX:` and `0.10.3` when `version_at_least` is false; call `run_argv` with the same pointer prompt the pane path delivers; on Err refuse SpawnFailed with `FIX: start the daemon with paseo daemon start (npm @getpaseo/cli 0.10.3+)`; parse the agent id and write `paseo_agent_id` into job.json at once; wait for the round through the same file-mailbox wait the no-pane path uses, with liveness from `inspect_argv` + `parse_inspect` (None never counts as alive); read the result only from result-N.json; on exit call `archive_argv` only when `should_close_pane` (run.rs:1226) is true and the id differs from env PASEO_AGENT_ID, else keep the agent and report its id. In `run` (run.rs:4286 and 4315), when not no_pane and `PaseoSpec::from_config(read_main_config(main_root), agent)` is Some(Ok), skip the pane transport and call execute_paseo with RealPaseoCli; Some(Err) refuses before any job file; None keeps today's path byte-identical. Tests with a fake PaseoCli: happy path archives; failed result keeps; own PASEO_AGENT_ID is never archived; old version refuses before run; failed run refuses with FIX; job.json carries paseo_agent_id; an agent with no paseo block takes the old path. No code comments; no `send` call.",
    "must_haves": {
      "truths": [
        "a run whose agent entry has a paseo block starts the worker through paseo run and never through a pane",
        "job.json records transport paseo and paseo_agent_id before the wait begins",
        "the result is read only from result-N.json",
        "the agent is archived only when should_close_pane is true and never when its id equals PASEO_AGENT_ID",
        "a paseo version below 0.10.3 refuses with a FIX line before any agent is created",
        "a failed paseo run refuses with a FIX line naming paseo daemon start",
        "an agent entry with no paseo block runs the existing herdr, tmux or no-pane path unchanged and the existing herding tests stay green"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "execute_paseo and its routing in run"}
      ],
      "key_links": [
        "run routes to execute_paseo when PaseoSpec::from_config returns Some(Ok)"
      ],
      "prohibitions": [
        "No code comments",
        "No paseo send call",
        "No change to PaneTransport, RealHerdr or RealTmux"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "pp-3",
    "feature": "paseo-pi",
    "lane": "high-risk",
    "role": "code",
    "change_class": "api",
    "title": "Report a Paseo agent as ready in dispatch prepare",
    "deps": ["pp-1"],
    "decisions": ["D1", "D4"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
    ],
    "read_first": [
      "docs/history/paseo-pi/plan.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
    ],
    "action": "Red first. In prepare.rs where the herding payload sets transport_ready (prepare.rs:2573), when the resolved herding agent's entry has a paseo block (`crate::herding::paseo::PaseoSpec::from_config`), set transport_ready true with reason `paseo agent <name>: <command> on PATH` when the configured paseo command resolves on PATH, else false with reason `paseo agent <name>: <command> not found — FIX: npm i -g @getpaseo/cli`; a Some(Err) block is not ready with its message. Every other agent keeps today's probe byte-identical. Unit tests beside the existing prepare tests. No code comments.",
    "must_haves": {
      "truths": [
        "a paseo-bound herding role is transport_ready true when the paseo command resolves on PATH, whatever herding.transport says",
        "a paseo-bound role is not ready with a FIX reason when the command does not resolve",
        "a role bound to a non-paseo herding agent gets the same transport_ready and reason as before"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "substantive": "the paseo arm of the readiness probe"}
      ],
      "key_links": [
        "prepare calls PaseoSpec::from_config for the resolved herding agent"
      ],
      "prohibitions": [
        "No code comments",
        "The returned command string stays byte-identical"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee prepare",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "pp-4",
    "feature": "paseo-pi",
    "lane": "high-risk",
    "role": "docs",
    "change_class": "docs",
    "title": "Write the Paseo channel concept",
    "deps": [],
    "decisions": ["D1", "D4", "D5", "D6"],
    "files": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md",
      "docs/knowledge/areas/bee-herding/index.md"
    ],
    "read_first": [
      "docs/history/paseo-pi/CONTEXT.md",
      "docs/history/paseo-pi/plan.md",
      "docs/knowledge/areas/bee-herding/the-mailbox-broker.md",
      "docs/knowledge/areas/bee-herding/index.md"
    ],
    "action": "Write docs/knowledge/areas/bee-herding/the-paseo-channel.md with the same frontmatter shape as the-mailbox-broker.md (type bee.area, id bee-herding-the-paseo-channel, areas [bee-herding], decisions citing paseo-pi D1, D4, D5, D6 by name, owns.code naming packages/bee-rs/crates/bee/src/herding/paseo.rs and herding/run.rs). Body in ASD-STE100 style, short sections: what the channel is (D1); the herding.agents.<name>.paseo block and herding.paseo.command, with one JSON example; how a run goes (paseo run, job.json paseo_agent_id, the file mailbox, the bee_job label); the liveness states and what Unknown means; when the agent is archived and when it is kept; the two FIX lines (version below 0.10.3, daemon down); the environment facts from CONTEXT.md (npm CLI starts the daemon, the AppImage CLI cannot); and one line each pointing at slice 2 (D6 heartbeat) and slice 3 (D5 answers) as planned work. Add one line for the new concept to docs/knowledge/areas/bee-herding/index.md in the same style as its other entries.",
    "must_haves": {
      "truths": [
        "the concept names the paseo block, its fields and its default command",
        "the concept states the liveness mapping and that Unknown never counts as alive",
        "the concept states the archive rule and the own-id guard",
        "the concept gives both FIX lines",
        "the bee-herding index links the new concept"
      ],
      "artifacts": [
        {"path": "docs/knowledge/areas/bee-herding/the-paseo-channel.md", "substantive": "the Paseo channel concept"},
        {"path": "docs/knowledge/areas/bee-herding/index.md", "substantive": "the index entry"}
      ],
      "key_links": [
        "index.md links the-paseo-channel.md"
      ],
      "prohibitions": [
        "No change to any other concept file"
      ]
    },
    "verify": ".bee/bin/bee knowledge check --json",
    "affects_skills": [],
    "affects_specs": []
  }
]
```

## Test matrix

High-risk: the applicable edge dimensions, each mapped to a cell truth.

| Dimension | Probe | Cell | Pass when |
|---|---|---|---|
| Input extremes | paseo block with no provider, provider not a string | pp-1 | `from_config` returns Err naming the agent |
| Input extremes | inspect body with unknown Status or not JSON | pp-1 | `parse_inspect` returns None |
| Version and compatibility | `paseo --version` prints 0.6.1 | pp-1, pp-2 | the run refuses with a line containing `FIX:` and `0.10.3` and no agent is created |
| External failure | `paseo run` exits non-zero (daemon down) | pp-2 | SpawnFailed with `FIX: start the daemon with paseo daemon start` |
| State and lifecycle | worker writes a valid result | pp-2 | outcome done and one `archive --force <id>` call |
| State and lifecycle | worker fails or times out | pp-2 | no archive call; the agent id is reported |
| Authorization and identity | the agent id equals env PASEO_AGENT_ID | pp-2 | no archive call |
| Concurrency and orphans | run returns an id | pp-2 | job.json holds `paseo_agent_id` before the wait; the argv holds `--label bee_job=<job id>` |
| Regression | an agent with no paseo block | pp-2, pp-3 | existing herding and prepare tests pass unchanged |
| Environment | paseo command not on PATH | pp-3 | transport_ready false with a FIX reason |
| User-visible path | live run on a paseo agent | pp-2 cap | `bee herding run --agent <paseo agent>` prints `outcome done` and `paseo inspect <id>` shows Archived true |

## Open Questions

- Slice 2: which bee command is the code tick the heartbeat runs (the Rust broker tick plus the TS result drain), and how the extension knows it is the leader and not a worker that loaded the same extension.
- Slice 3: an answer to the same idle Paseo agent through `paseo wait` then `paseo send` reads D5's "next round" as "next prompt to the same agent"; confirm at slice 3 shaping.
- An orphan sweep that finds agents by the `bee_job` label is not in slice 1; slice 1 only makes every agent findable.

## Out of scope

- The cockpit pane verbs, waves and `herding.transport` stay herdr and tmux only.
- Changing the user's `~/.local/bin/paseo` wrapper (the user owns it).
- A Paseo plugin and a Rust WebSocket client.
