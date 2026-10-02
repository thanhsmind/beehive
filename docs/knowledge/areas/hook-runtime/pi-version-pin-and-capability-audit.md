---
type: bee.area
title: Hook Runtime — the Pi version the belt runs on, and the 0.84.3 to 1.0.0 capability audits
description: "Which Pi version bee's extension belt was enumerated against, why the belt's true floor is 0.84.4 rather than the 0.84.3 it claims, which Pi surfaces an upgrade can break, and the keep/adapt/delete disposition for every extension-API change between 0.84.3 and 1.0.0."
timestamp: 2026-10-02
bee:
  id: hook-runtime-pi-version-pin-and-capability-audit
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md, areas/hook-runtime/catalog-projections-and-activation.md, areas/hook-runtime/codex-capability-probe-version-pin-and-re-probe-evidence.md]
  decisions: [pi-stage-dispatch D9 (version labels read 0.84-0.85), pi-beehive D5 / store 5d87f14e (pi means the pi binary only), pi-1-0-upgrade D1 (ceiling moves on live evidence), D2 (codemode and tool_search pass by name), D3 (systemPrompt carrier kept), D4 and D10 (backlog rows)]
  sources: [.pi/extensions/bee-guard.ts, packages/bee-rs/crates/bee/src/doctor.rs, .bee/verify/verify-app/features/pi-hat-wave.md, pi 0.84.3, 0.85.1 and 1.0.0 shipped docs and the 1.0.0 CHANGELOG under the mise install root]
  authoritative_for: "hook-runtime: the Pi version floor, the Pi upgrade re-probe list, and the per-version Pi capability audit"
---

# Hook Runtime — The Pi Version the Belt Runs On, and the 0.84.3 to 1.0.0 Capability Audits

The codex belt records the version it was probed against and re-checks it on
every `bee doctor` run. The Pi belt records nothing. This concept holds what
the Pi belt was enumerated against, which Pi surfaces an upgrade can break, and
what the 0.84.3 to 0.85.1 and 0.85.1 to 1.0.0 ranges actually changed.

The rule that a version pin moves only on live evidence is not restated here.
It lives in `codex-capability-probe-version-pin-and-re-probe-evidence.md` and
governs this audit too.

## Data Dictionary

| Element | Meaning |
|---|---|
| Claimed version | The version the belt's own comments name: `0.84.3` (`.pi/extensions/bee-guard.ts:19,317`). |
| Actual floor | `0.84.4`. The belt registers `ui_prompt_start` and `ui_prompt_end` (`.pi/extensions/bee-guard.ts:2179,2201`), and Pi added both events in 0.84.4. The belt cannot be the belt for 0.84.3. |
| Last live-evidence version | `1.0.0` — run `20261002-135348-2965648`, 2026-10-02 (`.bee/verify/verify-app/features/pi-hat-wave.md:136`). This is the proven ceiling the belt header names. The five-seat hat wave with seat injection was not re-run on 1.0.0; its newest evidence is still the 0.85.1 run `20260915-180023-56873`. |
| `PI_BUILTIN_TOOLS` | The named list of eight Pi built-in tool names the write guard routes on (`.pi/extensions/bee-guard.ts:354-363`). Its comment keeps it a named list, not a switch default, "because the fail-safe below depends on knowing exactly which names are enumerated". |
| Pi attestation | None. `bee doctor attest --runtime pi` refuses by design (`packages/bee-rs/crates/bee/src/doctor.rs:899`): "Pi has no trust-unknown rows, so mechanical green already reaches ready there — there is nothing to attest." |

## Behaviors & Operations

**Pi carries no version pin, and the recorded refusal reason does not cover API
drift.** The codex path stores `codex_version` in the attestation record and
compares it against the live CLI on every check, yielding `unprobed_version` or
`version_changed` (`doctor.rs:772-784`). Pi has no such record. The reason at
`doctor.rs:899` is that Pi has no trust-unknown rows, which is a statement
about capability trust. The risk this page covers is separate: the belt
hard-codes twelve Pi event names and eight built-in tool names, and a Pi
release that renamed any of them would not be caught by any mechanical check.

