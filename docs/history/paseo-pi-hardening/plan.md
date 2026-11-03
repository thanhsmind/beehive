---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: paseo-pi-hardening

## Summary

bee already runs a Pi leader and its workers on Paseo. This feature makes that
setup faster, cheaper and safer:

- A worker run can no longer hang on a stuck Paseo daemon, and bee asks Paseo
  for state every 3 seconds instead of every 200 ms.
- A worker that goes quiet with no answer gets one reminder, and a second quiet
  stop ends the round at once instead of after 15 minutes.
- A stuck worker shows as "stalled" in `bee herding status` within minutes.
- bee records the model each worker really runs and warns on a mismatch.
- A Pi worker can no longer push, open pull requests, start other agents or
  dispatch bee work.
- The leader learns of news within about 30 seconds, and the paid "no news"
  heartbeat turns drop from about 288 to about 48 a day; the heartbeat is
  deleted when the leader ends.
- `bee doctor` checks the Paseo setup.
- The supervisor can run on Pi.

Mode: `high-risk` — 4 risk flags: external-systems, audit-security, covered-contract-change, multi-domain
Why this is the least workflow that protects the work: a new guard on worker shell commands and a changed wake path for the leader can each stop a whole unattended run, so every cell carries red-first tests and the slice gets a live Paseo proof.

Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied).

## Requirements (from CONTEXT.md)

- D1: 15 s timeout on every Paseo CLI call in run and run --continue; inspect at most every 3 s; file checks keep 200 ms.
- D2 (revised): one nudge on the first silent idle; a second silent idle at least 3 s after the nudge ends the round as the existing idle-timeout outcome through a `PollTick` flag; a failed send counts as sent; agent kept.
- D3 (revised): `UpdatedAt` is observe-only; status shows `stalled` past 120 s; the run loop keeps Working fresh.
- D4 (revised): the run loop records observed model and thinking in job.json; `-` and `auto` are unverified; warn and flag on a confirmed mismatch; never act on it.
- D5 (revised): `worker-guard` judges Paseo-carried workers only; refuses outward forms, `paseo`, and bee dispatch, herding run, worktree merge and gate; the belt blocks fail closed, also when the binary does not know the hook; the Pi dispatch tools are not registered for such a worker.
- D6 (revised): the leader awaits the heartbeat delete on `session_shutdown` except `reload`; a failed delete keeps the marker with `delete_failed`; a changed cron re-creates the heartbeat.
- D7: report-only `paseo_ready` doctor row on the pi runtime.
- D8: broker tick on the leader's own timer (default 30 s); model turn only on news; heartbeat default `*/30 * * * *`; paseo-pi D6 holds.
- D9 (revised): `herding.supervisor_runtime` claude|pi; a pi-provider or plain model slot only; spawn in the main root with `--no-extensions -e <bee-guard>` and `BEE_SUPERVISOR_ALLOWED` through an env parameter on the spawn seam; refuse when the extension is absent.
- D10: the concept documents the parent-cascade archive.

## Load-bearing claims

