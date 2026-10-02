---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: pi-1-0-upgrade

## Summary
Two jobs. First, make bee work on Pi 1.0: today bee blocks Pi's new
`codemode` tool on every call and then removes it from the tool list, and the
newest Pi a bee run is proven on is 0.85.1. Second, and bigger: move the
workflow steps that small models skip into Pi itself. Today a model on Pi
must CHOOSE to dispatch a worker, call the advisor, or finish its cell, and
small models often do not. After this work, the leader on Pi cannot write
code in the execute phase (bee's write guard refuses it, and the tool list
shows it), it has a `bee_dispatch` and a `bee_advisor` tool, and a session
that tries to stop with work still owed gets one forced turn naming the exact
call. bee's CLI still decides WHAT is owed; Pi only forces the next call.
Third: text you type while bee is running reaches the work. The main model
already gets it; a running worker gets it too when you aim it at that
worker, and words that change the scope are held until they are recorded.

Mode: `high-risk` — 4 risk flags: external-systems, public-contracts, covered-contract-change, multi-domain
Why this is the least workflow that protects the work: it changes what the
leader model may do on Pi in every host repo (the belt ships inside the bee
binary), and it supersedes two locked Pi decisions.

Playbook: `skills/bee-planning/playbooks/refactor.md` (route class `refactor`); slices 2-3 also read `skills/bee-planning/playbooks/feature.md`.

## Requirements (from CONTEXT.md)
- D1: the ceiling moves to 1.0.0 only after a live end-to-end run on 1.0.0; floor stays 0.84.4.
- D2 (amended): `codemode` and `tool_search` pass the outer guard by name AND survive stage narrowing; nested calls stay guarded; MCP and other unknown tools keep the fail-safe.
- D3: the per-turn feed keeps `systemPrompt` this slice; the live run measures its cost.
- D4 (amended): out of scope are MCP server export, `context_with_system`, `appendContextEdit`, exported hook types and a `user_bash` guard.
- D5: the harness forces the next call; the bee CLI decides what that call is. The belt holds no workflow state (pi-native-stage-driver D6 stands).
- D6: a `bee_dispatch` tool runs `bee dispatch prepare` and then the exact command it returns — only the bee binary, argv form, no shell — detached, returning a job handle. The spawner stays in Rust. It is the herding path on Pi; the herding cockpit itself is out of scope.
- D7: an `agent_before_settle` obligation check, built over the obligations `session-close` already finds, adds one message naming the required call and requests one continuation — at most once per obligation key (feature + cell), never in a worker session, never when bee cannot record it. The close stays warn-only.
- D8 (amended): the leader-may-not-write rule is ENFORCED in `bee hook write-guard` by session role, lane and phase, which also sees codemode's nested calls; the tool loadout from `bee hook stage-tools` is the visible signal. Worker sessions keep every tool; an unknown role keeps every tool.
- D9: `bee_advisor` is a thin second name over `bee_dispatch` with `--kind advisor`, shipped in slice 2. The obligation check fires the plan-step hat wave once per feature when the plan is gate-ready, with a cost line first and a config off-switch. Gate approval never moves into the harness.
- D10 (amended): the phase-router virtual model is a backlog row, not a slice.
- D11: one bee config switch turns slices 2-3 on or off without a release; it ships on.
- D12: mid-run text for the leader rides Pi's own steer unchanged; a new belt `input` handler only TAGS steered text that reads as a scope change, never edits or drops it.
- D13: a running worker gets mid-run text through `bee herding steer <job> "<text>"` (a `bee_steer` tool on the leader): bee writes it to that job's steer inbox and the worker's belt injects it with `deliverAs: "steer"` at its next turn boundary. The text is the user's words, verbatim; the user picks the job (or there is exactly one running worker for the current cell). `bee herding interrupt` stays the hard stop.
- D14: a tagged scope-change input with no decision record is a slice-3 obligation: one continuation asks the leader to log it or ask the user. A change to an approved plan reopens the gate; only the user answers it.