**The claimed version is below the belt's real floor.** Both `0.84.3` labels
were checked against the shipped 0.85.1 docs for this audit, and the facts they
label still hold. The label itself does not: `ui_prompt_start` and
`ui_prompt_end` arrived in 0.84.4, so a belt registering them needs at least
0.84.4. A third label in the same file already reads as a range —
`.pi/extensions/bee-guard.ts:73`, "Pi 0.84–0.85 has no interactive permission
prompt event".

**Pi surfaces an upgrade can break.** These are the rows a future audit
re-checks. Each is a name the belt hard-codes:

- Thirteen registered events: `tool_call`, `session_start`,
  `before_agent_start`, `tool_execution_start`, `ui_prompt_start`,
  `ui_prompt_end`, `tool_result`, `agent_settled`, `turn_start`, `turn_end`,
  `session_tree`, `session_before_compact`, `session_shutdown`.
- The eight `PI_BUILTIN_TOOLS` names.
- The blocking return shape `{ block: true, reason }` on `tool_call`.
- `SessionManager.forkFrom` and `ctx.switchSession`, used by the five
  `bee-worktree-*` commands.
- `pi.sendUserMessage`, including the `{ deliverAs: "steer" }` form.
- `ctx.ui.notify`, `ctx.ui.setStatus`, `ctx.isIdle`, `ctx.cwd`,
  `ctx.sessionManager`.
- `pi.registerTool` and `ui.setWidget` with `{ placement: "belowEditor" }`,
  added by pi-worker-surface for the `verdict` tool and the in-flight worker
  widget.

**How much of Pi the belt actually drives, measured.** A distill of
pi-dynamic-workflows v3.12.0 (SHA `e29dbcae`) against the installed 0.85.1 docs
found bee's belt driving a LARGER host surface than that project does: 13 events
and 19 API members here against its 5 and 10. Its bulk — 172 files — is a
workflow ENGINE, which bee declined (`pi-native-stage-driver` D6), not host
integration. The brief is
`docs/history/research/pi-harness-session-surface-xia.md`. Three deltas it
found were real and two have since shipped: `ui.setWidget` belowEditor and the
`terminate: true` terminating tool (both pi-worker-surface); the third, a
sha256 baseline over bee's model-facing prose, is a filed backlog row under
`prose-guidance-baseline`. Two of its findings are anti-lessons worth keeping:
`.pi/agents/*.md` is NOT a Pi convention (0.85.1 documents only
`.agents/skills/`, which is where bee already installs), and that project
reaches its delivery path by stealing `sendCustomMessage` off
`AgentSession.prototype` — bee's documented `pi.sendUserMessage` with the
`steer` form is the cleaner path and stays.

## The 0.84.3 to 0.85.1 audit

Dispositions are `keep` (no local change needed), `adapt` (bee changed, or must
change), and `delete` (bee carries something the upstream no longer has). The
range covers three releases: 0.84.4 (2026-08-28), 0.85.0 (2026-09-04) and
0.85.1 (2026-09-05). No entry in any of the three carries a BREAKING marker.

