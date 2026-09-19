promote proposal for work item "pi-native-stage-driver" (docs/history/pi-native-stage-driver/CONTEXT.md + docs/history/pi-native-stage-driver/plan.md) — 7 capped cell(s): pnsd-1, pnsd-2, pnsd-3, pnsd-4, pnsd-5, pnsd-6, pnsd-7
anchor: history — docs/history/pi-native-stage-driver/CONTEXT.md, docs/history/pi-native-stage-driver/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/pi-native-stage-driver/delivery.md

---
type: bee.delivery
title: pi-native-stage-driver — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-native-stage-driver: 7 capped cell(s), 17 recorded deviation(s)."
timestamp: 2026-09-19
bee:
  id: pi-native-stage-driver-delivery
  lifecycle: active
  areas: [hook-runtime, bee-herding, workflow-state]
  required_context: [docs/history/pi-native-stage-driver/CONTEXT.md, docs/history/pi-native-stage-driver/plan.md]
  sources: [docs/history/pi-native-stage-driver/CONTEXT.md, docs/history/pi-native-stage-driver/plan.md, .bee/cells/pnsd-1.json, .bee/cells/pnsd-2.json, .bee/cells/pnsd-3.json, .bee/cells/pnsd-4.json, .bee/cells/pnsd-5.json, .bee/cells/pnsd-6.json, .bee/cells/pnsd-7.json]
---

# pi-native-stage-driver — Delivery

## What shipped

