---
type: bee.area
title: "Advisor Protocol — slots, transports, and staleness"
description: "Configuration is the authority: advice-class slot rules, dispatch transports and economics, and event-based consult staleness."
timestamp: 2026-07-19
bee:
  id: advisor-protocol-slots-and-tiers
  lifecycle: active
  areas: [advisor-protocol]
  required_context: [areas/advisor-protocol/overview.md]
  decisions: ["72f3d6dd (AO5 — config is the authority, no strength test, same-model no-op only)", AO8 (advice-class slots read-only), "AO2(b)/AO3/AO13 (one orchestrator trigger; execution-gate precondition; event-based staleness, never a TTL)", 0019 + 2A-iv GO (external gather proven through config), "codex-native-transport D1-D3, D5, D7 (3ceba8f5, cnt advisor conditions 69513d80, D3a c0cba64e)"]
  sources: ["advisor-and-orchestration Slices 2A-i..2A-iv, 2B, 3A, 3B, 4, 5 (cells ao-2ai-1..ao-5-1, traces in .bee/cells/, reports docs/history/advisor-and-orchestration/reports/, 2026-07-17)", dogfood run .bee/spikes/advisor-and-orchestration/2aiv-cli-gather-dogfood.md, "codex-native-transport cells cnt-1/cnt-2/cnt-3 (resolver + config native slot shape, capability classification, dispatch-prepare native branch + honest economics; traces in .bee/cells/, reports docs/history/codex-native-transport/reports/, 2026-07-19)", "codex-native-transport cell cnt-7 (Claude guard allowlist folds the adviser slot, closing a live adviser-dispatch refusal; trace in .bee/cells/, report docs/history/codex-native-transport/reports/cnt-7.md, 2026-07-19)", "docs/specs/advisor-protocol.md#R1", "docs/specs/advisor-protocol.md#R2", "docs/specs/advisor-protocol.md#R3", "docs/specs/advisor-protocol.md#R4", "docs/specs/advisor-protocol.md#R5", "docs/specs/advisor-protocol.md#R6", "docs/specs/advisor-protocol.md#R8", "docs/specs/advisor-protocol.md#R9", "docs/specs/advisor-protocol.md#E1", "docs/specs/advisor-protocol.md#E2", "docs/specs/advisor-protocol.md#E4", "docs/specs/advisor-protocol.md#E5", "docs/specs/advisor-protocol.md#E6", "docs/specs/advisor-protocol.md#P3", "docs/specs/advisor-protocol.md#P4", "docs/specs/advisor-protocol.md#P5", "docs/specs/advisor-protocol.md#P7"]
  authoritative_for: "advisor-protocol: slots, transports, staleness"
---

# Advisor Protocol — Slots, Transports, and Staleness

## Business Rules

- **R1 —** Config is the authority; the model does not get a vote (the ladder
  that once ranked models and silently skipped configured advisers is removed).
- **R2 —** The only skip is the literal same-model no-op.
- **R3 —** Advice-class slots are read-only, enforced at configuration checking
  (an honest blocklist of known write-granting/auto-approve tokens, stated as
  such — never a positive read-only guarantee).
- **R4 —** High-risk execution approval requires a live consult record;
  staleness is event-based (four events), never a time limit.
- **R5 —** Consult anchors are machine-stamped against the same record the verb
  mutates; callers cannot forge freshness.
- **R6 —** Advice never approves, never overrides, never writes; consults never
  substitute for the orchestrator's own verification re-run.
