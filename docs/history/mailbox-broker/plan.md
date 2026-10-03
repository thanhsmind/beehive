---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: mailbox-broker

## Summary
A worker that bee started with `bee herding run` can now stop and ask ONE
question instead of guessing or giving up. A small piece of bee code, the
broker, picks the question up on a timer. It asks the advisor model when the
question is technical, and it asks you when the question is a product or gate
choice, or when the advisor is not sure. When the answer is in, the broker
starts the same job again with the question and the answer in its brief. The
leader gets the final result the same way it gets any worker result today.
No LLM runs the broker, no second mailbox is made, and the supervisor still
only watches.

Mode: `standard` — 2 risk flags: public-contracts (the worker result file and
the `herding run` envelope gain a status), multi-domain (Rust herding, the Pi
extension, the supervisor store).
Why this size: one slice of five cells covers the whole loop end to end; a
smaller cut leaves a question that nobody answers.

Playbook: `skills/bee-planning/playbooks/feature.md` (route class `feature`).

## Requirements (from CONTEXT.md)
- D1: the broker is code on bee's own control loop, reuses the job mailbox and the supervisor delivery path, adds no second mailbox; the supervisor stays observer-only.
- D2: only dispatched workers ask; the leader keeps asking the human directly.
- D3: the worker stops with outcome `question`; the broker re-dispatches the same cell or seat as the next round with question and answer carried in; nothing is typed into a live pane.
- D4: technical questions go to the advisor first; product or gate questions, and "not sure" answers, go to the human.
- D5: a human question while away is queued in the presence queue and shown in the wake report; consent-sweep rules apply unchanged.
- D6 (planning, store `969f077a`): herding-channel workers only; a native Agent worker keeps returning `[BLOCKED]`; a non-herding advisor slot sends the question to the human.