- **pnsd-1** — Scope the approved plan packet to its own feature at both lookup sites (2 file(s) changed)
- **pnsd-2** — Add a no-pane child runner to bee herding run, with an isolated child environment (2 file(s) changed)
- **pnsd-3** — Select the no-pane runner from the dispatch door for pi-binary agents (1 file(s) changed)
- **pnsd-4** — Prove the no-pane dispatch path end to end and record its verify recipe (1 file(s) changed)
- **pnsd-5** — Prove the five-seat hat wave runs with no panes and names a dropped seat (1 file(s) changed)
- **pnsd-6** — Narrow the model's tools per bee stage, answered by a real bee hook, announced to user and model (5 file(s) changed)
- **pnsd-7** — Warn into the transcript when a session settles with a cell still claimed (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pnsd-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee plan_packets` — 20 passed, 0 failed; leader re-ran it rather than trusting the worker report. Truth 1 lands in plan_packets_lane_packet_with_foreign_feature_is_not_returned, truth 2 in plan_packets_state_packet_with…
- **pnsd-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding` — 576 passed, 0 failed; leader re-ran it and then confirmed the NEW tests actually executed rather than trusting the count (pattern 20260818). All eight ran: child_argv_construction_matches_pi_subagent…
- **pnsd-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee prepare` — 59 passed, 0 failed, 8 ignored; leader re-ran it and then confirmed each NEW test executed by name rather than trusting the count (pattern 20260818). All six ran: agent_argv_first_normalizes_both_con…
- **pnsd-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts` — 71 passed, 0 failed, and the live drive the cell actually exists for was run by the leader on the REBUILT binary after installing it at .bee/bin/bee (doctor first reported binary_freshness not_ok aga…
- **pnsd-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding` — 576 passed, 0 failed, and the leader independently reproduced D10 crux rather than accepting the worker report. Leader run, from the worktree against the rebuilt binary: printf a long task | bee herd…
- **pnsd-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts && .bee/bin/bee dev release-manifest --check` — 73 passed 0 failed (was 71 before this cell), and release_manifest --check reports 376 files match stored manifest. The leader did not accept a source-shape green: it ran the hook itself. Before the …
- **pnsd-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts && .bee/bin/bee dev release-manifest --check` — 77 passed 0 failed (was 73 after pnsd-6), and release_manifest --check reports 376 files match stored manifest. The leader confirmed the NEW tests execute by name rather than trusting the count: clos…

## Deviations

- **pnsd-1** — get_approved_preview_packet was widened from private fn to pub(crate) fn so the new tests can call it directly — the cell did not name a visibility change and the worker reported no deviations — hit an unforeseen obstacle
- **pnsd-1** — The live end-to-end reproduction could not be re-run: the leader own bee gate --preview during Gate 2 overwrote the stranded release-2-41-2 packet, so the precondition no longer exists in this store. Truth 4 is proven by the unit test that recreates that exact state instead — the plan was wrong about a fact
- **pnsd-1** — sync-ack: No skill text changes. This cell fixes an internal invariant in a private lookup (get_approved_preview_packet): an approved packet must belong to the feature that produced it. No bee-planning, bee-swarming, bee-reviewing or bee-capturing step describes that lookup or changes shape because of it — the fix restores the behavior those skills already assume, rather than introducing a new one. The user-visible effect (a new feature no longer inherits a stale role plan) belongs in the workflow-state knowledge area, and is carried by decision 539bfbb2 plus the feature close capture.
- **pnsd-2** — execute_no_pane is declared pub(super) while Options and ExecResult stay more private, so the build emits four private-interfaces warnings. Not an error and not worth widening the types for, but it is new warning noise this cell introduced — found a better route
- **pnsd-2** — The worker report listed prepare.rs among its changed paths. That was the working-tree state, not this commit: git show confirms the commit touches only herding/run.rs and herding/tests.rs. The prepare.rs edits belong to pnsd-3, which was running in parallel — the plan was wrong about a fact
- **pnsd-2** — sync-ack: No skill text changes. This cell adds a second LAUNCH path inside bee herding run and changes nothing a skill describes: the dispatch door vocabulary, the worker contract, the seat, the ceiling and the result drain are all untouched, which is the point of D11. The user-visible effect lands when pnsd-3 selects this path, and pnsd-4 records it in the verify-app feature file.
- **pnsd-3** — followed the plan
- **pnsd-3** — sync-ack: No skill text changes. The door keeps ONE shape and one vocabulary: the payload is the same bee herding run command it already returned, with one added flag, and the leader still never picks the transport (D8). Nothing in bee-planning, bee-swarming, bee-reviewing or bee-capturing describes how a slot resolves to a transport, because that has always been config. The user-visible effect is recorded by pnsd-4 in the verify-app feature file.
- **pnsd-4** — The cell told the worker to attempt a denied write inside the child and refuse to cap if it was not blocked. The worker did not run it; the leader did, separately, and recorded it in evidence.md section D. Result: a BARE child is refused by the write guard in the guard own words and the target file was untouched — but a no-pane WORKER carries BEE_HERDING_WORKER=1, and hooks/mod.rs:144-155 mutes every hook except activity under that marker, so a no-pane worker is unguarded exactly as a PANE worker is. That is pre-existing, deliberate posture (herding-worker-standalone D3, herding-adopt D7), identical on both transports, so D11 parity holds and this cell introduces no regression — found a better route
- **pnsd-4** — The worker returned BLOCKED asking whether to change that muting, offering two routes and a leaning. The leader did not take either: both would change PANE worker behavior and touch locked decisions, which is a separate feature, not this cell. Filed as a P2 finding instead — hit an unforeseen obstacle
- **pnsd-4** — doctor --runtime pi failed its first run on binary_freshness because .bee/bin/bee is a vendored copy, not the cargo output. The leader installed the rebuilt binary there before re-running. The cell said to rebuild but not to reinstall — the plan was wrong about a fact
- **pnsd-5** — followed the plan — the cell was written as proof, not construction, and no seat-naming, ceiling or drain mechanism was added: every one of them was inherited from the pane path as D11 intended, and the drive confirmed it
- **pnsd-6** — The cell was dispatched TWICE. The first run (b2cc78ddf) built the belt side exactly as written — it calls runAdvisoryHook for stage-tools instead of hardcoding a table — but stage-tools was not one of bee ten HOOK_NAMES, so bee answered unknown hook with exit 1, the advisory wrapper swallowed it, and the gate narrowed nothing. The cell files named only the belt, its tests and the manifest, so the worker could not have added the answering side without leaving its scope, and correctly did not. The leader widened the cell to carry hooks/mod.rs and a new hooks/stage_tools.rs and re-dispatched — the plan was wrong about a fact
- **pnsd-6** — The declared verify covers the contract suite and the manifest, but the narrowing logic lives in bee own unit tests, which that suite never runs. The leader ran cargo test -p bee stage_tools separately rather than reporting a green that never executed the new branch — found a better route
- **pnsd-6** — sync-ack: No skill text changes. The stage gate is a belt behaviour plus a new bee hook; it adds no step to bee-planning, bee-swarming, bee-reviewing or bee-capturing and changes no verb a skill instructs. The user-facing half is announced in-session by the belt itself and by /bee-tools-reopen, which is where a user meets it.
- **pnsd-7** — followed the plan — and unlike pnsd-6 this cell reused an EXISTING hook (session-close, already in HOOK_NAMES) and in fact reuses the verdict the belt had already fetched, so there was no second half to build and no inert-hook repeat
- **pnsd-7** — sync-ack: No skill text changes. The close guard is a belt behaviour on an event bee already hooks, using a verdict bee already returns; it adds no step to any skill and changes no verb. A user meets it in the session transcript, not in a document.

## Provenance

Proposed by `bee knowledge promote --work pi-native-stage-driver` from 7 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-native-stage-driver/CONTEXT.md`, `docs/history/pi-native-stage-driver/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "pi-native-stage-driver" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-19T03:29:01.085Z), the work item declares no bee.areas.

area hook-runtime:
  - [pnsd-1] Scope the approved plan packet to its own feature at both lookup sites — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pnsd-1.json)
  - [pnsd-2] Add a no-pane child runner to bee herding run, with an isolated child environment — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pnsd-2.json)
  - [pnsd-3] Select the no-pane runner from the dispatch door for pi-binary agents — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/pnsd-3.json)
  - [pnsd-4] Prove the no-pane dispatch path end to end and record its verify recipe — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/pnsd-4.json)
  - [pnsd-6] Narrow the model's tools per bee stage, answered by a real bee hook, announced to user and model — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/pnsd-6.json)
  - [pnsd-7] Warn into the transcript when a session settles with a cell still claimed — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/pnsd-7.json)

area bee-herding:
  - [pnsd-1] Scope the approved plan packet to its own feature at both lookup sites — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pnsd-1.json)
  - [pnsd-2] Add a no-pane child runner to bee herding run, with an isolated child environment — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pnsd-2.json)
  - [pnsd-3] Select the no-pane runner from the dispatch door for pi-binary agents — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/pnsd-3.json)
  - [pnsd-4] Prove the no-pane dispatch path end to end and record its verify recipe — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/pnsd-4.json)
  - [pnsd-6] Narrow the model's tools per bee stage, answered by a real bee hook, announced to user and model — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/pnsd-6.json)
  - [pnsd-7] Warn into the transcript when a session settles with a cell still claimed — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/pnsd-7.json)

area workflow-state:
  - [pnsd-1] Scope the approved plan packet to its own feature at both lookup sites — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pnsd-1.json)
  - [pnsd-2] Add a no-pane child runner to bee herding run, with an isolated child environment — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pnsd-2.json)
  - [pnsd-3] Select the no-pane runner from the dispatch door for pi-binary agents — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/pnsd-3.json)
  - [pnsd-4] Prove the no-pane dispatch path end to end and record its verify recipe — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/pnsd-4.json)
  - [pnsd-6] Narrow the model's tools per bee stage, answered by a real bee hook, announced to user and model — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/pnsd-6.json)
  - [pnsd-7] Warn into the transcript when a session settles with a cell still claimed — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/pnsd-7.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pnsd-1 — save as docs/knowledge/patterns/pi-native-stage-driver-pnsd-1-pitfall.md

---
type: bee.pattern
title: pi-native-stage-driver cell pnsd-1 — pitfall candidate
description: "Pitfall candidate mined from cell pnsd-1's capped trace: get_approved_preview_packet was widened from private fn to pub(crate) fn so the new tests can call it directly — the cell did not name a visibility change and …"
timestamp: 2026-09-18
bee:
  id: pi-native-stage-driver-pnsd-1-pitfall
  lifecycle: draft
  areas: [hook-runtime, bee-herding, workflow-state]
  sources: [.bee/cells/pnsd-1.json]
  polarity: pitfall
---

# pi-native-stage-driver cell pnsd-1 — pitfall candidate

## What the cell did

Scope the approved plan packet to its own feature at both lookup sites

## Recorded evidence (verbatim from .bee/cells/pnsd-1.json)

- **deviation** — get_approved_preview_packet was widened from private fn to pub(crate) fn so the new tests can call it directly — the cell did not name a visibility change and the worker reported no deviations — hit an unforeseen obstacle
- **deviation** — The live end-to-end reproduction could not be re-run: the leader own bee gate --preview during Gate 2 overwrote the stranded release-2-41-2 packet, so the precondition no longer exists in this store. Truth 4 is proven by the unit test that recreates that exact state instead — the plan was wrong about a fact
- **deviation** — sync-ack: No skill text changes. This cell fixes an internal invariant in a private lookup (get_approved_preview_packet): an approved packet must belong to the feature that produced it. No bee-planning, bee-swarming, bee-reviewing or bee-capturing step describes that lookup or changes shape because of it — the fix restores the behavior those skills already assume, rather than introducing a new one. The user-visible effect (a new feature no longer inherits a stale role plan) belongs in the workflow-state knowledge area, and is carried by decision 539bfbb2 plus the feature close capture.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pnsd-2 — save as docs/knowledge/patterns/pi-native-stage-driver-pnsd-2-pitfall.md

---
type: bee.pattern
title: pi-native-stage-driver cell pnsd-2 — pitfall candidate
description: "Pitfall candidate mined from cell pnsd-2's capped trace: execute_no_pane is declared pub(super) while Options and ExecResult stay more private, so the build emits four private-interfaces warnings. Not an error and no…"
timestamp: 2026-09-19
bee:
  id: pi-native-stage-driver-pnsd-2-pitfall
  lifecycle: draft
  areas: [hook-runtime, bee-herding, workflow-state]
  sources: [.bee/cells/pnsd-2.json]
  polarity: pitfall
---

# pi-native-stage-driver cell pnsd-2 — pitfall candidate

## What the cell did

Add a no-pane child runner to bee herding run, with an isolated child environment

## Recorded evidence (verbatim from .bee/cells/pnsd-2.json)

- **deviation** — execute_no_pane is declared pub(super) while Options and ExecResult stay more private, so the build emits four private-interfaces warnings. Not an error and not worth widening the types for, but it is new warning noise this cell introduced — found a better route
- **deviation** — The worker report listed prepare.rs among its changed paths. That was the working-tree state, not this commit: git show confirms the commit touches only herding/run.rs and herding/tests.rs. The prepare.rs edits belong to pnsd-3, which was running in parallel — the plan was wrong about a fact
- **deviation** — sync-ack: No skill text changes. This cell adds a second LAUNCH path inside bee herding run and changes nothing a skill describes: the dispatch door vocabulary, the worker contract, the seat, the ceiling and the result drain are all untouched, which is the point of D11. The user-visible effect lands when pnsd-3 selects this path, and pnsd-4 records it in the verify-app feature file.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pnsd-3 — save as docs/knowledge/patterns/pi-native-stage-driver-pnsd-3-pitfall.md

---
type: bee.pattern
title: pi-native-stage-driver cell pnsd-3 — pitfall candidate
description: "Pitfall candidate mined from cell pnsd-3's capped trace: followed the plan"
timestamp: 2026-09-19
bee:
  id: pi-native-stage-driver-pnsd-3-pitfall
  lifecycle: draft
  areas: [hook-runtime, bee-herding, workflow-state]
  sources: [.bee/cells/pnsd-3.json]
  polarity: pitfall
---

# pi-native-stage-driver cell pnsd-3 — pitfall candidate

## What the cell did

Select the no-pane runner from the dispatch door for pi-binary agents

## Recorded evidence (verbatim from .bee/cells/pnsd-3.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: No skill text changes. The door keeps ONE shape and one vocabulary: the payload is the same bee herding run command it already returned, with one added flag, and the leader still never picks the transport (D8). Nothing in bee-planning, bee-swarming, bee-reviewing or bee-capturing describes how a slot resolves to a transport, because that has always been config. The user-visible effect is recorded by pnsd-4 in the verify-app feature file.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pnsd-4 — save as docs/knowledge/patterns/pi-native-stage-driver-pnsd-4-pitfall.md

---
type: bee.pattern
title: pi-native-stage-driver cell pnsd-4 — pitfall candidate
description: "Pitfall candidate mined from cell pnsd-4's capped trace: The cell told the worker to attempt a denied write inside the child and refuse to cap if it was not blocked. The worker did not run it; the leader did, separat…"
timestamp: 2026-09-19
bee:
  id: pi-native-stage-driver-pnsd-4-pitfall
  lifecycle: draft
  areas: [hook-runtime, bee-herding, workflow-state]
  sources: [.bee/cells/pnsd-4.json]
  polarity: pitfall
---

# pi-native-stage-driver cell pnsd-4 — pitfall candidate

## What the cell did

Prove the no-pane dispatch path end to end and record its verify recipe

## Recorded evidence (verbatim from .bee/cells/pnsd-4.json)

- **deviation** — The cell told the worker to attempt a denied write inside the child and refuse to cap if it was not blocked. The worker did not run it; the leader did, separately, and recorded it in evidence.md section D. Result: a BARE child is refused by the write guard in the guard own words and the target file was untouched — but a no-pane WORKER carries BEE_HERDING_WORKER=1, and hooks/mod.rs:144-155 mutes every hook except activity under that marker, so a no-pane worker is unguarded exactly as a PANE worker is. That is pre-existing, deliberate posture (herding-worker-standalone D3, herding-adopt D7), identical on both transports, so D11 parity holds and this cell introduces no regression — found a better route
- **deviation** — The worker returned BLOCKED asking whether to change that muting, offering two routes and a leaning. The leader did not take either: both would change PANE worker behavior and touch locked decisions, which is a separate feature, not this cell. Filed as a P2 finding instead — hit an unforeseen obstacle
- **deviation** — doctor --runtime pi failed its first run on binary_freshness because .bee/bin/bee is a vendored copy, not the cargo output. The leader installed the rebuilt binary there before re-running. The cell said to rebuild but not to reinstall — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pnsd-5 — save as docs/knowledge/patterns/pi-native-stage-driver-pnsd-5-pitfall.md

---
type: bee.pattern
title: pi-native-stage-driver cell pnsd-5 — pitfall candidate
description: "Pitfall candidate mined from cell pnsd-5's capped trace: followed the plan — the cell was written as proof, not construction, and no seat-naming, ceiling or drain mechanism was added: every one of them was inherited …"
timestamp: 2026-09-19
bee:
  id: pi-native-stage-driver-pnsd-5-pitfall
  lifecycle: draft
  areas: [hook-runtime, bee-herding, workflow-state]
  sources: [.bee/cells/pnsd-5.json]
  polarity: pitfall
---

# pi-native-stage-driver cell pnsd-5 — pitfall candidate

## What the cell did

Prove the five-seat hat wave runs with no panes and names a dropped seat

## Recorded evidence (verbatim from .bee/cells/pnsd-5.json)

- **deviation** — followed the plan — the cell was written as proof, not construction, and no seat-naming, ceiling or drain mechanism was added: every one of them was inherited from the pane path as D11 intended, and the drive confirmed it

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pnsd-6 — save as docs/knowledge/patterns/pi-native-stage-driver-pnsd-6-pitfall.md

---
type: bee.pattern
title: pi-native-stage-driver cell pnsd-6 — pitfall candidate
description: "Pitfall candidate mined from cell pnsd-6's capped trace: The cell was dispatched TWICE. The first run (b2cc78ddf) built the belt side exactly as written — it calls runAdvisoryHook for stage-tools instead of hardcodin…"
timestamp: 2026-09-19
bee:
  id: pi-native-stage-driver-pnsd-6-pitfall
  lifecycle: draft
  areas: [hook-runtime, bee-herding, workflow-state]
  sources: [.bee/cells/pnsd-6.json]
  polarity: pitfall
---

# pi-native-stage-driver cell pnsd-6 — pitfall candidate

## What the cell did

Narrow the model's tools per bee stage, answered by a real bee hook, announced to user and model

## Recorded evidence (verbatim from .bee/cells/pnsd-6.json)

- **deviation** — The cell was dispatched TWICE. The first run (b2cc78ddf) built the belt side exactly as written — it calls runAdvisoryHook for stage-tools instead of hardcoding a table — but stage-tools was not one of bee ten HOOK_NAMES, so bee answered unknown hook with exit 1, the advisory wrapper swallowed it, and the gate narrowed nothing. The cell files named only the belt, its tests and the manifest, so the worker could not have added the answering side without leaving its scope, and correctly did not. The leader widened the cell to carry hooks/mod.rs and a new hooks/stage_tools.rs and re-dispatched — the plan was wrong about a fact
- **deviation** — The declared verify covers the contract suite and the manifest, but the narrowing logic lives in bee own unit tests, which that suite never runs. The leader ran cargo test -p bee stage_tools separately rather than reporting a green that never executed the new branch — found a better route
- **deviation** — sync-ack: No skill text changes. The stage gate is a belt behaviour plus a new bee hook; it adds no step to bee-planning, bee-swarming, bee-reviewing or bee-capturing and changes no verb a skill instructs. The user-facing half is announced in-session by the belt itself and by /bee-tools-reopen, which is where a user meets it.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pnsd-7 — save as docs/knowledge/patterns/pi-native-stage-driver-pnsd-7-pitfall.md

---
type: bee.pattern
title: pi-native-stage-driver cell pnsd-7 — pitfall candidate
description: "Pitfall candidate mined from cell pnsd-7's capped trace: followed the plan — and unlike pnsd-6 this cell reused an EXISTING hook (session-close, already in HOOK_NAMES) and in fact reuses the verdict the belt had alre…"
timestamp: 2026-09-19
bee:
  id: pi-native-stage-driver-pnsd-7-pitfall
  lifecycle: draft
  areas: [hook-runtime, bee-herding, workflow-state]
  sources: [.bee/cells/pnsd-7.json]
  polarity: pitfall
---

# pi-native-stage-driver cell pnsd-7 — pitfall candidate

## What the cell did

Warn into the transcript when a session settles with a cell still claimed

## Recorded evidence (verbatim from .bee/cells/pnsd-7.json)

- **deviation** — followed the plan — and unlike pnsd-6 this cell reused an EXISTING hook (session-close, already in HOOK_NAMES) and in fact reuses the verdict the belt had already fetched, so there was no second half to build and no inert-hook repeat
- **deviation** — sync-ack: No skill text changes. The close guard is a belt behaviour on an event bee already hooks, using a verdict bee already returns; it adds no step to any skill and changes no verb. A user meets it in the session transcript, not in a document.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 7 capped cell(s) mined, 1 delivery draft, 18 area bullet(s), 7 pattern candidate(s), 0 file(s) written.