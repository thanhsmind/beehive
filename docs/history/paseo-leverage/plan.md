---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: Paseo leverage

## Summary

bee gets more out of Paseo for the people who run herded workers. A worker
can now tell its leader something while it still works. Read-only pi workers
are made read-only for real, not only by prompt, and can still hand in their
result. `bee herding status` shows what a worker costs, which mode it runs
in, and when it is parked on a permission question. A dead Paseo lead can be
forked into a new agent that keeps its history. One opt-in test checks that
the installed Paseo still speaks the shape bee reads.

Mode: `high-risk` — 5 risk flags: external systems (Paseo CLI, daemon files,
pi session files), audit/security (a write guard for read-only workers),
public contracts (two new verbs, a new `herding run` flag, new `job.json` and
status fields), cross-platform (file paths under the Paseo home), multi-domain
(Rust verbs, the Pi extension, the supervisor prompt).
Why this is the least workflow that protects the work: the guard change and
the fork write files other tools own, so each needs its own proof and review.

Playbook: `skills/bee-planning/playbooks/feature.md` — cited, not copied.

## Requirements (from CONTEXT.md)

- D1 (+ refinements `6fc537f6`): `bee herding notify` appends a typed line to the job's `messages.jsonl`; unread = past `messages.cursor`; the Pi drain delivers for inbox-session jobs, fenced as data, cursor advanced only on steer/turn; status lists unread; never `paseo send`; never ends the turn; the brief allows this one bee command.
- D2 (revised, store `168b8037`): `--read-only` for gather and advisor; pi Paseo workers get `BEE_HERDING_READ_ONLY=1` and a fail-closed guard on every tool with the own mailbox as the one write zone; other Paseo providers refuse; herdr/tmux record it, shown as not enforced; no Paseo mode set.
- D3: record `paseo_mode_observed`; status marks `parked` when `Blocked` past 120 s; bee never changes the mode.
- D4: verified, no code — the Unknown arm already resets the died debounce.
- D5: `usage` in `job.json` from `LastUsage` or the pi session file; `null` plus `usage_unavailable` otherwise; status shows tokens and cost.
- D6 (+ refinements): `bee herding fork` — refuse a Working source without `--force`, refuse a session outside the default pi sessions root, copy with a new header, `paseo import`, print id, source, undo and model note; never stop or archive the source; the dead-lead resume line names it.
- D7 (+ refinements): on-disk agent state for status, the untracked report, usage and fork, with CLI fallback; home from `--home` in the configured command, else `PASEO_HOME`, else `~/.paseo`; the run loop keeps the CLI.
- D8: an `#[ignore]` live contract test for `inspect`, `ls -a` and the on-disk agent file including `persistence.nativeHandle`.

## Load-bearing claims

