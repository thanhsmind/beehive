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
- A stuck worker shows as "stalled" in minutes, not after 6 hours.
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
- D2: one nudge on the first silent idle; the second silent idle ends the round as the existing idle-timeout outcome; agent kept.
- D3: `UpdatedAt` is the liveness heartbeat for a Working agent; status shows `stalled` past 120 s; observe only.
- D4: record observed model and thinking in job.json; warn and show `model_mismatch` on a confirmed mismatch; never act on it.
- D5: `worker-guard` hook passes the marker; refuses outward forms, `paseo`, and bee dispatch, herding run, worktree merge and gate; the Pi belt routes a herded worker's shell calls to it, fail closed.
- D6: the leader deletes its heartbeat and marker on `session_shutdown` except `reload`.
- D7: report-only `paseo_ready` doctor row on the pi runtime.
- D8: broker tick on the leader's own timer (default 30 s); model turn only on news; heartbeat default `*/30 * * * *`; paseo-pi D6 holds.
- D9: `herding.supervisor_runtime` claude|pi; pi argv and `BEE_SUPERVISOR_ALLOWED` enforced by `worker-guard`.
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

## Discovery

Two advisor digests (2026-10-06) compared Seatworks `2e11099f` and slp
`paseo-pi-team` with bee's Paseo channel; the findings are summarized in
CONTEXT.md "Specific Ideas And References". The leader then opened every anchor
above. One correction: the worker-outward guard already judges Pi shell calls,
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

Risk map:

