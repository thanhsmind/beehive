---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: paseo-answers

## Summary

A worker on Paseo that stops to ask a question now keeps its agent. When
the answer comes, bee sends it to that same agent as the next round of the
same job, but only while Paseo shows the agent idle. If the agent is gone
or busy, bee starts a fresh worker as before. `bee herding steer` now
reaches a running Claude, Codex or OpenCode worker on Paseo through the
Paseo daemon; a Pi worker keeps its steer file. Nothing ever cuts into a
running turn.

Mode: `high-risk` — 2 risk flags: external systems (the Paseo daemon WebSocket, a hard-gate flag), multi-domain (Rust and a Node helper)
Why this is the least workflow that protects the work: a wrong send interrupts a worker and loses its work (paseo-pi D5), so each send is gated on Paseo's own state and proven live.

Playbook: `skills/bee-planning/playbooks/feature.md`. Step 1 (data shape) is job.json `transport` and `paseo_agent_id`, which slice 1 already writes. Step 2 is pa-1 to pa-3 driven live. Step 3 holds: one slice. Step 4 is the live question round and the live steer. Step 5 is pa-4.

Deviation (named): the hat wave is not re-run; the paseo-pi wave (`docs/history/paseo-pi/reports/hat-wave.md`, "Next slices", slice 3) is the consult, mapped in `docs/history/paseo-answers/reports/advisor.md`.

## Requirements (from CONTEXT.md)

- D1: a questioning worker keeps its agent; the answer runs as the next round of the same job on that agent, sent only when idle; otherwise a fresh child job.
- D2: steer to a claude, codex or opencode Paseo worker goes through the daemon with `activeTurnBehavior: "steer"`; Pi keeps the steer file; an undeliverable steer refuses; never an interrupting send.
- paseo-pi D5: never interrupt-and-replace a running turn.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Every answer path calls one function | read | packages/bee-rs/crates/bee/src/herding/broker.rs:538 | `let _ = start_child_job_for_answered_question(main_root, &job_id, round, leaning, spawner);` |
| 2 | The spawner always passes a job id and the task to `herding run` | read | packages/bee-rs/crates/bee/src/herding/broker.rs:186 | `cmd.arg("--job-id").arg(&args.job_id);` |
| 3 | `run` sends a continue to `execute_continue` | read | packages/bee-rs/crates/bee/src/herding/run.rs:2333 | `if opts.is_continue {` |
| 4 | `execute_paseo` archives on any valid result, a question included | read | packages/bee-rs/crates/bee/src/herding/run.rs:3444 | `let should_archive = should_close_pane(valid_result, opts.close_always) && !is_own_agent;` |
| 5 | Steer today only writes a steer file | read | packages/bee-rs/crates/bee/src/herding/job_verbs.rs:611 | `let final_file = mbox.join(format!("steer-{next_n}.json"));` |
| 6 | The daemon steer works from Node with the npm CLI's client | ran | paseo-pi spike 2: `node steer.mjs 3317e669-9340-4afe-a7a3-cc9f40c81822` | `steer sent after 2510 ms undefined` |

## Discovery

Read broker.rs, run.rs, job_verbs.rs and the paseo-pi spike notes. A
question ends the worker's round (mailbox-broker D3), so the agent is idle
when an answer arrives: the answer is a normal prompt to an idle agent,
not a steer. A live steer is only needed for a leader's `bee herding steer`
during a running turn.

## Approach

**Recommended path.**

- run.rs: `execute_paseo` keeps the agent when the result is a question
  (it reports the agent id; no archive). A `--continue <job>` whose job.json
  has `transport: "paseo"` goes to a new `execute_continue_paseo`: version
  check; `paseo inspect` must say Idle, else refuse with a FIX line and send
  nothing; write the round N+1 brief from `--task` exactly as
  `execute_continue` does; `paseo send <id> --no-wait <pointer>`; wait for
  `result-(N+1).json` through the paseo wait; same archive rule (keep on a
  question).
