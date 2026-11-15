# Delivery — pstack-playbook-port

Feature: bee carries the pstack poteto-mode playbook set as situations under its nine route classes, the leader routes to a situation, and the plan check holds the citation. Rules are CONTEXT.md D1-D5; the plan is `plan.md`; per-cell checker reports are in `reports/`; tool swaps and dropped steps are in `port-notes.md`.

## Concepts changed

- `docs/knowledge/areas/doctrine-layer/situation-layer-class-index-session-group-and-parity-fence.md`: new concept for the class index grammar, `situations.rs` and its compiled-in fallback, the orient situation lines, the plan check citation rule, the session group, the fallback skill, the eleven new skills, the thirteen new principles and the parity fence.
- `docs/knowledge/areas/doctrine-layer/per-task-route-tester-pairing-and-proof-floors.md`: the plan citation and orient listing (slice 1).
- `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`: the thirteen new principle rows and the merged improvements to nine existing ones (slice 4).
- `docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md`: slice 4.
- `docs/knowledge/areas/workflow-state/gates.md`: the plan check's playbook citation names a situation file.
- Re-graded with `bee knowledge confirm` after reading what this feature changed in their anchors (`main.rs` one `mod` line, `orient.rs` an additive situations list, `plan_group.rs`, `LEADER.md`, the planning and routing references, `spike.md`, the trigger fixture): 33 concepts across advisor-protocol, decision-memory, discovery-wayfinding, doctrine-layer, feedback-digest, hook-runtime, onboarding, product-backlog, project, rust-runtime, verify-pipeline, workflow-state and worktree-parallelism.

## What shipped

Slice 1 — selection and the bugfix skeleton (`547317f99` plan, `ca0ec5c23`):
- `bee orient --class` lists a class's situations through a new reader, `situations.rs` (`a8be04ffd`, tests `5ff9a89af`).
- `bee plan check` holds a plan to a situation file when the class lists situations (`d23bb0fb4`, tests `45ee41a92`).
- The first situation, bug-fix, merged with the bee bugfix playbook (`25bccf3b6`, checked `e9fc40b9a`).
- The parity fence reads situation files against their class index, red for each break (`2bddce22e`, `8c7823e62`).

Slice 2 — the other playbooks (plan `2e3d35daa`):
- docs, release and content situations plus authoring-a-skill (`ea91d0e27`, checked `b7032a750`).
- The session group, first half: pickup, pause, autonomous-run, orchestrate, worktree-cleanup (`0adfdf223`, checked `dc265a85e`); second half: babysit, shipping, autopilot, opening-a-pr (`051eda386`, checked `422729836`).
- perf-issue and hillclimb (`800d8342e`, checked `376db036d`); investigation, forensics and eval (`0f4a55e06`, checked `1dc4721e9`).
- refactoring and visual-parity (`1f29c38fb`, checked `6ab285be1`); prototype (`c710cdc42`, checked `ef0eb7e40`).
- feature and multi-phase-plan (`3ba27a8d1`, checked `e9a483943`).
- Checker fixes (`ffc6c41ff`, checked `91ccec8ad`); rendered trees (`6e3036823`).

Slice 3 — the eleven skills (plan `70057dcfc`), each with its checker:
- `bee-architect` (`52777acd0`, `269fad41b`), `bee-arena` (`1a72a53c5`, `e9913dd9b`), `bee-interrogate` (`261af76a7`, `348a5a28e`).
- `bee-fanout` (`ec92d65bc`, `d39ac4c6e`), `bee-show-me-your-work` (`6886cfdbf`, `2e4ecf2a0`), `bee-tdd` (`1e9958527`, `cd73a69b8`).
- `bee-reflect` (`7d06966ab`, `99c40190c`), `bee-recall` (`d5246e882`, `3cf7e9719`), `bee-blast-radius` (`c3d292870`, `bc47ed100`).
- `bee-comment-review` (`071e12824`, `45c311b90`), `bee-figure-it-out` (`a590767ab`, `51cc187d4`).
- Playbook steps that waited for the new skills restored (`d28cf9a9e`, checked `31649e0e5`).

Slice 4 — principles (plan `5d22d0f19`):
- foundational-thinking, redesign-from-first-principles, outcome-oriented-execution (`06e570157`, `fa9103f7c`).
- laziness-protocol, minimize-reader-load, model-the-domain (`58285028b`, `514da8503`).
- prove-it-works, sequence-verifiable-units, make-operations-idempotent (`f82a8ebbd`, `385bede82`).
- guard-the-context-window, never-block-on-the-human, exhaust-the-design-space, experience-first (`331557906`, `45c161bfa`).
- Improvements merged into nine existing principles (`578647b6d`, `02b15c336`) and into bee-how, bee-why, bee-teach, bee-unslop, bee-technical-writing and bee-verifying/bee-verify-upkeep (`7414b8ebe` through `dac33e0be`).
- Principle pointers restored and slice 3 leftovers closed (`4c0f0b358`, checked `cd6e14c1b`).

Slice 5 — doctrine and knowledge (plan `c4eef82f1`):
- `LEADER.md` teaches routing to a situation, the session situations and the fallback, and the trigger index gains eleven lines (`ecf77d86c`, `8bd04dcdf`, `e81cd5f5c`; checked `a963c71a4`).
- Knowledge consolidated and this record (cell ppb-39).

## Known limits

- The compiled-in fallback in `situations.rs` covers the nine class indexes only; it moves on a rebuild, and the session group is read from the repo file.
- The three consider findings in `reports/ppb-38c.md` are open.
- `bee knowledge check` still prints warnings that predate this feature (missing Read when lines, long concepts, directory anchors, evidence states); this cell added none.
- The re-grade confirms a concept when what this feature changed in its anchors does not touch what the concept claims; it did not re-audit the unrelated anchor changes that also made some of them suspect (for example the 602-line router-triage concept, whose other anchors moved before this feature).
- The three generated index files were updated by hand to the counts and line `bee knowledge index --check` expects, not regenerated.