| Upstream capability or change | Disposition | Evidence |
|---|---|---|
| Built-in tool registry: `read`, `bash`, `powershell`, `edit`, `write`, `grep`, `find`, `ls` | **keep** | `docs/settings.md:223` at 0.84.3 and `docs/settings.md:228` at 0.85.1 carry the identical sentence and the identical eight names, and no changelog entry in the range adds or removes a built-in. `PI_BUILTIN_TOOLS` is complete at 0.85.1. |
| New events `ui_prompt_start` / `ui_prompt_end` | **adapt — already done, and it raises the floor** | Added in 0.84.4 (issue 8355); present at 0.85.1 `docs/extensions.md:583-600`, absent at 0.84.3. The belt registers both (`.pi/extensions/bee-guard.ts:2179,2201`) and maps them onto the activity surface. Because the belt depends on them, its floor is 0.84.4. |
| Built-in tools `bash`, `edit`, `find`, `grep`, `ls`, `read`, `write` now honour `ctx.cwd` | **keep — behavior change, no code change** | 0.85.0 fix, issue 8627. The belt reads `ctx.cwd` itself (`.pi/extensions/bee-guard.ts:554,1238,1432,1547`) and routes on tool names, so the fix aligns the built-ins with what the belt already assumed. |
| Session fork fixes: forks losing their compaction boundary; in-memory forks before an active turn settled | **keep — no code change, relevant surface** | 0.85.0, issues 8990 and 8937. The belt forks sessions through `SessionManager.forkFrom` for the `bee-worktree-*` commands, so these fixes land under it. |
| Resumed sessions corrupting the next appended entry when the session JSONL lacks a trailing newline | **keep — no code change, relevant surface** | 0.84.4, issue 8345. |
| `pi.setModel` and `pi.setThinkingLevel` become session-scoped and are recorded in session history | **keep — no local impact** | 0.85.1 `docs/extensions.md:1706,1720-1722`. `rg 'setModel\|setThinkingLevel' .pi/extensions/bee-guard.ts` returns no call site. |
| `SessionManager.inMemory()` for externally managed session entries | **keep — no local impact** | 0.85.0, issue 8980. An SDK surface; the belt does not use it. |
| Extension messages sent with `triggerTurn: false` no longer land between a tool call and its result | **keep — no local impact** | 0.84.4, issue 8537. The belt sends through `pi.sendUserMessage`, not this path. |
| `CustomEditor` gains `{ embedWorkingStatus: true }` | **keep — no local impact** | 0.85.1 `docs/extensions.md:2832`. The belt registers no custom editor. |
| Interactive permission-prompt event | **keep — still absent** | No such event in either version's `docs/extensions.md`. The named exclusion at `.pi/extensions/bee-guard.ts:73` holds, and its label already reads `0.84–0.85`. |
| Provider, model-registry, auth, TUI, and terminal-capability changes across all three releases | **keep — no local impact** | The belt registers no provider, reads no model catalog, and draws no UI beyond `ctx.ui.notify` and `ctx.ui.setStatus`. |

Nothing in the range earns `delete`.

## The 0.85.1 to 1.0.0 audit

The range covers eight releases: 0.86.0 (2026-09-19), 0.86.1, 0.87.0
(2026-09-21), 0.87.1, 0.99.0 (2026-09-29), 0.99.1, 0.99.2 and 1.0.0
(2026-10-01). Only 0.86.0 and 0.87.0 carry a "Breaking Changes" section; each
of their eight entries has a row below. `CHANGELOG.md` lines cite the 1.0.0
install tree. Live evidence is run `20261002-135348-2965648`
(`.bee/verify/verify-app/features/pi-hat-wave.md:136`).

