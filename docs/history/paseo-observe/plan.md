---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: paseo-observe

## Summary

bee's watch and control verbs now see and act on Paseo workers. `status`
shows each Paseo worker's real state, including "blocked" when it waits on
a permission and which tools it asks for. A blocked worker keeps its turn;
the request is filed for a person, who answers with a new `bee herding
permit` verb, and the same turn goes on. `interrupt` stops a Paseo turn,
`cancel` stops and archives the agent, `occupancy` counts Paseo workers,
`pane read` shows a Paseo worker's log tail, and a timeout message carries
that tail. Herdr and tmux work as before.

Mode: `high-risk` — 3 risk flags: external systems (Paseo daemon, a hard-gate flag), public contracts (a new verb and new status fields), multi-domain (herding, supervisor, registry)
Why this is the least workflow that protects the work: cancel and permit change live agents another session may own, so each write is guarded and proven live.

Playbook: `skills/bee-planning/playbooks/feature.md`. Step 1 (data shape) is the status row fields below. Step 2 is po-1 plus the live blocked-permission proof. Step 3 holds: one slice. Step 4 is the live proof on po-3's cap. Step 5 is po-5.

Deviation (named): no hat wave. An advisor-tier inventory (`reports/advisor.md`) is the consult; the paseo-pi wave's risk findings (archive guard, fail closed) carry over as prohibitions.

## Requirements (from CONTEXT.md)