- **R8 —** **Dispatch payloads have one source of truth** (gh22-completion
  g22-1): `bee dispatch prepare --runtime <r> --kind cell|gather|reviewer|advisor`
  emits the guard-conformant payload (tier marker anchored exactly where the
  guard checks; pinned agent types from the guard's own map), a dispatch id,
  and the economics block. The guard's decision core is an exported pure
  function (`evaluateDispatch`) shared by the hook and prepare's tests —
  prepare's output is proven against the real guard, never a copied regex.
  Prepare surfaces tier refusals verbatim (a cli-shaped slot stays
  gather-only); the advisor kind resolves its model through the advisor
  resolver, never the generation slot. The same resolvers additionally accept,
  on any slot including advisor, a native model-override leaf and an
  explicit-fallback composite (codex-native-transport D2); the composite
  exposes its CLI fallback leg only when its fallback policy is stated
  explicitly (D1) — a bare native leaf or an unconfigured slot never invents a
  fallback command, and every pre-existing slot shape keeps resolving
  byte-identically. When the dispatched unit's feature runs in its own
  isolated worktree, the payload additionally names where the work happens
  and where the shared coordination store lives, and the rendered worker
  instructions carry both: without them, a worker starting inside an
  isolated worktree cannot tell where it is, and cannot tell that the
  record it must validate lives in a different workspace, leaving it to go
  looking by hand. A dispatch that executes a unit of work always names the
  write-capable execution worker's own helper identity, never a read-only
  helper's. The payload's own short label names the WORK, not the transport:
  a dispatch that executes a unit of work carries that unit's identifier and
  its recorded one-line title, cut to stay readable in a list; only a unit
  with no usable title falls back to the bare purpose-and-model form, and no
  label ever renders a dangling separator. The label matters because it is
  the only text a watching human sees per running helper — a label that is
  just a model name is the failure this rule exists to prevent, and the other
  dispatch purposes keep the short form because their subject is the
  caller's to state, not the record's to know.
- **R9 —** **Dispatch records tell the economic truth** (gh22-completion g22-2;
  refined by codex-native-transport D1/D3/D7): every record carries
  logical_tier, requested_model, effective_model, effective_model_status,
  channel, enforcement (additive; the legacy transport key is untouched). A
  model-param dispatch is `pinned`; a bare-marker budget dispatch is
  `unverified`. A second-runtime native spawn stays `inherited-or-unknown`
  with `prompt-budget` enforcement UNLESS a version- and configuration-scoped
  capability probe has classified the client as accepting a native model
  override for the resolved route — only then does the record carry
  `native-requested` status with `native-model-param` enforcement, the
  requested model/effort on the payload, and `effective_model` still null (the
  probe proves the client *accepted* the request, never that the runtime *ran*
  on it; a child's self-report is still never evidence). The stronger claim has
  its own reserved status name — `used-and-confirmed` — which no record may
  carry today: it becomes writable only if the runtime one day exposes
  confirmed effective-model metadata in its response, keeping the two trust
  stages non-synonymous by construction. A route that resolves
  native but the probe has not confirmed falls back to an explicitly
  configured CLI command with the reason recorded, or — with no such fallback
  configured — returns a typed refusal naming the classification that blocked
  it; config names the requested model, the log never silently claims it took
  effect.

## Edge Cases Settled

- **E1 —** External command reporting success while doing nothing →
  advice/gather output is accepted only between declared framing markers;
  missing or empty output is a failed run, surfaced loudly (proven by a real
  dogfood run).
- **E2 —** Adviser configured but the command cannot receive a prompt →
  refused at configuration checking (prompt transport is declared, never
  inferred).
- **E4 —** Execution gate revoked after a consult → the old consult is stale
  by rule; re-approval requires a fresh consult.
- **E5 —** A native-override route whose capability probe has not (yet)
  confirmed the client, and whose slot carries no explicit fallback policy,
  refuses the dispatch by name rather than silently downgrading to the
  budget-only path or inventing a CLI command (codex-native-transport D1/D3a).
- **E6 —** An adviser-kind dispatch prepared with a correctly-resolved adviser
  model was, until closed, still refused by the pre-spawn guard as an
  unconfigured model — the guard's allowlist recognized the tier slots but not
  the adviser slot, even though prepare's payload and the guard's decision
  core share the same resolver. The guard's allowlist now folds in a
  configured model-shaped adviser's own model (a cli-shaped, native, or
  unconfigured adviser adds nothing), closing the asymmetry; this can only
  turn a refusal into an allowance, never the reverse (codex-native-transport
  cnt-7; mechanism specced in `hook-runtime.md` B16).

## Open Gaps

- External advice/gather runs do not yet appear in the dispatch audit log
  (known, assigned to the measurement backlog — the passive tools log covers
  in-family calls only).

## Pointers (implementation)

- **P2a —** Payload label (R8): the `description` build in `prepare_dispatch`'s
  Agent arm, `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`, with
  `DESCRIPTION_TITLE_MAX` in the same file. Test:
  `a_cell_dispatch_description_says_what_the_work_is` (`drivers/tests.rs`),
  which also pins `gather (sonnet)` unchanged. Provenance: cell `ddi-1`,
  commit 537b9e85.

- **P3 —** Read-only validation: `validateModelsConfig` (advice-class token
  blocklist) + `validateAgentFilesDrift`, same lib; suite
  `scripts/tests/test_config_validate.mjs`.
- **P4 —** Guard allowlist fold (E6, cnt-7): `configuredModelSet` in
  `packages/bee/lib/dispatch-guard.mjs` (mirrored in
  `.bee/bin/lib/dispatch-guard.mjs`) calls `resolveAdvisor(root, 'claude')`
  directly, since the adviser slot is deliberately not a `CONFIGURABLE_SLOTS`
  member. Canary rows: `hooks/test_model_guard.mjs` rows 21-22.
- **P5 —** Resolution: `resolveAdvisor` (state.mjs); external gather contract:
  `skills/bee-hive/references/routing-and-contracts.md` (cli gather branch)
  and `docs/knowledge/areas/doctrine-layer/helper-classes-and-transports.md`
  B8/R12.
- **P7 —** Native-override transport (config shape, capability probe, dispatch
  prepare's native branch): capability classification and the doctor unlock
  row are specced in `docs/specs/hook-runtime.md`; the resolver's native leaf
  and explicit-fallback composite sit beside `resolveAdvisor`/`resolveTier`
  (`packages/bee/lib/state.mjs`); `prepareDispatch`'s
  native branch and `deriveEconomics`'s `native-requested` status share
  `packages/bee/lib/dispatch-prepare.mjs` with R8's
  `evaluateDispatch`.