- broker.rs: `start_child_job_for_answered_question` takes a
  `&dyn PaseoCli`. When the parent job.json says `transport: "paseo"` with a
  `paseo_agent_id` and inspect says Idle, it spawns
  `bee herding run --continue <parent-job> --task <answer-task>`, where the
  task is the same question-and-answer text, and records the redispatch with `mode: "continue"`; otherwise the existing
  child-job path runs unchanged.
- New `herding/paseo_steer.rs` with an embedded Node script
  (`herding/paseo_steer.mjs`, `include_str!`): resolve the `@getpaseo/cli`
  package dir from the configured paseo command (follow symlinks, walk up
  to the `package.json` named `@getpaseo/cli`), then run
  `node --input-type=module -e <script> <cliDir> <agentId> <text>`; the
  script imports `dist/utils/client.js` and `dist/utils/daemon-target.js`,
  calls `sendAgentMessage(id, text, {activeTurnBehavior: "steer"})`, prints
  `{"ok":true}` or `{"ok":false,"error":...}`.
- job_verbs.rs: `steer_job` on a job with `transport: "paseo"` reads the
  agent's PaseoSpec provider: `pi` keeps the steer file; `claude`, `codex`,
  `opencode` call the helper; any other provider, or a helper failure,
  refuses with a FIX line. It never calls `paseo send`.

**Rejected.**
- Steering the answer into a running turn: there is no running turn after a question.
- A Rust WebSocket client: a pre-1.0 protocol in a new dependency.
- Interrupting send as a fallback: forbidden by paseo-pi D5.

**Risk map.**

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Send into a busy agent | HIGH | pa-1, pa-2 | test: not Idle refuses and makes no send call; broker falls back to a child job |
| Agent archived before the answer | HIGH | pa-1 | test: a question result keeps the agent |
| Duplicate answer (continue and child) | MEDIUM | pa-2 | test: exactly one spawn per answered question |
| Steer lost or interrupting | HIGH | pa-3 | test: no send argv; helper failure refuses; live steer lands |
| herdr, tmux, no-pane change | MEDIUM | pa-1, pa-2, pa-3 | existing herding tests stay green |