Labels: `read` (file opened at that line), `ran` (command run, output held). Evidence is a verbatim substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | An Unknown liveness resets the died debounce (D4 needs no code) | read | packages/bee-rs/crates/bee/src/herding/run.rs:1279-1280 | Liveness::Unknown => { / absent_reads = 0; |
| 2 | Pi offers no Paseo mode, so D2 needs the guard path | ran | paseo provider ls --json \| jq -c '.[] \| select(.provider=="pi")' | "defaultMode":"default","modes":""} |
| 3 | `LastUsage` is null for a pi agent, so D5 needs the session file | ran | paseo inspect 156c9fdb-1b72-4688-bd5c-88c2680cfc22 --json \| jq .LastUsage | null |
| 4 | The on-disk agent file names the pi session file | ran | jq .persistence.nativeHandle ~/.paseo/agents/home-thanhsmind-Projects-goglbe-beehive/156c9fdb-1b72-4688-bd5c-88c2680cfc22.json | /home/thanhsmind/.pi/agent/sessions/ |
| 5 | `paseo import` exists in the installed CLI | ran | paseo import --help | Import an existing provider session as a Paseo agent |
| 6 | worker-guard judges Bash only today | read | packages/bee-rs/crates/bee/src/hooks/worker_guard.rs:31 | if tool_name != "Bash" { |
| 7 | The Pi belt routes only Bash calls to worker-guard | read | .pi/extensions/bee-guard/events.ts:94 | mapped.tool_name === "Bash" |
| 8 | Herding verbs register in one match table | read | packages/bee-rs/crates/bee/src/herding.rs:152 | "steer" => Some(job_verbs::steer(rest)), |
| 9 | dispatch prepare builds the herding run command in one place | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2585 | let mut command = ".bee/bin/bee herding run --task-file - --json".to_string(); |
| 10 | `paseo send` interrupts a running turn, so D1 must not use it | read | docs/knowledge/areas/bee-herding/the-paseo-channel.md:271 | an ordinary send interrupts and replaces a running turn |
| 11 | The parsed inspect struct has no mode or usage field yet | read | packages/bee-rs/crates/bee/src/herding/paseo.rs:184-188 | pub struct PaseoInspect { / pub state: Option<PaseoState>, / pub updated_at: Option<String>, / pub model: Option<String>, / pub thinking: Option<String>, |
| 12 | The supervisor never starts a successor itself | read | skills/bee-herding/references/supervisor-prompt.md:96 | You never start that session yourself. |
| 13 | A pi session file starts with a session header line that fork rewrites | ran | head -c 120 ~/.pi/agent/sessions/--mnt-data-Projects-goglbe-beehive--/2026-10-12T07-06-31-387Z_01a12eef-365a-76e0-a8e5-ee5a1a63101e.jsonl | {"type":"session","version":3,"id":"01a12eef-365a-76e0-a8e5-ee5a1a63101e","timestamp": |
| 14 | `paseo ls` lists archived agents with `-a` | ran | paseo ls --help | -a, --all              Include archived agents |
| 15 | The brief forbids every bee command today, so D1 must carve notify out | read | packages/bee-rs/crates/bee/src/herding/mailbox.rs:387 | Never run any `bee` command. |
| 16 | `sendLeaderMessage` returns false both for a skipped send and an idle turn, so D1 needs a distinct outcome | read | .pi/extensions/bee-guard/result-inbox.ts:304-305 | await pi.sendUserMessage(message, steer ? { deliverAs: "steer" } : undefined) / return steer |
| 17 | Mode B already has a fail-closed `Bash(...)` prefix allowlist to reuse | read | packages/bee-rs/crates/bee/src/hooks/worker_guard.rs:173 | if let Some(rest) = entry.strip_prefix("Bash(") { |
| 18 | Verb metadata lives in the generated registry payload | read | packages/bee-rs/crates/bee/src/registry.rs:13 | pub const REGISTRY_PAYLOAD: &str = include_str!("generated/registry_payload.json"); |

## Discovery

Three gathers read the slp repos and bee's herding code; five hat seats
checked the first draft (findings folded in below). CLI cost was measured:
`paseo --version` 349 ms, `inspect` 493 ms, `ls` 528 ms; a run makes three
CLI calls at startup, so bee is not the 7–14 s startup gap. A past verb
addition (`5d86b444e`) touched `catalog.rs`, `generated/registry_payload.json`
and the `herding.rs` match table.

## Approach

Build the on-disk reader first, since usage and fork read through it. The
observe fields (usage, mode, parked) share files, so they ride one cell.
Read-only, fork and notify are separate capabilities with separate guards.
The live contract test needs nothing else. Docs close the feature.

Reuse, from the alternatives seat: the read-only allowlist reuses mode B's
`Bash(...)` checker with its own constant and denial text; pi usage reuses the
field mapping in `run.rs` `parse_child_jsonl` through one shared function;
status usage extends `insert_usage` instead of a second emitter; parked reuses
`mailbox::ACTIVITY_FRESHNESS_SECS`.

Rejected: `paseo send` for D1 (interrupts a busy leader); Claude plan mode
for D2 (blocks the result file); auto mode change for D3 (the observer
decides nothing); replacing run-loop CLI calls with file reads for D7 (saves
under 1.5 s; the daemon stays the liveness truth); changing
`sendLeaderMessage`'s boolean contract (a sibling returns the three-way
outcome and the old function wraps it).

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Read-only guard bypass or self-block | HIGH | pl-3 | guard table with every bypass row denied and the mailbox write allowed; pi contract test |
| Session file copy and import | HIGH | pl-4 | header rewrite + byte-identical tail; refusal rows; cleanup on import failure |
| Message delivery to the leader | MEDIUM | pl-5 | node-run drain test: delivered once, skipped send retried, text fenced |
| Usage source fallback | MEDIUM | pl-2 | unit tests for each source and the null case |
| Paseo schema drift | LOW | pl-6 | ignored live test passes on 0.11.2 |

Waves (deps carry every file overlap): pl-1, pl-3, pl-6 run first; pl-2
after pl-1 and pl-3 (`run.rs`, `paseo.rs`, `herding.rs`); pl-4 after pl-1
(`herding.rs`); pl-5 after pl-2 and pl-4 (`herding.rs`, registry files,
`mailbox.rs` with pl-3 by way of pl-2); pl-7 after all.

Leader task after merge (D7): measure spawn-to-ack for `paseo-pi-agy-flash`
in main's `.bee/config.json` with and without `isolated_config: true` (four
runs each, same task). Record the numbers in a `bee decisions log` entry; set
`isolated_config` only if spawn-to-ack drops.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"pl-1 to pl-5 change Rust verbs, the run loop, the guard and the Pi extension."},
    {"stage":"test-and-live-proof","classification":"required","role":"test","reason":"pl-6 writes the live contract test."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"pl-7 updates the Paseo channel concept."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"conditional","role":"read","condition":"a worker report needs a multi-file check","reason":"Discovery is done; gathers only on demand."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"required","role":"review","reason":"The slice judge for the guard and fork cells."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the plan consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"High-risk plan check."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"High-risk plan check."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"High-risk plan check."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"High-risk plan check."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"High-risk plan check."}
  ]
}
```

## Shape

Epic map. Outcome: Paseo-carried workers are cheaper to watch, safer to run
read-only, and recoverable with history.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Observe | on-disk reader, usage, mode, parked | status today hides cost and permission waits | pl-1, pl-2 | unit tests + status JSON fields |
| Guard | read-only runs | gathers run with write tools today | pl-3 | guard table + pi contract test |
| Recover | fork | a dead lead loses its history | pl-4 | header rewrite + import argv tests |
| Talk | mid-flight messages | a worker speaks only at turn end | pl-5 | verb tests + drain test |
| Drift | live contract | Paseo upgrades silently | pl-6 | ignored test green on 0.11.2 |
| Record | docs | the concept must match | pl-7 | pointer check |

One slice; every cell is current.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pl-1 | Read Paseo's on-disk agent state | herding/paseo_state.rs (new), herding.rs | — | `bee herding status` lists untracked bee agents from disk, falling back to `paseo ls` | cargo test paseo_state |
| pl-2 | Record usage and mode, show cost and parked in status | herding/paseo.rs, herding/paseo_state.rs, herding/run.rs, herding.rs | pl-1, pl-3 | status shows tokens, cost, mode and `parked` with the unblock step per Paseo job | cargo test herding:: |
| pl-3 | Make read-only herded runs read-only | verbs/drivers/prepare.rs, herding/run.rs, hooks/worker_guard.rs, .pi/extensions/bee-guard/events.ts, tests/pi_worker_guard_contracts.rs | — | a pi gather worker that tries to edit a repo file is blocked with a clear reason, and still hands in its result | cargo test worker_guard; prepare; pi_worker_guard_contracts |
| pl-4 | Add `bee herding fork` for Paseo pi agents | herding/fork.rs (new), herding.rs, catalog.rs, generated/registry_payload.json, supervisor-prompt.md and its five generated copies | pl-1 | `bee herding fork --job <id>` prints a new agent id, the source, the undo command and the model note | cargo test fork |
| pl-5 | Add `bee herding notify` mid-flight messages | herding/notify.rs (new), herding.rs, catalog.rs, generated/registry_payload.json, herding/mailbox.rs, .pi/extensions/bee-guard/result-inbox.ts, tests/pi_plugin_contracts.rs | pl-2, pl-4 | a worker's `blocked` note reaches the Pi leader, fenced as data, while the worker keeps working | cargo test notify; render_brief; pi_plugin_contracts |
| pl-6 | Add the live Paseo contract test | tests/paseo_live_contract.rs (new) | — | `cargo test -- --ignored paseo_live` passes on Paseo 0.11.2 | the ignored test run |
| pl-7 | Document the Paseo leverage features | the-paseo-channel.md | pl-1..pl-6 | the concept names notify, fork, read-only, usage, parked and D4 | pointer check |

Paths under `herding/`, `hooks/`, `verbs/`, `generated/`, `catalog.rs` and
`herding.rs` are in `packages/bee-rs/crates/bee/src/`; `tests/` is
`packages/bee-rs/crates/bee/tests/`. The cell packet:

```json
[
  {
    "id": "pl-1",
    "feature": "paseo-leverage",
    "title": "Read Paseo's on-disk agent state",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["D7", "28b0b5c2", "6fc537f6"],
    "files": ["packages/bee-rs/crates/bee/src/herding/paseo_state.rs", "packages/bee-rs/crates/bee/src/herding.rs"],
    "read_first": ["docs/history/paseo-leverage/CONTEXT.md", "packages/bee-rs/crates/bee/src/herding/paseo.rs", "docs/knowledge/areas/bee-herding/the-paseo-channel.md"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"],
    "action": "Create herding/paseo_state.rs and declare it beside the other herding modules in herding.rs, per D7 and its refinement. Paseo home resolution: a `--home <path>` token inside the configured `herding.paseo.command`, else PASEO_HOME, else <home dir>/.paseo. Agent files live at <paseo home>/agents/<project slug>/<agent id>.json. Provide: find one agent record by id (search every project dir), list agent records whose labels carry a given key (bee_job), and a parsed record with id, provider, cwd, labels, lastStatus, archivedAt, runtimeInfo.model, runtimeInfo.thinkingOptionId, runtimeInfo.sessionId and persistence.nativeHandle (all optional except id). A missing home, a missing file or bad JSON gives None or an empty list, never a panic, and the reader never writes. Wire it into the untracked-agent report in herding.rs status (search `untracked_paseo_agents`): use the on-disk list when the home reads, fall back to the existing `paseo ls` path when it does not. The run loop is not touched. Tests use a temp PASEO_HOME with fixture files shaped like the fields in CONTEXT.md Environment facts.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee paseo_state",
    "must_haves": {
      "truths": ["An agent file under a temp PASEO_HOME is found by id and its nativeHandle is returned", "Agents labeled bee_job are listed from disk", "A missing or corrupt file gives None, not a panic", "A --home token in the configured command wins over PASEO_HOME", "status falls back to paseo ls when the disk read finds no home"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/herding/paseo_state.rs", "substantive": "reader with home resolution, find-by-id, list-by-label and a parsed record, plus unit tests"}],
      "key_links": ["herding.rs declares the module and the untracked report calls it"],
      "prohibitions": ["No change to the run loop's inspect or wait calls", "No write to any file under the Paseo home"]
    }
  },
  {
    "id": "pl-2",
    "feature": "paseo-leverage",
    "title": "Record usage and mode, show cost and parked in status",
    "lane": "high-risk",
    "role": "code",
    "deps": ["pl-1", "pl-3"],
    "decisions": ["D3", "D5", "33df9d0d", "1742a721"],
    "files": ["packages/bee-rs/crates/bee/src/herding/paseo.rs", "packages/bee-rs/crates/bee/src/herding/paseo_state.rs", "packages/bee-rs/crates/bee/src/herding/run.rs", "packages/bee-rs/crates/bee/src/herding.rs"],
    "read_first": ["docs/history/paseo-leverage/CONTEXT.md", "packages/bee-rs/crates/bee/src/herding/paseo.rs", "packages/bee-rs/crates/bee/src/herding/paseo_state.rs"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"],
    "action": "Extend PaseoInspect (paseo.rs `pub struct PaseoInspect`, parsed in `parse_inspect_full`) with the inspect `Mode` string and the `LastUsage` object. D3: in run.rs `record_observed_model` (same write-once read-modify-write) also record `paseo_mode_observed`. D5: when a Paseo round ends (any outcome), write `usage` into job.json as {input, output, cache_read, cache_write, total_tokens, cost_usd, source}: source `inspect` from LastUsage when not null; else for provider pi, source `pi-session`, summed over every line's `usage` object in the session file found through paseo_state's nativeHandle; else `usage: null` plus `usage_unavailable` naming the reason; never zero for unknown. The pi usage field mapping (input, output, cacheRead, cacheWrite, totalTokens, cost.total) already exists in run.rs `parse_child_jsonl` and its proof line; pull it into one shared function and use it in both places. Status: extend `insert_usage` in herding.rs (do not add a second emitter) to carry the D5 tokens and cost_usd from job.json when present; add `paseo_mode_observed`; add `parked: true` and `parked_reason` when the local `paseo_state` string in the status loop is `blocked` and the inspect UpdatedAt is older than `mailbox::ACTIVITY_FRESHNESS_SECS`; `parked_reason` names the mode, the pending tool and the unblock step `bee herding permit --job <id> allow|deny`. bee never calls `paseo agent mode`.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee herding::",
    "must_haves": {
      "truths": ["job.json gets usage with source inspect when LastUsage is present", "job.json gets usage with source pi-session summed from a fixture session file", "job.json gets usage null plus usage_unavailable when neither exists", "status marks a blocked job with UpdatedAt older than 120 s as parked, naming mode, tool and the permit command", "a blocked job 119 s old is not parked", "paseo_mode_observed is written once after spawn"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "usage write at round end and mode record, with tests on the fake Paseo CLI"}],
      "key_links": ["status reads usage and paseo_mode_observed from job.json through insert_usage", "parse_child_jsonl and the session sum share one usage mapping"],
      "prohibitions": ["No usage value of 0 for an unknown source", "No paseo agent mode call", "No change to the died debounce"]
    }
  },
  {
    "id": "pl-3",
    "feature": "paseo-leverage",
    "title": "Make read-only herded runs read-only",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["D2", "168b8037"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "packages/bee-rs/crates/bee/src/herding/run.rs", "packages/bee-rs/crates/bee/src/hooks/worker_guard.rs", ".pi/extensions/bee-guard/events.ts", "packages/bee-rs/crates/bee/tests/pi_worker_guard_contracts.rs"],
    "read_first": ["docs/history/paseo-leverage/CONTEXT.md", "packages/bee-rs/crates/bee/src/hooks/worker_guard.rs", ".pi/extensions/bee-guard/events.ts", ".pi/extensions/bee-guard/tool-map.ts", "docs/knowledge/areas/hook-runtime/hook-runtime-the-worker-outward-guard.md"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md", "docs/knowledge/areas/hook-runtime/hook-runtime-the-worker-outward-guard.md"],
    "action": "Per D2 as revised (store 168b8037). (1) `bee herding run` accepts `--read-only`; it lands in Options and job.json as `read_only: true`. (2) prepare.rs: the herding arm that builds `.bee/bin/bee herding run --task-file - --json` appends ` --read-only` for kinds gather and advisor; never for cell or reviewer. (3) Paseo spawn (run.rs `execute_paseo`): a read-only run on provider pi adds env `BEE_HERDING_READ_ONLY=1` (beside BEE_HERDING_WORKER and BEE_HERDING_JOB_ID); any other provider refuses before spawn with a FIX line naming pi as the supported read-only Paseo provider. No `--mode` is set. Herdr and tmux runs only record read_only. (4) worker_guard.rs, when BEE_HERDING_READ_ONLY and BEE_HERDING_JOB_ID are set: file-writing tools (Edit, Write, MultiEdit, NotebookEdit, and the pi write and edit tools as named in tool-map.ts) are allowed only when the target path resolves inside `.bee/mailbox/<BEE_HERDING_JOB_ID>/`; every other target is denied. Shell commands go through mode B's `Bash(...)` checker (`evaluate_mode_b`) with a new READ_ONLY_ALLOWED constant: cat, head, tail, wc, ls, rg, fd, jq, git log, git show, git diff, git status, git blame, bee status, bee orient, bee herding status, bee herding notify, and mv only when both paths are inside the job mailbox. The read-only path also denies: a truncated parse, $( and backticks, any > or >> whose target is outside the job mailbox, a pipe into a head not on the list, and the flags rg --pre, fd -x/-X/--exec/--exec-batch, git --output, sed, find, xargs, tee, env, sh -c, eval. Its denial text says the run is read-only, that the worker may still write its ack, report and result files in its mailbox, and that a blocker belongs in the result or `bee herding notify`. (5) events.ts: when BEE_HERDING_READ_ONLY is set for a herded Paseo worker, route every mapped tool call, not only Bash, to worker-guard, keeping the existing fail-closed checks. Tests: a guard table with one allow row (rg, a Write into the job mailbox, mv inside it) and one deny row per bypass listed above plus Write outside the mailbox; mode A and mode B tables unchanged; prepare rows (gather and advisor carry --read-only, cell and reviewer do not); a pi contract test that a write tool call outside the mailbox is blocked under the env, allowed inside it, and allowed without the env.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee worker_guard && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee prepare && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_worker_guard_contracts",
    "must_haves": {
      "truths": ["A gather or advisor dispatch command carries --read-only; cell and reviewer do not", "A read-only pi Paseo spawn sets BEE_HERDING_READ_ONLY=1 and no --mode", "worker-guard allows Write and mv inside the job mailbox and denies Write elsewhere", "worker-guard denies every listed bypass and allows rg", "A read-only spawn on another Paseo provider refuses before spawn with a FIX line"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/hooks/worker_guard.rs", "substantive": "read-only path on the mode B checker with the mailbox write zone and tests"}],
      "key_links": ["events.ts routes non-Bash tools to worker-guard under the env", "prepare passes the flag that run turns into the env"],
      "prohibitions": ["No change to mode A or mode B results for non-read-only workers", "No change to cell or reviewer dispatch commands", "No Paseo --mode set by bee"]
    }
  },
  {
    "id": "pl-4",
    "feature": "paseo-leverage",
    "title": "Add bee herding fork for Paseo pi agents",
    "lane": "high-risk",
    "role": "code",
    "deps": ["pl-1"],
    "decisions": ["D6", "12211a7d", "6fc537f6"],
    "files": ["packages/bee-rs/crates/bee/src/herding/fork.rs", "packages/bee-rs/crates/bee/src/herding.rs", "packages/bee-rs/crates/bee/src/catalog.rs", "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "skills/bee-herding/references/supervisor-prompt.md", ".agents/skills/bee-herding/references/supervisor-prompt.md", ".claude/skills/bee-herding/references/supervisor-prompt.md", ".claude-plugin/skills/bee-herding/references/supervisor-prompt.md", ".codex-plugin/skills/bee-herding/references/supervisor-prompt.md", ".opencode/skills/bee-herding/references/supervisor-prompt.md"],
    "read_first": ["docs/history/paseo-leverage/CONTEXT.md", "packages/bee-rs/crates/bee/src/herding/paseo_state.rs", "packages/bee-rs/crates/bee/src/herding/job_verbs.rs", "/home/thanhsmind/Projects/refs/slp/paseo-pi-team/scripts/team-fork.mjs"],
    "affects_skills": ["skills/bee-herding/references/supervisor-prompt.md"],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"],
    "action": "Per D6 and its refinement, add verb `bee herding fork --job <id> | --agent <paseo id> [--force] [--json]` in a new herding/fork.rs. Register it in the herding.rs match table beside `\"steer\" => Some(job_verbs::steer(rest)),`, add its entry to generated/registry_payload.json in the same shape as the `bee herding steer` entry, and extend the catalog.rs test that asserts `bee herding steer` to also assert `bee herding fork`. Resolve the source (job.json paseo_agent_id, or the id) through paseo_state. Refuse with a FIX line when: the provider is not pi; nativeHandle is missing or the file does not exist; the file is not under the default pi sessions root (<home dir>/.pi/agent/sessions/); a live `paseo inspect` says the source is Working and --force is absent. Copy: the first line must be a JSON object with type session; set id to a new UUIDv7, timestamp to now, and add parentSession = source path; copy every later line byte for byte; write `<target>.tmp-<pid>` then rename; target `<ISO time with : and . as ->_<new id>.jsonl` in the same directory. Then run `paseo import <new id> --provider pi --cwd <source cwd> --label bee_fork_of=<source id> --json` through the PaseoCli seam. On import failure remove the copied file and report. On success print (and in --json carry) the new agent id, the source agent id, the new session file, `undo: paseo archive <new id>; rm <new file>`, `model: provider default (not copied)`, and the next step `paseo send <new id> --prompt <text>`. Never stop, archive or send to the source. In skills/bee-herding/references/supervisor-prompt.md's dead-lead paragraph, add that when the dead lead is a Paseo agent the resume line also names `bee herding fork --agent <id>` for a human to run, keeping `You never start that session yourself.`; then run `bee dev regen` so the five generated copies update. Tests: header rewrite and byte-identical tail on a fixture; each refusal; import argv on the fake CLI; cleanup on import failure; no stop/archive/send call recorded.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee fork && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee catalog && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts",
    "must_haves": {
      "truths": ["A fork writes a new session file whose first line has a new id and parentSession and whose other lines match the source", "A fork calls paseo import with --provider pi, --cwd and the bee_fork_of label", "A non-pi source, a missing file, a file outside the pi sessions root, and a Working source without --force each refuse with a FIX line", "A failed import leaves no copied file", "Success output names the undo command and the model note", "The source agent gets no stop, archive or send call"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/herding/fork.rs", "substantive": "fork verb with tests"}],
      "key_links": ["herding.rs routes fork", "registry_payload.json lists bee herding fork", "the supervisor prompt names the fork command for a Paseo dead lead"],
      "prohibitions": ["No automatic fork from the supervisor or any loop", "No write to the source session file"]
    }
  },
  {
    "id": "pl-5",
    "feature": "paseo-leverage",
    "title": "Add bee herding notify mid-flight messages",
    "lane": "high-risk",
    "role": "code",
    "deps": ["pl-2", "pl-4"],
    "decisions": ["D1", "a7071d2b", "6fc537f6"],
    "files": ["packages/bee-rs/crates/bee/src/herding/notify.rs", "packages/bee-rs/crates/bee/src/herding.rs", "packages/bee-rs/crates/bee/src/catalog.rs", "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "packages/bee-rs/crates/bee/src/herding/mailbox.rs", ".pi/extensions/bee-guard/result-inbox.ts", "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"],
    "read_first": ["docs/history/paseo-leverage/CONTEXT.md", ".pi/extensions/bee-guard/result-inbox.ts", "packages/bee-rs/crates/bee/src/herding/mailbox.rs"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"],
    "action": "Per D1 and its refinement. (1) New verb `bee herding notify --kind <question|blocked|dependency|progress|reopen> --text <text> [--correlation-id <id>] [--json]` in herding/notify.rs, registered in the herding.rs match table, in generated/registry_payload.json (same shape as `bee herding steer`), and asserted in the catalog.rs test. It refuses unless BEE_HERDING_JOB_ID is set, and appends one newline-terminated JSON line {job_id, cell_id (from job.json), kind, text, correlation_id, at} to `.bee/mailbox/<that job>/messages.jsonl`, resolving the main root as other herding verbs do. It never calls paseo. (2) herding.rs status: each job gains `messages` (total lines), `unread_messages` (lines past the byte offset stored in the job's `messages.cursor`, counting only newline-terminated lines; absent cursor = 0) and `last_message` (kind and text). (3) mailbox.rs `render_brief`: change `Never run any \\`bee\\` command.` to name `bee herding notify` as the one exception, and add at most three lines saying the worker may run it while it works, that it never ends the turn, and that a question it must stop for still goes in the result; update the existing test that asserts the never-run wording. (4) result-inbox.ts: add `sendLeaderMessageOutcome` returning \"steer\" | \"turn\" | \"skipped\", and make `sendLeaderMessage` wrap it with its current boolean meaning unchanged. In `drainResultInbox`, for each tracked job (inbox-session jobs only), read messages.jsonl past messages.cursor (newline-terminated lines only), render each through `headerValue` inside a fixed `bee-message` fence carrying job, cell, kind, correlation id, the same data-not-instructions note results carry, and the action line `the worker keeps working; its answer comes in its result; redirect with bee herding steer --job <id>`, send with `sendLeaderMessageOutcome`, and advance messages.cursor only on steer or turn. The message pass runs even when the job has no result yet, and does not change the result-claim rename or the one-result-per-tick return. Tests: verb refusal without the env, append shape, unknown kind refusal, status counts with and without a cursor, brief exception present; a node-run drain test in pi_plugin_contracts.rs that a new line is delivered once, a skipped send is retried on the next tick, and the injected text sits inside the fence.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee notify && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee render_brief && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_paseo_heartbeat_contracts && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts",
    "must_haves": {
      "truths": ["notify without BEE_HERDING_JOB_ID refuses", "notify appends one line with the given kind to the job's messages.jsonl", "an unknown kind refuses", "status shows messages, unread_messages and last_message", "the brief names bee herding notify as the one allowed bee command", "the Pi drain delivers a new message once, retries a skipped send, and fences the text as data", "sendLeaderMessage keeps its boolean result"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/herding/notify.rs", "substantive": "notify verb with tests"}],
      "key_links": ["the brief tells the worker about notify", "the drain reads messages.jsonl and messages.cursor", "registry_payload.json lists bee herding notify"],
      "prohibitions": ["No paseo send for messages", "No change to the question outcome or result schema", "No change to the result-claim flow"]
    }
  },
  {
    "id": "pl-6",
    "feature": "paseo-leverage",
    "title": "Add the live Paseo contract test",
    "lane": "high-risk",
    "role": "test",
    "deps": [],
    "decisions": ["D8", "89d24cf0"],
    "files": ["packages/bee-rs/crates/bee/tests/paseo_live_contract.rs"],
    "read_first": ["docs/history/paseo-leverage/CONTEXT.md", "packages/bee-rs/crates/bee/src/herding/paseo.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D8, add an integration test file with tests named paseo_live_* and marked #[ignore]. Each first checks that `paseo --version` runs and `paseo ls --json` answers; if not, it prints a skip line and returns. Then: `paseo ls -a --json` parses as a JSON array; for the first agent id, `paseo inspect <id> --json` has keys Id, Status, UpdatedAt, PendingPermissions, ParentAgentId, Model, Thinking, Mode and LastUsage; and that agent's on-disk file under PASEO_HOME or ~/.paseo/agents/*/<id>.json has id, labels, lastStatus, runtimeInfo and persistence.nativeHandle. A missing key fails with the key name and the Paseo version. No agent is created, sent to, stopped or archived.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test paseo_live_contract -- --ignored",
    "must_haves": {
      "truths": ["The ignored tests pass on the installed Paseo 0.11.2", "With no daemon the tests print a skip line and pass"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/tests/paseo_live_contract.rs", "substantive": "ignored live tests for inspect, ls and the on-disk file"}],
      "key_links": ["the key list matches what paseo.rs and paseo_state.rs read"],
      "prohibitions": ["No mutating paseo command", "Not run by the default cargo test"]
    }
  },
  {
    "id": "pl-7",
    "feature": "paseo-leverage",
    "title": "Document the Paseo leverage features",
    "lane": "high-risk",
    "role": "docs",
    "deps": ["pl-1", "pl-2", "pl-3", "pl-4", "pl-5", "pl-6"],
    "decisions": ["D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8"],
    "files": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"],
    "read_first": ["docs/history/paseo-leverage/CONTEXT.md", "docs/knowledge/areas/bee-herding/the-paseo-channel.md"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"],
    "action": "Update the Paseo channel concept for D1-D8 in its existing style (ASD-STE100, short sentences, decision citations by store id: a7071d2b, 168b8037, 33df9d0d, 89d24cf0, 1742a721, 12211a7d, 28b0b5c2, 6fc537f6): mid-flight messages and their delivery limits, read-only runs and their write zone, observed mode and parked, the Unknown-never-died rule (one line), usage in job.json and status, the fork verb and its undo, the on-disk reader and its fallback, and the live contract test command. Add the new code files to Pointers and the new decisions to the frontmatter decisions list.",
    "verify": "rg -n 'herding notify|herding fork|read-only|parked|usage_unavailable|paseo_live' docs/knowledge/areas/bee-herding/the-paseo-channel.md",
    "must_haves": {
      "truths": ["The concept names notify, fork, read-only, parked, usage and the live test", "Pointers list paseo_state.rs, fork.rs and notify.rs"],
      "artifacts": [{"path": "docs/knowledge/areas/bee-herding/the-paseo-channel.md", "substantive": "sections for each new behavior"}],
      "key_links": ["each section cites its decision"],
      "prohibitions": ["No code comments added anywhere"]
    }
  }
]
```

## Test matrix

High-risk: the applicable edge dimensions, one probe each.

| Dimension | Probe | Cell | Pass when |
|---|---|---|---|
| Empty/absent | no Paseo home, no agent file | pl-1 | find returns None; status uses `paseo ls` |
| Malformed input | agent file with bad JSON; session file whose first line is not a session header | pl-1, pl-4 | None; fork refuses naming the file |
| Unknown vs zero | pi agent with no session file | pl-2 | `usage: null` and `usage_unavailable` set |
| Boundary time | blocked with UpdatedAt 119 s vs 121 s old | pl-2 | parked false at 119 s, true at 121 s |
| Permission/security | read-only pi worker: Write outside mailbox; `echo x > f`; `rg --pre sh`; `fd -x rm`; `$(…)`; `git diff --output=f` | pl-3 | each denied with the read-only text |
| Own write zone | read-only worker writes `ack-1.json.tmp` and renames it in its mailbox | pl-3 | allowed |
| Unsupported provider | read-only run on provider claude or codex | pl-3 | refused before spawn |
| Live source | fork a Working agent without `--force` | pl-4 | refused |
| Atomicity | import fails after copy | pl-4 | copied file removed |
| Idempotence | drain runs twice over one message; first send skipped | pl-5 | delivered exactly once, on the second tick |
| Injection | message text with backticks and an instruction | pl-5 | text inside the `bee-message` fence, backticks stripped |
| Wrong context | notify outside a herded worker | pl-5 | refused |
| External drift | live contract on Paseo 0.11.2 | pl-6 | all ignored tests pass |
| Regression | existing herding, guard and drain tests | each code cell | the cell's verify commands green |

## Open Questions

(none)

## Out of scope

- Daemon push, auto-archive and coordination signal (patched Paseo only).
- Killing orphaned child processes of a killed pi worker.
- Setting a forked agent's model (the Paseo CLI cannot).
- Read-only enforcement for herdr and tmux panes (recorded and shown as not enforced).