| Component | Risk | Reason | Lands in | Proof needed |
|---|---|---|---|---|
| worker-guard refuses a legal worker command | HIGH | a false deny stops every Pi worker | pph-3 | allow and deny table tests; herded marker still exits 0 for every other hook |
| broker timer starts turns in a loop | MEDIUM | a news flag stuck true costs money | pph-4 | node test: no news means no sendUserMessage; one in-flight tick |
| silent-idle nudge fires into a running turn | MEDIUM | breaks paseo-pi D5 | pph-1 | test: nudge only after Idle; never while Working |
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
| pph-1 | Bound Paseo calls and catch silent and stuck workers in the run loop | packages/bee-rs/crates/bee/src/herding/paseo.rs; packages/bee-rs/crates/bee/src/herding/run.rs | — | a stuck daemon no longer hangs a run; a quiet worker gets one reminder, then ends the round at once | herding tests green |
| pph-2 | Record each Paseo worker's real model and show stalled and mismatch in status | packages/bee-rs/crates/bee/src/herding/run.rs; packages/bee-rs/crates/bee/src/herding.rs | pph-1 | job.json names the model that ran; status shows stalled and model_mismatch | herding tests green |
| pph-3 | Guard herded workers and the Pi supervisor with a worker-guard hook | packages/bee-rs/crates/bee/src/hooks/mod.rs; packages/bee-rs/crates/bee/src/hooks/worker_guard.rs (new); packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs; .pi/extensions/bee-guard/events.ts; packages/bee-rs/crates/bee/tests/pi_worker_guard_contracts.rs (new) | — | a Pi worker's git push, gh write, agent launch, paseo call or bee dispatch is refused | hooks tests and the node contract test green |
| pph-4 | Wake the Pi leader from its own broker timer and delete its heartbeat on exit | .pi/extensions/bee-guard/paseo-heartbeat.ts; .pi/extensions/bee-guard/events.ts; packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs | pph-3 | news reaches the leader in about 30 s; the heartbeat fires every 30 min and is gone after the leader ends | node contract tests green |
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
    "title": "Bound Paseo calls and catch silent and stuck workers in the run loop",
    "deps": [],
    "decisions": ["D1", "D2", "D3"],
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
    "action": "Red first. paseo.rs: add a PaseoInspect struct (state: Option<PaseoState>, updated_at: Option<String>, model: Option<String>, thinking: Option<String>) and parse_inspect_full(stdout) that reads the same first JSON value parse_inspect reads, taking UpdatedAt, Model and Thinking with the same key tolerance (capitalized or lower-case); keep parse_inspect as a thin wrapper so every existing caller is unchanged. run.rs (find these by search, not by line): the two `let cli = RealPaseoCli::new(cmd);` sites in the Paseo run and continue paths gain `.with_timeout(Duration::from_secs(15))` (per D1). In wait_for_round_paseo_driven: call paseo inspect only when at least 3 s passed since the last inspect call, reusing the last state between calls; the result, log and ack file checks still run on every 200 ms tick (D1). Working is fresh only when updated_at differs from the last value read; when updated_at is None, Working stays fresh as today (D3). Silent idle (D2): once the round has seen Working, an Idle read with no result-N.json for the round sends ONE nudge through paseo::send_argv(agent_id, text) where text names the brief file and the result file the worker owes (use the mailbox path helpers); never send while the last state is Working or Blocked. A second Idle-with-no-result read after the nudge (and after the agent was seen Working again or 30 s passed, whichever comes first) ends the wait as PollDecision::TimedOutIdle, and the outcome message says the worker went idle twice with no result; the agent is kept (the existing failure path already keeps it). Add an injectable clock path so the tests drive time; reuse wait_for_round_paseo_driven's sleep and now parameters. Tests with a fake PaseoCli: inspect is called at most once per 3 s of fake time while file checks run each tick; Working with an unchanged UpdatedAt goes stale and times out at the idle timeout; Working with no UpdatedAt stays fresh; Working then Idle with no result sends exactly one nudge, and a second silent idle ends as idle timeout with the silent-idle message; a result after the nudge returns the result; no send happens while Working; parse_inspect_full reads all four fields and tolerates their absence. Herdr and tmux run paths are unchanged. No code comments.",
    "must_haves": {
      "truths": [
        "the Paseo run and continue paths build the CLI with a 15 second timeout",
        "the Paseo wait loop calls paseo inspect at most once every 3 seconds while file checks keep the 200 ms poll",
        "a Working agent whose UpdatedAt does not move goes stale and ends at the idle timeout; a Working agent with no UpdatedAt stays fresh",
        "the first silent idle after Working sends exactly one paseo send nudge and the second ends the round as idle timeout with a silent-idle message",
        "no nudge is ever sent while the agent is Working or Blocked",
        "herdr and tmux runs are unchanged and the existing herding tests stay green"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/paseo.rs", "substantive": "PaseoInspect and parse_inspect_full reading UpdatedAt, Model and Thinking"},
        {"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "timeouts, 3 s inspect cadence, UpdatedAt freshness and the silent-idle nudge"}
      ],
      "key_links": ["wait_for_round_paseo_driven reads parse_inspect_full"],
      "prohibitions": ["No code comments", "No new RunOutcome or PollDecision variant", "No paseo send while Working or Blocked", "No archive or stop call added"]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding",
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"]
  },
  {
    "id": "pph-2",
    "feature": "paseo-pi-hardening",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Record each Paseo worker's real model and show stalled and mismatch in status",
    "deps": ["pph-1"],
    "decisions": ["D3", "D4"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding.rs"
    ],
    "read_first": [
      "docs/history/paseo-pi-hardening/CONTEXT.md",
      "docs/history/paseo-pi-hardening/plan.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs"
    ],
    "action": "Red first. run.rs: after a Paseo spawn writes paseo_agent_id into job.json (search for `m.insert(\"paseo_agent_id\"`), the wait loop records the model on the first inspect that returns Model or Thinking: write paseo_model_observed and paseo_thinking_observed into job.json (a missing field writes the string unverified), once per job. Compare with the job's PaseoSpec: the configured model matches when the observed model equals it or ends with it after a provider prefix (Paseo may report provider/model); thinking matches when equal, ignoring case; a None in the spec never mismatches. On a confirmed mismatch print one stderr line `herding: paseo agent <id> runs model <observed> (thinking <t>), configured <model> (thinking <t>)` and write paseo_model_mismatch true into job.json; never archive, stop or retry (D4). herding.rs status, Paseo branch (search for `paseo_state`): use paseo::parse_inspect_full; add model_mismatch (bool, from job.json) to the JSON row and the text line; a Working agent whose UpdatedAt is older than the shared 120 s freshness constant the status already uses for stalled shows status stalled (D3), and recovered behaves as it does for panes; an unparseable UpdatedAt never shows stalled. Tests with fakes: observed model and thinking land in job.json; a mismatch writes the flag and one warning; a provider-prefixed equal model is not a mismatch; missing fields write unverified with no warning; status shows stalled for an old UpdatedAt and not for a fresh or missing one; status shows model_mismatch. No code comments.",
    "must_haves": {
      "truths": [
        "job.json carries paseo_model_observed and paseo_thinking_observed after the first answering inspect, or unverified when a field is missing",
        "a confirmed mismatch prints one warning and sets paseo_model_mismatch, and nothing is archived, stopped or retried",
        "a provider-prefixed model equal to the configured model is not a mismatch",
        "bee herding status shows stalled for a Working Paseo agent whose UpdatedAt is older than 120 s, and shows model_mismatch",
        "the existing herding tests stay green"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "observed model recording and mismatch warning"},
        {"path": "packages/bee-rs/crates/bee/src/herding.rs", "substantive": "stalled from UpdatedAt and model_mismatch in the Paseo status row"}
      ],
      "key_links": ["status reads paseo_model_mismatch from job.json"],
      "prohibitions": ["No code comments", "No archive, stop or retry on a mismatch", "No change to herdr or tmux status rows"]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding",
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"]
  },
  {
    "id": "pph-3",
    "feature": "paseo-pi-hardening",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Guard herded workers and the Pi supervisor with a worker-guard hook",
    "deps": [],
    "decisions": ["D5", "D9"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/mod.rs",
      "packages/bee-rs/crates/bee/src/hooks/worker_guard.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs",
      ".pi/extensions/bee-guard/events.ts",
      "packages/bee-rs/crates/bee/tests/pi_worker_guard_contracts.rs"
    ],
    "read_first": [
      "docs/history/paseo-pi-hardening/CONTEXT.md",
      "docs/history/paseo-pi-hardening/plan.md",
      "docs/knowledge/areas/hook-runtime/hook-runtime-the-worker-outward-guard.md",
      "packages/bee-rs/crates/bee/src/hooks/mod.rs",
      "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs"
    ],
    "action": "Red first. hooks/mod.rs: add worker-guard to HOOK_NAMES and route it to worker_guard::run; marker_short_circuits returns false for activity and worker-guard only (D5), and every other name still short-circuits under BEE_HERDING_WORKER. New hooks/worker_guard.rs: read the PreToolUse JSON from stdin (same shape write-guard reads: tool_name, tool_input.command, cwd); only a Bash tool is judged, anything else allows. Mode A, BEE_HERDING_WORKER set: deny when the command contains a git push, a gh command that is not a read-only form, an agent launch head (claude, codex, pi, opencode) that is not a sanctioned read-only codex exec, a paseo command head, or a bee command (bee or a path ending in /bee) whose verb is dispatch, `herding run`, `worktree merge` or gate; judge every segment of a compound line, independent of the working directory. Reuse the existing tokenizer and helpers in write_guard/checks.rs (find_git_invocations, outward_segment_head, gh_sub_and_args, gh_read_only_form, codex_read_only_exec, command_basename, is_separator) by making the ones you need pub(crate); do not copy them. Mode B, BEE_SUPERVISOR_ALLOWED set (D9): parse it as a comma-separated list of Claude-style entries; `Bash(<prefix>:*)` allows a command whose every segment starts with that prefix after trimming; `Read` entries are ignored; a command with a segment matching no prefix, or with command substitution or a redirect to a file, is denied. Both modes may be set; a deny in either denies. A deny prints the hook deny verdict the Pi belt already understands (the same exit code 2 and stderr reason write-guard uses) with a FIX line naming what to do instead (stop and report blocked for a worker). Neither variable set: allow, print nothing. events.ts tool_call: when process.env.BEE_HERDING_WORKER or process.env.BEE_SUPERVISOR_ALLOWED is non-empty and the mapped tool is the shell tool, run the blocking hook worker-guard through the same runBlockingHook path before the normal mapping; a deny, a crash or a missing binary blocks (fail closed, matching index.ts). With neither variable set nothing changes. Tests: Rust unit tests in worker_guard.rs for the allow and deny table (git push, gh pr create denied, gh pr view allowed, pi and claude launches denied, paseo ls denied, bee dispatch prepare and bee herding run denied, bee cells list, cargo test, git commit and git status allowed, a compound line with one denied segment denied, supervisor allowlist allows `.bee/bin/bee status --json` and denies `rm -rf x` and `.bee/bin/bee status; rm x`); a mod.rs test that under the marker worker-guard and activity reach their handlers and write-guard still short-circuits. New tests/pi_worker_guard_contracts.rs modeled on pi_paseo_heartbeat_contracts.rs (node probe, skip env): the belt blocks a herded worker's git push via a stub bee binary that denies, allows when the stub allows, and blocks when the binary is missing. Log a decision tagged contract:worker-guard naming what these tests check before claiming. No code comments.",
    "must_haves": {
      "truths": [
        "under BEE_HERDING_WORKER the worker-guard hook refuses git push, gh writes, agent launches, paseo commands and bee dispatch, herding run, worktree merge and gate in any working directory",
        "under BEE_HERDING_WORKER ordinary work commands such as cargo test, git commit, git status and bee cells list are allowed",
        "with BEE_SUPERVISOR_ALLOWED set, a shell command outside the allowlist is refused",
        "every other hook still exits 0 under the worker marker",
        "the Pi belt blocks a herded worker's shell call on a deny, a crash or a missing binary, and changes nothing when neither variable is set"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/worker_guard.rs", "substantive": "the worker and supervisor shell guard with its allow and deny tests"},
        {"path": ".pi/extensions/bee-guard/events.ts", "substantive": "routing of herded and supervisor shell calls to worker-guard"},
        {"path": "packages/bee-rs/crates/bee/tests/pi_worker_guard_contracts.rs", "substantive": "node contract tests for the belt routing"}
      ],
      "key_links": ["hooks/mod.rs routes worker-guard past the marker", "events.ts calls the worker-guard hook for herded shell calls"],
      "prohibitions": ["No code comments", "No copy of the outward tokenizer", "No change to write-guard verdicts in the main checkout or a worktree"]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee hooks && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_worker_guard_contracts --test pi_plugin_contracts",
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/hook-runtime-the-worker-outward-guard.md"]
  },
  {
    "id": "pph-4",
    "feature": "paseo-pi-hardening",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Wake the Pi leader from its own broker timer and delete its heartbeat on exit",
    "deps": ["pph-3"],
    "decisions": ["D6", "D8"],
    "files": [
      ".pi/extensions/bee-guard/paseo-heartbeat.ts",
      ".pi/extensions/bee-guard/events.ts",
      "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs"
    ],
    "read_first": [
      "docs/history/paseo-pi-hardening/CONTEXT.md",
      "docs/history/paseo-pi-hardening/plan.md",
      ".pi/extensions/bee-guard/result-inbox.ts",
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "action": "Red first. paseo-heartbeat.ts: DEFAULT_CRON becomes `*/30 * * * *` (D8). readPaseoSettings also reads herding.paseo.broker_tick_secs (a positive integer, default 30; anything else falls back to 30). Add deleteHeartbeat(mainRoot, agentId, settings, run): read the marker, call `<command> heartbeat delete <schedule_id>` when it holds one, then remove the marker; a failed delete still removes the marker and returns the reason; never throws. Add startBrokerTimer(pi, directory, settings, deps) and stopBrokerTimer(): every broker_tick_secs seconds, when no tick is in flight and the heartbeat tick is not running, run `bee herding broker tick --json` (the same call the heartbeat input handler makes, 120 s timeout), parse it with parseTick, and only when news is true call pi.sendUserMessage with the news text from heartbeatText(tick), passing { deliverAs: \"steer\" } when the leader is busy (the same busy state the result drain reads) and nothing otherwise; no news sends nothing; the timer is unref'd and armed only inside session_start for a Paseo leader (isPaseoLeader), never at module load. events.ts: in session_start, for a Paseo leader, arm the broker timer next to ensureHeartbeat; in session_shutdown, for every reason except reload, stop the broker timer and, for a Paseo leader, call deleteHeartbeat with the same execFile runner ensureHeartbeat uses (D6). The heartbeat input handler is unchanged: it still transforms, never swallows (paseo-pi D6). Update tests/pi_paseo_heartbeat_contracts.rs: the default cron row now expects */30; add node tests that the timer sends nothing on no news, sends once on news, steers when busy, never overlaps two ticks, reads broker_tick_secs, and that deleteHeartbeat calls heartbeat delete with the marker's id and removes the marker even when the delete fails. These tests change the contracts pinned by decisions 7f0788a0 and 67f07142: cite them and log a decision tagged contract:paseo-heartbeat naming the new default and the timer contract before claiming. No code comments.",
    "must_haves": {
      "truths": [
        "the default heartbeat cron is */30 * * * *",
        "the leader runs the broker tick on its own timer every broker_tick_secs seconds, default 30",
        "a tick with no news sends nothing; a tick with news sends one message, as a steer when the leader is busy",
        "two broker ticks never run at once",
        "session_shutdown for every reason except reload deletes the leader's heartbeat and its marker",
        "a heartbeat prompt is still transformed and never swallowed"
      ],
      "artifacts": [
        {"path": ".pi/extensions/bee-guard/paseo-heartbeat.ts", "substantive": "new default cron, broker timer and deleteHeartbeat"},
        {"path": ".pi/extensions/bee-guard/events.ts", "substantive": "timer arming at session_start and heartbeat delete at session_shutdown"},
        {"path": "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs", "substantive": "node tests for the timer and the delete"}
      ],
      "key_links": ["events.ts session_start arms startBrokerTimer for a Paseo leader", "events.ts session_shutdown calls deleteHeartbeat"],
      "prohibitions": ["No code comments", "No timer at module load", "No model turn on a tick with no news", "No change to the heartbeat input handler's never-swallow behavior"]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_paseo_heartbeat_contracts --test pi_plugin_contracts",
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"]
  },
  {
    "id": "pph-5",
    "feature": "paseo-pi-hardening",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Check the Paseo setup in bee doctor",
    "deps": [],
    "decisions": ["D7"],
    "files": [
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/doctor/tests.rs"
    ],
    "read_first": [
      "docs/history/paseo-pi-hardening/CONTEXT.md",
      "docs/history/paseo-pi-hardening/plan.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs"
    ],
    "action": "Red first. doctor.rs: in the pi runtime block, after the `rows.push(pi_herding_transport_row_with_env(root, env));` line, push a paseo_ready row only when some team.pi slot is a herding slot whose agent has a paseo block (read with the same folded config every_pi_slot_runs_no_pane uses, and herding::paseo::PaseoSpec::from_config). The row checks, through a PaseoCli built with a 5 s timeout so tests can inject a fake: the daemon answers (paseo::daemon_reachable or a version call); `<command> --version` output passes paseo::version_at_least((0,10,3)); when any configured provider is pi, `~/.pi/agent/auth.json` exists (home from the env closure); and every marker file under `<root>/.bee/runtime/paseo-heartbeat/` names an agent that `paseo ls` (paseo::ls_label_argv or the plain list) still lists as not archived. ok is true only when every check passes; the detail names each failed check with a FIX line (start the daemon with paseo daemon start; upgrade to 0.10.3 with npm @getpaseo/cli; log in with pi; delete the orphan heartbeat with paseo heartbeat delete <id> and remove the marker). Report only: doctor changes nothing. No Paseo slot: no row. Tests in doctor/tests.rs with a fake CLI and a temp root: no row without a Paseo slot; ok with every check passing; each failing check named; an orphan marker named. No code comments.",
    "must_haves": {
      "truths": [
        "bee doctor on the pi runtime shows a paseo_ready row only when a team.pi slot uses a Paseo agent",
        "the row fails and names the fix when the daemon is down, the version is below 0.10.3, the pi auth file is missing, or a heartbeat marker names a missing agent",
        "doctor changes no file and no Paseo state",
        "the existing doctor tests stay green"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/doctor.rs", "substantive": "the paseo_ready row"},
        {"path": "packages/bee-rs/crates/bee/src/doctor/tests.rs", "substantive": "tests for each check"}
      ],
      "key_links": ["the pi runtime block pushes the paseo_ready row"],
      "prohibitions": ["No code comments", "No Paseo call that changes state from doctor"]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee doctor",
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"]
  },
  {
    "id": "pph-6",
    "feature": "paseo-pi-hardening",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Run the supervisor tick on Pi when configured",
    "deps": ["pph-3"],
    "decisions": ["D9"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/control_loop.rs"
    ],
    "read_first": [
      "docs/history/paseo-pi-hardening/CONTEXT.md",
      "docs/history/paseo-pi-hardening/plan.md",
      "packages/bee-rs/crates/bee/src/hooks/worker_guard.rs"
    ],
    "action": "Red first. control_loop.rs: read herding.supervisor_runtime from the main root config; absent or claude keeps today's behavior byte-identical; pi selects the Pi path; any other value refuses the supervisor tick before any spawn with a FIX line naming the two legal values. Pi path: resolve the supervisor slot from team.pi (the same resolver supervisor_model uses, with runtime pi); a herding slot whose agent has a paseo block gives model = provider/model when model is set, or the paseo model alone when the provider is pi, plus its thinking; a native or model slot gives its model; nothing usable falls back to no --model flag and prints the existing unconfigured warning. When herding.control_command is set it still wins and {MODEL} gets that model. Otherwise the default argv is [pi, --print, <prompt>, --model, <model>, --thinking, <level> (only when set), --no-session, --tools, read,grep,find,ls,bash]. The spawned process gets the environment variable BEE_SUPERVISOR_ALLOWED set to SUPERVISOR_ALLOWED_TOOLS, so the worker-guard hook from pph-3 enforces the allowlist. Find the spawn site by searching for build_control_argv and the env it passes. Tests: absent and claude give the exact current argv; pi with a paseo slot gives the pi argv with model and thinking; pi with control_command uses the template; an illegal value refuses with no spawn; the Pi path sets BEE_SUPERVISOR_ALLOWED to the allowlist. No code comments.",
    "must_haves": {
      "truths": [
        "with supervisor_runtime absent or claude the supervisor argv is byte-identical to today",
        "with supervisor_runtime pi the supervisor runs pi --print with the team.pi supervisor model and thinking and the read,grep,find,ls,bash tools",
        "the Pi supervisor process carries BEE_SUPERVISOR_ALLOWED equal to the supervisor allowlist",
        "an illegal supervisor_runtime refuses before any spawn",
        "the existing control_loop tests stay green"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/control_loop.rs", "substantive": "supervisor runtime selection and the Pi argv"}
      ],
      "key_links": ["the Pi supervisor spawn sets BEE_SUPERVISOR_ALLOWED"],
      "prohibitions": ["No code comments", "No change to dispatch, merge or route argv"]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee control_loop",
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md"]
  },
  {
    "id": "pph-7",
    "feature": "paseo-pi-hardening",
    "lane": "high-risk",
    "role": "docs",
    "change_class": "docs",
    "title": "Document the hardened Paseo channel, worker-guard and the Pi supervisor",
    "deps": ["pph-1", "pph-2", "pph-3", "pph-4", "pph-5", "pph-6"],
    "decisions": ["D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8", "D9", "D10"],
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
    "action": "Update the concepts to match the shipped code, citing each rule as paseo-pi-hardening D1 to D10 with its store id from bee decisions search --text paseo-pi-hardening. the-paseo-channel.md: the 15 s timeout and 3 s inspect cadence (D1), the silent-idle nudge and second-idle end (D2), UpdatedAt liveness and stalled (D3), the observed model fields and model_mismatch (D4), heartbeat delete on shutdown (D6), the doctor row (D7), the broker timer and the */30 backstop replacing the 5-minute default (D8, superseding paseo-pi D7), and the parent cascade: paseo run from inside the leader records it as parent, so archiving the leader archives every in-flight worker (D10); add the new files to Pointers. hook-runtime-the-worker-outward-guard.md: a section on worker-guard, the second hook past the herded marker, its deny list and the supervisor allowlist mode (D5, D9). the-supervisor-observer-and-its-interventions.md: herding.supervisor_runtime and the Pi argv (D9). docs/config-reference.md: herding.paseo.broker_tick_secs, the new heartbeat_cron default, herding.supervisor_runtime. Plain technical English, no code comments, no deferral words.",
    "must_haves": {
      "truths": [
        "the Paseo channel concept describes the timeouts, nudge, stalled, model check, heartbeat delete, doctor row, broker timer and parent cascade with their decision ids",
        "the worker-outward guard concept describes worker-guard and the supervisor allowlist",
        "the supervisor concept and the config reference name herding.supervisor_runtime and herding.paseo.broker_tick_secs",
        "bee knowledge check passes"
      ],
      "artifacts": [
        {"path": "docs/knowledge/areas/bee-herding/the-paseo-channel.md", "substantive": "the hardened channel"},
        {"path": "docs/knowledge/areas/hook-runtime/hook-runtime-the-worker-outward-guard.md", "substantive": "worker-guard"},
        {"path": "docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md", "substantive": "Pi supervisor"},
        {"path": "docs/config-reference.md", "substantive": "new keys"}
      ],
      "key_links": ["each concept cites paseo-pi-hardening decision ids"],
      "prohibitions": ["No change outside docs"]
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
| Rate | 10 s of fake time with 200 ms ticks | pph-1 | inspect called at most 4 times |
| State transition | Working → Idle, no result | pph-1 | exactly one send call |
| State transition | second silent idle | pph-1 | outcome idle timeout with the silent-idle message |
| Concurrency | send never during Working or Blocked | pph-1 | zero send calls in those states |
| Absent data | no UpdatedAt, no Model | pph-1, pph-2 | Working stays fresh; job.json says unverified; no warning |
| Mismatch | Model differs from config | pph-2 | one warning; paseo_model_mismatch true; no archive call |
| Security: deny | git push, gh pr create, pi -p, paseo ls, bee dispatch prepare | pph-3 | each denied with a FIX line |
| Security: allow | cargo test, git commit, bee cells list | pph-3 | each allowed |
| Security: compound | `cargo test && git push` | pph-3 | denied |
| Fail closed | missing bee binary in the belt | pph-3 | the tool call is blocked |
| Regression | every other hook under the marker | pph-3 | exits 0 with no output |
| Cost | broker tick with no news | pph-4 | zero sendUserMessage calls |
| Concurrency | slow tick longer than the interval | pph-4 | one tick in flight |
| Cleanup | shutdown with a failing delete | pph-4 | marker removed, no throw |
| Setup | daemon down, old version, no auth, orphan marker | pph-5 | each named in the row detail |
| Config | supervisor_runtime absent, claude, pi, bogus | pph-6 | today's argv, today's argv, pi argv, refusal |
| Live | one Paseo Pi worker run in a sandbox repo after merge | leader | job.json carries the observed model; a worker git push is refused |

## Open Questions

(none)

## Out of scope

- Seatworks' per-role Pi config folder (`PI_CODING_AGENT_DIR`) and readable worker titles in the Paseo app: not among the eight items the user named.
- Watching worker timelines with a coded detector: a separate past decision kept it out.