- D1: a blocked Paseo worker keeps its turn; status shows blocked and the tools; it is filed as a human decision; `bee herding permit` answers it; the same turn goes on.
- D2: cancel stops and archives (never the caller's own agent); interrupt stops the turn and keeps the agent; untracked labelled agents are only reported.
- paseo-pi D5: no interrupting send except these explicit stop verbs.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Status reads only `pane_id` from job.json | read | packages/bee-rs/crates/bee/src/herding.rs:915 | `let pane_id = job_obj` |
| 2 | The orphan sweep skips a job with no `pane_id` | read | packages/bee-rs/crates/bee/src/herding/mailbox.rs:934 | `let pane_id = match obj.get("pane_id").and_then(Value::as_str) {` |
| 3 | Pane verbs build only a herdr or tmux transport | read | packages/bee-rs/crates/bee/src/herding/pane_verbs.rs:162 | `pub(crate) fn cockpit_transport_for(main_root: &Path) -> Result<Box<dyn CockpitTransport>, String> {` |
| 4 | A blocked observation ends the run | read | packages/bee-rs/crates/bee/src/herding/run.rs:1269 | `if observed.blocked && !observed.result_ready {` |
| 5 | The supervisor may run only an allowlist of verbs | read | packages/bee-rs/crates/bee/src/herding/control_loop.rs:324 | `const SUPERVISOR_ALLOWED_TOOLS: &str = "Bash(.bee/bin/bee status:*),\` |
| 6 | Human-decision kinds are a fixed list | read | packages/bee-rs/crates/bee/src/verbs/supervisor.rs:283 | `[WAITING_ON_GATE, WAITING_ON_QUESTION, "escalation", "urgent", ADVISOR_NUDGE_KIND];` |
| 7 | Paseo can answer a permission from the CLI | ran | `paseo permit --help` (npm @getpaseo/cli 0.10.3) | `Manage permission requests` |
| 8 | Paseo stop interrupts only a running agent | ran | `paseo stop --help` | `Interrupt an agent if it is running (no-op for idle agents)` |

## Discovery

The advisor inventory (`reports/advisor.md`) read status, pane, wave,
mailbox, job, run, control-loop and supervisor code plus the Paseo CLI
commands. Root cause: a Paseo job carries `paseo_agent_id`, not `pane_id`,
and writes no wave-ledger row.

## Approach

**Recommended path.** Every verb branches on job.json `transport` first:
`paseo` takes the new arm, anything else runs today's code unchanged.
Paseo state always comes from `paseo inspect` through `parse_inspect`; a
failed inspect is unknown, never alive and never dead.

- Status row additions for a Paseo job: `transport`, `paseo_agent_id`,
  `paseo_state` (working, idle, blocked, dead, unknown) and, when blocked,
  `permissions` (request id and tool name each). The orphan sweep marks a
  Paseo job interrupted only when inspect says Dead (or the agent is
  missing from the labelled list) and it has no result and no mark. Status
  also lists `untracked_paseo_agents`: labelled agents whose job is gone or
  finished. Nothing is archived by the sweep (D2).
- Interrupt runs `paseo stop`; cancel writes the cancelled mark, then
  `paseo stop`, then `paseo archive --force`, never on the caller's own
  `PASEO_AGENT_ID`. A new verb `bee herding permit` takes a job id and
  allow or deny, with optional request id and `--all`, and runs
  `paseo permit allow|deny`. It refuses a non-Paseo job.
- `run`: the Paseo poll treats Blocked as waiting, not done: the idle
  timeout still applies, and the run never ends on Blocked alone (D1). A
  ledger row is written with the agent id. A non-result exit appends
  `paseo logs <id> --tail 40` to the message, and the text says
  "agent kept for inspection".
- Occupancy adds labelled, non-archived Paseo agents to the live set.
- `pane read` on a Paseo job id or agent id shows `paseo logs --tail`.
  The supervisor allowlist gains status. The broker tick files a blocked
  permission once per request id as a human decision of the new kind
  `permission`, so the leader heartbeat sees it as news.

**Rejected.**
- A Paseo `CockpitTransport`: pane geometry has no Paseo meaning (paseo-pi plan).
- Auto-answering permissions: the user chose a person decides (D1).
- Auto-archiving untracked agents: D2.

**Risk map.**

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Cancel archives the wrong agent | HIGH | po-2 | test: own `PASEO_AGENT_ID` never archived; non-Paseo job unchanged |
| Permit on the wrong agent | HIGH | po-2 | test: permit refuses a non-Paseo job and uses the job's own agent id |
| Sweep marks a live agent | HIGH | po-1 | test: unknown inspect never marks; only Dead or missing does |
| Blocked worker ends the run | MEDIUM | po-3 | test: Blocked then a result returns the result |
| Over-spawn | MEDIUM | po-3 | test: a labelled Paseo agent counts in the live set |
| Herdr and tmux drift | MEDIUM | all | existing herding, supervisor and registry tests stay green |

Waves: wave 1 runs po-1 and po-5. Wave 2 runs po-2, po-3 and po-4 in
parallel after po-1: they call its `paseo.rs` helpers and own disjoint files
(po-2 takes `herding.rs` after po-1 is capped).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"po-1 to po-4 change status, control verbs, run, occupancy, pane read, the broker and the supervisor list."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Each code cell writes its own tests red-first; the leader drives the live proof."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"po-5 updates the Paseo channel and supervisor concepts."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"required","role":"review","reason":"The slice judge for the behavior cells dispatches the review role."},
    {"stage":"generic-advisor","classification":"required","role":"advisor","reason":"The advisor inventory is the plan consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Advisor inventory instead (named deviation)."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Advisor inventory instead (named deviation)."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Advisor inventory instead (named deviation)."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Advisor inventory instead (named deviation)."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Advisor inventory instead (named deviation)."}
  ]
}
```

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | Paseo workers in watch and control verbs | D1, D2 | status shows a blocked Paseo worker with its tool; `bee herding permit` lets it go on; cancel archives it | an orchestrator runs Paseo workers with the same control it has over panes |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| po-1 | Show Paseo worker state in status and sweep Paseo orphans | packages/bee-rs/crates/bee/src/herding/paseo.rs; packages/bee-rs/crates/bee/src/herding.rs; packages/bee-rs/crates/bee/src/herding/mailbox.rs | — | `bee herding status` shows working, idle, blocked (with tools) or dead for a Paseo worker | herding tests green |
| po-2 | Interrupt, cancel and answer permissions for Paseo jobs | packages/bee-rs/crates/bee/src/herding/job_verbs.rs; packages/bee-rs/crates/bee/src/catalog.rs; packages/bee-rs/crates/bee/src/generated/registry_payload.json; packages/bee-rs/crates/bee/src/herding.rs | po-1 | interrupt stops a Paseo turn, cancel archives the agent, and the new permit verb answers a permission | herding and registry tests green |
| po-3 | Keep a blocked Paseo worker waiting, count it, and attach its log tail | packages/bee-rs/crates/bee/src/herding/run.rs; packages/bee-rs/crates/bee/src/herding/wave.rs | po-1 | a blocked worker waits for an answer; occupancy counts Paseo workers; a timeout shows the agent's last log lines | herding tests green, then a live run |
| po-4 | Let the observer read Paseo workers and file blocked permissions | packages/bee-rs/crates/bee/src/herding/pane_verbs.rs; packages/bee-rs/crates/bee/src/herding/control_loop.rs; packages/bee-rs/crates/bee/src/herding/broker.rs; packages/bee-rs/crates/bee/src/verbs/supervisor.rs | po-1 | pane read shows a Paseo worker's log tail; a blocked permission appears in the human-decision queue | herding and supervisor tests green |
| po-5 | Document Paseo observation and control | docs/knowledge/areas/bee-herding/the-paseo-channel.md; docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md | — | the concepts explain the Paseo status fields, the verbs and the permission flow | knowledge check green |

```json
[
  {
    "id": "po-1",
    "feature": "paseo-observe",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Show Paseo worker state in status and sweep Paseo orphans",
    "deps": [],
    "decisions": ["D1", "D2"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding.rs",
      "packages/bee-rs/crates/bee/src/herding/mailbox.rs"
    ],
    "read_first": [
      "docs/history/paseo-observe/CONTEXT.md",
      "docs/history/paseo-observe/plan.md",
      "docs/history/paseo-observe/reports/advisor.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding.rs",
      "packages/bee-rs/crates/bee/src/herding/mailbox.rs"
    ],
    "action": "Red first. paseo.rs: add stop_argv(id) = [stop, id]; ls_label_argv(key) = [ls, --label, key, --json] (the label filter with a key only, plus the all-directories flag if the CLI needs it, checked against paseo ls --help); logs_tail_argv(id, n, filter: Option<&str>) = [logs, id, --tail, n] plus [--filter, f] when given; permit_argv(agent, allow: bool, request: Option<&str>, all: bool) = [permit, allow|deny, agent] plus the request id or --all; parse_ls_agents(stdout) -> Vec<(id, label bee_job value, archived)> reading the first JSON value (array or object with an array), tolerant of capitalized keys like inspect; parse_pending_permissions(stdout) -> Vec<(request id, tool name)> from inspect's PendingPermissions (keys id or Id, and tool, name or toolName; keep unknown shapes as id only). herding.rs status: for a job whose job.json transport is paseo, take paseo_agent_id, run inspect through RealPaseoCli on the configured command, and add to its row transport, paseo_agent_id, paseo_state (working, idle, blocked, dead or unknown, from parse_inspect; a failed inspect is unknown) and, when blocked, permissions. A Paseo job is never stalled while paseo_state is working or blocked. mailbox.rs: give mark_orphans a Paseo arm that marks a Paseo job interrupted only when its state is dead or its agent is missing from the labelled list, it has no result for its round and no mark; an unknown state never marks. Status JSON gains untracked_paseo_agents: labelled, non-archived agents whose bee_job has no mailbox, or whose job has a final result or a mark; report only, never archive. Put the Paseo calls behind the PaseoCli trait so tests use a fake. Herdr and tmux rows stay byte-identical. No code comments.",
    "must_haves": {
      "truths": [
        "a Paseo job's status row shows transport, paseo_agent_id and paseo_state working, idle, blocked, dead or unknown",
        "a blocked Paseo job's row lists each pending permission's request id and tool",
        "a failed inspect shows unknown and never marks the job interrupted",
        "the orphan sweep marks a Paseo job interrupted only when its agent is dead or missing and it has no result and no mark",
        "status lists untracked labelled Paseo agents and archives nothing",
        "herdr and tmux status rows and sweeps are unchanged and the existing herding tests stay green"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/paseo.rs", "substantive": "stop, ls, logs-tail and permit argv and the ls and permission parsers"},
        {"path": "packages/bee-rs/crates/bee/src/herding.rs", "substantive": "the Paseo arm of status"},
        {"path": "packages/bee-rs/crates/bee/src/herding/mailbox.rs", "substantive": "the Paseo arm of the orphan sweep"}
      ],
      "key_links": ["status branches on job.json transport before reading pane_id"],
      "prohibitions": ["No code comments", "No archive call anywhere in status or the sweep", "No change to herdr or tmux output"]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "po-2",
    "feature": "paseo-observe",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Interrupt, cancel and answer permissions for Paseo jobs",
    "deps": ["po-1"],
    "decisions": ["D1", "D2"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/job_verbs.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/herding.rs"
    ],
    "read_first": [
      "docs/history/paseo-observe/plan.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/job_verbs.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs",
      "packages/bee-rs/crates/bee/src/herding.rs"
    ],
    "action": "Red first. job_verbs.rs: interrupt and cancel read job.json first; when transport is paseo they never build a pane transport. Interrupt calls stop_argv on the job's paseo_agent_id and writes the interrupted mark the same way the pane path does. Cancel writes the cancel marks the same way, then stop_argv, then archive_argv, but skips both calls when the id equals the env PASEO_AGENT_ID and says so. Add permit_job(main_root, job, allow, request, all, cli): refuse a missing job, a non-Paseo job or a missing paseo_agent_id with a FIX line; otherwise call permit_argv with the job's own agent id and report the CLI output. Add the CLI entry for the new verb `bee herding permit` (positional job id and allow or deny, flags request, all, json, main-root) and route the subverb permit to it in the herding dispatch match in herding.rs (po-1 has finished with that file by then). Declare the verb in catalog.rs and in generated/registry_payload.json in the same shape as the herding.steer entry, with a first example that runs cleanly in the registry dispatch test. Tests with a fake PaseoCli: interrupt calls stop and keeps the agent; cancel calls stop then archive; cancel never archives the caller's own agent; permit sends allow or deny with the job's agent id; permit refuses a herdr job; a herdr interrupt and cancel are unchanged. No code comments.",
    "must_haves": {
      "truths": [
        "interrupt on a Paseo job runs paseo stop on its agent and keeps the agent",
        "cancel on a Paseo job runs paseo stop then archive and never touches the caller's own PASEO_AGENT_ID",
        "the herding permit verb answers allow or deny for the job's own agent and refuses a non-Paseo job",
        "the herding permit verb is declared in the catalog and the registry payload and its example runs",
        "interrupt and cancel on a herdr or tmux job are unchanged and the existing tests stay green"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/job_verbs.rs", "substantive": "Paseo interrupt, cancel and permit"},
        {"path": "packages/bee-rs/crates/bee/src/catalog.rs", "substantive": "the permit verb entry"},
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "the permit verb declaration"}
      ],
      "key_links": ["interrupt and cancel branch on job.json transport before any pane transport"],
      "prohibitions": ["No code comments", "No archive of the caller's own agent", "No paseo send anywhere"]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "po-3",
    "feature": "paseo-observe",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Keep a blocked Paseo worker waiting, count it, and attach its log tail",
    "deps": ["po-1"],
    "decisions": ["D1"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/wave.rs"
    ],
    "read_first": [
      "docs/history/paseo-observe/plan.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/wave.rs"
    ],
    "action": "Red first. run.rs: in the Paseo poll (wait_for_round_paseo_driven and its liveness mapping), a Blocked state is alive and waiting, never a reason to end the run; the idle and ceiling timeouts still apply, and a result still ends it. In execute_paseo and execute_continue_paseo call record_dispatch with the agent id where the pane path passes a pane id, so the wave ledger gets a row. On every non-result exit after the agent exists (idle timeout, ceiling, died, blocked at timeout) append the output of logs_tail_argv(id, 40, None) to the message, say 'agent kept for inspection: paseo logs <id>' instead of the pane wording, and keep the original message when the logs call fails. wave.rs: the live set for occupancy adds the ids of labelled, non-archived Paseo agents from ls_label_argv(bee_job) through parse_ls_agents; a failed call keeps today's behaviour. Tests with fakes: Blocked then a result returns the result; Blocked until the idle timeout times out with the log tail in the message; a Paseo run writes one ledger row; a labelled agent counts in occupancy; a herdr run is unchanged. No code comments.",
    "must_haves": {
      "truths": [
        "a Paseo worker that is Blocked keeps the run waiting until a result or a timeout",
        "a Paseo run writes a wave-ledger row with its agent id",
        "a Paseo timeout message carries the agent's last log lines and says the agent is kept",
        "occupancy counts labelled, non-archived Paseo agents",
        "herdr and tmux runs and occupancy are unchanged and the existing herding tests stay green"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "blocked waiting, ledger row and log-tail diagnosis"},
        {"path": "packages/bee-rs/crates/bee/src/herding/wave.rs", "substantive": "Paseo agents in the live set"}
      ],
      "key_links": ["the Paseo poll treats Blocked as alive"],
      "prohibitions": ["No code comments", "No send or archive change", "No change to the herdr poll"]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "po-4",
    "feature": "paseo-observe",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Let the observer read Paseo workers and file blocked permissions",
    "deps": ["po-1"],
    "decisions": ["D1"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/pane_verbs.rs",
      "packages/bee-rs/crates/bee/src/herding/control_loop.rs",
      "packages/bee-rs/crates/bee/src/herding/broker.rs",
      "packages/bee-rs/crates/bee/src/verbs/supervisor.rs"
    ],
    "read_first": [
      "docs/history/paseo-observe/plan.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/pane_verbs.rs",
      "packages/bee-rs/crates/bee/src/herding/control_loop.rs",
      "packages/bee-rs/crates/bee/src/herding/broker.rs",
      "packages/bee-rs/crates/bee/src/verbs/supervisor.rs",
      "docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md"
    ],
    "action": "Red first. pane_verbs.rs: pane read with an id that is a Paseo job id, or the paseo_agent_id of a Paseo job in the mailbox, prints the output of logs_tail_argv(agent, lines, None) (lines from the existing lines flag, default 40) instead of asking herdr or tmux; any other id keeps today's path. control_loop.rs: add the herding status verb to SUPERVISOR_ALLOWED_TOOLS in the same Bash(...) shape. verbs/supervisor.rs: add a human-decision kind permission to HUMAN_DECISION_KINDS. broker.rs: in the tick, for each Paseo job whose inspect shows Blocked, record one supervisor intervention of kind permission per pending request id (a marker file in the job mailbox stops repeats), naming the job, the tool and the answer verb `bee herding permit`; count it in notices_sent so the leader heartbeat sees news. Use the existing intervention path the broker already uses for a human-routed question. Tests with fakes: pane read on a Paseo job prints the log tail; a herdr pane read is unchanged; one blocked request files one intervention across two ticks; a supervisor record of kind permission is accepted. No code comments.",
    "must_haves": {
      "truths": [
        "pane read on a Paseo job or agent id shows the agent's log tail",
        "the supervisor may run the herding status verb",
        "a blocked Paseo permission is filed once per request as a human decision of kind permission and counts as broker news",
        "herdr and tmux pane verbs and existing broker routing are unchanged and the existing tests stay green"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/pane_verbs.rs", "substantive": "Paseo log tail in pane read"},
        {"path": "packages/bee-rs/crates/bee/src/herding/control_loop.rs", "substantive": "status in the supervisor allowlist"},
        {"path": "packages/bee-rs/crates/bee/src/herding/broker.rs", "substantive": "blocked permission filing"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/supervisor.rs", "substantive": "the permission human-decision kind"}
      ],
      "key_links": ["the broker tick inspects Paseo jobs and files blocked permissions"],
      "prohibitions": ["No code comments", "The broker never answers a permission itself"]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "po-5",
    "feature": "paseo-observe",
    "lane": "high-risk",
    "role": "docs",
    "title": "Document Paseo observation and control",
    "deps": [],
    "decisions": ["D1", "D2"],
    "files": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md",
      "docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md"
    ],
    "read_first": [
      "docs/history/paseo-observe/CONTEXT.md",
      "docs/history/paseo-observe/plan.md",
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md",
      "docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md"
    ],
    "action": "In the-paseo-channel.md add a section 'Watching and controlling Paseo workers' in the same ASD-STE100 style: the status fields (transport, paseo_agent_id, paseo_state, permissions, untracked_paseo_agents) and what unknown means; interrupt (paseo stop, agent kept), cancel (stop and archive, own-agent guard); the permit verb and the blocked-permission flow (the run keeps waiting, the broker files a permission decision, a person answers, the same turn goes on); occupancy counting; pane read showing the log tail; untracked agents reported only. Write commands as inline code that holds the verb name only, for example `bee herding permit`, never with flags or placeholders. Add 'paseo-observe D1, D2' to its frontmatter decisions. In the supervisor concept add one short paragraph: the supervisor may run the herding status verb, and a permission kind exists in the human-decision queue for blocked Paseo workers. Change nothing else.",
    "must_haves": {
      "truths": [
        "the Paseo concept documents the status fields and that unknown is neither alive nor dead",
        "the Paseo concept documents interrupt, cancel and the own-agent guard",
        "the Paseo concept documents the permit verb and the blocked-permission flow",
        "the supervisor concept documents the status verb and the permission kind",
        "the Paseo concept frontmatter cites paseo-observe D1, D2"
      ],
      "artifacts": [
        {"path": "docs/knowledge/areas/bee-herding/the-paseo-channel.md", "substantive": "the watching and controlling section"},
        {"path": "docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md", "substantive": "the Paseo paragraph"}
      ],
      "key_links": ["the sections extend the existing concepts"],
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
| Identity | cancel where the agent is the caller's own | po-2 | no stop, no archive, a clear message |
| Identity | permit on a herdr job | po-2 | refusal with a FIX line |
| State | inspect fails during status or the sweep | po-1 | unknown; no mark |
| State | Blocked then a result | po-3 | the result returns |
| Idempotence | the same blocked request over two ticks | po-4 | one intervention |
| Concurrency | a labelled agent of another session | po-1 | listed as untracked, never archived |
| Regression | herdr and tmux status, pane, interrupt, cancel, occupancy | all | existing tests green |
| User-visible path | live: a Claude worker in Paseo default mode asks to run a shell command | po-3 cap | status shows blocked with the tool; the permit verb allows it; the same turn finishes with done |

## Open Questions

(none)

## Out of scope

- The Pi workers widget state glyph.
- Auto-answering permissions.