| Upstream capability or change | Disposition | Evidence |
|---|---|---|
| 0.86.0 BREAKING: provider stream inputs become `TranscriptContext`; custom providers read the system prompt from `context.messages` | **keep — no local impact** | `CHANGELOG.md:282`. The belt registers no provider. |
| 0.86.0 BREAKING: `ToolCall.arguments` and `ToolResultMessage.details` restricted to JSON values | **keep — no local impact** | `CHANGELOG.md:283`. The belt reads `event.input` and `event.args` as plain JSON (`.pi/extensions/bee-guard.ts:2452,2561,2625`) and builds no tool result. |
| 0.86.0 BREAKING: `user_bash` fails closed | **keep — no local impact; guard is a backlog row** | `CHANGELOG.md:284`. The belt registers no `user_bash` handler, so a user `!` command is not guarded. A `user_bash` guard is out of scope (pi-1-0-upgrade D4) and filed as a backlog row. |
| 0.87.0 BREAKING: `shouldStopAfterTurn` removed for `finishTurn` | **keep — no local impact** | `CHANGELOG.md:220`. `rg shouldStopAfterTurn .pi/extensions/bee-guard.ts` returns nothing. |
| 0.87.0 BREAKING: `ContextEditEntry` joins the `SessionEntry` union | **keep — no local impact** | `CHANGELOG.md:221`. The belt has no exhaustive switch over session entries. `appendContextEdit` as a carrier is a D4 backlog row. |
| 0.87.0 BREAKING: `SessionManager` is canonical for provider context; assigning `session.agent.state.messages` no longer replaces history | **keep — no local impact** | `CHANGELOG.md:222`. `rg 'state\.messages' .pi/extensions/bee-guard.ts` returns nothing. The belt forks and switches sessions only through `SessionManager.forkFrom` and `ctx.switchSession`, and the live run relocated a session into a worktree on 1.0.0. |
| 0.87.0 BREAKING: `TurnEndEvent` gains required fields; `AgentBeforeSettleEvent` joins `ExtensionEvent`; `emit()` no longer takes `turn_end` | **keep — no code change** | `CHANGELOG.md:223`. The belt only receives `turn_end` (`.pi/extensions/bee-guard.ts:2955`) and ignores the event body. It neither constructs events nor switches over the union. `agent_before_settle` is the boundary pi-1-0-upgrade D7 builds on, in a later slice. |
| 0.87.0 BREAKING: runs requested from `agent_settled` handlers are deferred until all settled handlers finish | **keep — proven live** | `CHANGELOG.md:224`. The belt's worktree relocation rides `agent_settled` (`.pi/extensions/bee-guard.ts:2680`). On 1.0.0, `/bee-worktree-new --feature demo` relocated the session and `ctx.cwd` then named the worktree (`pi-hat-wave.md:145`). |
| `codemode` and `tool_search` built in (0.99.0) | **adapt — done in p1u-1** | `CHANGELOG.md:119-120`. Before p1u-1 both names fell to the write-capable fail-safe, and stage narrowing stripped them. They now pass the outer guard by name (`.pi/extensions/bee-guard.ts:501-503`, pi-1-0-upgrade D2) and survive narrowing. Pi runs every nested call through `tool_call` (`docs/extensions.md:148` at 1.0.0), so bee-guard judges each real read and write inside a script. Live: a codemode `read` passed, and a codemode `write` before the gate was denied with the nested `write` named (`pi-hat-wave.md:141-142`). |
| Codemode partial writes: a failed script does not undo earlier tool calls | **keep — accepted risk, recorded** | `docs/codemode.md:18` at 1.0.0: "calls made before a failure are not undone". A script whose third write is denied keeps its first two. Write-guard judged each nested write on its own; a deny stops the script, not what already passed. The live error says so: "Tool calls made before the failure (they are not undone)" (`pi-hat-wave.md:142`). Found by the hat wave (`docs/history/pi-1-0-upgrade/reports/hat-wave.md`). |
| Transcript-backed mid-conversation system prompt and tool changes (0.86.0) | **keep — measured (D3)** | `CHANGELOG.md:288`. On 1.0.0 the per-turn `systemPrompt` from `before_agent_start` added 0 system messages over six turns. The 3 system messages were the session's first and one per tool-set change (`pi-hat-wave.md:143`). The carrier stays; the pi-1-0-upgrade D3 guess is answered. |
| `/reload` enables tools newly added to `defaultTools`; tools turned off stay off unless newly added (0.99.2) | **keep — measured** | `CHANGELOG.md:64`. In the TUI, `/reload` restored all nine tools, and the next turn narrowed them again to `["read","bash"]`. Write-guard blocked writes in every state (`pi-hat-wave.md:144`). In RPC mode `/reload` is not a command and reaches the model as text. |
| Experimental virtual models: `pi.registerVirtualModel()` (0.99.0) | **keep — no local impact; router is a backlog row** | `CHANGELOG.md:114,122`. Out of scope (pi-1-0-upgrade D10); filed as a backlog row. |
| `context_with_system` event, `appendContextEdit`, exported hook types (0.86.0, 0.87.0) | **keep — no local impact; backlog row** | `CHANGELOG.md:215,228,231,296`. Out of scope (pi-1-0-upgrade D4), with MCP server export; filed as a backlog row. |
| Built-in tool registry | **keep** | `docs/settings.md:44` at 1.0.0 names the same eight built-ins and adds that `defaultTools` can also name `codemode` and `tool_search`. `PI_BUILTIN_TOOLS` is complete at 1.0.0. |
| The thirteen registered events and the API members under "Pi surfaces an upgrade can break" | **keep — checked in the binary and live** | The 1.0.0 docs were rewritten and no longer list every event: `ui_prompt_start`, `ui_prompt_end` and `session_tree` appear in no 1.0.0 doc. All three names are in the 1.0.0 `pi` binary (`rg -a -c`), and the live run drove `turn_start`, `tool_call`, `agent_settled` and the worktree commands. |
| Interactive permission-prompt event | **keep — still absent** | 1.0.0 adds tool `annotations` that a permission extension can read (`docs/extensions.md:166`), but no prompt event. The named exclusion at `.pi/extensions/bee-guard.ts:73` holds. |
| Fullscreen TUI, codemode token cuts, image generation, MCP OAuth, `/login` and provider changes | **keep — no local impact** | `CHANGELOG.md:5-45`. The belt registers no provider and draws no UI beyond notify, status and the widget. |

