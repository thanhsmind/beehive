---
type: bee.area
title: "Hook Runtime — the catalog of record, its rendered projections, the third runtime's own belt, and checkpoint activation"
description: "One logical definition of every checkpoint, rendered deterministically into one projection per runtime that can consume a rendered file; a runtime whose own before-tool surface cannot consume one carries its own hand-authored belt instead, held to the identical parity guarantee by a coverage gate — every difference named rather than drift — and the separate question of whether a project's checkpoints are enabled, rooted, and trusted enough to run at all."
timestamp: 2026-08-12
bee:
  id: hook-runtime-catalog-projections-and-activation
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: ["codex-hook-state-parity D1-D3, D8-D13", "codex-runtime-parity D1, D2", d91a8398-2d63-426b-a133-341568453200, "opencode-support D1, D2, D5, D6"]
  sources: ["codex-hook-state-parity cells 2, 3, 5 (paired Codex subagent audit, package authority, exclusive hook-source arbitration, and fresh-host handler delivery; capped traces and reports, 2026-07-16)", "codex-runtime-parity Safety foundation — cells codex-parity-2, 2b, 3, 4 (traces in .bee/cells/), reports in docs/history/codex-runtime-parity/reports/", "codex-native-runtime-v2 cnr2-2 (state-sync trigger extended at the generator sources to both runtimes' plan tools)", "opencode-support cells oc-2, oc-3, oc-6, oc-8, oc-9, oc-10 (OpenCode's own guard belt: live-proved throw-blocking, the apply_patch/lsp/list coverage-gate closures, and exit-0 repair/ask verdict honoring; capped traces in .bee/cells/, evidence in docs/history/opencode-support/discovery.md, 2026-08-11)", "opencode-support cell oc-12 (live nested-dispatch proof: the belt engaged correctly on a real cross-session hold and a real concurrent-worker git guard, and surfaced the session-identity gap named below; trace .bee/cells/oc-12.json)", "opencode-support cell oc-13 (bee onboard --apply installs the belt; this concept's own Pointers correction landed in passing; trace .bee/cells/oc-13.json)", "docs/specs/hook-runtime.md#B5", "docs/specs/hook-runtime.md#B6", "docs/specs/hook-runtime.md#R1", "docs/specs/hook-runtime.md#R6", "docs/specs/hook-runtime.md#E4", "docs/specs/hook-runtime.md#E5", "docs/specs/hook-runtime.md#E7", "docs/specs/hook-runtime.md#P3"]
  authoritative_for: "hook-runtime: the catalog of record, projection parity, and checkpoint activation"
---

# Hook Runtime — the catalog of record, its rendered projections, the third runtime's own belt, and checkpoint activation

Which checkpoints exist is one question; whether they fire in a given project is
another. The first is answered by a single catalog of record. Two of bee's three
runtimes each consume a projection rendered from that catalog, where every
difference between the two rendered projections is a named export rather than
drift. The third runtime's own before-tool checkpoint surface cannot consume a
rendered projection at all, so it carries its own hand-authored belt instead —
itself one more named difference under the same rule, held to the identical
parity guarantee by proof rather than by shared generation. The second question
— whether a project's checkpoints are enabled at all — is answered by the
project's own configuration and by the human owner's trust in each command
definition.

## Entry Points & Triggers

- Which checkpoints are active comes from one **catalog of record** rendered
  into projections. Claude Code and Codex each consume only their own
  rendered projection; the projections differ only by an explicitly named
  allowed list. The directional differences: both carry a pre-spawn dispatch
  guard — Claude on its dispatch tools, Codex on its native spawn call,
  judging only the envelope shape actually observed on the probed runtime
  version and passing every unobserved shape through open — while Codex
  alone has child-start and child-stop lifecycle audits
  (codex-native-runtime-v2, cnr2-8).
- OpenCode's own before-tool checkpoint surface has no abort or deny return
  value at all — the surface can only proceed normally or raise an error —
  so it cannot consume a rendered projection the way Claude Code and Codex
  do. OpenCode instead loads its own belt: one project file, checked into
  the repository and vendored into a host project by onboarding, that
  raises the same checkpoints through the same helper commands every other
  runtime calls. This is a named per-runtime difference the catalog's own
  parity rule already anticipates, never drift (opencode-support D1, D2).

## Data Dictionary

