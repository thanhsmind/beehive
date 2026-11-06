---
artifact_contract: bee-research/v1
topic: forgent-xia
depth: deep
date: 2026-11-06
---

## Bottom Line

- Mode: `xia` (distill and discuss; nothing is built).
- Recommendation (ladder rung): adapt-upstream, item by item. Nine forgent mechanisms are real (code and tests) and fill a gap that bee has. The rest are prose in forgent too, or bee already has them.
- Strongest picks: crash-safe state writes, a per-cell question park, a path-to-rigor table, and the missing `bee config` verbs.
- Confidence: 80%. Every pick below was checked in both repos; items marked `Inference` were not.
- Suggested next step: bee-shaping, one feature per pick the user selects.

## Source manifest

| Field | Value |
|---|---|
| Repo or path | /mnt/data/Projects/refs/forgent (fgOS, github vantt/forgent) |
| Ref | main |
| Resolved commit SHA | aeb6e668e7e38b0865ceaa5055c23c7258614675 |
| Narrowed scope | design laws and governance; Rust runtime packages; workflow skills and domains; install, config and doctor; knowledge, docs and test discipline |

## Ranked findings

Rank weighs user value, how real the forgent version is, and the cost to adapt it.

| # | Mechanism | Forgent evidence | Bee today | Gap |
|---|---|---|---|---|
| 1 | Crash-safe file install: tmp write, fsync the file, install the name, fsync the parent; the error names the failed step | `packages/durable-file/rust/src/lib.rs:1-31`, tests `tests/install.rs`, `creation_modes.rs` (Upstream) | `packages/bee-rs/crates/bee/src/fsutil.rs:138-165` writes tmp and renames with no fsync; one `sync_all` in the whole crate (Local) | A power loss or kernel crash can leave `.bee/*.json` empty or torn |
| 2 | Per-item park on a human question: the item goes `awaiting-human`, leaves the frontier, sibling items keep running; the answer re-queues it | `core/skills/fgos-routing/SKILL.md:207-226`, `src/intake/discovery.mjs` (Upstream) | `waiting_on` is one mark per session; cell `blocked` exists only for swept claims, `src/verbs/cells/handlers_select.rs:318` (Local) | One open question stalls the whole lane in bee |
| 3 | Path-to-rigor table: write paths set kind, review pattern and rigor; agents can raise, never lower; `askOwnerFirst` for migrations and SQL; a request size rule (≤3 paths and ≤standard = one run) | `domains/coding/slice-kinds.yaml`, `src/workflow/propose-classify.mjs`, test `test/workflow/propose-classify.test.mjs` (Upstream) | Lane and risk are agent judgment, `skills/bee-planning/references/planning-reference.md:228-235` (Local) | Bee risk sizing is doctrine; forgent makes it a check with a floor |
| 4 | Config verbs `show / explain / set --reason` on a schema plus a bundled standard, three layers (release, global, project), locked compare-and-swap writes | `packages/config/rust/src/{inspect,write,locks,scope}.rs`, tests `inspect_verbs.rs`, `registry_defaults.rs` (Upstream) | `bee config get` prints "not built into this binary … read and edit .bee/config.json directly" (Local, command output) | Bee tells the agent to hand-edit state, which breaks its own CLI-only rule |
| 5 | Content check on owner questions: five parts (situation, cause, options with gain/harm/cost, recommendation, scope of the yes) | `core/skills/_shared/decision-question.md`, `src/state/decision-question.mjs:30,59`, test `test/scripts/decision-question-hook.test.mjs` (Upstream) | Shape check only, `src/hooks/write_guard/detectors.rs:174-215` (Local) | Bee does not check that a question carries its cost and recommendation |
| 6 | Per-item attempt limit (anti-loop breaker) for unattended runs | `docs/knowledge/design-safeguards-in-fgos-s-runner-anti-loop-breakers-model/runner-execution-safety.md:15-40` (Upstream, prose; runner code not located) | Stall detection and a ceiling in `src/herding.rs:669-705`; no repeat-attempt cap found (Local) | A stuck cell can be dispatched again and again under herding |
| 7 | Architecture layer manifest plus a test that fails on an upward import | `docs/architecture-manifest.json`, `test/architecture.test.mjs` (Upstream) | No layer check in the bee crate (Local, rg) | Layer drift in a large crate goes unnoticed |
| 8 | Test-ownership map: source file → owning tests; a lint refuses duplicates; test selection reads it | `test/test-ownership.mjs`, `test/scripts/test-ownership-lint.test.mjs`, `test-select.test.mjs` (Upstream) | Agent picks proof scope by judgment (AGENTS.md "Prove, then say so") (Local) | Scoped proof cannot be derived from the changed files |
| 9 | Doctor rows registry with stable ids and `doctor --fix` | `packages/config/contracts/doctor-rows.v1.json`, `doctor.rs`, `repair.rs`, test `golden_doctor.rs` (Upstream) | Rows carry fix text only, `src/doctor.rs:896-918`; no `--fix` (Local) | Bee prints the fix and the human runs it |

## Strategic, larger

- Domain layer for non-coding work (`domains/marketing/workflows/content-publish.yaml`): milestones with an accepter, `must` guards, a named `unenforced` list. Bee playbooks are coding-only. Value is high only if bee aims at business workflows.
- Per-project runtime pin by digest (`.fgos/distribution.json`, `packages/distribution/rust/src/verify.rs`). Bee records `bee_version` at onboarding but does not enforce it.
- Project coexistence file and project uninstall (`.fgos/coexistence.json`, `test/install/coexist.test.mjs`). Bee has no project uninstall.

## Checked and not recommended

- Event store with seal, hash and Git publish: real but dormant in forgent itself (`packages/event-store/rust/src/lib.rs:1`).
- Kernel holder locks with a predecessor rule: real; bee's pid lock works today, so the gain is small.
- Approved frame with a lines/days budget, batch-the-questions rule, maturity ladder F0-F5, durability ladder D1-D5: prose only in forgent; no proof it works.
- Doctrine anchor suite: forgent took it from bee (`beegog:doctrine-layer-always-loaded`).
- Fan-out of children, friction journal: bee already has these (`skills/bee-swarming`, `bee mailbox reflect`).

## Risks, unknowns, follow-ups

- Item 6: the forgent runner code for the breaker was not found; treat the design as `Inference`.
- Items 2 and 3 touch cell state and the planning route; both are public contracts and need their own shaping.
- Item 4 must decide whether a global layer is wanted; that is the user's call.

## Source pack

- Forgent: the files named above, plus `AGENTS.md`, `README.md`, `docs/platform/platform-foundations.md`, `docs/platform/governance.md`, `docs/platform/config/spec.md`.
- Bee: `packages/bee-rs/crates/bee/src/{fsutil.rs,herding.rs,doctor.rs,router.rs}`, `src/verbs/cells/handlers_select.rs`, `src/hooks/write_guard/detectors.rs`, `skills/bee-planning/references/planning-reference.md`, `docs/knowledge/areas/workflow-state/workflow-records-and-projections.md`.