## Load-bearing claims
Labels are `read` / `ran`; evidence is a verbatim substring of the anchor; multi-line evidence joins lines with " / ".

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Pi runs codemode's nested calls through `tool_call`, so the guard still sees each real write | ran | `sed -n 148,148p ~/.local/share/mise/installs/pi/latest/pi/docs/extensions.md` | Nested calls go through argument validation and the `tool_call` and `tool_result` handlers like model-issued calls, |
| 2 | An unknown tool with no command and no path is sent to write-guard as a Write with an empty path | read | .pi/extensions/bee-guard.ts:533-534 | tool_name: "Write", /         tool_input: { ...args, file_path: "" }, |
| 3 | write-guard denies that empty-path Write, so codemode is always blocked | ran | `echo '{"hook_event_name":"PreToolUse","session_id":"probe","cwd":"'$PWD'","tool_name":"Write","tool_input":{"code":"await tools.read({path:\"x\"})","file_path":""}}' \| .bee/bin/bee hook write-guard` | bee write guard denied this target: it could not be canonically contained inside the physical worktree. |
| 4 | `codemode` and `tool_search` are new tool names a 1.0 session can enable | ran | `sed -n 44,44p ~/.local/share/mise/installs/pi/latest/pi/docs/settings.md` | `defaultTools` can also name `codemode` and `tool_search`, which built-in extensions register inactive |
| 5 | The stage-tools full set does not name `codemode` or `tool_search`, so narrowing removes them | read | packages/bee-rs/crates/bee/src/hooks/stage_tools.rs:16-17 | pub(crate) const FULL_TOOL_SET: [&str; 8] = [ /     "read", "bash", "edit", "write", "find", "grep", "ls", "powershell", |
| 6 | The belt keeps only allowed names when it narrows | read | .pi/extensions/bee-guard.ts:2913 | const targetActive = basePool.filter((t) => allowedSet.has(t)) |
| 7 | Codemode calls ignore the active tool set, so hiding `edit`/`write` alone cannot stop a write | ran | `sed -n 204,204p ~/.local/share/mise/installs/pi/latest/pi/docs/mcp.md` | Codemode calls do not depend on the active tool set, |
| 8 | The 1.0 binary still carries the event and API names the belt hard-codes | ran | `for e in ui_prompt_start ui_prompt_end turn_start session_tree session_before_compact agent_settled forkFrom switchSession setWidget setStatus isIdle sendUserMessage registerTool belowEditor deliverAs; do printf "%s: %s\n" $e "$(rg -a -c "\b$e\b" pi)"; done` (in the 1.0.0 install dir) | ui_prompt_start: 1 |
| 9 | Pi 0.86 stores mid-conversation system prompt changes in the transcript (the D3 premise) | ran | `sed -n 288,288p ~/.local/share/mise/installs/pi/latest/pi/CHANGELOG.md` | Added transcript-backed mid-conversation system prompt and tool changes |
| 10 | A forced `systemPrompt` from `before_agent_start` is now sent as the leading system prompt | ran | `sed -n 344,344p ~/.local/share/mise/installs/pi/latest/pi/CHANGELOG.md` | the forced prompt is now sent as the provider's leading system prompt instead of being appended as a section patch after the original prompt. |
| 11 | bee's per-turn feed returns a new `systemPrompt` | read | .pi/extensions/bee-guard.ts:2544 | return { systemPrompt: `${base}\n\n${parts.join("\n\n")}` } |
| 12 | `/reload` keeps tools turned off during the session off, unless newly added to `defaultTools` | ran | `sed -n 64,64p ~/.local/share/mise/installs/pi/latest/pi/CHANGELOG.md` | tools turned off during the session stay off unless newly added |
| 13 | Follow-up runs from `agent_settled` now start later | ran | `sed -n 224,224p ~/.local/share/mise/installs/pi/latest/pi/CHANGELOG.md` | Deferred runs requested from `agent_settled` handlers until all settled handlers finish. |
| 14 | The relocate flow is a follow-up sent from `agent_settled` | read | .pi/extensions/bee-guard.ts:2699 | await pi.sendUserMessage(`/bee-worktree-relocate ${token}`, { |
| 15 | The belt ships inside the bee binary, so a belt change reaches host repos only by release | read | packages/bee-rs/crates/bee/src/doctor.rs:47 | const PI_EXTENSION_SOURCE: &str = include_str!("../../../../../.pi/extensions/bee-guard.ts"); |
| 16 | The belt header still names 0.85.1 as the newest proven Pi | read | .pi/extensions/bee-guard.ts:82 | // CEILING: none proven. 0.85.1 is the newest Pi a bee workflow has been driven |
| 17 | Dispatch on Pi is prose only: model-guard cannot fire because Pi has no subagent tool | read | .pi/extensions/bee-guard.ts:52-53 | // model-guard is a NAMED EXCLUSION on this belt — n/a — Pi has NO native / // subagent surface: no Agent tool, no Task tool, no subagent_type parameter |
| 18 | Weak models invent middle options that prose did not foreclose | read | .bee/decisions.jsonl:21 | middle option |
| 19 | Pi 1.0 gives an actionable settle boundary that can add entries and request one continuation (the D7 carrier) | ran | `sed -n 66,66p ~/.local/share/mise/installs/pi/latest/pi/docs/extensions.md` | `agent_before_settle` is the final actionable boundary: it can append entries and request one continuation. |
| 20 | The old warn-only close rested on Pi having no stop event | read | docs/history/pi-native-stage-driver/CONTEXT.md:51 | pi 0.85.1 ships no `session_stop` event |
| 21 | The locked decision D6 supersedes in part says the belt gains no dispatch tool | read | docs/history/pi-native-stage-driver/CONTEXT.md:57 | `.pi/extensions/bee-guard.ts` gains NO dispatch tool and NO spawner |
| 22 | Today the stage-tools answer knows only phase and gate, not lane or role | read | packages/bee-rs/crates/bee/src/hooks/stage_tools.rs:56 | pub(crate) fn allowed_tools_for(phase: &str, gate_approved: bool) -> &'static [&'static str] { |
| 23 | omp switches phase from the harness, not from the model's choice | ran | `omp --help \| rg -i plan-yolo` | Force read-only plan mode at start, auto-approve the plan on the model's first resolve call |
| 24 | Pi delivers a steer after the running turn's tool calls, before the next model call | ran | `sed -n 26,26p ~/.local/share/mise/installs/pi/latest/pi/docs/rpc-commands.md` | It is delivered after the current assistant turn finishes executing its tool calls, before the next LLM call. |
| 25 | An extension `input` handler can see that typed text is a steer | ran | `sed -n 19,21p ~/.local/share/mise/installs/pi/latest/pi/examples/extensions/input-transform-streaming.ts` | pi.on("input", async (event) => { / 		// During steering, skip the exec call — corrections should be fast / 		if (event.streamingBehavior === "steer") { |
| 26 | The no-pane Pi worker gets no input after it starts | read | packages/bee-rs/crates/bee/src/herding/run.rs:2984 | cmd.stdin(Stdio::null()); |
| 27 | The no-pane worker is a one-shot `pi --mode json -p` run | read | packages/bee-rs/crates/bee/src/herding/run.rs:2777-2779 | "--mode".to_string(), /         "json".to_string(), /         "-p".to_string(), |
| 28 | The belt already steers into a running turn — the carrier D13 reuses in the worker | read | .pi/extensions/bee-guard.ts:935 | steer ? { deliverAs: "steer" } : undefined, |

## Discovery
Read the Pi 1.0.0 CHANGELOG from 0.85.1 up (eight releases: 0.86.0, 0.86.1,
0.87.0, 0.87.1, 0.99.0-0.99.2, 1.0.0), the 1.0.0 `extensions.md`,
`settings.md`, `mcp.md` and `virtual-models.md`, and the 0.85.1 audit page.
Checked each hard-coded name against the 1.0.0 binary (row 8). Probed
write-guard with a codemode-shaped call (row 3): denied. A read worker mapped
bee-on-Pi steps: dispatch, the hat wave, phase moves and cell caps are prose
only (row 17); the write guard, the high-risk advisor ref and the result drain
are mechanism. omp 18.4.10 starts phases, subagents and its advisor from the
harness (row 23). The five-seat hat wave critiqued the first redirect draft;
its synthesis is `docs/history/pi-1-0-upgrade/reports/hat-wave.md`.

## Approach
Recommended path: the rule from D5 — **the harness forces the next call; the
bee CLI decides what it is.** Every new belt behavior reads one bee verb and
enforces its answer; no phase logic is written in TypeScript. Hard rules sit
on the one blocking surface (`tool_call` → write-guard), which also sees
codemode's nested calls (rows 1, 7). The tool list is the signal, never the
wall.

1. Slice 1 — compat (D1-D3): unblock codemode in the guard and in the stage
   list, prove 1.0 live, move the ceiling.
2. Slice 2 — the only legal move is the right one (D6, D8, D9 tools, D11):
   write-guard refuses leader writes on `small`+ in execute; the loadout shows
   it; `bee_dispatch` and `bee_advisor` exist. Walking skeleton: a small model
   on a `small` cell dispatches; its worker writes and returns a `verdict`.
3. Slice 3 — nothing owed is dropped (D7, D9 auto-fire): the settle
   obligation check over `session-close`'s existing obligation finder.
4. Slice 4 — mid-run input reaches the work (D12-D14): the `input` tag, the
   worker steer inbox and `bee_steer`, and the scope-change obligation (which
   rides slice 3's check). Pattern: omp steers its subagents only on an
   explicit parent call (`steer_subagent`), never by itself — bee does the same.

Rejected alternatives:
- More or louder prompt text — what fails today (row 18).
- Hiding tools as the wall — codemode ignores the active set (row 7).
- `prepareLoadout` / `exposure: hidden` — keeps tools callable, or hides them from workers too (hat-alternatives).
- A workflow engine in the belt — rejected before (pi-native-stage-driver D6).
- A spawner in TypeScript — the Rust runner already ships job ids, seats, ceilings and the result drain.
- A new obligations engine — `session-close` already finds uncapped cells and advisor debt.
- A virtual-model router slice — does not make dispatch or the advisor fire (hat-value); backlog row.
- Forward every typed line to every running worker — wrong-job noise, and the leader would be writing in the user's name.
- omp's `interruptMode: immediate` (skip pending tool calls) — Pi has no such mode; `bee herding interrupt` already is the hard stop.
- Keep the worker's stdin open in RPC mode — a second transport beside the mailbox the drain already uses.

Risk map:
| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Codemode allowed past the outer guard | HIGH | p1u-1 | nested `write` and nested `bash` redirect without a gate both denied |
| Narrowing strips new tools | HIGH | p1u-1 (codemode, tool_search); slice 2 (bee_dispatch, bee_advisor, verdict) | contract test: every tool bee registers survives each phase it belongs to |
| Workers stuck without writes | HIGH | slice 2 | explicit worker signal; unknown role keeps all tools; herding pane run writes |
| Rollback needs a release | HIGH | slice 2 | D11 config switch off restores today's behavior |
| Continuation loop at settle | HIGH | slice 3 | once per (feature, cell) key; no record → no continuation; never in a worker |
| `bee_dispatch` running arbitrary output | MEDIUM | slice 2 | argv-only, bee binary only; detached, returns a handle |
| Partial writes when a codemode script is denied midway | MEDIUM | p1u-1 | deny text names the nested tool; documented in the audit page |
| Per-turn feed on the 1.0 transcript | MEDIUM | p1u-2 | system entries per turn, counted |
| A steer reaching a worker after its commit | MEDIUM | slice 4 | the steer inbox is refused once the cell is capped; the user is told |
| Scope tag read as approval | MEDIUM | slice 4 | a tagged change to an approved plan reopens the gate; never auto-approved |

Waves: slice 1 is serial (p1u-1 and p1u-3 share `.pi/extensions/bee-guard.ts`;
p1u-2 needs p1u-1's binary). In slice 2 the binary work (write-guard rule,
stage-tools lane input) and the belt tools can run in parallel on disjoint
files.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"p1u-1 changes the guard mapping and its contract test."},
    {"stage":"test-and-live-proof","classification":"required","role":"test","reason":"p1u-2 drives a real bee workflow on Pi 1.0.0 and records evidence."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"p1u-3 moves the ceiling and writes the 0.85.1 to 1.0.0 audit table."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"The changelog digest already ran."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"required","role":"advisor","reason":"High-risk gate: the hat-wave synthesis is recorded as the advisor ref."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape, no competing designs."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Plan-step hat wave."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"Plan-step hat wave."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"Plan-step hat wave."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Plan-step hat wave."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Plan-step hat wave."}
  ]
}
```


## Shape

Epic map. Feature outcome: bee runs on Pi 1.0, and on Pi the workflow steps
small models skip today are forced by the harness, decided by the bee CLI.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Compat | codemode unblock (guard + stage list), 1.0 live proof, ceiling | the one proven break, and D1 | 1 | contract tests; live run id on 1.0.0 |
| Only the right move | write-guard leader rule; lane-aware loadout; `bee_dispatch`, `bee_advisor`; kill switch | small models write inline instead of dispatching, and rarely call the advisor | 2 | live: a small model on a `small` cell dispatches; a leader write is refused with the hint |
| Nothing owed is dropped | settle obligation check; auto hat wave | small models stop with a cell uncapped or a plan unreviewed | 3 | live: one forced turn per obligation; the hat wave fires once |
| Mid-run input reaches the work | `input` tag; worker steer inbox + `bee_steer`; scope-change obligation | typed context today never reaches a running no-pane worker, and scope changes typed mid-run are not recorded | 4 | live: text typed during a worker run appears in that worker's next turn; a scope change gets one forced log-or-ask turn |

Slice queue: 1 → 2 → 3 → 4 (4 reuses 2's dispatch path and 3's obligation
check). Current slice to prepare: slice 1. Slices 2-4 are headlines; their
cells are drafted after p1u-2's live run.

Deferred coverage (on purpose, not a gap): D5-D9 and D11 land in slice 2-3
cells; D12-D14 in slice 4 cells; D10 lands as a backlog row in p1u-3.

What the user sees in slices 2-3 (from the hat-user-impact seat):
```
ℹ bee: edit/write are off for the leader in the execute phase (lane: small).
  Workers write. To edit by hand: Esc, then /bee-tools-reopen
▸ bee_dispatch p1u-2 → worker started (job 7f3a) — you can keep typing
```
```
⚠ bee: work is still owed — cell p1u-2 is claimed but not capped.
  One extra turn starts now to run: bee cells cap p1u-2 …
  (Esc stops it. This message will not repeat for p1u-2.)
```
Named defaults (each a one-line decision at slice 2-3 drafting): a reopened
leader can write again until the stage changes; Esc on a forced turn counts the
obligation as served; a new user prompt runs beside a running worker, with a
status line naming it; `/bee-obligation-skip` dismisses one obligation.

What the user sees in slice 4:
```
▸ bee_steer job 7f3a ← "use the v2 endpoint, not v1" — lands at its next turn
⚠ bee: that reads as a scope change to an approved plan — logging it reopens the gate.
```

## Cells — current slice (preview)

Slice 1 (p1u-1 to p1u-3) is capped and merged (main 4d16fde). Plan revision 1 prepares slice 2.
Contracts: `eba50fb9` pi-leader-write-lock, `034373cc` pi-stage-loadout, `690c84f5` pi-dispatch-tool; named defaults `c7bde4b8`.
Waves: p1u-4 and p1u-5 in parallel (disjoint files); then p1u-6; then p1u-7, run by the leader because the worker-outward guard refuses a worker that starts pi.

```json
[
  {
    "id": "p1u-4",
    "feature": "pi-1-0-upgrade",
    "lane": "high-risk",
    "role": "code",
    "change_class": "security",
    "title": "Refuse source writes from a Pi leader in the execute phase so it dispatches instead",
    "deps": [],
    "decisions": [
      "D8",
      "D11",
      "eba50fb9-8c3c-49c9-899c-9ffe202b77f0"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs"
    ],
    "read_first": [
      "docs/history/pi-1-0-upgrade/CONTEXT.md",
      "docs/history/pi-1-0-upgrade/plan.md",
      "docs/history/pi-1-0-upgrade/reports/hat-wave.md",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs",
      "packages/bee-rs/crates/bee/src/hooks/mod.rs"
    ],
    "action": "Red first, then add one check to write-guard that implements contract pi-leader-write-lock exactly: refuse a source write, and a bash call that runs `bee herding run`, from a Pi leader (payload bee_runtime == \"pi\", env BEE_HERDING_WORKER not \"1\") when phase is swarming, the execution gate is approved and the route lane is small, standard or high-risk. Allow lanes tiny, docs and spike, worker sessions, non-Pi payloads, payload tools_reopened true, and config pi_harness_workflow false. The refusal text names bee_dispatch and /bee-tools-reopen. Reuse the record and config readers check_worktree_first already uses. No code comments.",
    "must_haves": {
      "truths": [
        "a Pi leader source write on a small lane in an approved swarming phase is refused, naming bee_dispatch and /bee-tools-reopen",
        "the same leader's bash `bee herding run` is refused",
        "a worker session (BEE_HERDING_WORKER=1) on the same record may write",
        "lane tiny may write",
        "a non-Pi payload is unchanged",
        "tools_reopened true may write",
        "pi_harness_workflow false restores today's behavior"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs",
          "substantive": "the leader write-lock check"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs",
          "substantive": "one test per truth"
        }
      ],
      "key_links": [
        "write_guard/main.rs calls the new check on the PreToolUse path"
      ],
      "prohibitions": [
        "No code comments",
        "Claude and Codex payloads unchanged",
        "No change to the docs or tiny exemptions"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee write_guard",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "p1u-5",
    "feature": "pi-1-0-upgrade",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Make the Pi stage tool set lane- and worker-aware so the leader is handed bee_dispatch",
    "deps": [],
    "decisions": [
      "D8",
      "D9",
      "D11",
      "034373cc-aa8d-4df4-a569-c9a761f80710"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/stage_tools.rs"
    ],
    "read_first": [
      "docs/history/pi-1-0-upgrade/CONTEXT.md",
      "docs/history/pi-1-0-upgrade/plan.md",
      "docs/history/pi-1-0-upgrade/reports/hat-wave.md",
      "packages/bee-rs/crates/bee/src/hooks/stage_tools.rs"
    ],
    "action": "Red first, then change allowed_tools_for to take lane and worker and implement contract pi-stage-loadout exactly. Read the lane from the resolved record's route, the worker from env BEE_HERDING_WORKER, and pi_harness_workflow from config. Keep both notice sentences and add the leader case's sentence naming bee_dispatch and /bee-tools-reopen. No code comments.",
    "must_haves": {
      "truths": [
        "a leader in an approved swarming phase on lane small gets the full set minus edit and write, plus bee_dispatch and bee_advisor",
        "a worker session gets the full set plus verdict",
        "lane tiny keeps the full set",
        "the gated phases keep read and bash only",
        "the full set names bee_dispatch, bee_advisor and verdict",
        "pi_harness_workflow false restores today's answer"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/stage_tools.rs",
          "substantive": "lane- and worker-aware allowed_tools_for with its tests"
        }
      ],
      "key_links": [
        "the belt's turn_start narrowing reads allowed_tools from this hook unchanged"
      ],
      "prohibitions": [
        "No code comments",
        "READ_ONLY_TOOLS unchanged"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee stage_tools",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "p1u-6",
    "feature": "pi-1-0-upgrade",
    "lane": "high-risk",
    "role": "code",
    "change_class": "api",
    "title": "Give the Pi leader bee_dispatch and bee_advisor tools that start workers through bee",
    "deps": [
      "p1u-4",
      "p1u-5"
    ],
    "decisions": [
      "D6",
      "D9",
      "690c84f5-d884-4524-ad73-1289e805dd64",
      "eba50fb9-8c3c-49c9-899c-9ffe202b77f0",
      "c7bde4b8-00bc-4541-9486-89932d04b996"
    ],
    "files": [
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "read_first": [
      "docs/history/pi-1-0-upgrade/CONTEXT.md",
      "docs/history/pi-1-0-upgrade/plan.md",
      "docs/history/pi-1-0-upgrade/reports/hat-wave.md",
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "action": "Red first in pi_plugin_contracts.rs, then in the belt: register bee_dispatch and bee_advisor per contract pi-dispatch-tool, reusing tokenizeArgv, resolveBeeBinary and the result drain; add bee_runtime \"pi\" and tools_reopened to the write-guard payload; show each running job in the worker widget. Rebuild .bee/bin/bee, run bee dev regen, and run bee doctor --runtime pi. No code comments.",
    "must_haves": {
      "truths": [
        "bee_dispatch runs dispatch prepare and then the returned herding command as argv, detached, and returns a job id at once",
        "bee_dispatch refuses a payload whose command is not `.bee/bin/bee herding run`",
        "bee_advisor runs prepare with --kind advisor",
        "a prepare refusal comes back as the tool error verbatim",
        "the write-guard payload carries bee_runtime pi and tools_reopened",
        "bee doctor --runtime pi is READY after rebuild"
      ],
      "artifacts": [
        {
          "path": ".pi/extensions/bee-guard.ts",
          "substantive": "bee_dispatch and bee_advisor tools"
        },
        {
          "path": "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
          "substantive": "contract tests for both tools and the payload fields"
        }
      ],
      "key_links": [
        "doctor.rs:47 embeds the belt, so .bee/bin/bee is rebuilt in this cell"
      ],
      "prohibitions": [
        "No code comments",
        "No spawner in TypeScript beyond running the returned bee command",
        "No shell parsing of the returned command"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts",
    "affects_skills": [],
    "affects_specs": [],
    "regen_obligation_ack": "regen chain runs inside this cell (bee dev regen named in action)"
  },
  {
    "id": "p1u-7",
    "feature": "pi-1-0-upgrade",
    "lane": "high-risk",
    "role": "test",
    "change_class": "test",
    "title": "Prove on Pi 1.0 that a small model dispatches instead of writing inline",
    "deps": [
      "p1u-6"
    ],
    "decisions": [
      "D1",
      "eba50fb9-8c3c-49c9-899c-9ffe202b77f0",
      "034373cc-aa8d-4df4-a569-c9a761f80710",
      "690c84f5-d884-4524-ad73-1289e805dd64"
    ],
    "files": [
      ".bee/verify/verify-app/features/pi-harness-dispatch.md",
      ".bee/verify/verify-app/features/README.md",
      "docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md"
    ],
    "read_first": [
      "docs/history/pi-1-0-upgrade/CONTEXT.md",
      "docs/history/pi-1-0-upgrade/plan.md",
      "docs/history/pi-1-0-upgrade/reports/hat-wave.md",
      ".bee/verify/verify-app/features/pi-hat-wave.md"
    ],
    "action": "Leader-run (the worker-outward guard refuses a worker that starts pi). In a fresh verify-app sandbox with the candidate vendored, a small-lane feature in an approved swarming phase and one open cell, drive a Pi 1.0 leader on deepseek/deepseek-flash: record the tool list, a refused inline write with its text, a bee_dispatch call, the worker's verdict, and the cell's file written by the worker. Write the new feature file and its README index row, and add the mechanism to the audit page.",
    "must_haves": {
      "truths": [
        "the leader's tool list has bee_dispatch and no edit or write",
        "an inline leader write is refused with the bee_dispatch hint",
        "bee_dispatch starts a worker that writes the cell's file and returns a verdict",
        "the feature file names the run id and evidence path"
      ],
      "artifacts": [
        {
          "path": ".bee/verify/verify-app/features/pi-harness-dispatch.md",
          "substantive": "how to drive it, the run evidence, gotchas"
        }
      ],
      "key_links": [
        "README.md index lists pi-harness-dispatch"
      ],
      "prohibitions": [
        "No product source edits"
      ]
    },
    "verify": "bash .bee/verify/verify-app/control-bee doctor",
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md"
    ]
  }
]
```

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| p1u-4 | Refuse source writes from a Pi leader in the execute phase | write_guard `hook_local.rs`, `main.rs`, `tests.rs` | — | a Pi leader that tries to edit code is told to use `bee_dispatch` | `cargo test -p bee write_guard` |
| p1u-5 | Make the Pi stage tool set lane- and worker-aware | `stage_tools.rs` | — | the leader's tool list shows `bee_dispatch` and no `edit`/`write` | `cargo test -p bee stage_tools` |
| p1u-6 | Give the Pi leader `bee_dispatch` and `bee_advisor` | belt, `pi_plugin_contracts.rs` | p1u-4, p1u-5 | one tool call starts a worker and returns a job id | `cargo test --test pi_plugin_contracts`; doctor pi READY |
| p1u-7 | Prove on Pi 1.0 that a small model dispatches | new `pi-harness-dispatch.md`, README index, audit page | p1u-6 | the verify map shows a small model dispatching on Pi 1.0 | green:live, leader-run |

## Test matrix
Slice 1:
- Happy: outer `codemode` call → allowed. Pass when the guard returns no block.
- Happy: after gate approval, a stage turn keeps `codemode` and `tool_search` active. Pass when `allowed_tools` from `bee hook stage-tools` names both.
- Edge: nested `write` inside codemode, no approved gate → denied. Pass when the `tool_call` result is `{ block: true }` with the write-guard reason.
- Edge: nested `bash` with `> file` inside codemode, no approved gate → denied. Pass when blocked.
- Edge: `tool_search` → allowed. Pass when no block.
- Error: unknown tool with no command and no path (an MCP tool) → still denied. Pass when the deny text from claim row 3 appears.
- Same-scenario on main vs head: outer `codemode` call. Pass when main denies and head allows.

Slices 2-3 (headline rows; the 12-dimension matrix lands with their cells):
- Leader on a `small` cell in execute writes → refused by write-guard, also through codemode. Pass when the refusal names `bee_dispatch` and `/bee-tools-reopen`.
- Worker session on the same cell writes → allowed. Pass when no block.
- Kill switch off → today's behavior. Pass when the leader write is allowed after the gate.
- Settle with a claimed uncapped cell → one continuation naming the cap call. Pass when a second settle only warns, and a worker session gets none.
- Plan reaches gate-ready → the hat wave fires once, after a cost line. Pass when a second settle does not refire it.
- Text sent with `bee_steer` to a running no-pane worker → it appears in the worker's next turn. Pass when the worker's session JSONL holds the text as a user message.
- `bee_steer` to a capped cell's job → refused. Pass when the refusal names the cell as capped.
- Typed steer with no scope words → passes through untagged. Pass when no obligation is recorded.

## Open Questions
- Does each `systemPrompt` change add one transcript entry per turn, and does it break the prompt cache? (p1u-2 answers.)
- Does `/reload` keep stage narrowing in practice? Row 12 says tools turned off stay off unless newly added to `defaultTools`; p1u-2 checks the one case left — a user whose `defaultTools` adds a narrowed tool.
- Should hand-typed `bee herding run` in bash be blocked once `bee_dispatch` exists? (slice 2 drafting; default: block for the leader, so a small model has one path.)
- What marks text as a scope change for the D12 tag — a word list, or a cheap classifier call? (slice 4 drafting; default: a word list from bee config, so the rule has one home.)

## Out of scope
- MCP server export of bee, `context_with_system`, `appendContextEdit`, exported hook types, a `user_bash` guard (D4).
- A virtual-model phase router (D10, backlog row).
- The herding cockpit on Pi; `bee_dispatch` is the herding path (D6).
- Moving gate approval, phase truth, or the role→model map into the belt (D5, D9).
- Pi Durable (a separate runtime; no bee surface uses it).
- `clear_queue` before abort — an RPC-host command the belt cannot call.
- The release that ships the new belt — a separate ask through `scripts/release.sh`.