| Element | Meaning |
|---|---|
| catalog of record | The single logical definition of every checkpoint: event, matcher, handler. Claude Code's and Codex's rendered projections are both derived from it deterministically — rendering again must reproduce both byte-for-byte. |
| projection | The runtime-specific checkpoint list Claude Code or Codex actually loads. One per runtime, checked in, never hand-divergent. |
| runtime's own belt | OpenCode's hand-authored equivalent of a projection: one project file translating OpenCode's own before-tool events into the same helper calls every projection makes, held to the catalog's coverage guarantee by a derived registry check rather than by shared generation. |
| allowed difference | A named, exported exception explaining why one projection — or OpenCode's own belt — carries or omits a checkpoint the others do not. Any un-named difference is a defect. |
| reviewed definition | The exact command definition the owner has inspected and trusted. A new or changed non-managed definition does not run until it is reviewed again. |

## Behaviors & Operations

**B5 — Two projections, one truth.** Changing the catalog of record and
re-rendering updates both projections in the same change; the parity check in
the installation suite compares the assistant-facing settings against the
correct projection for that runtime and fails on any un-allowed drift. Each
difference is declared by runtime, event, and handler, and each projection is
proved independently.

**B6 — Project checkpoints are active, rooted, and reviewed.** Project
checkpoints are enabled unless an active configuration explicitly disables
them. A checkpoint command starts with the session's working directory, which
may be below the project root, so a project-local command first resolves the
project root and then launches its handler. A new or changed non-managed
definition is listed for review and skipped until the human owner trusts that
exact definition. Afterwards, a fresh lifecycle event uses the reviewed
definition; until then the assistant continues without that checkpoint and the
owner sees the pending-review warning.

**B7 — OpenCode's own belt earns the same parity guarantee by proof, not by
generation.** Because this belt is hand-authored, no re-render can prove it
stayed in step with the catalog the way B5 proves the two rendered
projections. Parity instead rests on three independent proofs, all run in
the installation suite: (1) every checkpoint the catalog marks blocking is
shown to deny, allow, fail closed on a crash, and fail closed on a missing
helper, through this belt specifically; (2) a derived inventory of every
tool OpenCode's own installed program registers is checked against the
belt's own routing, so a newly registered write- or read-capable tool the
belt does not yet route is caught by name instead of silently reaching an
unchecked default — this has already caught two real gaps this way, one a
patch-apply tool and one a code-navigation tool (opencode-support
oc-3/oc-9/oc-10); and (3) every checkpoint the catalog marks advisory-only
that this belt does not wire is named on its own line in the feature's own
record, never left to blend into surrounding prose, so a future catalog
addition this belt silently fails to carry is caught by name too.

**B8 — A verdict reaching OpenCode's belt has only two shapes to arrive in,
and both must be honored.** OpenCode's own before-tool surface returns void
on success or raises an error on failure — it carries no structured deny
value the way Claude Code's and Codex's checkpoint surfaces do. The belt
therefore raises a checkpoint's denial text as that error. Because a
checkpoint's verdict is not always a bare allow-or-deny, the belt also
inspects what a not-denying checkpoint returned: a mechanically repaired
argument is applied before the call proceeds, and a checkpoint that answers
"ask" rather than "allow" is still raised as an error rather than treated
as a pass-through allow — "ask" is a checkpoint's own dominant way of
stopping a repaired call for human confirmation, and honoring it only at
the deny value while silently allowing at "ask" would make that stopping
mechanism inert on OpenCode alone while it keeps working on every other
runtime (opencode-support D6, oc-8).

## Business Rules

- R1 — One catalog of record; Claude Code's and Codex's projections are
  rendered, never hand-edited; all directional differences between them
  must be exported by name (codex-runtime-parity D1; codex-hook-state-parity
  D1-D3). OpenCode's own before-tool surface cannot consume a rendered
  projection at all and carries its own hand-authored belt instead — one
  more named difference under this same rule, never drift, and held to the
  identical coverage guarantee by the proofs in B7 rather than by shared
  generation (opencode-support D1, D2).

- R6 — Project checkpoints are enabled by default, resolve project-local
  handlers from the project root even when a session starts below it, and any
  changed non-managed definition requires fresh human review before execution
  (decision d91a8398-2d63-426b-a133-341568453200).

- R28 — A proof whose only job is to show a runtime's own belt still has
  full checkpoint coverage must fail when the environment it needs (the
  runtime program itself, or the capability that lets its fixtures run) is
  unavailable, never quietly report success by skipping — a coverage proof
  that goes green having proved nothing is worse than no proof at all. A
  named, explicit opt-out may still choose to degrade to a visible skip
  that states its own reason; the default is fail (opencode-support oc-9).

## Edge Cases Settled

- Explicitly disabling checkpoints produces no project lifecycle execution;
  the absence of an opt-in flag does not disable them.

