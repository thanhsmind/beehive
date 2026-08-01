---
area: hook-runtime
updated: 2026-07-22
migrated_to: docs/knowledge/areas/hook-runtime/
---

# Hook Runtime (lifecycle guardrails) (migrated — pointer stub)

This area's current truth now lives in the knowledge bundle:
[`docs/knowledge/areas/hook-runtime/`](../knowledge/areas/hook-runtime/index.md)
(okf-foundation D20/D29/D37). Twelve concepts, split by TOPIC rather than the old spec's
headings: `overview.md` owns the frame every checkpoint sits inside — purpose, actors,
hostile-input immunity and the safety-net-not-authority rule;
`catalog-projections-and-activation.md` owns the one catalog of record, its two rendered
projections with every difference named, and whether a project's checkpoints are enabled,
rooted and trusted enough to run; `write-guard-request-shapes.md` owns the request shapes the
write guard can read — batch envelopes, workflow-command shape checks, and both recognised
command forms; `governed-paths-and-the-intake-gate.md` owns which write targets are governed,
the only-ever-shrinking always-writable set, and the intake gate that reads the phase rather
than a closed feature's leftover approvals; `advisories-and-turn-control.md` owns the advisory
contract, session-stop output, and the one deliberate turn-control exception;
`delivery-targets-and-the-fallback-command.md` owns the two rendering targets and the launch
contract a rendered fallback command owes, Windows form included;
`hook-source-exclusivity.md` owns the proof-gated arbitration that keeps exactly one hook
source active; `dispatch-guard.md` owns pre-spawn judgement of tier, model and helper type;
`native-spawn-and-transport-classification.md` owns the deliberate override pass-through and
the capability probe's scoped verdict; `child-agent-attribution-and-audit.md` owns the three
checkpoints that observe without authorising; `coordination-refresh-and-session-init.md` owns
the durable state a checkpoint maintains as a side job; and
`health-checks-and-proof-surfaces.md` owns how the guardrails are inspected by a human and
proven by the chain. This path stays alive as a pointer stub — it is never deleted in this
feature (D20) — and the anchor map below sends every numbered anchor the old spec exposed to
the concept that now owns it, so existing citations keep resolving. Coverage is machine-checked
by `scripts/okf_migrate.mjs --check hook-runtime` in the verify chain (D35), against the pinned
pre-migration blob `a8907ce` (81 anchors — 22 B / 24 R / 17 E / 18 P — 8 unparsed blocks —
okf-migration-f2 F8/F9).

## The duplicate `R14`, and why one rule now reads `R14a`

The pre-migration source carried the rule id **`R14` twice**, and it had done so since the
gate-bypass work added its own `R14` beside an existing one. They are two genuinely different
rules — one about the session-stop handler's block verdict, one about the write guard's
command-shape recognition — not one rule stated twice. Because anchors are keyed by id, the first
`R14`'s text was silently overwritten by the second's: it was unmeasurable by the coverage
gate's fidelity floor, permanently, and a set-equality check could not see the pair's second
member at all. Neither rule may be dropped or merged to make the collision go away, so the
source was **repaired before the migration pin was captured**: the second occurrence in document
order — the write-guard rule — was renumbered **`R14a`**, one token on one line.

The gate-bypass rule keeps `R14` because every live citation of `hook-runtime R14` means it:
this spec's own `R4` and `R10`, the gate-bypass pointer,
`skills/bee-hive/references/routing-and-contracts.md`, and decision `4c1c5921`. So a reader
arriving from any of those lands on
the rule they meant, unchanged. A reader arriving from the write-guard side finds the same rule
one row down as `R14a` — and the suffix here is a **disambiguation**, not a refinement of
`R14` the way `R8a`/`R8b` refine `R8`. Both rows appear in the map below.

The 8 unparsed blocks are all in "Behaviors & Operations" and none was invented into an anchor
(D10): `B2`'s wrapped continuation line that happens to open with a bold run, `B3`'s three
un-ided outcome bullets, and `B16`'s four un-ided case bullets. Each travels, verbatim, with
the anchor whose block it sits in.

## Anchor map