Waves: wave 1 runs pa-1, pa-3 and pa-4 in parallel (disjoint files); pa-2 runs after pa-1 because it spawns the continue path pa-1 builds.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"pa-1 to pa-3 change run, the broker and steer."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Each code cell writes its own tests red-first; the leader drives the live runs."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"pa-4 updates the Paseo channel concept."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"required","role":"review","reason":"The slice judge for the behavior cells pa-1, pa-2 and pa-3 dispatches the review role."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The paseo-pi hat wave is the advisor consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Covered by the paseo-pi wave (named deviation)."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Covered by the paseo-pi wave (named deviation)."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Covered by the paseo-pi wave (named deviation)."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Covered by the paseo-pi wave (named deviation)."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Covered by the paseo-pi wave (named deviation)."}
  ]
}
```

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | Answers and steer on Paseo | paseo-pi D5, D1, D2 | a Paseo worker asks, gets answered in the same agent, and finishes; a steer reaches a running Claude worker | the Paseo channel is complete |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pa-1 | Continue a Paseo job in the same idle agent | packages/bee-rs/crates/bee/src/herding/run.rs; packages/bee-rs/crates/bee/src/herding/paseo.rs | — | `bee herding run --continue <paseo job>` runs the next round in the same agent, or refuses when it is busy | herding tests green |
| pa-2 | Send a broker answer to the same Paseo agent | packages/bee-rs/crates/bee/src/herding/broker.rs | pa-1 | an answered question on a Paseo job continues the same agent | herding tests green, then a live question round |
| pa-3 | Steer a running Claude, Codex or OpenCode worker on Paseo | packages/bee-rs/crates/bee/src/herding/paseo_steer.rs (new); packages/bee-rs/crates/bee/src/herding/paseo_steer.mjs (new); packages/bee-rs/crates/bee/src/herding/job_verbs.rs; packages/bee-rs/crates/bee/src/herding.rs | — | `bee herding steer` reaches a running Claude worker on Paseo, or refuses clearly | herding tests green, then a live steer |
| pa-4 | Document answers and steering on Paseo | docs/knowledge/areas/bee-herding/the-paseo-channel.md | — | the concept explains same-agent answers and steer | knowledge check green |

```json
[
  {
    "id": "pa-1",
    "feature": "paseo-answers",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Continue a Paseo job in the same idle agent",
    "deps": [],
    "decisions": ["D1"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs"
    ],
    "read_first": [
      "docs/history/paseo-answers/CONTEXT.md",
      "docs/history/paseo-answers/plan.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "action": "Red first. paseo.rs: add send_argv(id, text) = [\"send\", id, \"--no-wait\", text]. run.rs: (1) in execute_paseo (run.rs:3262) keep the agent when the outcome is a Result whose status is question: no archive call, eprintln that the agent is kept for the answer. (2) Add execute_continue_paseo(opts, cli: &dyn PaseoCli): read job.json; refuse (RunOutcome::SpawnFailed with a FIX line) when transport is not paseo or paseo_agent_id is missing; run the same --version check as execute_paseo; inspect the agent with inspect_argv + parse_inspect and refuse with `paseo agent <id> is not idle — FIX: wait for it to finish, then run bee herding run --continue <job>` unless the state is Idle, making no send call; compute the next round and write its brief from opts.task exactly as execute_continue (run.rs:3602) does (refuse the same way when there is no prior result); call send_argv with the pointer prompt for the next round's brief; update job.json round; wait with wait_for_round_paseo using min_round = next round; read result-(next).json; archive with the same rule as execute_paseo (keep on question, keep on own PASEO_AGENT_ID, keep on failure). (3) In run, when opts.is_continue and the job's job.json has transport paseo, call execute_continue_paseo with RealPaseoCli on the configured paseo command instead of the pane transport; every other continue is unchanged. Tests with a fake PaseoCli: a question result keeps the agent; a continue on an Idle agent sends one prompt naming brief round 2 and returns the round 2 result; a continue on a Working agent refuses with no send; a continue with no paseo_agent_id refuses; an old version refuses before inspect. No code comments.",
    "must_haves": {
      "truths": [
        "a Paseo worker result with status question does not archive the agent",
        "bee herding run --continue on a paseo job sends the round N+1 pointer to the same agent only when it is Idle",
        "a continue on a Working, Blocked, Dead or unknown agent refuses with a FIX line and makes no send call",
        "the continued round's result is read from result-(N+1).json",
        "a continue on a non-paseo job runs the existing execute_continue unchanged and the existing herding tests stay green"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "execute_continue_paseo, its routing, and keep-on-question"},
        {"path": "packages/bee-rs/crates/bee/src/herding/paseo.rs", "substantive": "send_argv"}
      ],
      "key_links": ["run routes a continue on a paseo job to execute_continue_paseo"],
      "prohibitions": ["No code comments", "No send call unless inspect says Idle", "No change to PaneTransport, RealHerdr or RealTmux"]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "pa-2",
    "feature": "paseo-answers",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Send a broker answer to the same Paseo agent",
    "deps": ["pa-1"],
    "decisions": ["D1"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/broker.rs"
    ],
    "read_first": [
      "docs/history/paseo-answers/plan.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/broker.rs",
      "docs/knowledge/areas/bee-herding/the-mailbox-broker.md"
    ],
    "action": "Red first. In broker.rs add `continue_job: Option<String>` to SpawnArgs; RealJobSpawner passes `--continue <id>` instead of `--job-id <id>` when it is set, and still passes --task, --cwd and --main-root (question flags only for the child path). Give start_child_job_for_answered_question (broker.rs:307) a `paseo: &dyn crate::herding::paseo::PaseoCli` parameter. When the parent job.json has transport \"paseo\" and a paseo_agent_id, call inspect on that id; only when parse_inspect says Idle, spawn with continue_job = the parent job id and the same task text the child path builds (the original task plus the question and answer), write the same redispatched-<round>.json record with an added \"mode\": \"continue\", and return the parent job id. Any other state, an inspect error, or a non-paseo parent runs the existing child-job path unchanged. Thread a RealPaseoCli on the configured paseo command (crate::herding::paseo::paseo_command over the main config) through tick_with and answer_with at their real entry points (tick, answer); tests pass a fake. Tests: an Idle paseo parent spawns once with continue_job set and no --job-id; a Working paseo parent spawns one child job; a non-paseo parent is unchanged; exactly one spawn per answered question in each case. No code comments.",
    "must_haves": {
      "truths": [
        "an answered question on a paseo job whose agent is Idle spawns bee herding run --continue on the same job, once",
        "an answered question on a paseo job whose agent is not Idle spawns one fresh child job as before",
        "a non-paseo job's answer path is unchanged",
        "the redispatch record marks mode continue for the same-agent path"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/broker.rs", "substantive": "the same-agent continue branch and the spawner's continue flag"}
      ],
      "key_links": ["start_child_job_for_answered_question inspects the parent paseo agent before choosing continue"],
      "prohibitions": ["No code comments", "No paseo send call in broker.rs"]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "pa-3",
    "feature": "paseo-answers",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Steer a running Claude, Codex or OpenCode worker on Paseo",
    "deps": [],
    "decisions": ["D2"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/paseo_steer.rs",
      "packages/bee-rs/crates/bee/src/herding/paseo_steer.mjs",
      "packages/bee-rs/crates/bee/src/herding/job_verbs.rs",
      "packages/bee-rs/crates/bee/src/herding.rs"
    ],
    "read_first": [
      "docs/history/paseo-answers/CONTEXT.md",
      "docs/history/paseo-answers/plan.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/job_verbs.rs"
    ],
    "action": "Red first. Create herding/paseo_steer.mjs: an ES module script run as `node --input-type=module -e <script> <cliDir> <agentId> <text>`; it imports pathToFileURL(cliDir + '/dist/utils/client.js') for connectToDaemon and pathToFileURL(cliDir + '/dist/utils/daemon-target.js') for selectDaemonTarget, connects with connectToDaemon({ target: selectDaemonTarget({}, process.env, false) }), calls client.sendAgentMessage(agentId, text, { activeTurnBehavior: 'steer' }), closes, and prints {\"ok\":true}; on any error prints {\"ok\":false,\"error\":\"...\"} and exits 1. No comments. Create herding/paseo_steer.rs (declare `pub(crate) mod paseo_steer;` in herding.rs): STEER_SCRIPT = include_str!(\"paseo_steer.mjs\"); `steerable(provider) -> bool` true for claude, codex, opencode; `cli_package_dir(command: &str, path_lookup) -> Option<PathBuf>` resolves the command on PATH when not absolute, canonicalizes symlinks, walks up to a directory whose package.json has name @getpaseo/cli; `steer_argv(cli_dir, agent_id, text) -> Vec<String>` = [\"--input-type=module\", \"-e\", STEER_SCRIPT, cli_dir, agent_id, text]; `trait SteerRunner` with a real impl that runs node and parses the JSON line. In job_verbs.rs steer_job (job_verbs.rs:500), after the existing checks and before the steer file is written: when job.json transport is \"paseo\", read PaseoSpec for job.json's agent from the main config; provider pi keeps today's steer-file path; a steerable provider calls the runner with paseo_agent_id and returns success without writing a steer file; a missing cli dir, a runner error, a missing paseo_agent_id or any other provider returns a JobVerbError with code paseo_steer_failed or paseo_steer_unsupported and a FIX line. Never build a paseo send argv. Unit tests: steerable set; cli_package_dir on a temp tree with a symlinked bin; steer_argv shape; steer_job with a fake runner for claude (runner called once, no steer file), pi (steer file written, runner not called), unknown provider (refused), runner error (refused). No code comments.",
    "must_haves": {
      "truths": [
        "bee herding steer on a paseo job with a claude, codex or opencode provider calls the daemon steer helper and writes no steer file",
        "bee herding steer on a paseo job with a pi provider writes the steer file as today",
        "a helper failure, a missing paseo_agent_id or an unsupported provider refuses with a FIX line",
        "the steer helper sends with activeTurnBehavior steer and no code path builds a paseo send argv",
        "steer on a non-paseo job is unchanged and the existing herding tests stay green"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/paseo_steer.rs", "substantive": "steer helper resolution and runner"},
        {"path": "packages/bee-rs/crates/bee/src/herding/paseo_steer.mjs", "substantive": "the daemon steer script"},
        {"path": "packages/bee-rs/crates/bee/src/herding/job_verbs.rs", "substantive": "the paseo steer branch"},
        {"path": "packages/bee-rs/crates/bee/src/herding.rs", "substantive": "the module declaration"}
      ],
      "key_links": ["steer_job routes a paseo job by provider"],
      "prohibitions": ["No code comments", "No paseo send argv", "No new crate dependency"]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "pa-4",
    "feature": "paseo-answers",
    "lane": "high-risk",
    "role": "docs",
    "title": "Document answers and steering on Paseo",
    "deps": [],
    "decisions": ["D1", "D2"],
    "files": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "read_first": [
      "docs/history/paseo-answers/CONTEXT.md",
      "docs/history/paseo-answers/plan.md",
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "action": "In docs/knowledge/areas/bee-herding/the-paseo-channel.md replace the planned-work line about slice 3 (answers) with a section 'Answers and steering' in the same ASD-STE100 style: a question keeps the agent; the broker sends the answer to the same agent as the next round of the same job only when Paseo shows it idle, else a fresh child job; `bee herding run --continue` on a Paseo job and its refusal when the agent is busy; `bee herding steer` on Paseo: claude, codex, opencode through the daemon helper (needs node and the npm @getpaseo/cli), pi through the steer file, other providers refused; never an interrupting send. Add 'paseo-answers D1, D2' to the frontmatter decisions. Change nothing else.",
    "must_haves": {
      "truths": [
        "the concept states that a question keeps the agent and the answer continues the same job only when the agent is idle",
        "the concept states the fallback to a fresh child job",
        "the concept states the steer routes per provider and the refusal",
        "the frontmatter cites paseo-answers D1, D2"
      ],
      "artifacts": [
        {"path": "docs/knowledge/areas/bee-herding/the-paseo-channel.md", "substantive": "the answers and steering section"}
      ],
      "key_links": ["the section extends the existing concept"],
      "prohibitions": ["No change to any other file"]
    },
    "verify": ".bee/bin/bee knowledge check --json",
    "affects_skills": [],
    "affects_specs": []
  }
]
```

## Test matrix

| Dimension | Probe | Cell | Pass when |
|---|---|---|---|
| State | question result | pa-1 | no archive call |
| Concurrency | continue on a Working agent | pa-1 | refusal with FIX, zero send calls |
| Concurrency | answer while the agent is Working | pa-2 | one child job, no continue |
| Idempotence | one answered question | pa-2 | exactly one spawn |
| External failure | steer helper fails or node is missing | pa-3 | refusal with FIX, no steer file for claude |
| Identity | pi provider steer | pa-3 | steer file written, helper not called |
| Regression | non-paseo continue, answer and steer | pa-1, pa-2, pa-3 | existing herding tests green |
| User-visible path | live: a Pi worker on Paseo asks, `bee herding answer` answers | pa-2 cap | the same agent id runs round 2 and the job ends `done` |
| User-visible path | live: `bee herding steer` on a running Claude worker on Paseo | pa-3 cap | the worker's timeline shows the steer text inside the run |

## Open Questions

(none)

## Out of scope

- Steering a herdr or tmux worker.
- The leader-side `bee_steer` Pi tool text; it calls `bee herding steer` and inherits the change.