Labels: `read` (file opened at that line), `ran` (command executed, output kept). Evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The run loop polls every 200 ms | read | packages/bee-rs/crates/bee/src/herding/run.rs:69 | const POLL_INTERVAL: Duration = Duration::from_millis(200); |
| 2 | The run path builds the Paseo CLI with no timeout | read | packages/bee-rs/crates/bee/src/herding/run.rs:4967 | let cli = RealPaseoCli::new(cmd); |
| 3 | A timeout builder already exists on the real CLI | read | packages/bee-rs/crates/bee/src/herding/paseo.rs:489 | pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self { |
| 4 | Working always sets the heartbeat fresh | read | packages/bee-rs/crates/bee/src/herding/run.rs:3848 | heartbeat_fresh = true; |
| 5 | Idle maps to alive with no other effect | read | packages/bee-rs/crates/bee/src/herding/run.rs:3851 | Some(PaseoState::Idle) => { |
| 6 | Paseo inspect output carries UpdatedAt, Model and Thinking | ran | rg -n "UpdatedAt\|\"Model\"\|Thinking" /home/thanhsmind/Projects/refs/paseo/packages/cli/src/commands/agent/inspect.ts | 171:    { key: "UpdatedAt", value: agent.UpdatedAt }, |
| 7 | Every hook but activity short-circuits under the worker marker | read | packages/bee-rs/crates/bee/src/hooks/mod.rs:102 | name != "activity" |
| 8 | The outward arm judges only linked worktrees | read | packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:991 | if worktree_resolution != "linked-valid" { |
| 9 | The supervisor model is read from team.claude only | read | packages/bee-rs/crates/bee/src/herding/control_loop.rs:452 | match resolve_role(&models, &[SUPERVISOR_MODEL_ROLE], "claude", "cell") { |
| 10 | The heartbeat default is every 5 minutes | read | .pi/extensions/bee-guard/paseo-heartbeat.ts:5 | export const DEFAULT_CRON = "*/5 * * * *" |
| 11 | An empty heartbeat still forces a model turn | read | .pi/extensions/bee-guard/paseo-heartbeat.ts:237 | return "bee heartbeat: no news. Reply with the single word ok and do nothing else." |
| 12 | The drain timer already runs every 2 s inside the leader | read | .pi/extensions/bee-guard/result-inbox.ts:49 | export const DRAIN_POLL_MS = 2000 |
| 13 | The drain steers into a busy leader | read | .pi/extensions/bee-guard/result-inbox.ts:358 | steer ? { deliverAs: "steer" } : undefined, |
| 14 | session_shutdown runs only the close and activity hooks | read | .pi/extensions/bee-guard/events.ts:776 | runAdvisoryHook(directory, "session-close", { |
| 15 | The Pi doctor block ends with the transport row, where a Paseo row fits | read | packages/bee-rs/crates/bee/src/doctor.rs:400 | rows.push(pi_herding_transport_row_with_env(root, env)); |
| 16 | Pi has a one-shot, sessionless print mode | ran | rg -n -- "--no-session" /home/thanhsmind/.local/share/mise/installs/pi/0.87.1/pi/docs/cli.md | 97:- `--no-session`<br> |
| 17 | Status already has the stalled word | read | packages/bee-rs/crates/bee/src/herding.rs:851 | return "stalled".to_string(); |
| 18 | The run writes the agent id into job.json at spawn | read | packages/bee-rs/crates/bee/src/herding/run.rs:3399 | m.insert("paseo_agent_id".into(), Value::String(agent_id.clone())); |
| 19 | Paseo fills an unknown model with "-" and an unknown thinking level with "auto" | ran | rg -n "Thinking: snapshot" /home/thanhsmind/Projects/refs/paseo/packages/cli/src/commands/agent/inspect.ts | 135:    Thinking: snapshot.effectiveThinkingOptionId ?? "auto", |
| 20 | Pi extensions load in print mode | ran | rg -n "Extensions load in" /home/thanhsmind/.local/share/mise/installs/pi/0.87.1/pi/docs/extensions.md | 189:Extensions load in interactive, RPC, JSON, and print modes. |
| 21 | An explicit -e extension still loads under --no-extensions | ran | rg -n "Explicit .-e. paths still load" /home/thanhsmind/.local/share/mise/installs/pi/0.87.1/pi/docs/cli.md | 153:  Disables discovered and configured extensions. Explicit `-e` paths still load. |
| 22 | The supervisor spawn seam takes argv only, no env | read | packages/bee-rs/crates/bee/src/herding/control_loop.rs:600 | fn spawn(&self, argv: &Argv) -> std::io::Result<Child>; |
| 23 | Two outward helpers live outside checks.rs and are already pub(crate) | read | packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs:169 | pub(crate) fn find_git_invocations(tokens: &[String]) -> Vec<GitInvocation> { |

## Discovery

Two advisor digests (2026-10-06) compared Seatworks `2e11099f` and slp
`paseo-pi-team` with bee's Paseo channel; the findings are summarized in
CONTEXT.md "Specific Ideas And References". The leader then opened every anchor
above. The plan-step hat wave (five seats, 2026-10-06) found five blockers
(worker self-dispatch on herdr panes, the supervisor allowlist depending on
extension discovery, Paseo's `-`/`auto` sentinels, an invalid Pi `--model`
built from a non-pi provider, and the unproven extension load); D2-D6 and D9
were revised and every cell below carries the fixes. One correction: the worker-outward guard already judges Pi shell calls,
but only inside a linked worktree (claim 8), and a herded worker never reaches
it (claim 7).

## Approach

Recommended path: extend `parse_inspect` into a richer inspect read (state,
`UpdatedAt`, model, thinking) and use it in the run loop (D1-D4); add one Rust
hook `worker-guard` that reuses the outward helpers and also enforces the
supervisor allowlist from an environment variable (D5, D9); move the broker tick
onto a leader-side timer next to the result drain (D8); add the heartbeat delete
(D6), the doctor row (D7) and the docs (D10).

Rejected:
- A TypeScript denylist in the Pi extension — a second copy of the outward rules (one fact, one home).
- Archiving a worker on a model mismatch — automatic behavior against herding-cockpit-completeness 9615be76.
- Dropping the heartbeat — paseo-pi D6 needs a backstop, and Paseo must still see a turn end.
- A new `stalled` outcome word — the existing idle-timeout outcome carries the message.
- Ending a round on a stale `UpdatedAt` — Paseo does not move it during one long silent tool call (D3 revised).
- A second broker timer beside the drain with its own busy logic — the drain's busy and steer helper is shared instead.

Risk map:

| Component | Risk | Reason | Lands in | Proof needed |
|---|---|---|---|---|
| worker-guard refuses a legal worker command | HIGH | a false deny stops every Pi worker | pph-3 | allow and deny table tests; herded marker still exits 0 for every other hook |
| broker timer starts turns in a loop | MEDIUM | a news flag stuck true costs money | pph-4 | node test: no news means no sendUserMessage; one in-flight tick |
| silent-idle nudge fires into a running turn | MEDIUM | breaks paseo-pi D5 | pph-1 | test: nudge only after Idle; never while Working |
| reused inspect state defeats the died debounce | MEDIUM | one transient Dead read would end the run | pph-1 | test: non-inspect ticks report no liveness; one Dead then Working keeps running |
| spawn timeout leaves an untracked agent | MEDIUM | a re-dispatch puts two workers in one worktree | pph-1 | test: a timed-out spawn looks the agent up by its job label and names it |
| old binary silently allows | MEDIUM | an unknown hook name short-circuits to allow | pph-3 | node test: a binary without worker-guard in its hook list blocks |
| heartbeat delete races the next start | MEDIUM | the leader loses its backstop | pph-4 | node test: awaited delete, then a new start creates one heartbeat |
| Pi supervisor argv | MEDIUM | wrong flags kill the tick | pph-6 | argv test plus a live tick |

Waves: wave 1 runs pph-1, pph-3 and pph-5 in parallel (disjoint files). Wave 2 runs pph-2 (needs pph-1's inspect read and run.rs), pph-4 and pph-6 (both need pph-3: pph-4 shares `events.ts`, pph-6 needs the hook). Wave 3 runs pph-7 (docs describe the shipped code).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"pph-1 to pph-6 change the run loop, status, a hook, the Pi extension, doctor and the control loop."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Each code cell writes its own tests red-first; the leader drives the live proof."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"pph-7 updates the Paseo, hook and supervisor concepts and the config reference."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"conditional","role":"review","condition":"the slice judge for a behavior cell","reason":"A semantic judge may run on the guard cells."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the plan consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"High-risk plan check, structure mandate."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"High-risk plan check."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"High-risk plan check."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"High-risk plan check."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"High-risk plan check."}
  ]
}
```

## Shape

Epic map. Outcome: an unattended Pi leader on Paseo that cannot hang, wastes no
model turns, and whose workers cannot reach outside their cell.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Run loop | D1-D4 | hangs, slow silent idles, invisible stuck turns, unverified models | 1 | fake-CLI run-loop tests |
| Guards | D5, D9 | unguarded Pi workers; Pi supervisor | 1 | hook allow/deny tests, node belt test, argv test |
| Leader wake | D6, D8 | paid empty turns, orphan heartbeats | 1 | node extension tests |
| Setup check | D7 | silent setup faults | 1 | doctor tests |
| Docs | D10 + all | the concepts must match the code | 1 | knowledge check |

Current slice: all seven cells (one slice).

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pph-1 | Bound Paseo calls, nudge silent workers and record the real model in the run loop | packages/bee-rs/crates/bee/src/herding/paseo.rs; packages/bee-rs/crates/bee/src/herding/run.rs | — | a stuck daemon no longer hangs a run; a quiet worker gets one reminder, then ends the round at once; job.json names the model that ran | herding tests green |
| pph-2 | Show stalled and model mismatch for Paseo workers in status | packages/bee-rs/crates/bee/src/herding.rs | pph-1 | status shows stalled for a stuck Paseo worker and model_mismatch | herding tests green |
| pph-3 | Guard Paseo workers and the Pi supervisor with a worker-guard hook | packages/bee-rs/crates/bee/src/hooks/mod.rs; packages/bee-rs/crates/bee/src/hooks/worker_guard.rs (new); packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs; .pi/extensions/bee-guard/events.ts; .pi/extensions/bee-guard/index.ts; packages/bee-rs/crates/bee/tests/pi_worker_guard_contracts.rs (new) | — | a Paseo worker's git push, gh write, agent launch, paseo call or bee dispatch is refused | hooks tests and the node contract test green |
| pph-4 | Wake the Pi leader from its own broker timer and delete its heartbeat on exit | .pi/extensions/bee-guard/paseo-heartbeat.ts; .pi/extensions/bee-guard/events.ts; .pi/extensions/bee-guard/result-inbox.ts; packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs | pph-3 | news reaches the leader in about 30 s; the heartbeat fires every 30 min and is gone after the leader ends | node contract tests green |
| pph-5 | Check the Paseo setup in bee doctor | packages/bee-rs/crates/bee/src/doctor.rs; packages/bee-rs/crates/bee/src/doctor/tests.rs | — | bee doctor shows a paseo_ready row with what is missing | doctor tests green |
| pph-6 | Run the supervisor tick on Pi when configured | packages/bee-rs/crates/bee/src/herding/control_loop.rs | pph-3 | with supervisor_runtime pi, the tick runs pi with the team.pi model and the guarded allowlist | control_loop tests green |
| pph-7 | Document the hardened Paseo channel, worker-guard and the Pi supervisor | docs/knowledge/areas/bee-herding/the-paseo-channel.md; docs/knowledge/areas/hook-runtime/hook-runtime-the-worker-outward-guard.md; docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md; docs/config-reference.md | pph-1, pph-2, pph-3, pph-4, pph-5, pph-6 | the concepts explain the timeouts, nudge, stalled, model check, guard, timer wake, doctor row, Pi supervisor and the parent cascade | knowledge check green |

```json
[
  {
    "id": "pph-1",
    "feature": "paseo-pi-hardening",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Bound Paseo calls, nudge silent workers and record the real model in the run loop",
    "deps": [],
    "decisions": [
      "D1",
      "D2",
      "D4"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "read_first": [
      "docs/history/paseo-pi-hardening/CONTEXT.md",
      "docs/history/paseo-pi-hardening/plan.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "action": "Red first. paseo.rs: add a PaseoInspect struct (state: Option<PaseoState>, updated_at: Option<String>, model: Option<String>, thinking: Option<String>) and parse_inspect_full(stdout) that reads the same first JSON value parse_inspect reads, taking UpdatedAt, Model and Thinking with the same key tolerance; Model \"-\" and Thinking \"auto\" (Paseo's sentinels) parse as None (D4). Keep parse_inspect as a thin wrapper so every existing caller is unchanged. run.rs, timeouts (D1): the two `let cli = RealPaseoCli::new(cmd);` sites in the Paseo run and continue paths gain `.with_timeout(Duration::from_secs(15))`. When the spawn call itself fails or times out, call paseo::ls_label_argv for the job's bee_job label once, and when an agent for this job exists name its id in the SpawnFailed message (report only, no adopt, no archive). run.rs, cadence (D1): in wait_for_round_paseo_driven call paseo inspect only when at least 3 s passed since the last inspect call; the result, log and ack file checks still run every 200 ms tick; a tick that does not call inspect reports liveness None, so the died debounce in run_poll_loop still needs three consecutive real Dead reads. Working stays fresh exactly as today. run.rs, silent idle (D2 revised): once the round has seen Working, an Idle inspect with no result-N.json for the round sends ONE nudge with paseo::send_argv(agent_id, text), where text names the brief file and the result file the worker owes (mailbox path helpers); never send while the last state is Working or Blocked; a failed send counts as sent. A later Idle-with-no-result inspect read at least one inspect interval after the nudge ends the wait: add a bool field to PollTick (false in every herdr and tmux tick) that decide_poll turns into PollDecision::TimedOutIdle at once, before the idle timeout, and have the Paseo caller pass the silent-idle message to diagnose_giveup_paseo in place of the generic idle_timeout_message text: the message `herding: paseo agent <id> went idle twice with no result for round <n> — agent kept for inspection: paseo logs <id>` (pass the flag back through a Cell or the closure's captured state). The agent is kept by the existing failure path. run.rs, observed model (D4 revised): on the first inspect whose model or thinking is Some, write paseo_model_observed and paseo_thinking_observed into job.json (None writes the string unverified), once per job. Compare with the job's PaseoSpec: models match when equal, or when the observed one ends with `/<configured>` or the configured one ends with `/<observed>`; thinking matches equal ignoring case; a None on either side never mismatches. On a confirmed mismatch print one stderr line `herding: paseo agent <id> runs model <observed> (thinking <t>), configured <model> (thinking <t>)` and write paseo_model_mismatch true into job.json; never archive, stop or retry. Tests with a fake PaseoCli and the driven clock: inspect at most once per 3 s of fake time while file checks run each tick; one Dead read followed by Working keeps the run going; Working then Idle with no result sends exactly one nudge and a second silent idle 3 s later ends as idle timeout with the silent-idle message; a failed send still lets the second silent idle end the round; a result after the nudge returns the result; no send while Working or Blocked; observed model and thinking land in job.json; `-`/`auto` write unverified with no warning; a provider-prefixed equal model is no mismatch; a real mismatch writes the flag; a timed-out spawn names a labelled agent; parse_inspect_full reads all four fields and tolerates their absence. Herdr and tmux runs unchanged. No code comments.",
    "must_haves": {
      "truths": [
        "the Paseo run and continue paths build the CLI with a 15 second timeout, and a failed spawn names an agent found under its job label",
        "the Paseo wait loop calls paseo inspect at most once every 3 seconds while file checks keep the 200 ms poll, and non-inspect ticks report no liveness",
        "the first silent idle after Working sends exactly one paseo send nudge and a second silent idle at least 3 s later ends the round as idle timeout with a silent-idle message",
        "no nudge is ever sent while the agent is Working or Blocked",
        "job.json records the observed model and thinking, with - and auto recorded as unverified, and a confirmed mismatch sets paseo_model_mismatch and prints one warning",
        "herdr and tmux runs are unchanged and the existing herding tests stay green"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/herding/paseo.rs",
          "substantive": "PaseoInspect and parse_inspect_full reading UpdatedAt, Model and Thinking with sentinel handling"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/herding/run.rs",
          "substantive": "timeouts, 3 s inspect cadence, silent-idle nudge and end flag, observed-model record"
        }
      ],
      "key_links": [
        "wait_for_round_paseo_driven reads parse_inspect_full",
        "decide_poll turns the PollTick silent-idle flag into TimedOutIdle"
      ],
      "prohibitions": [
        "No code comments",
        "No new RunOutcome or PollDecision variant",
        "No paseo send while Working or Blocked",
        "No archive, stop or retry added",
        "Working freshness in the run loop is unchanged"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding",
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ]
  },
  {
    "id": "pph-2",
    "feature": "paseo-pi-hardening",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Show stalled and model mismatch for Paseo workers in status",
    "deps": [
      "pph-1"
    ],
    "decisions": [
      "D3",
      "D4"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/herding.rs"
    ],
    "read_first": [
      "docs/history/paseo-pi-hardening/CONTEXT.md",
      "docs/history/paseo-pi-hardening/plan.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs"
    ],
    "action": "Red first. herding.rs status, Paseo branch (search for `paseo_state`): read the inspect with paseo::parse_inspect_full (added by pph-1). Add model_mismatch (bool, from job.json paseo_model_mismatch, false when absent) and paseo_model_observed (string or null) to the JSON row, and model_mismatch=true to the text line only when true. A Working Paseo agent whose UpdatedAt (RFC 3339) is older than the shared 120 s freshness constant the status already uses for stalled shows status stalled and goes through the same stalled/recovered transition helper the pane path uses (D3 revised); a missing or unparseable UpdatedAt never shows stalled; finished, idle, blocked and dead rows are unchanged. Tests with a fake CLI and a temp mailbox: stalled for an old UpdatedAt; working for a fresh or missing one; recovered after stalled when UpdatedAt moves; model_mismatch true and false rows. No code comments.",
    "must_haves": {
      "truths": [
        "bee herding status shows stalled for a Working Paseo agent whose UpdatedAt is older than 120 s, and recovered when it moves again",
        "a missing or unparseable UpdatedAt never shows stalled",
        "the Paseo status row carries model_mismatch and paseo_model_observed from job.json",
        "the existing herding tests stay green"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/herding.rs",
          "substantive": "stalled from UpdatedAt and model fields in the Paseo status row"
        }
      ],
      "key_links": [
        "status reads paseo_model_mismatch from job.json"
      ],
      "prohibitions": [
        "No code comments",
        "No change to herdr or tmux status rows",
        "No Paseo call that changes state"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding",
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ]
  },
  {
    "id": "pph-3",
    "feature": "paseo-pi-hardening",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Guard Paseo workers and the Pi supervisor with a worker-guard hook",
    "deps": [],
    "decisions": [
      "D5",
      "D9"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/mod.rs",
      "packages/bee-rs/crates/bee/src/hooks/worker_guard.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs",
      ".pi/extensions/bee-guard/events.ts",
      ".pi/extensions/bee-guard/index.ts",
      "packages/bee-rs/crates/bee/tests/pi_worker_guard_contracts.rs"
    ],
    "read_first": [
      "docs/history/paseo-pi-hardening/CONTEXT.md",
      "docs/history/paseo-pi-hardening/plan.md",
      "docs/knowledge/areas/hook-runtime/hook-runtime-the-worker-outward-guard.md",
      "packages/bee-rs/crates/bee/src/hooks/mod.rs",
      "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs"
    ],
    "action": "Red first. hooks/mod.rs: add worker-guard to HOOK_NAMES and route it to worker_guard::run; marker_short_circuits returns false for activity and worker-guard only, and every other name still short-circuits under BEE_HERDING_WORKER. New hooks/worker_guard.rs: read the PreToolUse JSON from stdin (same shape write-guard reads: tool_name, tool_input.command, cwd); only a Bash tool is judged, anything else allows. Mode A (D5 revised), BEE_HERDING_WORKER and PASEO_AGENT_ID both non-empty: deny when any segment of the command is a git push, a gh command that is not a read-only form, an agent launch head (claude, codex, pi, opencode) that is not a read-only codex exec, a paseo command head, or a bee command (head bee or a path ending in /bee) whose verb is dispatch, `herding run`, `worktree merge` or gate; independent of the working directory. Only BEE_HERDING_WORKER set (a herdr or tmux pane worker): allow everything, as today. Mode B (D9 revised), BEE_SUPERVISOR_ALLOWED non-empty: parse it as a comma-separated list of Claude-style entries; `Bash(<prefix>:*)` allows a segment that starts with that prefix after trimming, where a leading `bee`, `./.bee/bin/bee` or an absolute path ending in `/.bee/bin/bee` counts as `.bee/bin/bee`; other entries such as Read are ignored; a segment matching no prefix, or a command with $( ), backticks or a > redirect, is denied. A deny in either mode denies. Reuse the tokenizer and helpers instead of copying them: find_git_invocations (write_guard/paths.rs) and is_separator (write_guard/guards.rs) are already pub(crate); in write_guard/checks.rs make command_basename, outward_segment_head, gh_sub_and_args, gh_read_only_form and codex_read_only_exec pub(crate), and reuse the same deep tokenize call check_git_bash_command uses. A deny exits 2 with the reason on stderr, the verdict shape write-guard uses, plus a FIX line (for a worker: stop and report blocked; the leader does this step). Neither variable set: exit 0, print nothing. .pi/extensions/bee-guard/events.ts tool_call: when the env carries BEE_HERDING_WORKER and PASEO_AGENT_ID, or BEE_SUPERVISOR_ALLOWED, and the mapped tool is the shell tool, run the blocking hook worker-guard through runBlockingHook before the normal mapping; block on a deny, a crash, a missing binary, or a binary whose `bee hook --help` output (read once per session and cached) does not list worker-guard. index.ts: when BEE_HERDING_WORKER and PASEO_AGENT_ID are both set, do not register beeDispatchTool, beeAdvisorTool and beeSteerTool (the verdict tool stays). With none of the variables set nothing changes. Tests: Rust unit tests in worker_guard.rs for the table (git push, gh pr create, pi -p x, claude, paseo ls, bee dispatch prepare, .bee/bin/bee herding run, bee gate denied in mode A; gh pr view, cargo test, git commit, git status, bee cells list allowed; `cargo test && git push` denied; pane-only marker allows git push; mode B allows `.bee/bin/bee status --json` and denies `rm -rf x`, `.bee/bin/bee status; rm x` and `.bee/bin/bee status > f`); a mod.rs test that under the marker worker-guard and activity reach their handlers and write-guard still short-circuits. New tests/pi_worker_guard_contracts.rs modeled on pi_paseo_heartbeat_contracts.rs (node probe, skip env): with a stub bee binary, the belt blocks a Paseo worker's shell call when the stub denies, allows when it allows, blocks when the binary is missing, blocks when the stub's hook help lacks worker-guard, and does not call the stub when no variable is set; index.ts registers no dispatch tools for a Paseo worker. Cite decision 9659395e (contract:worker-guard). No code comments.",
    "must_haves": {
      "truths": [
        "for a Paseo worker (BEE_HERDING_WORKER and PASEO_AGENT_ID set) worker-guard refuses git push, gh writes, agent launches, paseo commands and bee dispatch, herding run, worktree merge and gate in any working directory",
        "for a Paseo worker ordinary work commands such as cargo test, git commit, git status and bee cells list are allowed, and a herdr or tmux pane worker keeps today's posture",
        "with BEE_SUPERVISOR_ALLOWED set, a shell command outside the allowlist is refused",
        "every other hook still exits 0 under the worker marker",
        "the Pi belt blocks such a shell call on a deny, a crash, a missing binary or a binary that does not list worker-guard, and changes nothing when no variable is set",
        "the Pi extension registers no bee_dispatch, bee_advisor or bee_steer tool for a Paseo worker"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/worker_guard.rs",
          "substantive": "the worker and supervisor shell guard with its allow and deny tests"
        },
        {
          "path": ".pi/extensions/bee-guard/events.ts",
          "substantive": "routing of Paseo worker and supervisor shell calls to worker-guard with the hook-list check"
        },
        {
          "path": ".pi/extensions/bee-guard/index.ts",
          "substantive": "no dispatch tools for a Paseo worker"
        },
        {
          "path": "packages/bee-rs/crates/bee/tests/pi_worker_guard_contracts.rs",
          "substantive": "node contract tests for the belt"
        }
      ],
      "key_links": [
        "hooks/mod.rs routes worker-guard past the marker",
        "events.ts calls the worker-guard hook for Paseo worker and supervisor shell calls"
      ],
      "prohibitions": [
        "No code comments",
        "No copy of the outward tokenizer",
        "No change to write-guard verdicts",
        "No change for herdr or tmux pane workers"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee hooks && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_worker_guard_contracts --test pi_plugin_contracts",
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/hook-runtime/hook-runtime-the-worker-outward-guard.md"
    ]
  },
  {
    "id": "pph-4",
    "feature": "paseo-pi-hardening",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Wake the Pi leader from its own broker timer and delete its heartbeat on exit",
    "deps": [
      "pph-3"
    ],
    "decisions": [
      "D6",
      "D8"
    ],
    "files": [
      ".pi/extensions/bee-guard/paseo-heartbeat.ts",
      ".pi/extensions/bee-guard/events.ts",
      ".pi/extensions/bee-guard/result-inbox.ts",
      "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs"
    ],
    "read_first": [
      "docs/history/paseo-pi-hardening/CONTEXT.md",
      "docs/history/paseo-pi-hardening/plan.md",
      ".pi/extensions/bee-guard/result-inbox.ts",
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "action": "Red first. paseo-heartbeat.ts: DEFAULT_CRON becomes `*/30 * * * *` (D8). readPaseoSettings also reads herding.paseo.broker_tick_secs (a positive integer, default 30; anything else falls back to 30). ensureHeartbeat (D6 revised): when the marker holds a schedule id and a cron different from settings.cron, delete that schedule (`<command> heartbeat delete <id>`) and create a new one, rewriting the marker; same cron keeps today's exists answer. Add deleteHeartbeat(mainRoot, agentId, settings, run) returning a Promise: read the marker; with a schedule id run `<command> heartbeat delete <id>`; on success remove the marker; on failure keep the marker and write delete_failed: <reason> into it; never throws. result-inbox.ts: export one helper that tells whether the leader is busy and sends a message through pi.sendUserMessage with { deliverAs: \"steer\" } when busy and no option otherwise; the drain uses this helper too, so the busy and steer rule lives in one place. paseo-heartbeat.ts: add startBrokerTimer(pi, directory, settings, deps) and stopBrokerTimer(): every broker_tick_secs seconds run `bee herding broker tick --json` (120 s timeout) unless a broker tick is already in flight — the heartbeat input handler's tickRunning flag in events.ts and the timer share ONE in-flight flag (export it from paseo-heartbeat.ts and use it in both places); parse with parseTick; only when news is true send heartbeatText(tick) through the result-inbox helper; no news sends nothing; the timer is unref'd and armed only inside session_start for a Paseo leader (isPaseoLeader), never at module load. events.ts: in session_start, for a Paseo leader, arm the broker timer next to ensureHeartbeat; in session_shutdown, for every reason except reload, stop the broker timer and, for a Paseo leader, await deleteHeartbeat with the same execFile runner ensureHeartbeat uses before the handler returns (D6 revised). The heartbeat input handler still transforms and never swallows (paseo-pi D6). Tests in tests/pi_paseo_heartbeat_contracts.rs (node): the default cron row expects */30; the timer sends nothing on no news, sends once on news, steers when busy, never overlaps a slow tick nor runs while the heartbeat tick runs, reads broker_tick_secs; deleteHeartbeat calls heartbeat delete with the marker's id and removes the marker, and on a failing delete keeps the marker with delete_failed; ensureHeartbeat re-creates on a changed cron and answers exists on the same cron; shutdown(new) then start(new) ends with exactly one heartbeat. Cite decisions a7326910 (contract:paseo-heartbeat), 7f0788a0 and 67f07142. No code comments.",
    "must_haves": {
      "truths": [
        "the default heartbeat cron is */30 * * * * and a leader whose marker holds another cron gets its heartbeat re-created",
        "the leader runs the broker tick on its own timer every broker_tick_secs seconds, default 30, sharing one in-flight flag with the heartbeat tick",
        "a tick with no news sends nothing; a tick with news sends one message, as a steer when the leader is busy, through the same helper the result drain uses",
        "session_shutdown for every reason except reload awaits the heartbeat delete; a successful delete removes the marker and a failed one keeps it with delete_failed",
        "a heartbeat prompt is still transformed and never swallowed"
      ],
      "artifacts": [
        {
          "path": ".pi/extensions/bee-guard/paseo-heartbeat.ts",
          "substantive": "new default cron, cron re-create, broker timer and deleteHeartbeat"
        },
        {
          "path": ".pi/extensions/bee-guard/result-inbox.ts",
          "substantive": "the shared busy and steer send helper"
        },
        {
          "path": ".pi/extensions/bee-guard/events.ts",
          "substantive": "timer arming at session_start and awaited heartbeat delete at session_shutdown"
        },
        {
          "path": "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs",
          "substantive": "node tests for the timer, the delete and the cron re-create"
        }
      ],
      "key_links": [
        "events.ts session_start arms startBrokerTimer for a Paseo leader",
        "events.ts session_shutdown awaits deleteHeartbeat",
        "the drain and the broker timer send through one helper"
      ],
      "prohibitions": [
        "No code comments",
        "No timer at module load",
        "No model turn on a tick with no news",
        "No change to the heartbeat input handler's never-swallow behavior"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_paseo_heartbeat_contracts --test pi_plugin_contracts",
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ]
  },
  {
    "id": "pph-5",
    "feature": "paseo-pi-hardening",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Check the Paseo setup in bee doctor",
    "deps": [],
    "decisions": [
      "D7"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/doctor/tests.rs"
    ],
    "read_first": [
      "docs/history/paseo-pi-hardening/CONTEXT.md",
      "docs/history/paseo-pi-hardening/plan.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs"
    ],
    "action": "Red first. doctor.rs: in the pi runtime block, after the `rows.push(pi_herding_transport_row_with_env(root, env));` line, push a paseo_ready row only when some team.pi slot is a herding slot whose agent has a paseo block (read with the same folded config every_pi_slot_runs_no_pane uses, and herding::paseo::PaseoSpec::from_config). The row checks, through a PaseoCli built with a 5 s timeout so tests can inject a fake: the daemon answers (paseo::daemon_reachable or a version call); `<command> --version` output passes paseo::version_at_least((0,10,3)); when any configured provider is pi, `~/.pi/agent/auth.json` exists (home from the env closure); and for every marker file under `<root>/.bee/runtime/paseo-heartbeat/`, `paseo inspect <agent id> --json` (paseo::inspect_argv) answers and the agent is not archived (an Archived true field, a failed inspect or a Dead state is an orphan), and no marker carries a delete_failed field (D6 revised). ok is true only when every check passes; the detail names each failed check with a FIX line (start the daemon with paseo daemon start; upgrade to 0.10.3 with npm @getpaseo/cli; log in with pi; delete the orphan heartbeat with paseo heartbeat delete <id> and remove the marker). Report only: doctor changes nothing. No Paseo slot: no row. Tests in doctor/tests.rs with a fake CLI and a temp root: no row without a Paseo slot; ok with every check passing; each failing check named; an orphan marker named; a delete_failed marker named. No code comments.",
    "must_haves": {
      "truths": [
        "bee doctor on the pi runtime shows a paseo_ready row only when a team.pi slot uses a Paseo agent",
        "the row fails and names the fix when the daemon is down, the version is below 0.10.3, the pi auth file is missing, a heartbeat marker names an archived or missing agent, or a marker records a failed delete",
        "doctor changes no file and no Paseo state",
        "the existing doctor tests stay green"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/doctor.rs",
          "substantive": "the paseo_ready row"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/doctor/tests.rs",
          "substantive": "tests for each check"
        }
      ],
      "key_links": [
        "the pi runtime block pushes the paseo_ready row"
      ],
      "prohibitions": [
        "No code comments",
        "No Paseo call that changes state from doctor"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee doctor",
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ]
  },
  {
    "id": "pph-6",
    "feature": "paseo-pi-hardening",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Run the supervisor tick on Pi when configured",
    "deps": [
      "pph-3"
    ],
    "decisions": [
      "D9"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/control_loop.rs"
    ],
    "read_first": [
      "docs/history/paseo-pi-hardening/CONTEXT.md",
      "docs/history/paseo-pi-hardening/plan.md",
      "packages/bee-rs/crates/bee/src/hooks/worker_guard.rs"
    ],
    "action": "Red first. control_loop.rs: read herding.supervisor_runtime from the main root config; absent or claude keeps today's behavior byte-identical; pi selects the Pi path; any other value refuses the supervisor tick before any spawn with a FIX line naming the two legal values. Pi path (D9 revised): resolve the supervisor slot from team.pi with the same resolver supervisor_model uses (runtime pi). A herding slot whose agent's paseo block has provider pi gives its paseo model (required) and thinking; a plain model or native slot gives its model; a paseo provider other than pi, or no usable model, refuses with a FIX line (set team.pi.supervisor to a pi-provider agent). Refuse when `<main root>/.pi/extensions/bee-guard` does not exist. When herding.control_command is set it still wins and {MODEL} gets that model. Otherwise the default argv is [pi, --print, <prompt>, --model, <model>, --thinking, <level> (only when set), --no-session, --no-extensions, -e, <main root>/.pi/extensions/bee-guard, --tools, read,grep,find,ls,bash]. Spawn seam: IterationSpawner::spawn takes argv only today; add an env list and a cwd (the main root) to the spawn call (a second parameter or a small spawn-spec struct), have RealSpawner apply them, keep dispatch, merge, route and the claude supervisor passing an empty env and the current cwd, and pass BEE_SUPERVISOR_ALLOWED = SUPERVISOR_ALLOWED_TOOLS on the Pi path so worker-guard enforces the allowlist. Tests with the existing fake spawner: absent and claude give the exact current argv; pi with a pi-provider paseo slot gives the pi argv with model, thinking and the -e path; a claude-provider slot refuses; a missing extension directory refuses; pi with control_command uses the template; an illegal value refuses with no spawn; the Pi spawn carries BEE_SUPERVISOR_ALLOWED and the main root cwd. No code comments.",
    "must_haves": {
      "truths": [
        "with supervisor_runtime absent or claude the supervisor argv and spawn are byte-identical to today",
        "with supervisor_runtime pi and a pi-provider slot the supervisor runs pi --print with that model and thinking, --no-extensions -e <main root>/.pi/extensions/bee-guard and the read,grep,find,ls,bash tools, in the main root",
        "the Pi supervisor process carries BEE_SUPERVISOR_ALLOWED equal to the supervisor allowlist",
        "a non-pi provider slot, a missing bee-guard extension directory or an illegal supervisor_runtime refuses before any spawn",
        "the existing control_loop tests stay green"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/herding/control_loop.rs",
          "substantive": "supervisor runtime selection, the Pi argv and the env-carrying spawn seam"
        }
      ],
      "key_links": [
        "the Pi supervisor spawn sets BEE_SUPERVISOR_ALLOWED and loads bee-guard with -e"
      ],
      "prohibitions": [
        "No code comments",
        "No change to dispatch, merge or route argv or environment"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee control_loop",
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md"
    ]
  },
  {
    "id": "pph-7",
    "feature": "paseo-pi-hardening",
    "lane": "high-risk",
    "role": "docs",
    "change_class": "docs",
    "title": "Document the hardened Paseo channel, worker-guard and the Pi supervisor",
    "deps": [
      "pph-1",
      "pph-2",
      "pph-3",
      "pph-4",
      "pph-5",
      "pph-6"
    ],
    "decisions": [
      "D1",
      "D2",
      "D3",
      "D4",
      "D5",
      "D6",
      "D7",
      "D8",
      "D9",
      "D10"
    ],
    "files": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md",
      "docs/knowledge/areas/hook-runtime/hook-runtime-the-worker-outward-guard.md",
      "docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md",
      "docs/config-reference.md"
    ],
    "read_first": [
      "docs/history/paseo-pi-hardening/CONTEXT.md",
      "docs/history/paseo-pi-hardening/plan.md"
    ],
    "action": "Update the concepts to match the shipped code, citing each rule as paseo-pi-hardening D1 to D10 with its store id from bee decisions search --text paseo-pi-hardening. the-paseo-channel.md: the 15 s timeout and 3 s inspect cadence (D1), the silent-idle nudge and second-idle end (D2), UpdatedAt liveness and stalled (D3), the observed model fields and model_mismatch (D4), heartbeat delete on shutdown (D6), the doctor row (D7), the broker timer and the */30 backstop replacing the 5-minute default (D8, superseding paseo-pi D7), and the parent cascade: paseo run from inside the leader records it as parent, so archiving the leader archives every in-flight worker (D10); add the new files to Pointers. hook-runtime-the-worker-outward-guard.md: a section on worker-guard, the second hook past the herded marker, its deny list and the supervisor allowlist mode (D5, D9). the-supervisor-observer-and-its-interventions.md: herding.supervisor_runtime and the Pi argv (D9); in the Paseo channel concept, say that status is bee's freshness view and paseo_state is the daemon's raw state, so status=stalled with paseo_state=working is expected. docs/config-reference.md: herding.paseo.broker_tick_secs, the new heartbeat_cron default, herding.supervisor_runtime. Plain technical English, no code comments, no deferral words.",
    "must_haves": {
      "truths": [
        "the Paseo channel concept describes the timeouts, nudge, stalled, model check, heartbeat delete, doctor row, broker timer and parent cascade with their decision ids",
        "the worker-outward guard concept describes worker-guard and the supervisor allowlist",
        "the supervisor concept and the config reference name herding.supervisor_runtime and herding.paseo.broker_tick_secs",
        "bee knowledge check passes"
      ],
      "artifacts": [
        {
          "path": "docs/knowledge/areas/bee-herding/the-paseo-channel.md",
          "substantive": "the hardened channel"
        },
        {
          "path": "docs/knowledge/areas/hook-runtime/hook-runtime-the-worker-outward-guard.md",
          "substantive": "worker-guard"
        },
        {
          "path": "docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md",
          "substantive": "Pi supervisor"
        },
        {
          "path": "docs/config-reference.md",
          "substantive": "new keys"
        }
      ],
      "key_links": [
        "each concept cites paseo-pi-hardening decision ids"
      ],
      "prohibitions": [
        "No change outside docs"
      ]
    },
    "verify": ".bee/bin/bee knowledge check --json",
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md",
      "docs/knowledge/areas/hook-runtime/hook-runtime-the-worker-outward-guard.md",
      "docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md"
    ]
  }
]
```

## Test matrix

High-risk: the dimensions of `edge-dimensions.md` that apply, by cell.

| Dimension | Probe | Cell | Pass when |
|---|---|---|---|
| Timeout / hang | fake CLI sleeps past the timeout during a run | pph-1 | the call returns an error and the loop treats the tick as Unknown |
| Timeout / orphan | spawn times out but an agent exists under the job label | pph-1 | SpawnFailed names that agent id; no archive call |
| Debounce | one Dead read, then Working | pph-1 | the run keeps waiting |
| Rate | 10 s of fake time with 200 ms ticks | pph-1 | inspect called at most 4 times |
| State transition | Working → Idle, no result | pph-1 | exactly one send call |
| State transition | second silent idle | pph-1 | outcome idle timeout with the silent-idle message |
| Concurrency | send never during Working or Blocked | pph-1 | zero send calls in those states |
| Absent data | no UpdatedAt; Model "-"; Thinking "auto" | pph-1, pph-2 | status not stalled; job.json says unverified; no warning |
| Stale | Working with UpdatedAt 10 min old | pph-2 | status shows stalled; the run keeps waiting |
| Mismatch | Model differs from config | pph-2 | one warning; paseo_model_mismatch true; no archive call |
| Security: deny | git push, gh pr create, pi -p, paseo ls, bee dispatch prepare | pph-3 | each denied with a FIX line |
| Security: allow | cargo test, git commit, bee cells list | pph-3 | each allowed |
| Security: compound | `cargo test && git push` | pph-3 | denied |
| Scope | herdr pane worker (no PASEO_AGENT_ID) runs git push | pph-3 | allowed, as today |
| Old binary | hook help lacks worker-guard | pph-3 | the belt blocks |
| Fail closed | missing bee binary in the belt | pph-3 | the tool call is blocked |
| Regression | every other hook under the marker | pph-3 | exits 0 with no output |
| Cost | broker tick with no news | pph-4 | zero sendUserMessage calls |
| Concurrency | slow tick longer than the interval | pph-4 | one tick in flight |
| Cleanup | shutdown with a failing delete | pph-4 | marker kept with delete_failed, no throw |
| Race | shutdown(new) then start(new) | pph-4 | exactly one heartbeat exists |
| Migration | marker cron */5, setting */30 | pph-4 | heartbeat deleted and re-created with */30 |
| Setup | daemon down, old version, no auth, orphan marker | pph-5 | each named in the row detail |
| Config | supervisor_runtime absent, claude, pi, bogus | pph-6 | today's argv, today's argv, pi argv, refusal |
| Config | pi runtime with a claude-provider slot, or no bee-guard directory | pph-6 | refusal before spawn |
| Live | one Paseo Pi worker run in a sandbox repo after merge | leader | job.json carries the observed model; a worker git push is refused |
| Live | one Pi supervisor tick in a sandbox repo after merge | leader | the tick runs; a shell command outside the allowlist is refused |

## Open Questions

(none)

## Out of scope

- An exclusive broker claim for several leaders ticking at once: one leader runs per repo here; the hat wave noted the race.

- Seatworks' per-role Pi config folder (`PI_CODING_AGENT_DIR`) and readable worker titles in the Paseo app: not among the eight items the user named.
- Watching worker timelines with a coded detector: a separate past decision kept it out.