| Anchor | Now owned by | Was |
|---|---|---|
| B1 | [docs/knowledge/areas/hook-runtime/overview.md](../knowledge/areas/hook-runtime/overview.md) | hostile input never crashes a turn; fail-open is visible and never a flipped decision |
| B2 | [docs/knowledge/areas/hook-runtime/advisories-and-turn-control.md](../knowledge/areas/hook-runtime/advisories-and-turn-control.md) | advisories never steer the conversation — with one deliberate exception |
| B3 | [docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md](../knowledge/areas/hook-runtime/write-guard-request-shapes.md) | batch file-change requests are guarded per target |
| B3a | [docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md](../knowledge/areas/hook-runtime/write-guard-request-shapes.md) | workflow-command requests are shape-checked against the published catalog |
| B4 | [docs/knowledge/areas/hook-runtime/child-agent-attribution-and-audit.md](../knowledge/areas/hook-runtime/child-agent-attribution-and-audit.md) | worker nudges reach the right worker by registered identity |
| B5 | [docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md](../knowledge/areas/hook-runtime/catalog-projections-and-activation.md) | two projections, one truth — each proved independently |
| B6 | [docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md](../knowledge/areas/hook-runtime/catalog-projections-and-activation.md) | project checkpoints are active, rooted, and reviewed |
| B7 | [docs/knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md](../knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md) | the source-repository fallback is derived, not authored |
| B8 | [docs/knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md](../knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md) | fallback checkpoint commands are environment-independent |
| B9 | [docs/knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md](../knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md) | launch-setup failure and a computed decision are different things |
| B10 | [docs/knowledge/areas/hook-runtime/advisories-and-turn-control.md](../knowledge/areas/hook-runtime/advisories-and-turn-control.md) | session-stop handlers speak JSON or say nothing |
| B11 | [docs/knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md](../knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md) | a repo-root disposable-experiment location is no longer always-writable |
| B12 | [docs/knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md](../knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md) | no active work means no source writes — a finished feature is not an open door |
| B13 | [docs/knowledge/areas/hook-runtime/child-agent-attribution-and-audit.md](../knowledge/areas/hook-runtime/child-agent-attribution-and-audit.md) | Codex records paired native-subagent lifecycle evidence |
| B14 | [docs/knowledge/areas/hook-runtime/hook-source-exclusivity.md](../knowledge/areas/hook-runtime/hook-source-exclusivity.md) | exactly one bee hook source is active |
| B15 | [docs/knowledge/areas/hook-runtime/advisories-and-turn-control.md](../knowledge/areas/hook-runtime/advisories-and-turn-control.md) | the gate-bypass net mechanizes "zero stops" at the close-time checkpoint |
| B16 | [docs/knowledge/areas/hook-runtime/dispatch-guard.md](../knowledge/areas/hook-runtime/dispatch-guard.md) | the pre-spawn dispatch guard reads the declared tier before judging the model |
| B18 | [docs/knowledge/areas/hook-runtime/dispatch-guard.md](../knowledge/areas/hook-runtime/dispatch-guard.md) | work-tier dispatches ride pinned helper types, not the catch-all |
| B19 | [docs/knowledge/areas/hook-runtime/native-spawn-and-transport-classification.md](../knowledge/areas/hook-runtime/native-spawn-and-transport-classification.md) | the native spawn checkpoint deliberately does not judge an override |
| B20 | [docs/knowledge/areas/hook-runtime/coordination-refresh-and-session-init.md](../knowledge/areas/hook-runtime/coordination-refresh-and-session-init.md) | per-prompt and post-task-update checkpoints refresh coordination state, never blocking on it |
| B21 | [docs/knowledge/areas/hook-runtime/coordination-refresh-and-session-init.md](../knowledge/areas/hook-runtime/coordination-refresh-and-session-init.md) | session-init persists its own runtime-provided transcript path |
| B17 | [docs/knowledge/areas/hook-runtime/child-agent-attribution-and-audit.md](../knowledge/areas/hook-runtime/child-agent-attribution-and-audit.md) | a passive usage log records every tool call, and enforces nothing |
| R1 | [docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md](../knowledge/areas/hook-runtime/catalog-projections-and-activation.md) | one catalog of record; projections rendered, never hand-edited |
| R2 | [docs/knowledge/areas/hook-runtime/overview.md](../knowledge/areas/hook-runtime/overview.md) | a checkpoint failure never flips an allow/deny decision |
| R3 | [docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md](../knowledge/areas/hook-runtime/write-guard-request-shapes.md) | an intercepted batch with unprovable targets is denied, not fail-opened |
| R4 | [docs/knowledge/areas/hook-runtime/advisories-and-turn-control.md](../knowledge/areas/hook-runtime/advisories-and-turn-control.md) | advisory events never emit turn-control verdicts — one scoped exception |
| R5 | [docs/knowledge/areas/hook-runtime/dispatch-guard.md](../knowledge/areas/hook-runtime/dispatch-guard.md) | every dispatch carries a config-agreeing tier transport and is audit-logged |
| R6 | [docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md](../knowledge/areas/hook-runtime/catalog-projections-and-activation.md) | project checkpoints are enabled by default, rooted, and re-reviewed on change |
| R7 | [docs/knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md](../knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md) | the fallback checkpoint file is generated from the catalog, never hand-authored |
| R8 | [docs/knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md](../knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md) | a fallback command must not depend on packaged-only environment |
| R8a | [docs/knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md](../knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md) | every second-runtime fallback entry carries a shell-agnostic Windows command |
| R8b | [docs/knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md](../knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md) | both transport forms reference one committed checkpoint path |
| R8c | [docs/knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md](../knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md) | every rendered command prefers the native executable, keeping the script handler as its fallback arm |
| R9 | [docs/knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md](../knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md) | a pre-handoff launch-setup failure fails open visibly |
| R10 | [docs/knowledge/areas/hook-runtime/advisories-and-turn-control.md](../knowledge/areas/hook-runtime/advisories-and-turn-control.md) | every session-stop handler exits success and emits a single JSON object |
| R14 | [docs/knowledge/areas/hook-runtime/advisories-and-turn-control.md](../knowledge/areas/hook-runtime/advisories-and-turn-control.md) | the session-stop handler emits a block verdict ONLY for the gate-bypass net — **kept this id**: every external citation of `hook-runtime R14` means this rule |
| R11 | [docs/knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md](../knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md) | the always-writable set no longer includes the repo-root experiment location |
| R12 | [docs/knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md](../knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md) | the intake gate fires in every terminal state |
| R13 | [docs/knowledge/areas/hook-runtime/overview.md](../knowledge/areas/hook-runtime/overview.md) | the guardrails are a safety net, not the authority |
| R14a | [docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md](../knowledge/areas/hook-runtime/write-guard-request-shapes.md) | the write guard accepts both the unified dispatcher form and the retired helper form — **shipped as a second `R14`**; renumbered here so both rules are individually measurable |
| R15 | [docs/knowledge/areas/hook-runtime/hook-source-exclusivity.md](../knowledge/areas/hook-runtime/hook-source-exclusivity.md) | Codex plugin delivery loads the catalog-derived projection from the installed package |
| R16 | [docs/knowledge/areas/hook-runtime/hook-source-exclusivity.md](../knowledge/areas/hook-runtime/hook-source-exclusivity.md) | plugin and project hooks are mutually exclusive bee sources |
| R17 | [docs/knowledge/areas/hook-runtime/child-agent-attribution-and-audit.md](../knowledge/areas/hook-runtime/child-agent-attribution-and-audit.md) | the Codex native-subagent audit is bounded, audit-only, and post-start |
| R18 | [docs/knowledge/areas/hook-runtime/native-spawn-and-transport-classification.md](../knowledge/areas/hook-runtime/native-spawn-and-transport-classification.md) | the native spawn checkpoint never judges an override until the probe observes it |
| R19 | [docs/knowledge/areas/hook-runtime/coordination-refresh-and-session-init.md](../knowledge/areas/hook-runtime/coordination-refresh-and-session-init.md) | a checkpoint's own coordination refresh never waits on the lock |
| R20 | [docs/knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md](../knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md) | a Windows fallback command resolves the git repository root itself |
| R21 | [docs/knowledge/areas/hook-runtime/coordination-refresh-and-session-init.md](../knowledge/areas/hook-runtime/coordination-refresh-and-session-init.md) | session-init persists a runtime-provided transcript path into the session record |
| E1 | [docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md](../knowledge/areas/hook-runtime/write-guard-request-shapes.md) | a whitespace-only change-line path counts as unprovable → deny |
| E2 | [docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md](../knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md) | regenerating the RED-baseline evidence report is timestamp-stable in content |
| E3 | [docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md](../knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md) | requesting the evidence-baseline and catalog-only test modes together is rejected |
| E4 | [docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md](../knowledge/areas/hook-runtime/catalog-projections-and-activation.md) | explicitly disabling checkpoints produces no project lifecycle execution |
| E5 | [docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md](../knowledge/areas/hook-runtime/catalog-projections-and-activation.md) | editing a reviewed command definition makes only that definition pending review |
| E6 | [docs/knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md](../knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md) | fallback root resolution succeeds from paths with spaces and non-ASCII characters |
| E7 | [docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md](../knowledge/areas/hook-runtime/catalog-projections-and-activation.md) | the state-sync trigger matches the plan/task tools of BOTH runtimes |
| E8 | [docs/knowledge/areas/hook-runtime/coordination-refresh-and-session-init.md](../knowledge/areas/hook-runtime/coordination-refresh-and-session-init.md) | concurrent hook invocations can no longer corrupt a state write |
| E9 | [docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md](../knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md) | a read-only, fail-closed doctor command reports per-runtime health with evidence |
| E10 | [docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md](../knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md) | doctor's overall verdict is three-state, with a version-scoped attestation |
| E11 | [docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md](../knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md) | doctor resolves hook handlers at host topology (dual-location) |
| E12 | [docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md](../knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md) | skill install checks are a deep inventory audit against a render sidecar |
| E13 | [docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md](../knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md) | a scripted canary drives the REAL second-runtime CLI against a throwaway fixture |
| E14 | [docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md](../knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md) | a scripted conformance suite drives the guard and CLI binaries as subprocesses |
| E15 | [docs/knowledge/areas/hook-runtime/native-spawn-and-transport-classification.md](../knowledge/areas/hook-runtime/native-spawn-and-transport-classification.md) | the native-transport capability probe is version- and configuration-scoped |
| E16 | [docs/knowledge/areas/hook-runtime/native-spawn-and-transport-classification.md](../knowledge/areas/hook-runtime/native-spawn-and-transport-classification.md) | the probe's live check observed a real cross-build regression |
| E17 | [docs/knowledge/areas/hook-runtime/dispatch-guard.md](../knowledge/areas/hook-runtime/dispatch-guard.md) | the second-runtime tier-marker check alone recognizes an advisor role token |
| P1 | [docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md](../knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md) | shared isolated runner for nested test entrypoints |
| P2 | [docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md](../knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md) | real external integration stays external, graded on concrete status/output |
| P3 | [docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md](../knowledge/areas/hook-runtime/catalog-projections-and-activation.md) | catalog + renderer, and the two checked-in projections |
| P4 | [docs/knowledge/areas/hook-runtime/advisories-and-turn-control.md](../knowledge/areas/hook-runtime/advisories-and-turn-control.md) | shared adapter — `encodeAdvisory` / `encodeBlock` and the eight handlers |
| P5 | [docs/knowledge/areas/hook-runtime/advisories-and-turn-control.md](../knowledge/areas/hook-runtime/advisories-and-turn-control.md) | gate-bypass net — `maybeBypassBlock`, fire matrix, loop-guard, level |
| P6 | [docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md](../knowledge/areas/hook-runtime/write-guard-request-shapes.md) | batch guard — `extractApplyPatchTargets` |
| P7 | [docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md](../knowledge/areas/hook-runtime/write-guard-request-shapes.md) | CLI-shape guard including 3-token verb resolution |
| P8 | [docs/knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md](../knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md) | always-writable set — `GATE_ALLOWED_PREFIXES` and the close-time nudge's mirror |
| P9 | [docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md](../knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md) | hook-contract, write-guard and model-guard suites plus the parity check |
| P10 | [docs/knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md](../knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md) | package/fallback distribution proof |
| P11 | [docs/knowledge/areas/hook-runtime/hook-source-exclusivity.md](../knowledge/areas/hook-runtime/hook-source-exclusivity.md) | parity evidence — codex-hook-state-parity cells and reports |
| P12 | [docs/knowledge/areas/hook-runtime/hook-source-exclusivity.md](../knowledge/areas/hook-runtime/hook-source-exclusivity.md) | evidence — codex-runtime-parity red baseline, cell reports, commits |
| P13 | [docs/knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md](../knowledge/areas/hook-runtime/delivery-targets-and-the-fallback-command.md) | the Codex source-repository fallback projection and its runtime contract |
| P14 | [docs/knowledge/areas/hook-runtime/native-spawn-and-transport-classification.md](../knowledge/areas/hook-runtime/native-spawn-and-transport-classification.md) | native-transport classification, probe record reader/writer, doctor row |
| P15 | [docs/knowledge/areas/hook-runtime/native-spawn-and-transport-classification.md](../knowledge/areas/hook-runtime/native-spawn-and-transport-classification.md) | override-field pass-through gap documented inline, with canary rows |
| P16 | [docs/knowledge/areas/hook-runtime/native-spawn-and-transport-classification.md](../knowledge/areas/hook-runtime/native-spawn-and-transport-classification.md) | capability probe live leg and offline self-check |
| P17 | [docs/knowledge/areas/hook-runtime/coordination-refresh-and-session-init.md](../knowledge/areas/hook-runtime/coordination-refresh-and-session-init.md) | opportunistic coordination refresh — `heartbeatTouch`, `renewHoldsBySession`, the lock primitive |
| P18 | [docs/knowledge/areas/hook-runtime/dispatch-guard.md](../knowledge/areas/hook-runtime/dispatch-guard.md) | Claude model-param allowlist advisor fold |