## Load-bearing claims
Labels are `read` / `ran`; evidence is a verbatim substring of the anchor.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | A worker result accepts only done and blocked today | read | packages/bee-rs/crates/bee/src/herding/mailbox.rs:545-548 | pub(crate) enum MailboxStatus { |
| 2 | Rust requires a proof string on every result | read | packages/bee-rs/crates/bee/src/herding/mailbox.rs:682-686 | .ok_or(MailboxError::MissingField { round, field: "proof" })? |
| 3 | The Pi verdict tool accepts only done and blocked | read | .pi/extensions/bee-guard/tool-verdict.ts:24 | enum: ["done", "blocked"], |
| 4 | The Pi verdict tool requires proof only for done | read | .pi/extensions/bee-guard/tool-verdict.ts:95 | if (params.status === "done" && params.proof.trim().length === 0) { |
| 5 | The Pi drain picks the highest result file of a job, whatever its status | read | .pi/extensions/bee-guard/result-inbox.ts:139 | export function latestResultFile(mailbox: string): { round: number; file: string } \| null { |
| 6 | The supervisor role is an observer and never dispatches | read | packages/bee-rs/crates/bee/src/herding/control_loop.rs:63 | It is NOT a router: it never dispatches, never |
| 7 | Every control-loop role builds its argv from one shared control_command template | read | packages/bee-rs/crates/bee/src/herding/control_loop.rs:550 | read_command_template_tokens(main_root, "control_command") |
| 8 | A non-urgent supervisor row is queued while the human is away | read | packages/bee-rs/crates/bee/src/verbs/supervisor.rs:871 | let queued = kind != "urgent" && current_window(control).is_some(); |
| 9 | job.json today holds task, cwd, round, ceiling and expertise, not agent, seat or session | read | packages/bee-rs/crates/bee/src/herding/run.rs:2480-2490 | "expertise": opts.expertise, |
| 10 | A plain-model Claude slot dispatches as the Agent tool, which code cannot call | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2715 | tool = "Agent".into(); |
| 11 | --continue refuses when the job's pane is gone | read | packages/bee-rs/crates/bee/src/herding/run.rs:3215 | return refused(ContinueRefusal::PaneGone { |

## Discovery
- Three read gathers mapped the result contract, the control loop and supervisor store, and the advisor and cell retry fields (reports under main `.bee/mailbox/job-1791026302998-4029731-1`, `-4029732-1`, `-4029733-1`).
- The plan-step hat wave ran three seats (facts-gaps, alternatives, user-impact). Synthesis: `docs/history/mailbox-broker/reports/hat-wave.md`.

## Approach
The question lives where the worker already writes: `result-N.json` in its job
mailbox gets `status: "question"` and a `question {text, kind}` object
(contract `f58bfe89`). `job.json` gains every fact a re-dispatch needs (agent,
seat, cell_id, no_pane, inbox_session, leader_session, question_of,
question_round). The envelope of `bee herding run` gets outcome `question`,
the question object, `broker_running`, and a `next` line saying the broker owns
the job.

On Pi, the verdict tool accepts `question`, and the result drain leaves the
marker in place while the job's newest result is a question, so the final
round arrives through the same marker (contract `8e9a5eb0`).

`bee herding broker tick` is plain Rust (contract `91608561`). Each tick
claims one unanswered question by atomic rename and routes it:

| `kind` | Advisor | Route |
|---|---|---|
| gate or product | not called | human, signal `big-decision` (never consent-swept) |
| technical | herding advisor returns done | answer to the worker |
| technical | advisor blocked, failed, timed out, native or not configured | human, signal `worker-question` |

A human question is one supervisor intervention aimed at `leader_session`, so
the away queue, the wake report and consent-sweep work unchanged (D5). The human
answers with `bee herding answer --job <id> --text <t>`, which writes
`answer-N.json`. A consent-swept technical question takes the worker's
`leaning` as its answer. An answered question starts a new `bee herding run`
job with the same agent, cwd, seat, cell and inbox session, `question_of` set,
and a brief that carries the original task, the question, the answer and the
files the earlier round changed. Above `broker.max_question_rounds` (default 2)
the brief does not offer `question`. A finished re-dispatched job with no inbox
session gets one `broker-notice` row to the leader session. Each tick writes a
heartbeat that `herding run` reads for `broker_running`.

`bee herding control-loop --role broker` runs the bee binary itself as each
iteration's child: no prompt, no model, default 30 s (contract `d3031edb`).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"mb-1 to mb-4 change the result contract, the Pi extension, the broker verbs and the control loop."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Each code cell writes its own tests red-first; the leader drives the live loop after merge."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"mb-5 writes the broker concept and the question outcome."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"Standard lane; the hat wave is the plan check."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape, no competing designs."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Plan-step hat wave."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Standard lane runs three seats."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Standard lane runs three seats."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Plan-step hat wave."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Plan-step hat wave."}
  ]
}
```

## Shape
One slice. Waves: mb-1 first; then mb-2, mb-3 and mb-4 in parallel (disjoint
files); then mb-5.

## Cells — current slice (preview)

```json
[
  {
    "id": "mb-1",
    "feature": "mailbox-broker",
    "lane": "standard",
    "role": "code",
    "change_class": "api",
    "title": "Let a herded worker end a round with a question",
    "deps": [],
    "decisions": ["D1", "D3", "D6", "f58bfe89-4ec9-45c6-9834-cbdfa28ddd5b"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/mailbox.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "read_first": [
      "docs/history/mailbox-broker/CONTEXT.md",
      "docs/history/mailbox-broker/plan.md",
      "packages/bee-rs/crates/bee/src/herding/mailbox.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "action": "Red first, then implement contract worker-question (store f58bfe89). In mailbox.rs add MailboxStatus::Question; parse_result_text accepts status \"question\" only with a question object {text: non-empty string, kind: technical|product|gate}, makes proof optional for question and keeps it required for done and blocked; add a MailboxError for a missing or bad question. In the brief that render_brief builds, add one short section telling the worker it may end the round with status question and one question when it cannot go on without an answer, and that it must first write its partial work to the report; omit that section when a new BriefSpec field allow_question is false. In run.rs map the question result to outcome label \"question\"; the envelope carries the question object, broker_running (true when <control root>/.bee/supervisor/broker-heartbeat.json has a ts no older than 90 s) and a next line saying the broker owns the job and the final result arrives as a new job. job.json (pane and no-pane paths) records agent, seat, cell_id, no_pane, inbox_session, leader_session (from BEE_SESSION_ID, else CLAUDE_CODE_SESSION_ID, else PI_SESSION_ID, else null), question_of and question_round; add flags --question-of <job-id>, --question-round <n> and --no-question that set them and allow_question. No code comments.",
    "must_haves": {
      "truths": [
        "a result with status question and a valid question object parses as MailboxStatus::Question",
        "a question result with no proof field parses",
        "a done result with no proof field is still refused",
        "a question result with an empty text or an unknown kind is refused",
        "bee herding run reports outcome question with the question object, broker_running and next in its envelope",
        "job.json records agent, seat, cell_id, no_pane, inbox_session, leader_session, question_of and question_round on the pane and the no-pane path",
        "the brief offers the question status by default and omits it under --no-question"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/mailbox.rs", "substantive": "MailboxStatus::Question and its parse rules"},
        {"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "the question outcome, envelope fields and job.json facts"}
      ],
      "key_links": [
        "outcome_label maps MailboxStatus::Question to \"question\""
      ],
      "prohibitions": [
        "No code comments",
        "done and blocked parsing unchanged",
        "--continue behaviour unchanged"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "mb-2",
    "feature": "mailbox-broker",
    "lane": "standard",
    "role": "code",
    "change_class": "api",
    "title": "Let a Pi worker ask through the verdict tool and keep the drain waiting for the final round",
    "deps": ["mb-1"],
    "decisions": ["D3", "8e9a5eb0-0ca2-43eb-b36e-165bc6e97296"],
    "files": [
      ".pi/extensions/bee-guard/tool-verdict.ts",
      ".pi/extensions/bee-guard/result-inbox.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "read_first": [
      "docs/history/mailbox-broker/plan.md",
      ".pi/extensions/bee-guard/tool-verdict.ts",
      ".pi/extensions/bee-guard/result-inbox.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "action": "Red first in pi_plugin_contracts.rs, then implement contract pi-question-drain (store 8e9a5eb0). tool-verdict.ts: the status enum and check accept \"question\"; add an optional question parameter {text, kind: technical|product|gate}, required and validated when status is question; proof stays required only for done; the written result carries the question object. result-inbox.ts: when the newest result-N.json of a marked job has status question, do not claim, inject or delete the marker; leave it for the next round. No code comments.",
    "must_haves": {
      "truths": [
        "the verdict tool writes a question result with the question object and no proof",
        "the verdict tool refuses status question with no question text",
        "the drain leaves a marker unclaimed while the newest result is a question",
        "the drain delivers the next round's done result through the same marker",
        "the existing pi_plugin_contracts tests stay green"
      ],
      "artifacts": [
        {"path": ".pi/extensions/bee-guard/tool-verdict.ts", "substantive": "the question status"},
        {"path": ".pi/extensions/bee-guard/result-inbox.ts", "substantive": "the skip for question results"}
      ],
      "key_links": [
        "result-inbox.ts reads the status of latestResultFile before it claims"
      ],
      "prohibitions": [
        "No code comments",
        "done and blocked delivery unchanged"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "mb-3",
    "feature": "mailbox-broker",
    "lane": "standard",
    "role": "code",
    "change_class": "api",
    "title": "Add the broker tick and the answer verb that route a worker question and start the next round",
    "deps": ["mb-1"],
    "decisions": ["D1", "D3", "D4", "D5", "D6", "91608561-4012-4e55-a7cc-272ce2ab22ad"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/broker.rs",
      "packages/bee-rs/crates/bee/src/herding.rs",
      "packages/bee-rs/crates/bee/src/verbs/supervisor.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json"
    ],
    "read_first": [
      "docs/history/mailbox-broker/CONTEXT.md",
      "docs/history/mailbox-broker/plan.md",
      "docs/history/mailbox-broker/reports/hat-wave.md",
      "packages/bee-rs/crates/bee/src/herding/job_verbs.rs",
      "packages/bee-rs/crates/bee/src/herding.rs",
      "packages/bee-rs/crates/bee/src/verbs/supervisor.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
    ],
    "action": "Red first, then implement contract broker-tick (store 91608561) in a new module herding/broker.rs, routed from herding.rs as `broker tick` and `answer`, and declared in catalog.rs and registry_payload.json the way `herding steer` is. tick: find jobs under the control root .bee/mailbox whose highest result has status question and no answer-N.json; claim one per tick by atomic rename of a claim file (job_id + round is the dedupe key); route by the plan's truth table. The advisor consult calls the dispatch-prepare internals for --kind advisor on the job's runtime with a brief holding the question; only a herding payload is run (its command, synchronously, ceiling below 600 s); done = answer from its summary and report_path, anything else = human. A human route calls record_intervention_into with target leader_session, signal big-decision for gate or product and worker-question for technical; add the mailbox kinds the broker needs (broker-notice) to MAILBOX_KINDS and delivery_line; a consented technical question takes the worker's leaning as its answer, or stays waiting when there is none. answer --job <id> --text <t> writes answer-N.json (refuses an unknown job or a round with no question). An answered question starts a new bee herding run job with the parent job's agent, cwd, seat, cell_id, no_pane and inbox_session, --question-of and --question-round set, --no-question above broker.max_question_rounds (config, default 2), and a task holding the original task, the question, the answer and the parent round's files_changed. A finished child job with no inbox_session gets one broker-notice row to leader_session. Every tick writes .bee/supervisor/broker-heartbeat.json {ts}. Inject the spawner and the advisor runner behind traits so tests run with no herdr and no model. No code comments.",
    "must_haves": {
      "truths": [
        "a gate or product question becomes one intervention to leader_session with signal big-decision and no advisor call",
        "a technical question with a herding advisor that returns done writes answer-N.json from the advisor",
        "a technical question whose advisor is native, unconfigured, blocked or failed becomes one intervention with signal worker-question",
        "bee herding answer writes answer-N.json and refuses an unknown job",
        "an answered question starts one child job with question_of set and the parent's agent, cwd, seat and inbox_session",
        "the child brief carries the original task, the question, the answer and the parent round's files_changed",
        "above broker.max_question_rounds the child job runs with --no-question",
        "a second tick never routes the same job and round twice",
        "every tick writes the broker heartbeat",
        "bee herding broker tick --help and bee herding answer --help print their flags"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/broker.rs", "substantive": "the tick, the routing table, the answer verb and the re-dispatch"}
      ],
      "key_links": [
        "herding.rs routes broker and answer to herding/broker.rs",
        "broker.rs writes human questions through supervisor::record_intervention_into"
      ],
      "prohibitions": [
        "No code comments",
        "No LLM call in the broker itself",
        "No keys typed into a live pane",
        "No new store beside the job mailbox and the supervisor store",
        "consent-sweep code unchanged"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee -- herding catalog supervisor && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "mb-4",
    "feature": "mailbox-broker",
    "lane": "standard",
    "role": "code",
    "change_class": "behavior",
    "title": "Run the broker as a code-only control-loop role",
    "deps": ["mb-1"],
    "decisions": ["D1", "d3031edb-1f62-46b9-9a83-4df3f539e389"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/control_loop.rs"
    ],
    "read_first": [
      "docs/history/mailbox-broker/plan.md",
      "packages/bee-rs/crates/bee/src/herding/control_loop.rs"
    ],
    "action": "Red first, then implement contract broker-role (store d3031edb). Add Role::Broker (parse \"broker\", as_str, default interval 30 s, usage text). For Broker, resolve_iteration_argv returns [current bee executable, \"herding\", \"broker\", \"tick\", \"--json\"] and reads no prompt file, no model and no control_command template; allowed_tools_for is not consulted. Interval, timeout, backoff, max iterations and the stop file behave as for every role. No code comments.",
    "must_haves": {
      "truths": [
        "Role::parse(\"broker\") returns Role::Broker",
        "the broker iteration argv is the bee binary with herding broker tick --json",
        "a broker iteration needs no broker-prompt.md file",
        "the other roles' argv are unchanged",
        "the stop file ends a broker loop like any role"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/control_loop.rs", "substantive": "Role::Broker and its code-only argv"}
      ],
      "key_links": [
        "resolve_iteration_argv branches on Role::Broker before reading a prompt file"
      ],
      "prohibitions": [
        "No code comments",
        "Supervisor role unchanged"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee control_loop",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "mb-5",
    "feature": "mailbox-broker",
    "lane": "standard",
    "role": "docs",
    "change_class": "docs",
    "title": "Document the mailbox broker and the question outcome",
    "deps": ["mb-2", "mb-3", "mb-4"],
    "decisions": ["D1", "D2", "D3", "D4", "D5", "D6"],
    "files": [
      "docs/knowledge/areas/bee-herding/the-mailbox-broker.md",
      "docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md",
      "docs/knowledge/areas/bee-herding/index.md"
    ],
    "read_first": [
      "docs/history/mailbox-broker/CONTEXT.md",
      "docs/history/mailbox-broker/plan.md",
      "docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md",
      "docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md",
      "docs/knowledge/areas/bee-herding/index.md"
    ],
    "action": "Write docs/knowledge/areas/bee-herding/the-mailbox-broker.md as a bee.area concept in the shape of the supervisor concept: what the broker is and is not, the question record, the routing table, the human answer verb, the round limit, the heartbeat, how to start it (bee herding control-loop --role broker), what is out (native Agent workers, leader questions), citing mailbox-broker D1-D6 and the four contract decisions by store id; Pointers name broker.rs, run.rs, mailbox.rs, control_loop.rs, tool-verdict.ts and result-inbox.ts. Add the question outcome to the-run-verb-and-worker-outcomes.md and link the new concept from index.md. Plain technical English.",
    "must_haves": {
      "truths": [
        "the new concept cites mailbox-broker D1 through D6",
        "the run-verb concept lists the question outcome",
        "index.md links the new concept"
      ],
      "artifacts": [
        {"path": "docs/knowledge/areas/bee-herding/the-mailbox-broker.md", "substantive": "the broker concept"}
      ],
      "key_links": [
        "index.md links the-mailbox-broker.md"
      ],
      "prohibitions": [
        "No promise of later work in the prose"
      ]
    },
    "verify": "rg -n \"mailbox-broker D6\" docs/knowledge/areas/bee-herding/the-mailbox-broker.md && rg -n \"question\" docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md && rg -n \"the-mailbox-broker.md\" docs/knowledge/areas/bee-herding/index.md",
    "affects_skills": [],
    "affects_specs": []
  }
]
```

## Test matrix
| Behavior | Cell | Proof |
|---|---|---|
| question result parse, proof rule, envelope, job.json facts | mb-1 | unit tests in herding mailbox and run |
| Pi verdict question and drain skip | mb-2 | pi_plugin_contracts |
| routing table, answer verb, re-dispatch, dedupe, heartbeat | mb-3 | broker unit tests with a fake spawner and advisor |
| code-only broker role | mb-4 | control_loop unit tests |
| whole loop live | leader | after merge: control-bee sandbox, one herded worker asks, broker answers, round 2 finishes |

## Open Questions
None block the gate.

## Out of scope
- Native Agent-tool workers (D6).
- Questions from the leader session (D2).
- Answer injection into a live worker (D3).