Nothing in the range earns `delete`.

**Harness-native dispatch on Pi (pi-1-0-upgrade slice 2).** On lanes `small`, `standard` and `high-risk` in an approved swarming phase, write-guard refuses a Pi leader's source write (`check_pi_leader_write_lock`, contract `eba50fb9`), stage-tools hands the leader `bee_dispatch` and `bee_advisor` instead of `edit`/`write` (contract `034373cc`), and the belt's `bee_dispatch` runs `bee dispatch prepare` plus the returned `bee herding run` detached (contract `690c84f5`). Live proof: `.bee/verify/verify-app/features/pi-harness-dispatch.md`, run 20261002-153242-3268921.

**Settle obligations on Pi (pi-1-0-upgrade slice 3).** Pi 1.0's actionable `agent_before_settle` boundary carries one forced turn when bee work is owed. `bee hook session-close` answers an `obligations_only` payload with each owed item once (kinds `cap` and `advisor`, contract `5f6f7020` with amendments `90077144`, `295f1277`, `562588f5`), the belt shows the notice and returns `continue: true` (contract `5495627d`), and the agent_settled nudge does not repeat it (`210c86e4`). The planning tool set carries `bee_advisor` (amendment `82179c5d`). Live proof: `.bee/verify/verify-app/features/pi-harness-dispatch.md`, runs 20261002-181622-3758076 and 20261002-184508-3853826.

**Mid-run input on Pi (pi-1-0-upgrade slice 4).** `bee herding steer` (contract `ccdae0ed`, amendment `01fd5793`) writes a steer file into a running job's mailbox; the Pi worker belt delivers it once as a context-only steer and the verdict tool resolves the main-checkout mailbox without a newest-dir guess when `BEE_HERDING_JOB_ID` is set (contract `8972e270`, amendment `8b0387f5`). The belt's `input` handler records steer and follow-up text through session-close without changing it, and a scope-phrase match is owed as one log-or-ask turn (contract `47677410`, amendment `bf3483c6`). Live proof: `.bee/verify/verify-app/features/pi-harness-dispatch.md`, run 20261002-201522-52484.

