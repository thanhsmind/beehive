---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: SLP Detector

## Summary

bee gets a cheap watcher for herded workers. It is plain code, no model.
Every supervisor tick, and every Paseo heartbeat of a Pi leader, it reads
the jobs that are running and writes a small event when one of four things
happens: a worker is close to a budget, a worker edits tests while the
contract they test is still open, a worker touches a file its cell did not
declare, or a worker undoes a change it made earlier. The supervisor reads
those events, looks at the context, and either asks its open question or
records that it looked and chose silence. Nothing speaks to a worker or to
the human from the watcher itself.

Mode: `standard` — 3 risk flags: external-systems (the Pi extension's
Paseo heartbeat path), public-contracts (two new verbs, a new store, a new
`--events` flag, four new signal names), multi-domain (Rust, TypeScript
extension, the supervisor prompt).
Why this is the least workflow that protects the work: the new verbs and
the store are a contract other code reads, so each slice needs its own
proof, but no hard-gate flag is present.

Playbook: `skills/bee-planning/playbooks/feature.md` — cited, not copied.

## Requirements (from CONTEXT.md)

- D1 (store d19eec05): `bee detector tick --json`, code only, reads existing surfaces, appends to `.bee/supervisor/events.jsonl` at the control root, writes nothing else, speaks to nobody.
- D2 (5231cdf4): Event row `{id, ts, signal, job_id, cell_id, target_session, point_key, excerpt, source}`; four closed signal names; the same four join `KNOWN_SIGNALS` and `INTERVENTION_SIGNALS`; `point_key` is the supervisor's intervention point key.
- D3 (936eb9a8): the four deterministic rules over live jobs — budget-80 (ceiling, claims, tokens), test-on-unstable-contract, boundary-approach, self-correction with per-job state in `.bee/supervisor/detector/<job_id>.json`; `estimate_minutes` stays unwritten.
- D4 (699974db): `bee detector events [--unjudged] [--json]` in the supervisor's allowlist and prompt; `--events <id,...>` on `bee supervisor record` marks events judged; one prompt paragraph per signal; an event never creates an intervention by itself; delivery unchanged.
- D5 (a6232035): the tick runs in code at the start of every supervisor control-loop tick, and beside `bee herding broker tick --json` in the Pi leader's heartbeat handler; no second heartbeat; never news for the leader; once per (job, signal, point_key).
- D6 (36568aaa): `bee herding status` shows per-job event counts by signal.
- D7 (8fac7f27): out of scope — model classifier, mid-turn delivery, new heartbeat, far-eye, `estimate_minutes`.
- Contract decision for test-writing cells: `03838ea8` (`contract:detector-events`).
- D3 refinement (32492fc4, hat wave): changed set = diff --name-only against `base_sha` plus untracked files, run against the job cwd; `base_sha` = the cwd's HEAD recorded in job.json at spawn, else captured at the first tick into the detector state file; `.bee/` and `docs/decisions/` excluded; boundary-approach uses the cells judge's `declared_covers`; path point keys `<signal>-<job_id>-<8 hex of sha256(path)>`; ceiling from `continued_at` when present; token rule reads the last finished round.
- D4 refinement (e3de051a, hat wave): `--events` legal on every record kind; judged = named by any row in observations.jsonl or interventions.jsonl; silence keeps `--signal none`; `No detector events.` on a missing or empty store; status carries `events` counts plus `unjudged`; the heartbeat runs the tick in the broker tick's `finally` with a 30 s timeout.

## Load-bearing claims

