# Stale promote-proposal audit (read-only), round 1

Paths read: docs/history/{handoff-self-relaunch,durable-state-writes,opencode-2x-tool-ids,bee-config-verbs,cell-question-park,path-rigor-floor,context-mark-advisory,handoff-flag-doctrine}/promote-proposals.md; docs/knowledge/areas/{doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:253-262, hook-runtime/token-economy-of-injected-context.md:122-133, hook-runtime/write-guard-request-shapes.md:455-462, workflow-state/dissent-and-the-verdict-duty.md:223-310, workflow-state/gates.md:409-420, workflow-state/cells-authoring-and-revision.md:236-246, worktree-parallelism/returning-and-the-merge-gate.md:150-155}; patterns/index.md; patterns/*.md titles and lines matched by rg.

Searches: rg -n -i over docs/knowledge for fsync (zero hits anywhere in docs/knowledge), handoff/leader/waggledance, park, RIGOR_FLOOR/OWNER_FIRST, context mark/advisory, CARGO_TARGET/shared target, red-first, token list, registry generator.

Two knowledge statements conflict on the context mark and need a human decision before any proposal is applied:
- router-triage...:261 says a context mark never stops work; handoff only on user ask or at feature close.
- token-economy...:127-130 says the mark line is a directive: hand off at the next clean stop. It says it replaced the advisory line.
The proposal header for handoff-self-relaunch (line 5) already marks its stop-at-mark rule SUPERSEDED. Do not promote hsrl-1 or hsrl-2 text.

### handoff-self-relaunch
b: doctrine-layer | hsrl-1 | CONFLICT docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:261 | handoff at mark via leader door; bundle says user word or close
b: doctrine-layer | hsrl-2 | CONFLICT docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:261 | leader:true dispatch past mark; bundle says user word or close
b: hook-runtime | hsrl-1 | CONFLICT docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md:127-129 | mark line directs handoff at next clean stop
b: hook-runtime | hsrl-2 | ABSENT | mark line names no waggledance dispatch; leader:true only at docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md:460
c: hsrl-1 | ROUTINE | sync-ack: only one AGENTS.md rule changed | n/a
c: hsrl-2 | ROUTINE | sync-ack: sibling hsrl-1 owns skill text | n/a

### durable-state-writes
b: workflow-state | dsw-1 | ABSENT | fsync tmp file, parent dir and jsonl appends (no fsync anywhere in docs/knowledge)
c: dsw-1 | ROUTINE | write_json_atomic delegates to write_text_atomic | n/a
c: dsw-1 | LESSON | worktree checkout lacks CONTEXT.md and plan.md | UNCOVERED (nearest docs/knowledge/patterns/20260819-inside-a-worktree-the-tracked-copy-of-a-runtime-store-answers-stale.md, different trap)
c: dsw-1 | LESSON | base suite red from installed opencode binary, not the cell | COVERED docs/knowledge/patterns/20260719-a-shared-checkout-with-a-second-live-session.md

### opencode-2x-tool-ids
b: none
c: oc2x-1 | LESSON | 2.x loader skips the 1.x default-export plugin, so the guard never loads | COVERED docs/knowledge/patterns/20260805-source-that-ships-without-reinstalling-the-binary-the-hooks-call-is-inert.md (nearest; loader-shape specific)
c: oc2x-1 | ROUTINE | 2.x renames write/edit/read args; belt port is follow-up work | n/a
c: oc2x-1 | ROUTINE | 2.x extra anchors and exclusions named as gaps | n/a

### bee-config-verbs
b: none
c: cfg-1 | LESSON | edited tests/registry_dispatch.rs outside the cell files | COVERED docs/knowledge/patterns/20260811-a-cells-declared-file-is-a-hypothesis.md
c: cfg-1 | ROUTINE | governed-key refusal reuses guard's deny check | n/a
c: cfg-1 | LESSON | registry generator script gone; payload hand-edited by JSON round-trip | UNCOVERED
c: cfg-1 | ROUTINE | config get reads tracked file only | n/a
c: cfg-1 | ROUTINE | set and unset results carry local and warning | n/a

### cell-question-park
b: workflow-state | cqp-1 | PRESENT docs/knowledge/areas/workflow-state/dissent-and-the-verdict-duty.md:237-240 | open park question read from trace.park; refusals name answer
b: workflow-state | cqp-2 | PRESENT docs/knowledge/areas/workflow-state/dissent-and-the-verdict-duty.md:259-260 | close parked-cell door; merge refuses parked cell
b: workflow-state | cqp-4 | PRESENT docs/knowledge/areas/workflow-state/dissent-and-the-verdict-duty.md:261-263 | parked cell counted as unfinished in start; status text not confirmed
b: worktree-parallelism | cqp-1 | PRESENT docs/knowledge/areas/workflow-state/dissent-and-the-verdict-duty.md:237-240 (found outside this area) | refusals name answer; supervisor reads live plus archive
b: worktree-parallelism | cqp-2 | PRESENT docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md:150-152 | merge refuses WORKTREE_MERGE_PARKED_CELL
b: worktree-parallelism | cqp-4 | PRESENT docs/knowledge/areas/workflow-state/dissent-and-the-verdict-duty.md:261-263 (found outside this area) | parked cells counted as unfinished work
c: cqp-1 | ROUTINE | refuse_parked guard moved into apply_block_mutation | n/a
c: cqp-1 | LESSON | dissent.rs edited outside the cell files, one line | COVERED docs/knowledge/patterns/20260811-a-cells-declared-file-is-a-hypothesis.md
c: cqp-1 | ROUTINE | answer stamps dropped or blocked park entries | n/a
c: cqp-1 | ROUTINE | round-2 remedy text on claim, unclaim, update, reroute | n/a
c: cqp-1 | ROUTINE | (d) residual: uncited decision row on archive race | n/a
c: cqp-1 | ROUTINE | judge advisory out of scope, filed to backlog | n/a
c: cqp-1 | ROUTINE | sync-ack: skill text landed in sibling cqp-3 | n/a
c: cqp-2 | ROUTINE | parked-cell door pushed only when a parked cell exists | n/a
c: cqp-2 | ROUTINE | supervisor reads per-feature parked cells via list_cells | n/a
c: cqp-2 | LESSON | tests written after code, not red-first | UNCOVERED (nearest docs/knowledge/patterns/20260724-red-first-oracle-fed-by-live-environment.md, different trap)
c: cqp-2 | LESSON | verify run with private CARGO_TARGET_DIR because shared target was stale | COVERED docs/knowledge/patterns/20260908-the-vendored-binary-is-behind-the-change-it-would-prove.md (verify under a different cargo target dir)
c: cqp-3 | ROUTINE | golden native-cell prompt updated, one line | n/a
c: cqp-3 | ROUTINE | regen rewrote rendered skill copies | n/a
c: cqp-3 | LESSON | fourth token list in gates-and-delegation.md also needed [PARKED] | COVERED docs/knowledge/patterns/20260818-a-rule-checked-at-two-points-needs-one-shared.md
c: cqp-3 | LESSON | chain_nudge.rs token list left stale, outside cell | COVERED docs/knowledge/patterns/20260818-a-rule-checked-at-two-points-needs-one-shared.md
c: cqp-3 | ROUTINE | path-rigor-floor also edits same file; second merge re-runs regen | n/a
c: cqp-4 | ROUTINE | render.rs reuses orient parked remedy helper | n/a
c: cqp-4 | ROUTINE | Cells line always prints parked=N | n/a
c: cqp-4 | ROUTINE | start_lane and start_default share helpers | n/a
c: cqp-4 | LESSON | private CARGO_TARGET_DIR because shared target held a stale path | COVERED docs/knowledge/patterns/20260908-the-vendored-binary-is-behind-the-change-it-would-prove.md
c: cqp-4 | ROUTINE | sync-ack: skill text owned by cqp-3 | n/a

### path-rigor-floor
b: workflow-state | rfl-1 | PRESENT docs/knowledge/areas/workflow-state/cells-authoring-and-revision.md:236-246 (RIGOR_FLOOR); docs/knowledge/areas/workflow-state/gates.md:409-418 (GATE_OWNER_FIRST after auto gate)
c: rfl-1 | LESSON | verify run with private CARGO_TARGET_DIR, shared cache stale | COVERED docs/knowledge/patterns/20260908-the-vendored-binary-is-behind-the-change-it-would-prove.md
c: rfl-1 | LESSON | cells add and update tests written after wiring, only 2 seen red | UNCOVERED
c: rfl-1 | ROUTINE | update test spawns own child for readable refusal text | n/a
c: rfl-1 | ROUTINE | after-lock GATE_OWNER_FIRST re-check has no race test | n/a
c: rfl-1 | LESSON | unrelated .bee/onboarding.json left unstaged in shared worktree | COVERED docs/knowledge/patterns/20260806-a-guard-on-the-shared-index-does-not-stop-a-path-scoped-commit-naming-someone-elses-file.md
c: rfl-1 | ROUTINE | sync-ack: skill text in sibling rfl-2 | n/a
c: rfl-2 | ROUTINE | regen rewrote 15 rendered copies | n/a
c: rfl-2 | ROUTINE | regen bumped .bee/onboarding.json updated_at, left uncommitted | n/a

### context-mark-advisory
b: hook-runtime | cma-1 | CONFLICT docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md:127-130 | mark line is a directive to hand off at clean stop, not advisory
b: doctrine-layer | cma-1 | PRESENT docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:261 | a context mark never stops work
c: cma-1 | LESSON | dispatch prepare refused limits.concurrency, leader ran inline | UNCOVERED (nearest docs/knowledge/patterns/20261027-a-dispatched-stage-that-fails-preflight-gets-run-inline-by-the-leader-and-the-preflight-never-gets-fixed.md, preflight-specific)
c: cma-1 | ROUTINE | sync-ack: only prompt_context.rs changed | n/a
c: cma-2 | LESSON | same inline run after limits.concurrency refusal | UNCOVERED (same nearest as cma-1)
c: cma-2 | ROUTINE | regen rewrote skill copies, onboarding.json, feature-gates.md | n/a
c: cma-2 | ROUTINE | sync-ack: only one AGENTS.md rule changed | n/a

### handoff-flag-doctrine
b: none
c: hfd-1 | LESSON | leader ran inline after limits.concurrency refusal | UNCOVERED (same nearest as cma-1)
c: hfd-1 | ROUTINE | regen rewrote skill copies and onboarding.json | n/a
c: hfd-1 | ROUTINE | sync-ack: only one AGENTS.md rule changed | n/a