## Business Rules

- A row above is true for the versions named in its evidence and for no others.
  An upgrade past 1.0.0 re-checks each row against the new shipped docs before
  the row is carried forward. Never carry a row forward on a version label
  alone.
- The claimed version, the actual floor, and the last live-evidence version are
  three different facts. A live run proves the belt worked on that version. It
  does not prove the belt's enumerated lists were re-read against it.
- Pi's lack of an attestation pin is a recorded design position scoped to
  capability trust (`doctor.rs:899`). It is not a finding that Pi API drift is
  harmless.

## Known gaps

- ~~The belt's floor is undocumented.~~ **Settled** by pi-version-floor, cell
  `pvf-1`. The belt header now states the range directly — floor 0.84.4,
  ceiling unproven — and says that a version named anywhere else in the file
  records which docs were READ, so the two are not confused again. The same
  note rides both contract tests. The `0.84.3` provenance citations at
  `.pi/extensions/bee-guard.ts:19,317` were deliberately left as written, per
  `docs/history/pi-stage-dispatch/plan.md:207` ("leave comments that name which
  Pi docs or binary were read unchanged"): they are true statements about what
  was read. The range is a decision, tagged `contract:pi-version-range`.
- **A delivery record claims a label change that did not reach every copy.**
  `docs/knowledge/work/pi-stage-dispatch/delivery.md:43` cites "the absence of
  the old Pi 0.84.x label". The label remains at `.pi/extensions/bee-guard.ts:19,317`,
  `docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md:336`,
  `.bee/verify/verify-app/features/semantic-role-routing.md:56`, and the two
  test files above.
- **One verify feature names a version below the API it drives.**
  `.bee/verify/verify-app/features/semantic-role-routing.md:56` says "installed
  Pi 0.84.x runtime", while its acceptance drives `ctx.switchSession`, which
  `docs/knowledge/areas/worktree-parallelism/entering-creating-and-registering.md:146`
  attributes to 0.85.1.
- **The codex sibling page has two dead pointers.** It names
  `PROBED_CODEX_VERSION` as a constant "in the bee binary" and
  `scripts/canary_codex.mjs` as the canary. Neither exists at this commit:
  `rg 'PROBED' packages/bee-rs/crates/bee/src` and `fd -t f 'canary_codex*'`
  both return nothing. The mechanism that survived the Rust port is the
  attestation record read at `doctor.rs:772-784`. Recorded here rather than
  edited, because that page is another concept's home.
- **No mechanical check covers Pi API drift.** The event names and the built-in
  tool list are prose-verified, on the cadence of this page being re-run.
- **The 1.0.0 docs no longer enumerate every event.** The 0.85.1 to 1.0.0
  audit read three event names out of the `pi` binary. The docs diff in the
  Pointers below is no longer enough on its own.

## Pointers (implementation)

- Belt: `.pi/extensions/bee-guard.ts` — built-in registry at `:354-363`, event
  registrations from `:2446`, command registrations from `:3035`.
- Attestation path and the Pi refusal:
  `packages/bee-rs/crates/bee/src/doctor.rs:772-784,899`.
- Last live evidence: `.bee/verify/verify-app/features/pi-hat-wave.md:136` (Pi 1.0.0).
- Upstream compared: `docs/settings.md`, `docs/extensions.md` and
  `CHANGELOG.md` in the Pi 0.84.3, 0.85.1 and 1.0.0 install trees.
- Regenerate the built-in-tool row:
  `rg -n 'Available built-ins are' <pi-install>/pi/docs/settings.md`
- Regenerate the extension-API delta:
  `diff <(sed 's/[0-9]\+\.[0-9]\+\.[0-9]\+//g' <old>/pi/docs/extensions.md) <(sed 's/[0-9]\+\.[0-9]\+\.[0-9]\+//g' <new>/pi/docs/extensions.md)`
