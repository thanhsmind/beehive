---
type: bee.area
title: Hook Runtime — the Pi version the belt runs on, and the 0.84.3 to 0.85.1 capability audit
description: "Which Pi version bee's extension belt was enumerated against, why the belt's true floor is 0.84.4 rather than the 0.84.3 it claims, which Pi surfaces an upgrade can break, and the keep/adapt/delete disposition for every extension-API change between 0.84.3 and 0.85.1."
timestamp: 2026-09-18
bee:
  id: hook-runtime-pi-version-pin-and-capability-audit
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md, areas/hook-runtime/catalog-projections-and-activation.md, areas/hook-runtime/codex-capability-probe-version-pin-and-re-probe-evidence.md]
  decisions: [pi-stage-dispatch D9 (version labels read 0.84-0.85), pi-beehive D5 / store 5d87f14e (pi means the pi binary only)]
  sources: [.pi/extensions/bee-guard.ts, packages/bee-rs/crates/bee/src/doctor.rs, .bee/verify/verify-app/features/pi-hat-wave.md, pi 0.84.3 and 0.85.1 shipped docs and the 0.85.1 CHANGELOG under the mise install root]
  authoritative_for: "hook-runtime: the Pi version floor, the Pi upgrade re-probe list, and the per-version Pi capability audit"
---

# Hook Runtime — The Pi Version the Belt Runs On, and the 0.84.3 to 0.85.1 Capability Audit

The codex belt records the version it was probed against and re-checks it on
every `bee doctor` run. The Pi belt records nothing. This concept holds what
the Pi belt was enumerated against, which Pi surfaces an upgrade can break, and
what the 0.84.3 to 0.85.1 range actually changed.

The rule that a version pin moves only on live evidence is not restated here.
It lives in `codex-capability-probe-version-pin-and-re-probe-evidence.md` and
governs this audit too.

## Data Dictionary

| Element | Meaning |
|---|---|
| Claimed version | The version the belt's own comments name: `0.84.3` (`.pi/extensions/bee-guard.ts:19,317`). |
| Actual floor | `0.84.4`. The belt registers `ui_prompt_start` and `ui_prompt_end` (`.pi/extensions/bee-guard.ts:2179,2201`), and Pi added both events in 0.84.4. The belt cannot be the belt for 0.84.3. |
| Last live-evidence version | `0.85.1` — run `20260915-180023-56873`, 2026-09-15 (`.bee/verify/verify-app/features/pi-hat-wave.md:53`). |
| `PI_BUILTIN_TOOLS` | The named list of eight Pi built-in tool names the write guard routes on (`.pi/extensions/bee-guard.ts:332-341`). Its comment keeps it a named list, not a switch default, "because the fail-safe below depends on knowing exactly which names are enumerated". |
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

- Twelve registered events: `tool_call`, `session_start`, `before_agent_start`,
  `tool_execution_start`, `ui_prompt_start`, `ui_prompt_end`, `tool_result`,
  `agent_settled`, `turn_end`, `session_tree`, `session_before_compact`,
  `session_shutdown`.
- The eight `PI_BUILTIN_TOOLS` names.
- The blocking return shape `{ block: true, reason }` on `tool_call`.
- `SessionManager.forkFrom` and `ctx.switchSession`, used by the five
  `bee-worktree-*` commands.
- `pi.sendUserMessage`, including the `{ deliverAs: "steer" }` form.
- `ctx.ui.notify`, `ctx.ui.setStatus`, `ctx.isIdle`, `ctx.cwd`,
  `ctx.sessionManager`.

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

## Business Rules

- A row above is true for the versions named in its evidence and for no others.
  An upgrade past 0.85.1 re-checks each row against the new shipped docs before
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

## Pointers (implementation)

- Belt: `.pi/extensions/bee-guard.ts` — built-in registry at `:332-341`, event
  registrations from `:2051`, command registrations from `:2460`.
- Attestation path and the Pi refusal:
  `packages/bee-rs/crates/bee/src/doctor.rs:772-784,899`.
- Last live evidence: `.bee/verify/verify-app/features/pi-hat-wave.md:53`.
- Upstream compared: `docs/settings.md`, `docs/extensions.md` and
  `CHANGELOG.md` in the Pi 0.84.3 and 0.85.1 install trees.
- Regenerate the built-in-tool row:
  `rg -n 'Available built-ins are' <pi-install>/pi/docs/settings.md`
- Regenerate the extension-API delta:
  `diff <(sed 's/[0-9]\+\.[0-9]\+\.[0-9]\+//g' <old>/pi/docs/extensions.md) <(sed 's/[0-9]\+\.[0-9]\+\.[0-9]\+//g' <new>/pi/docs/extensions.md)`
