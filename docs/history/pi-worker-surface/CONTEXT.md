# Pi Worker Surface — Context

**Feature slug:** pi-worker-surface
**Date:** 2026-09-20
**Shaping session:** complete
**Scope:** Standard
**Domain types:** SEE | RUN

## Feature Boundary

A Pi leader running native stage dispatch shows its in-flight workers in a live
widget below the input, and a Pi worker ends its run by calling one terminating
tool that hands back a schema-validated verdict instead of prose the leader must
read. It ends at the Pi session surface: bee's store, gates, cells, proof,
worktrees, and the Rust spawn path in `bee herding run` are unchanged.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

Decision log: `a4b0368c` (scope), `7dfd593d` (panel), `dae51a75` (verdict
fallback), `e29aa9cd` (model-guard, agent-sourced). All four carry
`touches:31fb9e15` or stand alone; none supersedes anything.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | A1 (live worker widget panel) and A2 (structured worker verdict tool) ship together as this one feature. A3 (sha256 baseline over bee's model-facing prose) is a SEPARATE feature and is filed as a P2 `proposal` backlog row under slug `prose-guidance-baseline`. | `a4b0368c`. The two file sets are disjoint and A3 blocks on nothing this feature produces, so bundling would gate belt changes behind release-gate changes. |
| D2 | The panel is drawn with Pi's own `ui.setWidget(<key>, factory, { placement: "belowEditor" })`. No component library, no vendored panel code from the source. | `extensions.md:2613-2619` documents the surface. The source's own panel is 1677 lines and carries state this feature does not need. |
| D3 | The panel lists in-flight workers ONLY. A worker's row is removed the moment that worker finishes, and the widget is not drawn at all when zero workers are in flight. | `7dfd593d`. A finished worker's result already reaches the user through the result-inbox drain; a surviving row would duplicate the transcript and grow without bound. |
| D4 | The panel is informational and takes no input. | Matches the belt's existing advisory posture — every surface here except `tool_call` is non-blocking. |
| D5 | Row content derives from bee's existing progress-tick vocabulary (`▸` started, `✓` green, `⚡` auto-approved, `✗` red). No new status vocabulary is invented for this surface. | AGENTS.md, "Communication". One vocabulary, one home. |
| D6 | A2 registers ONE terminating tool. The worker calls it to end its run, returning `terminate: true` with a schema-validated verdict. **The tool MIRRORS the verdict schema bee already owns — it does not define a new one.** | `extensions.md:2019` documents `terminate: true` and ships `examples/extensions/structured-output.ts`. **Amended 2026-09-20, decision `6b7e8f49`** — see the amendment note below this table. |
| D7 | A Pi worker that ends WITHOUT calling the verdict tool falls back to today's behavior: the leader reads the worker's report file. Transport outcome classification (`succeeded`, `timed_out`, `send_failed`, `resolution_failed`, `unverifiable_after_send`, `flipped_before_send`, `unsafe_at_preflight`) is unchanged. | `dae51a75`. Makes the tool a strict improvement where it fires and regresses nothing where it does not. A mandatory verdict would turn every prose-only worker, including ones running older prompts, into a blocker on day one. |
| D8 | The verdict tool does NOT touch model-guard's named exclusion on the Pi belt. It starts no worker and spawns no process, so D11's premise (`31fb9e15`) that the belt carries no dispatch tool and no spawner holds. | `e29aa9cd`, verified at `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:1878-1884`: the parity assertion walks routed `(tool, hook)` pairs and fails only when a hook is not `write-guard`. A non-spawning tool leaves that invariant intact. |
| D9 | `.pi/extensions/bee-guard.ts` stays hand-written and authoritative; the Rust binary is rebuilt so `doctor`'s byte-compare agrees. No generator is introduced. | Inherited from `pi-native-stage-driver` D7. `hook_manifests.rs:46-64` names Pi a NAMED EXCLUSION and `bee dev regen` writes nothing here. |
| D10 | Existing Claude, Codex and OpenCode behavior does not change. No shared payload, prompt or hook route changes shape for them. | Inherited from `pi-native-stage-driver` D9. Three belts share `bee hook <name>`; a regression there is a harness-wide outage. |

**Amendment, 2026-09-20 (decision `6b7e8f49`).** D6 was shaped on a claim from
`pi-harness-session-surface-xia.md` that bee never machine-parses a worker
verdict. Planning's reality touch **disproved it**: `herding/mailbox.rs:490`
already defines `MailboxResult` with a required `status` (`"done"` |
`"blocked"`), `summary`, `files_changed` and `proof`, validated at `:655-690`,
and every real job under `.bee/mailbox/` carries a `result-N.json` in that
shape. The feature boundary is unchanged and no decision is superseded — what
changes is D6's basis and its honest payoff:

- The tool **mirrors** `MailboxResult`; Rust stays the single source of truth
  for the schema (`bee-principle-single-source-of-truth`).
- The gains are `terminate: true` (one saved assistant turn per worker) and
  host-side field validation before `execute()` runs, so a worker cannot emit
  the `malformed_result` + exit-1 case this repo has already hit — **not**
  "the outcome becomes data", which it already is.
- D7's fallback is unaffected and still correct.

### Agent's Discretion

- The widget key, row layout and truncation rule are the agent's, within D3–D5.
- The verdict tool's name, and how it reports a write failure, are the agent's.

<!-- bee:not-a-deferral: This sentence names the CONTEXT template's own "Deferred To Planning" section and states the OPPOSITE of a deferral — both items are closed, with their evidence recorded below. It promises no future action. -->
The other two discretion items were settled by planning's reality touches and
are recorded under Deferred To Planning below — they are no longer open.
<!-- /bee:not-a-deferral -->

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Verdict | The worker's own conclusion about its cell — the structured form of today's `[DONE]` / `[BLOCKED]` / `[HANDOFF]` / `[NOOP]` prose token. Never a transport outcome. |
| Transport outcome | What `classify_outcome` already records about the *delivery* of a worker — `timed_out`, `send_failed` and siblings. Untouched by this feature (D7). |
| In-flight worker | A dispatched worker whose run has started and not yet finished. The only thing the panel draws (D3). |

## Specific Ideas And References

- `docs/history/research/pi-harness-session-surface-xia.md` — the brief this
  feature comes from. Its A1 and A2 are D2–D8 here; its A4 and A5 (hook
  `model_select`; guard `registerCommand` with `pi.getCommands()`) are polish
  and are NOT in this feature's scope.
- The brief's "Do not take" list holds: no `.pi/agents` convention (not a Pi
  convention — `extensions.md` Q8 ABSENT), no `AgentSession.prototype`
  monkey-patch (bee's `pi.sendUserMessage` path is cleaner), no workflow engine
  (declined by `pi-native-stage-driver` D6).

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `.pi/extensions/bee-guard.ts:2028-2060` — `refreshModelUsageStatus`, the
  existing `ctx.ui.setStatus` surface and its refresh points
  (`session_start`, `turn_end`, `session_tree`). The widget is a sibling of
  this, not a replacement.
- `.pi/extensions/bee-guard.ts:905-925` — the result-inbox drain's injection via
  `pi.sendUserMessage` with `deliverAs: "steer"` and the `turnStartPending`
  latch. This is what already delivers a finished worker's result, and is why
  D3 removes the row rather than keeping it.
- `.pi/extensions/bee-guard.ts:2874-2912` — `bee-tools-reopen`, the existing
  `registerCommand` shape with its `ctx.isIdle()` refusal and `ui.notify`
  pattern.
- `packages/bee-rs/crates/bee/src/herding/wave.rs:471-500` — `classify_outcome`
  and `outcome_retryable`, the transport taxonomy D7 leaves alone.

### Established Patterns

- Two failure policies, never a third: BLOCKING on `tool_call` only, ADVISORY
  everywhere else, never mixed on one call (`bee-guard.ts:14-40`). The widget
  and the verdict tool are both advisory-class.
- Per-call passivity: no `.bee` directory means the handler returns without
  running or printing anything (`beeStorePresent`). Every new handler follows it.
- Module-level state is re-derived, not carried: `fullToolSet` re-captures from
  `getAllTools()` at the next `turn_start` (`:2482-2490`), which is why the belt
  survives `/reload` without a handoff slot.

### Integration Points

- `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` — the belt contract
  suite and the parity test that D8 must keep green.
- `packages/bee-rs/crates/bee/src/doctor.rs:47`, `:310-312` — `include_str!` and
  the byte-compare that makes D9 a rebuild obligation.
- `packages/bee-rs/crates/bee/src/herding/mailbox.rs:490`, `:655-690`,
  `:20-26` — `MailboxResult`, its parser, and the mailbox file contract. The
  verdict schema's one home (D6 amendment).
- `packages/bee-rs/crates/bee/src/herding/run.rs:2269` — `write_inbox_marker`,
  the pending marker written before spawn. The panel's data source.
- `.pi/extensions/bee-guard.ts:493-528` — `mapToolCall`'s fail-safe default
  arm, which denies the verdict tool without an explicit row.
- `.bee/verify/verify-app/features/pi-runtime.md` — the user-facing map. This
  feature is a CHANGE to that mapped feature, not a new door: it adds
  sub-features, it does not create a second Pi runtime entry.

## Canonical References

- `~/.local/share/mise/installs/pi/0.85.1/pi/docs/extensions.md` — the
  version-matched host contract. `:2019` (`terminate`), `:2616` (the
  `belowEditor` widget call), `:2033` (schema validation runs before
  `execute()`), `:1677` (active-tool APIs), `:1529` (duplicate command names).
- `docs/history/pi-native-stage-driver/CONTEXT.md` — D6, D7, D9, D11 are
  inherited premises, cited above and never reinterpreted here.
- `docs/history/research/pi-harness-session-surface-xia.md` — evidence pack.

<!-- bee:not-a-deferral: These sections are CONTEXT.md's own record of what was resolved and what was consciously left out of scope. "Deferred To Planning" is a fixed template heading whose every item is answered and checked off with a command or a file:line; "Deferred Ideas" names three FILED backlog rows (A4 a P3 proposal, A5 a P3 debt row, A3 a P2 proposal under slug prose-guidance-baseline), which are their home and where they get picked up; the Handoff Note is template boilerplate describing what a planning agent reads. None is an unregistered promise to act later. -->
## Outstanding Questions

### Resolve Before Planning

None. The one question raised at intake — whether A2 breaks model-guard's named
exclusion — was resolved on the evidence and locked as D8.

### Deferred To Planning

All three were answered by reality touches on 2026-09-20, before the plan was
drafted. Kept here with their evidence so a later cell does not re-open them.

- [x] **Where does the verdict schema live?** It already exists:
  `MailboxResult` at `packages/bee-rs/crates/bee/src/herding/mailbox.rs:490`,
  parsed and validated at `:655-690`, carried by `result-N.json` whose
  appearance at the final name is the done signal (`:20-26`). The tool mirrors
  it; Rust keeps ownership. (Evidence: read, plus a real
  `job-1789890452645-4031915-1/result-1.json` on disk.)
- [x] **Does the verdict tool need an explicit `mapToolCall` row?** **Yes.**
  An unmapped tool whose arguments carry no `command`, path field or `url`
  falls to the default arm's last branch — `tool_name: "Write"` with
  `file_path: ""` (`.pi/extensions/bee-guard.ts:522-526`). Running that payload
  through the real hook returns **exit 2, denied**: *"bee write guard denied
  this target: it could not be canonically contained inside the physical
  worktree."* Without its own row, every verdict call is blocked. (Evidence:
  ran.)