Labels: `read` (file opened at that line), `ran` (command run, output held). Evidence is a verbatim substring of the anchored line(s).

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The signal set is a closed const the record verb validates against, so adding four names is one edit | read | packages/bee-rs/crates/bee/src/verbs/supervisor.rs:303 | pub(crate) const KNOWN_SIGNALS: [&str; 7] = [ |
| 2 | Intervention rows have their own closed set that must also learn the names | read | packages/bee-rs/crates/bee/src/verbs/supervisor.rs:313 | pub(crate) const INTERVENTION_SIGNALS: [&str; 8] = [ |
| 3 | Observation rows refuse unknown signals on read, so the event store must be a separate file | read | packages/bee-rs/crates/bee/src/verbs/supervisor.rs:376 | if !KNOWN_SIGNALS.contains(&signal.as_str()) { |
| 4 | A verb group hooks into the dispatcher through a `try_native` guard on its first word | read | packages/bee-rs/crates/bee/src/verbs/supervisor.rs:3239 | if args.first()?.to_str()? != "supervisor" { |
| 5 | The dispatcher chains `try_native` calls in verbs/mod.rs | read | packages/bee-rs/crates/bee/src/verbs/mod.rs:136 | if let Some(code) = supervisor::try_native(args, t0) { |
| 6 | Verb metadata lives in the generated registry payload compiled into the binary | read | packages/bee-rs/crates/bee/src/registry.rs:13 | pub const REGISTRY_PAYLOAD: &str = include_str!("generated/registry_payload.json"); |
| 7 | catalog.rs pins each registered verb with a resolve test | read | packages/bee-rs/crates/bee/src/catalog.rs:957 | let (entry, rest) = resolve(&["herding", "notify"]).expect("herding.notify is in the registry"); |
| 8 | The supervisor's tool surface is an enumerated string, one verb per line | read | packages/bee-rs/crates/bee/src/herding/control_loop.rs:333 | Bash(.bee/bin/bee supervisor record:*),\ |
| 9 | The shipped prompt is pinned to KNOWN_SIGNALS by a test, so the prompt must learn the names in the same cell | read | packages/bee-rs/crates/bee/src/herding/control_loop.rs:1311 | for signal in KNOWN_SIGNALS { |
| 10 | resolve_iteration has a supervisor-only arm where the in-code tick can run | read | packages/bee-rs/crates/bee/src/herding/control_loop.rs:578 | if role == Role::Supervisor { |
| 11 | The supervisor tick interval is 900 s | read | packages/bee-rs/crates/bee/src/herding/control_loop.rs:116 | const SUPERVISOR_DEFAULT_INTERVAL: u64 = 900; |
| 12 | job.json carries `ceiling_secs` for budget-80's time rule | read | packages/bee-rs/crates/bee/src/herding/run.rs:2593 | "ceiling_secs": opts.ceiling_secs, |
| 13 | job.json gets a `usage` map after a round for budget-80's token rule | read | packages/bee-rs/crates/bee/src/herding/run.rs:3136 | fn record_round_usage( |
| 14 | A cell's budget is `max_claims` and friends, counted in claims, not minutes | read | packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:124 | const BUDGET_KEYS: [&str; 3] = ["max_claims", "max_failed_attempts", "max_same_signature"]; |
| 15 | The claims count is already exposed crate-wide | read | packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:740 | pub(crate) fn attempts_since_budget_reset(cell: &Map<String, Value>) -> MR<Vec<Value>> { |
| 16 | Derived contract status exists as a crate function | read | packages/bee-rs/crates/bee/src/verbs/decisions/read.rs:419 | pub(crate) fn contract_status(root: &Path, id: &str) -> Ex<ContractStatus> { |
| 17 | Nothing writes `estimate_minutes`, so budget-80 cannot read it | read | packages/bee-rs/crates/bee/src/verbs/supervisor.rs:2589 | /// The estimate field the overrun counter reads. NOTHING in bee writes it |
| 18 | The stall read uses `git diff HEAD` on the job cwd, the same source the changed set comes from | read | packages/bee-rs/crates/bee/src/herding.rs:715 | let diff = git(&["diff", "HEAD"])?; |
| 19 | A round's result file is `result-<round>.json` in the job mailbox, the live-job test | read | packages/bee-rs/crates/bee/src/herding/mailbox.rs:125 | format!("result-{round}.json") |
| 20 | The Pi heartbeat handler already runs the broker tick where the detector tick goes beside it | read | .pi/extensions/bee-guard/events.ts:384 | ["herding", "broker", "tick", "--json"], |
| 21 | The heartbeat prompt detector is an exported function with a node-run contract test | read | .pi/extensions/bee-guard/paseo-heartbeat.ts:99 | export function isHeartbeatPrompt(text: unknown): boolean { |
| 22 | The prompt says there is no eighth surface and names the Detector as a separate feature — the wording to change | read | skills/bee-herding/references/supervisor-prompt.md:61 | That is the whole list. There is no eighth surface: if a fact is not visible |
| 23 | Status already composes per-job display fields in one place where event counts join | read | packages/bee-rs/crates/bee/src/herding.rs:1260 | + &insert_usage(&mut job_map, job_usage(bee_dir, &job_id), &job_obj) |
| 24 | A cell record carries a `files` list the boundary rule compares against | read | packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:534 | match cell.get("files") { |
| 25 | The team roster hash for the role plan | ran | .bee/bin/bee team show --runtime claude --json (canonical sorted {description,role} rows, sha256) | 30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112 |
| 26 | Workers commit while the job is live, so a diff against HEAD loses committed paths | read | .claude/agents/bee-build.md:3 | write the code, commit, and cap |
| 27 | The only `base_sha` bee computes today sits on the done-result object, never on the live job record | read | packages/bee-rs/crates/bee/src/herding/run.rs:5131 | object {branch, head_sha, base_sha, ahead, dirty, changed_paths} |
| 28 | The spawn-time job record writes `created_at`, the line beside which `base_sha` joins | read | packages/bee-rs/crates/bee/src/herding/run.rs:2595 | "created_at": chrono::Utc::now().to_rfc3339(), |
| 29 | A continued job stamps `continued_at`, the ceiling clock's restart | read | packages/bee-rs/crates/bee/src/herding/run.rs:4936 | m.insert("continued_at".into(), Value::String(chrono::Utc::now().to_rfc3339())); |
| 30 | A cell record loads by id outside the cells verbs | read | packages/bee-rs/crates/bee/src/verbs/cells/read.rs:180 | pub(crate) fn read_cell(root: &Path, id: &str) -> Result<Option<Value>, Delegate> { |
| 31 | The cells judge already decides whether a declared `files` entry covers a changed path | read | packages/bee-rs/crates/bee/src/verbs/cells/judge.rs:119 | pub(crate) fn declared_covers(declared: &[Value], file: &str) -> bool { |
| 32 | The contract walk over a cell's decisions hoists its reads into `contract_status_over` | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs:1286 | match crate::verbs::decisions::contract_status_over( |
| 33 | The control loop's real provider calls `resolve_iteration`, so the in-code tick sits in the provider, not in `resolve_iteration` which tests call directly | read | packages/bee-rs/crates/bee/src/herding/control_loop.rs:889 | resolve_iteration(&self.main_root, self.role, self.turn_ceiling) |
| 34 | The record verb has no `events` flag today | read | packages/bee-rs/crates/bee/src/verbs/supervisor.rs:3235 | const RECORD_FLAGS: [&str; 7] = |
| 35 | The heartbeat handler has a `finally` where the detector tick runs after any broker outcome | read | .pi/extensions/bee-guard/events.ts:404 | } finally { |
| 36 | The config reference carries a `herding.paseo.*` row to mirror for the token budget key | read | docs/config-reference.md:389 | herding.paseo.heartbeat_cron |
| 37 | The empty-store wording to mirror | read | packages/bee-rs/crates/bee/src/verbs/supervisor.rs:3323 | "No supervisor observations.".to_string() |

## Discovery

Scout report: `.bee/mailbox/job-1792075577054-1658640-1/report-1.md`
(gather, 2026-10-15). It found the closed sets, the control-loop role arm,
the budget fields, the usage writer, the heartbeat handler and the contract
status function. Two facts it did not know were read by the leader: the
stall fingerprint is `git diff HEAD` (claim 18) and a live job is one whose
`result-<round>.json` is absent (claim 19). The prompt-pin test (claim 9)
decides the cell split: the prompt learns the four words in the same cell
that widens the sets.

Hat wave (three seats, 2026-10-15): one blocker — a changed set read from
`git diff HEAD` loses committed paths (claim 26) and untracked files — fixed
by the D3 refinement (claims 27–29); warnings applied: the tick's home is
the real argv provider (claim 33), `declared_covers` is the boundary
matcher (claim 31), the contract walk hoists its reads (claim 32), `--events`
is legal on every kind (claim 34), the heartbeat runs the tick in `finally`
(claim 35), the token key gets a config-reference row (claim 36).
Dismissed: folding det-4 into det-1 and det-3 (prose belongs to the docs
role; a second regen is cheap); the shared-worktree attribution limit is
named in the D3 refinement, not solved.

## Approach

Recommended path: one new verb group `bee detector` in
`verbs/detector.rs` (D1, D2, D4) holding the store, the read verb and the
tick; the four rules read only what claims 12–19, 24 and 27–32 name (D3 and
its refinement); the two trigger points are a one-line in-code call in the
real argv provider's supervisor branch and one more `execFile` in the broker
tick's `finally` (D5); status counts ride the existing per-job composer (D6).

Rejected alternatives:
- Events inside `observations.jsonl` — claim 3 shows the reader refuses the shape.
- A detector role in the control loop with its own model — D1 forbids a model; the code tick costs nothing.
- A new Paseo heartbeat named for the detector — paseo-pi D8 (2aa8417a) allows one per agent.

Risk map:
- Rule false positives (boundary-approach on a cell with a trailing-`*` pattern) / MEDIUM / Lands in: det-2 / proof: fixture tests for exact and `*` patterns.
- Prompt-pin test red between cells / MEDIUM / Lands in: det-1 / proof: `cargo test control_loop` green in det-1.
- Heartbeat turn slowed by the tick / LOW / Lands in: det-3 / proof: 60 s timeout, errors logged, news unchanged, node test.
- Status output shape change breaks a reader / LOW / Lands in: det-3 / proof: field added, no field removed, existing status tests green.

Waves: det-1 alone (it owns the sets and the prompt words); then det-2 and
det-4 in parallel (disjoint files); then det-3 (needs the tick from det-2).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"det-1 to det-3 change Rust verbs, the control loop, status and the Pi extension."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests ride each code cell; the live proof is the leader's."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"det-4 rewrites the supervisor prompt paragraphs and the bee-herding concepts."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"conditional","role":"read","condition":"a worker report needs a multi-file check","reason":"Discovery is done; gathers only on demand."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Review is user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the plan consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Standard plan check, 7 product files."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Three seats on standard work."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Three seats on standard work."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Standard plan check."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Standard plan check."}
  ]
}
```

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 store and names | `events.jsonl`, `bee detector events`, four names in both sets, `--events` on record, the prompt knows the words | Every following piece reads this contract | `bee supervisor record --signal budget-80 --kind silence --note x` is accepted; `bee detector events` prints an empty list | det-2, det-4 |
| 2 rules | `bee detector tick` fires the four rules once per point | The store exists to receive them | A fixture job past 80% of its ceiling makes one `budget-80` row; a second tick adds none | det-3 |
| 3 triggers, status, prompt | supervisor loop pre-step, heartbeat handler, status counts, prompt paragraphs and area docs | The tick exists to be run | `bee herding status` shows `events=budget-80:1`; the supervisor prompt names ten signals | close |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| det-1 | Add the detector event store, the read verb and the four signal names | verbs/detector.rs (new), verbs/mod.rs, verbs/supervisor.rs, generated/registry_payload.json, catalog.rs, supervisor-prompt.md and its five generated copies, release-manifest.json | — | `bee supervisor record` accepts the four new signals and `--events`; `bee detector events --unjudged` lists what no tick judged | cargo test supervisor, detector, catalog, control_loop, registry_contracts |
| det-2 | Add `bee detector tick` with the four deterministic rules | verbs/detector.rs, generated/registry_payload.json, catalog.rs | det-1 | one tick on a fixture job writes one row per rule hit and a second tick writes none | cargo test detector, catalog, registry_contracts |
| det-3 | Run the tick from the supervisor loop and the Pi heartbeat, count events in status | herding/control_loop.rs, herding.rs, events.ts, tests/pi_paseo_heartbeat_contracts.rs | det-2 | a supervisor tick fires events before the model reads; a heartbeat runs the tick after the broker tick; `bee herding status` shows per-job counts | cargo test control_loop, herding, pi_paseo_heartbeat_contracts |
| det-4 | Teach the supervisor prompt and the bee-herding concepts the detector | supervisor-prompt.md and its five copies, release-manifest.json, the-supervisor-observer concept, the-paseo-channel concept, area index | det-1 | the prompt reads events as its seventh command, judges ten signals, names reviewed event ids on silence | cargo test control_loop; bee knowledge index --check; bee dev release-manifest --check |

Paths under `packages/bee-rs/crates/bee/src/` in the table are spelled in full in the packet below.

```json
[
  {
    "id": "det-1",
    "feature": "slp-detector",
    "title": "Add the detector event store, the read verb and the four signal names",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["D1", "D2", "D4", "03838ea8"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/detector.rs",
      "packages/bee-rs/crates/bee/src/verbs/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/supervisor.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/catalog.rs",
      "skills/bee-herding/references/supervisor-prompt.md",
      ".agents/skills/bee-herding/references/supervisor-prompt.md",
      ".claude/skills/bee-herding/references/supervisor-prompt.md",
      ".claude-plugin/skills/bee-herding/references/supervisor-prompt.md",
      ".codex-plugin/skills/bee-herding/references/supervisor-prompt.md",
      ".opencode/skills/bee-herding/references/supervisor-prompt.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/slp-detector/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/supervisor.rs — `rg -n 'KNOWN_SIGNALS|INTERVENTION_SIGNALS|RECORD_FLAGS|fn record_into|fn from_value|fn supervisor_dir|fn control_root_path|pub fn try_native'` — hits today: 303 `pub(crate) const KNOWN_SIGNALS: [&str; 7] = [`, 313 `pub(crate) const INTERVENTION_SIGNALS: [&str; 8] = [`, 336 `fn record_into(`, 376 `if !KNOWN_SIGNALS.contains(&signal.as_str()) {`, 3238 `pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {`",
      "packages/bee-rs/crates/bee/src/verbs/mod.rs — `rg -n 'supervisor::try_native'` — hit today: 136 `if let Some(code) = supervisor::try_native(args, t0) {`",
      "packages/bee-rs/crates/bee/src/catalog.rs — `rg -n 'resolve\\(&\\[\"herding\", \"notify\"\\]\\)'` — hit today: 957 `let (entry, rest) = resolve(&[\"herding\", \"notify\"]).expect(\"herding.notify is in the registry\");`",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json — the `supervisor.list` and `supervisor.record` entries, the shape to copy",
      "packages/bee-rs/crates/bee/src/herding/control_loop.rs — `rg -n 'fn the_shipped_prompt_pins_the_record_verbs_own_closed_sets'` — hit today: 1302; this test pins the prompt to KNOWN_SIGNALS",
      "skills/bee-herding/references/supervisor-prompt.md — `rg -n -- '--signal \\{'` — the record command line whose signal list grows"
    ],
    "affects_skills": ["skills/bee-herding/references/supervisor-prompt.md"],
    "affects_specs": [],
    "action": "Per D1, D2 and D4. (1) Add module verbs/detector.rs with `pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode>` that answers only when args[0] is `detector`, and hook it in verbs/mod.rs right after the `supervisor::try_native` block in the same shape. (2) The store: `.bee/supervisor/events.jsonl`, resolved against the control root with the same helpers supervisor.rs uses (`supervisor_dir`, `control_root_path`; widen their visibility to pub(crate) if needed). Event row per D2: `id` (uuid v4 string), `ts` (RFC 3339 UTC), `signal`, `job_id`, `cell_id` (string or null), `target_session` (string or null), `point_key` (the same slug rule `--point-key` enforces, 80 chars max), `excerpt` (one sentence, 300 chars max, whitespace collapsed), `source` (the surface read). Expose pub(crate) `EVENT_SIGNALS: [&str; 4]` holding exactly self-correction, boundary-approach, test-on-unstable-contract, budget-80; pub(crate) `append_event(control_root, &Event) -> Result<(), String>` that validates signal, point_key and excerpt before the append and refuses by name otherwise, creating the directory on first write; pub(crate) `read_events(control_root) -> Vec<Event>` oldest first, skipping malformed lines; pub(crate) `events_for_job(control_root, job_id) -> Vec<Event>`. (3) Verb `bee detector events [--unjudged] [--json]`: text form one line per row `- <ts> [<signal>] <job_id> <cell_id or -> <point_key> — <excerpt>`, JSON form `{events: [rows]}`; `--unjudged` keeps only rows whose id no row in observations.jsonl or interventions.jsonl names in its `events` array (D4 refinement e3de051a); a missing or empty store prints `No detector events.` in text form (JSON `{events: []}`) and exits 0. Register `detector.events` (flags unjudged, json) in generated/registry_payload.json in the same shape as `supervisor.list` (the precedent: `herding.notify` and `herding.fork` were added by hand in that shape), and add a catalog.rs resolve test beside the `herding notify` one. Do NOT register or implement `detector tick` here (det-2 owns it). (4) supervisor.rs: add the four names to KNOWN_SIGNALS (11 entries) and INTERVENTION_SIGNALS (12); add optional `--events <id,...>` to RECORD_FLAGS (today `const RECORD_FLAGS: [&str; 7] =`), legal on every record kind, every id must exist in events.jsonl (refused by name otherwise, nothing written), stored on the row as `events: [..]` only when given, on observation rows and on mailbox rows alike; Observation::from_value and the mailbox row reader read an optional `events` array and tolerate its absence; `supervisor list` and `supervisor pending` print ` [events: a,b]` after the note or question when present. (5) In skills/bee-herding/references/supervisor-prompt.md change only the record command line block: extend the `--signal {...}` set with the four names in this order after dead-lead, and add `[--events <id,...>]` after `[--tick <n>]`; then run `.bee/bin/bee dev regen` so the five generated copies and the release manifest update, and commit them in the same commit. Tests (author only what is not pinned): unknown signal still refused naming the set; `--events` happy on a silence row, happy on an intervention row (the row carries the ids and the event leaves `--unjudged`), and refused with an unknown id; from_value with and without `events`; `detector events` empty store prints `No detector events.`, two rows, `--unjudged` after a silence row names one and after an intervention row names the other; catalog resolve for detector events; registry_contracts green; the control_loop prompt-pin test green.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee supervisor && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee detector && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee catalog && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee control_loop && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "bee supervisor record --kind silence --signal budget-80 --note x is accepted",
        "bee supervisor record --events <unknown id> is refused by name and writes nothing",
        "bee detector events --unjudged lists only rows no observation, silence or mailbox row names",
        "bee detector events with no store prints No detector events. and exits 0"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/detector.rs", "substantive": "Event struct, EVENT_SIGNALS, append_event, read_events, events_for_job, the events verb and their tests"},
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "a detector.events entry"}
      ],
      "key_links": [
        "verbs/mod.rs calls detector::try_native",
        "supervisor record validates --events against events.jsonl",
        "the supervisor prompt's record command line names the four signals and --events, five copies regenerated"
      ],
      "prohibitions": [
        "No change to the Observation row fields that exist today",
        "No detector tick verb or rule in this cell",
        "No write to any store other than events.jsonl and observations.jsonl",
        "No code comments"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "det-2",
    "feature": "slp-detector",
    "title": "Add bee detector tick with the four deterministic rules",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": ["det-1"],
    "decisions": ["D1", "D3", "D5", "32492fc4", "03838ea8"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/detector.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/catalog.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "read_first": [
      "docs/history/slp-detector/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/detector.rs (det-1's module: Event, EVENT_SIGNALS, append_event, read_events)",
      "packages/bee-rs/crates/bee/src/herding/mailbox.rs — `rg -n 'pub\\(crate\\) fn mailbox_dir|pub\\(crate\\) fn job_path|pub\\(crate\\) fn result_path|fn result_filename'` — hits today: 60 `pub(crate) fn mailbox_dir(bee_dir: &Path, job_id: &str) -> PathBuf {`, 66 `pub(crate) fn job_path(bee_dir: &Path, job_id: &str) -> PathBuf {`, 116 `pub(crate) fn result_path(bee_dir: &Path, job_id: &str, round: u32) -> PathBuf {`, 125 `format!(\"result-{round}.json\")`",
      "packages/bee-rs/crates/bee/src/herding/run.rs — `rg -n '\"ceiling_secs\": opts.ceiling_secs|\"created_at\"|\"cell_id\"|\"leader_session\"|\"round\"|fn record_round_usage'` — the job.json fields the rules read",
      "packages/bee-rs/crates/bee/src/herding.rs — `rg -n 'fn worktree_fingerprint|let diff = git'` — hits today: 708 `fn worktree_fingerprint(cwd: &str) -> Option<String> {`, 715 `let diff = git(&[\"diff\", \"HEAD\"])?;` — the git invocation shape to mirror",
      "packages/bee-rs/crates/bee/src/verbs/cells/validate.rs — `rg -n 'pub\\(crate\\) fn resolve_cell_budgets|pub\\(crate\\) fn attempts_since_budget_reset|pub\\(crate\\) fn check_cell_budgets'` — hits today: 722, 740, 777; the claims count rule reuses these",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs — `rg -n 'fn event_has_contract_tag|fn contract_citation_refusal'` — how a cell's decisions resolve to contract-tagged decisions",
      "packages/bee-rs/crates/bee/src/verbs/decisions/read.rs — `rg -n 'pub\\(crate\\) enum ContractStatus|pub\\(crate\\) fn contract_status_over|pub\\(crate\\) fn contract_status\\('` — hits today: 324, 394, 419",
      "packages/bee-rs/crates/bee/src/verbs/cells/read.rs — `rg -n 'pub\\(crate\\) fn read_cell'` — hit today: 180 `pub(crate) fn read_cell(root: &Path, id: &str) -> Result<Option<Value>, Delegate> {` — the one reader for a cell record by id; never a second parser",
      "packages/bee-rs/crates/bee/src/verbs/cells/judge.rs — `rg -n 'pub\\(crate\\) fn declared_covers|pub\\(crate\\) fn frozen_judge_hits'` — hits today: 119 `pub(crate) fn declared_covers(declared: &[Value], file: &str) -> bool {`, 200 `pub(crate) fn frozen_judge_hits(changed: &Value, declared: &Value) -> Vec<(String, &'static str)> {` — the boundary matcher to reuse",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs — `rg -n 'contract_status_over\\('` — hit today: 1286 `match crate::verbs::decisions::contract_status_over(` — the hoisted-reads pattern for a whole decisions walk",
      "packages/bee-rs/crates/bee/src/herding/run.rs — `rg -n '\"created_at\": chrono::Utc::now\\(\\).to_rfc3339\\(\\),|\"continued_at\"'` — hits today: 2595 (and the two sibling spawn sites near 3194 and 3500) `\"created_at\": chrono::Utc::now().to_rfc3339(),`, 4936 `m.insert(\"continued_at\".into(), Value::String(chrono::Utc::now().to_rfc3339()));`"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D1, D3 with its refinement 32492fc4, and D5. (1) run.rs: at each of the three spawn sites that write `\"created_at\": chrono::Utc::now().to_rfc3339(),` also write `base_sha`: the cwd's HEAD commit read through one small helper that runs the version-control CLI with `-C <cwd>` and `rev-parse HEAD` the way worktree_fingerprint in herding.rs runs it, null when the cwd is not a checkout; one helper, three call sites. (2) Add `bee detector tick [--json]` to verbs/detector.rs, register `detector.tick` (flag json) in generated/registry_payload.json in the same shape as `detector.events`, and add a catalog.rs resolve test. The tick resolves the control root, lists every `.bee/mailbox/<job>/job.json`, and treats a job as live when `result-<round>.json` is absent for the job's `round` (default 1). It loads the job's cell record with `crate::verbs::cells::read_cell` when `cell_id` is set; with no cell only the ceiling and token rules apply. The changed set: `diff --name-only <base_sha>` plus `ls-files --others --exclude-standard`, both through the same CLI helper against the job cwd, repo-relative paths, minus every path under `.bee/` or `docs/decisions/`; `base_sha` is job.json `base_sha`, else the value stored in the job's detector state file, else the cwd's HEAD captured now and stored there (the first tick of an old job then sees only untracked paths). Rules, each firing at most one event per (job_id, signal, point_key) — before appending, skip any triple already present in events.jsonl: budget-80 (a) ceiling: now minus `continued_at` when present else `created_at`, at or above 0.8 times `ceiling_secs`, point_key `budget-80-ceiling-<job_id>`; (b) claims: `attempts_since_budget_reset(cell).len() + 1` at or above 0.8 times `resolve_cell_budgets(cell).max_claims`, point_key `budget-80-claims-<job_id>`; (c) tokens: job.json `usage.total_tokens` (the last finished round) at or above 0.8 times config `herding.budget.tokens_per_job` when that key is a positive number, point_key `budget-80-tokens-<job_id>`. test-on-unstable-contract: walk the cell's `decisions` once with the hoisted reads the way handlers_write.rs does around `contract_status_over(`; for each id naming an active decision with a `contract:<name>` tag whose status is Unsettled, when the changed set holds a test path (a `tests/` or `spec/` path segment, or a file name matching `*_test.*`, `*.test.*` or `*_tests.*`), fire with point_key `unstable-contract-<name>-<job_id>`. boundary-approach: when the cell's `files` is non-empty, every changed path for which `declared_covers(files, path)` is false fires with point_key `boundary-approach-<job_id>-<key>` where `<key>` is the first 8 hex chars of sha256(path); the excerpt carries the path. self-correction: read `.bee/supervisor/detector/<job_id>.json` (`{base_sha, changed_paths: [..], at}`); every path present there and absent from the current changed set fires with point_key `self-correction-<job_id>-<key>` (same key rule); then write the current set and base_sha to that file (create the directory on first write). Every event carries `cell_id`, `target_session` = job.json `leader_session`, `source` naming the surface (`job.json`, `cell`, `diff`, `config`), and an `excerpt` of one sentence holding the numbers or the path. A job whose job.json is unreadable, whose cwd is missing, or whose cwd is not a checkout prints one warning line on stderr and is skipped; the tick exits 0 unless the store itself cannot be written. Output: with `--json` `{fired: [events], scanned: n}`; text form one line per fired event in the `detector events` shape and a final line `detector: scanned <n> jobs, fired <m>`. Expose pub(crate) `tick_quiet(control_root: &Path) -> usize` returning the fired count with warnings on stderr, for det-3. Tests on tempdir fixtures (a real checkout per job cwd, job.json and cell records written as plain JSON in the shapes the writers use): every det-2 row of the plan's test matrix, plus the dedupe case (second tick fires none).",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee detector && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee catalog && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts",
    "must_haves": {
      "truths": [
        "a live job at 80% of ceiling_secs produces exactly one budget-80 row across two ticks",
        "a job with result-<round>.json present produces no row and is not counted in scanned",
        "a changed path that declared_covers accepts fires no boundary-approach row; a path under .bee/ never fires",
        "a path committed mid-round stays in the changed set (diff against base_sha), so no self-correction row fires on a commit",
        "an untracked new test file counts as a changed path",
        "the tick never calls a model, paseo, or the network"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/detector.rs", "substantive": "the tick verb, the four rules, tick_quiet, the per-job state file, and fixture tests for every matrix row"},
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "a detector.tick entry"}
      ],
      "key_links": [
        "the tick reads cells through the existing cell reader and budgets through validate.rs",
        "the tick reads contract status through decisions/read.rs contract_status",
        "tick_quiet is pub(crate) for the control loop"
      ],
      "prohibitions": [
        "No write outside .bee/supervisor/events.jsonl and .bee/supervisor/detector/",
        "No write to estimate_minutes",
        "No second parser for cells, job.json or decisions",
        "No code comments"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "det-3",
    "feature": "slp-detector",
    "title": "Run the tick from the supervisor loop and the Pi heartbeat, count events in status",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": ["det-2"],
    "decisions": ["D4", "D5", "D6", "e3de051a", "03838ea8"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/control_loop.rs",
      "packages/bee-rs/crates/bee/src/herding.rs",
      ".pi/extensions/bee-guard/events.ts",
      ".pi/extensions/bee-guard/paseo-heartbeat.ts",
      "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/slp-detector/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/herding/control_loop.rs — `rg -n 'const SUPERVISOR_ALLOWED_TOOLS|Bash\\(.bee/bin/bee supervisor list:\\*\\)|SUPERVISOR_FORBIDDEN_TOOL_TOKENS|pub\\(crate\\) fn resolve_iteration|if role == Role::Supervisor \\{'` — hits today: 325 `const SUPERVISOR_ALLOWED_TOOLS: &str = \"Bash(.bee/bin/bee status:*),\\`, 558 `pub(crate) fn resolve_iteration(main_root: &Path, role: Role, turn_ceiling: u64) -> Result<(Argv, SpawnSpec), String> {`, 578 `if role == Role::Supervisor {`",
      "packages/bee-rs/crates/bee/src/herding.rs — `rg -n 'insert_usage\\(&mut job_map|fn insert_usage|fn insert_read_only'` — hits today: 1031 `fn insert_usage(`, 1260 and 1349 `+ &insert_usage(&mut job_map, job_usage(bee_dir, &job_id), &job_obj)` — the per-job composer, two call sites",
      ".pi/extensions/bee-guard/events.ts — `rg -n '\\[\"herding\", \"broker\", \"tick\", \"--json\"\\]|const tick = parseTick|quietHeartbeatTurn ='` — hit today: 384 `[\"herding\", \"broker\", \"tick\", \"--json\"],` inside the heartbeat branch of the input handler",
      ".pi/extensions/bee-guard/paseo-heartbeat.ts — `rg -n 'export function isHeartbeatPrompt|export function parseTick|export function heartbeatText'` — hits today: 99, 329, 346 — exported helpers the node contract tests import",
      "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs — `rg -n 'fn run_js_test|fn test_is_heartbeat_prompt'` — the node-run test harness to extend",
      "packages/bee-rs/crates/bee/src/verbs/detector.rs (det-2: tick_quiet, events_for_job)"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D4 with its refinement e3de051a, D5 and D6. (1) control_loop.rs: in `impl ArgvProvider for RealArgvProvider`, inside `fn spec_for`, before the line `resolve_iteration(&self.main_root, self.role, self.turn_ceiling)`, when `self.role == Role::Supervisor` call `crate::verbs::detector::tick_quiet(&self.main_root)` and, when it returns more than zero, print one stderr line `supervisor: detector fired <n> event(s) this tick`; a tick failure never fails the iteration; resolve_iteration itself does not change (tests call it directly). Add the line `Bash(.bee/bin/bee detector events:*),\\` to SUPERVISOR_ALLOWED_TOOLS directly before the `supervisor list` line, and extend the existing allowlist test to assert it while the forbidden-token test stays green. (2) herding.rs status: add `insert_events(job_map, bee_dir, job_id) -> String` beside insert_usage and call it at both composer call sites; it reads `detector::events_for_job` and the judged ids, writes JSON `events` as an object `{<signal>: <count>}` over all events (an empty object when none) and `unjudged` as a number, and returns the text suffix ` events=<signal>:<n>[,<signal>:<n>] unjudged=<k>` sorted by signal name, or an empty string when there are no events. No existing field moves or disappears. (3) paseo-heartbeat.ts: add `export async function runDetectorTick(beeBinary: string, directory: string, env: Record<string,string>, execFileFn): Promise<boolean>` that runs `[\"detector\", \"tick\", \"--json\"]` with a 30000 ms timeout, resolves true on exit 0 and false otherwise, logging `bee detector tick (heartbeat): <message>` on error, never throwing. events.ts heartbeat branch: inside the `finally` block that follows the broker tick, before `setTickRunning(false)`, `await runDetectorTick(beeBinary, directory, beltEnv(), cp.execFile)`; the result is discarded so quietHeartbeatTurn, heartbeatText and the news rule do not change, and a failed broker tick never skips it. (4) Tests: in control_loop.rs a test that RealArgvProvider::spec_for for Role::Supervisor on a tempdir main root with an empty mailbox still resolves and leaves no events file, and that resolve_iteration alone writes nothing under .bee/supervisor; herding.rs status tests for a job with two events of one signal (JSON object, unjudged count and text suffix) and a job with none (empty object, no suffix); in pi_paseo_heartbeat_contracts.rs a run_js_test that runDetectorTick calls execFile with argv `[\"detector\",\"tick\",\"--json\"]` and resolves false without throwing when execFile reports an error, plus a source-level assertion that events.ts calls runDetectorTick inside the heartbeat branch's finally block. (5) The two .pi/extensions files sit under a root the release manifest hashes: after the edits run `.bee/bin/bee dev regen` and commit the manifest with the change.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee control_loop && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee herding && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_paseo_heartbeat_contracts && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "a supervisor iteration runs the detector tick in the real argv provider before its argv resolves; resolve_iteration alone never ticks",
        "the supervisor allowlist carries detector events and no forbidden token",
        "bee herding status --json carries events as an object and unjudged as a number per job; the text line carries events=<signal>:<n> unjudged=<k> only when events exist",
        "a Paseo heartbeat runs detector tick in the broker tick's finally block and its outcome never changes the heartbeat turn text"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/control_loop.rs", "substantive": "the supervisor-arm pre-step and the widened allowlist with tests"},
        {"path": ".pi/extensions/bee-guard/paseo-heartbeat.ts", "substantive": "runDetectorTick exported and tested"}
      ],
      "key_links": [
        "resolve_iteration calls detector::tick_quiet in the supervisor arm",
        "events.ts calls runDetectorTick in the heartbeat branch",
        "status composes insert_events at both call sites"
      ],
      "prohibitions": [
        "No new paseo heartbeat or schedule",
        "No change to quietHeartbeatTurn, heartbeatText or the news rule",
        "No status field removed or renamed",
        "No code comments"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "det-4",
    "feature": "slp-detector",
    "title": "Teach the supervisor prompt and the bee-herding concepts the detector",
    "lane": "standard",
    "role": "docs",
    "status": "open",
    "deps": ["det-1"],
    "decisions": ["D1", "D3", "D4", "D5", "D6"],
    "files": [
      "skills/bee-herding/references/supervisor-prompt.md",
      ".agents/skills/bee-herding/references/supervisor-prompt.md",
      ".claude/skills/bee-herding/references/supervisor-prompt.md",
      ".claude-plugin/skills/bee-herding/references/supervisor-prompt.md",
      ".codex-plugin/skills/bee-herding/references/supervisor-prompt.md",
      ".opencode/skills/bee-herding/references/supervisor-prompt.md",
      "docs/history/codex-harness-hardening/release-manifest.json",
      "docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md",
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md",
      "docs/knowledge/areas/bee-herding/index.md",
      "docs/config-reference.md"
    ],
    "read_first": [
      "docs/history/slp-detector/CONTEXT.md",
      "docs/config-reference.md — `rg -n 'herding.paseo.heartbeat_cron'` — hits today: 389 (table row) and 537 (section) — the shape for the new `herding.budget.tokens_per_job` row",
      "docs/specs/slp-supervisor-lead-peer/slp-supervisor-lead-peer.md — sections 1 (Detector), 4.7, 4.8 and 5 S4",
      "skills/bee-herding/references/supervisor-prompt.md — `rg -n 'There is no eighth surface|Judge against exactly six signals|You look for six things|Record the silence, with its reason|supervisor record --kind'` — hits today: 61 `That is the whole list. There is no eighth surface: if a fact is not visible`, 67 `### 2. Judge against exactly six signals`",
      "docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md (frontmatter decisions and owns.code, the 'One tick, one record' and 'Four stores, one root' sections)",
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md — `rg -n 'The leader heartbeat and broker timer'`",
      ".claude/skills/bee-technical-writing/SKILL.md"
    ],
    "affects_skills": ["skills/bee-herding/references/supervisor-prompt.md"],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md",
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "action": "Per D1, D3, D4, D5 and D6, with bee-technical-writing. (1) supervisor-prompt.md: in step 1 add a seventh command `.bee/bin/bee detector events --unjudged` with one line saying what it lists; rewrite the 'no eighth surface' paragraph so it says the seven commands are the whole list, that the Detector is bee code (not you) which reads the jobs and writes one event per fact it sees, that an event is a fact and never a finding, and that you still do not scan transcripts or poll; rename step 2 to 'Judge against exactly ten signals' and update 'six things' wording; add four paragraphs in the existing style for self-correction, boundary-approach, test-on-unstable-contract and budget-80, each stating what the Detector saw (the D3 rule in plain words), what to read before speaking (the job's pane with `herding pane read`, the cell with `cells list`), that a healthy stream is a silence row, and that an intervention on an event carries the event's point_key as `--point-key`; in step 3 say that an observation or silence row made after reading events names the ids it judged with `--events`, and that an event nobody names stays in `--unjudged` for the next tick; keep the record command line det-1 already updated; keep every existing rule and the `Record the silence, with its reason` sentence. Then run `.bee/bin/bee dev regen` and commit the five copies and the manifest. (2) the-supervisor-observer concept: add a section 'The Detector' (what it is per D1, the rules per D3, the judged mark per D4, the two trigger points per D5), add verbs/detector.rs to owns.code, add the D-ids with their one-line gist to the decisions list, and replace the sentence that says no new poller was built with the current truth; keep the diagram, adding one state for events read. (3) the-paseo-channel concept: in 'The leader heartbeat and broker timer' add one sentence that the heartbeat also runs `bee detector tick --json` after the broker tick and that its outcome never changes the turn. (4) docs/config-reference.md: add a table row and a section for `herding.budget.tokens_per_job` in the shape of `herding.paseo.heartbeat_cron` (default: unset, meaning the token rule is off; a positive whole number of tokens per herded job; read by `bee detector tick`). (5) Run `.bee/bin/bee knowledge index` so index.md regenerates. Also say in step 2 of the prompt that an event names a job and that a job in a shared worktree can carry a sibling's path, so the pane read decides. Proof: the control_loop prompt-pin test green, `.bee/bin/bee knowledge index --check` green, `.bee/bin/bee dev release-manifest --check` green.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee control_loop && .bee/bin/bee knowledge index --check && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "the supervisor prompt lists seven read commands and ten signals",
        "the prompt tells a tick to name judged event ids with --events on its observation or silence row",
        "the five generated copies equal the source and the manifest check is green",
        "the bee-herding concept names verbs/detector.rs in owns.code and the slp-detector decisions"
      ],
      "artifacts": [
        {"path": "skills/bee-herding/references/supervisor-prompt.md", "substantive": "seventh command, ten-signal section with four new paragraphs, the judged-events rule"},
        {"path": "docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md", "substantive": "a 'The Detector' section with pointers"}
      ],
      "key_links": [
        "the prompt's seventh command is the verb det-1 built",
        "the concept's owns.code names verbs/detector.rs"
      ],
      "prohibitions": [
        "No change to the record command line beyond what det-1 wrote",
        "No new signal name outside the four",
        "No hand edit of the five generated copies (regen only)"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  }
]
```

## Test matrix

Triad per cell, smallest demonstrating size; writers judge existing coverage first.

| Cell | Case | Pass when |
|---|---|---|
| det-1 | happy: record `--kind silence --signal budget-80 --events <id>` with the id present | the observation row carries `events: ["<id>"]` and `detector events --unjudged` no longer lists it |
| det-1 | edge: `--events` on an intervention row | the mailbox row carries the ids; the event leaves `--unjudged` |
| det-1 | error: `--events` naming an unknown id | refused, nothing written |
| det-1 | edge: empty or missing store | `bee detector events` prints `No detector events.`, exit 0 |
| det-1 | existing: the prompt-pin test with ten signals | `cargo test control_loop` green |
| det-2 | happy: job at 80% of `ceiling_secs` | one `budget-80` row with `point_key` `budget-80-ceiling-<job>`; a second tick writes none |
| det-2 | edge: continued job with `continued_at` recent and `created_at` old | no `budget-80` ceiling row |
| det-2 | happy: cell cites an Unsettled `contract:x` decision and an untracked `tests/foo_test.rs` exists | one `test-on-unstable-contract` row naming `x` |
| det-2 | happy: diff touches `src/other.rs` while files is `["src/a.rs", "src/b/*"]` | one `boundary-approach` row for `src/other.rs`; `src/b/c.rs` fires none; `src/b/d/e.rs` fires one (declared_covers semantics) |
| det-2 | edge: diff touches `.bee/decisions.jsonl` | no row |
| det-2 | happy: `src/a.rs` in the changed set at tick 1, restored to base at tick 2, job live | one `self-correction` row |
| det-2 | edge: `src/a.rs` changed at tick 1, committed before tick 2, job live | no `self-correction` row (diff against base_sha) |
| det-2 | edge: job with `result-<round>.json` present | zero rows, `scanned` excludes it |
| det-2 | edge: old job.json without `base_sha` | first tick stores the cwd HEAD in the state file and fires only on untracked paths |
| det-2 | error: unreadable job.json or a cwd that is not a checkout | tick prints one warning line, exit 0, other jobs still scanned |
| det-3 | happy: RealArgvProvider::spec_for for the supervisor | the detector pre-step ran before argv resolution; resolve_iteration alone writes nothing |
| det-3 | happy: heartbeat handler | node test sees `detector tick --json` run from the finally block after `broker tick --json`, also when the broker tick failed; `quietHeartbeatTurn` logic unchanged |
| det-3 | happy: status | a job with two rows shows `events: {"budget-80": 2}`, `unjudged: 2` and text ` events=budget-80:2 unjudged=2`; a job with none shows `events: {}` and no suffix |
| det-3 | same scenario on main and head | status JSON on head carries every field main carried |
| det-4 | parity | the five copies equal the source after `bee dev regen`; `bee knowledge index --check` green |

## Open Questions

(none)

## Out of scope

D7: model-based transcript classification (backlog proposal, 2026-10-15),
mid-turn delivery of interventions, any new Paseo heartbeat or schedule, the
waggledance far-eye supervisor, writing `estimate_minutes`.