- Editing a reviewed command definition makes only the changed definition
  pending review; automation never rewrites or bypasses the owner's trust
  record.

- The state-sync trigger matches the plan/task tools of BOTH runtimes as a
  superset — Codex's native plan tool (`update_plan`) alongside the legacy
  Claude names — extended at the generator sources (catalog + both host
  renderers), never by hand-editing a rendered manifest; behavior proven by a
  contract row driving a real `update_plan` payload (codex-native-runtime-v2,
  cnr2-2).

## Open Gaps

- The pre-spawn dispatch guard's explicit-model-choice check still only
  recognizes the models configured for Claude Code and Codex. OpenCode's
  own dispatch tool carries no model argument at all today, so this gap has
  not yet misfired in practice, but a future OpenCode version that adds one
  would reach this unwidened check unchanged (opencode-support oc-11/oc-13).
- OpenCode's own command line can reach a pinned helper identity only
  through a nested spawn call made from a running session — there is no
  direct "run as this helper" entry point the way a primary session has one
  for itself; a caller expecting that shortcut finds none (opencode-support
  oc-12).
- A file reservation made by the session that goes on to dispatch a nested
  helper on OpenCode is not automatically recognized as the same actor once
  that nested helper performs the write: the outer session and the nested
  helper carry different native session identities on this runtime, so the
  reservation's own cross-session check can deny a nested helper writing
  the exact file reserved on its behalf. Proven live and not yet resolved;
  the remedy needs either the belt to forward a shared identity, or the
  reservation to accept a not-yet-known child identity, or a documented
  convention for who reserves in a nested dispatch (opencode-support oc-12).
- Worker dispatch through OpenCode is proven functional end to end — a real
  unit of work claimed, written, committed, and capped from inside a nested
  session — but, under OpenCode's current dispatch mechanism, one dispatch
  runs at a time; concurrent dispatch is expected once OpenCode's own
  upstream limitation closes (opencode-support D5, oc-12).

## Pointers (implementation)

- Catalog + renderer: `packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs`
  — the Rust port of the deleted Node `packages/bee/hooks/catalog.mjs`
  (retired at the R6 cutover, commit 5c62cad0). It exports `Runtime {
  Claude, Codex }`, `Target { Plugin, Repo }`, and the `ALLOWED_DIFFERENCES`
  table; `render_projection`/`render_projection_text` take an explicit
  `Target` (`Plugin` default, `Repo`) so both rendering targets still share
  one function, never forked logic. Projections: `packages/bee/hooks/hooks.json`
  (Codex, plugin target), `packages/bee/hooks/claude-hooks.json`
  (Claude, plugin target; `.claude-plugin/plugin.json` points here).
  OpenCode is a NAMED EXCLUSION from this catalog (opencode-support D1/D2,
  R1): its belt is the checked-in TypeScript plugin at
  `.opencode/plugins/bee-guard.ts`, not a rendered JSON manifest sharing this
  catalog's `Entry`/`Group` rows — see `hook_manifests.rs`'s own `Runtime`
  doc comment for why `Runtime::Opencode` would be the wrong shape.
- OpenCode belt internals (B7, B8): `mapToolCall` (tool→hook routing),
  `runBlockingHook` (throw-on-deny, exit-0 `updatedInput`/`ask`/unparseable
  handling), `runAdvisoryHook` (swallow + log), all in
  `.opencode/plugins/bee-guard.ts`. Parity/coverage proofs:
  `packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs` —
  `every_blocking_mapped_row_denies_allows_crashes_and_reports_a_missing_binary`,
  `every_registered_write_or_read_capable_opencode_tool_is_mapped_or_named_as_a_gap`,
  `advisory_gaps_the_plugin_does_not_wire_are_named_not_silent`,
  `three_belt_parity_every_blocking_rule_hits_helper_claude_codex_and_opencode`;
  the fail-not-skip opt-out (R28) is `BEE_OPENCODE_SUITE_ALLOW_SKIP`.
  Model-parameter allowlist gap (Open Gaps): `evaluate_claude_dispatch` in
  `packages/bee-rs/crates/bee/src/hooks/model_guard.rs` (no `models.opencode`
  branch yet). Nested-dispatch session-identity gap (Open Gaps): reservation
  identity is the acting CLI session (`verbs/reservations/reserve.rs`);
  the belt forwards OpenCode's own `input.sessionID` as `session_id`
  (`.opencode/plugins/bee-guard.ts`); the mismatch is evaluated by
  `find_session_conflicts` (`hooks/write_guard/checks.rs`). Evidence for all
  three gaps: `docs/history/opencode-support/discovery.md` (oc-12).