- [x] **What refresh trigger keeps the widget current without a timer?** The
  result-inbox itself. `write_inbox_marker`
  (`packages/bee-rs/crates/bee/src/herding/run.rs:2269`) writes the pending
  marker — `job_id`, `seat`, `mailbox`, `cell_id`, `created_at` — **before the
  worker is spawned**, and the belt's drain already polls that directory. The
  widget re-renders off the drain's existing tick; no second timer. (Evidence:
  read.)
  - **Known limit, recorded not hidden:** the marker is written only when
    `--inbox-session` is passed (`run.rs:2269-2271`). The panel therefore shows
    DETACHED workers only. That is the honest set — a foreground run blocks the
    leader, so there is nothing to draw while it runs.

## Deferred Ideas

Each of these is a filed backlog row. This section records why it is not in
this feature, never a promise to return to it here.

- A4 — hook `model_select` and `thinking_level_select` to refresh the model-usage
  status line. Real but cosmetic: the line self-corrects at the next `turn_end`,
  so the harm is latency, not correctness. Filed as a P3 `proposal`.
- A5 — guard the six `registerCommand` calls with `pi.getCommands()`. Defensive
  only: Pi re-creates extensions on session replacement, so no live defect was
  found. Filed as a P3 `debt` row.
- A3 — the prose guidance baseline. Filed as a P2 `proposal` backlog row under
  `prose-guidance-baseline`, with its drift policy already locked (`460a639f`).

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
<!-- /bee:not-a-deferral -->
