promote proposal for work item "pi-stage-dispatch" (docs/history/pi-stage-dispatch/CONTEXT.md + docs/history/pi-stage-dispatch/plan.md) — 14 capped cell(s): psd-1, psd-2, psd-3, psd-4, psd-5, psd-6, psd-7, psd-8, psd-9, psd-10, psd-11, psd-12, psd-13, psd-14
anchor: history — docs/history/pi-stage-dispatch/CONTEXT.md, docs/history/pi-stage-dispatch/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/pi-stage-dispatch/delivery.md

---
type: bee.delivery
title: pi-stage-dispatch — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-stage-dispatch: 14 capped cell(s), 33 recorded deviation(s)."
timestamp: 2026-09-15
bee:
  id: pi-stage-dispatch-delivery
  lifecycle: active
  areas: [bee-herding, hook-runtime, doctrine-layer, okf-profile, advisor-protocol, worktree-parallelism]
  required_context: [docs/history/pi-stage-dispatch/CONTEXT.md, docs/history/pi-stage-dispatch/plan.md]
  sources: [docs/history/pi-stage-dispatch/CONTEXT.md, docs/history/pi-stage-dispatch/plan.md, .bee/cells/archive/pi-stage-dispatch/psd-1.json, .bee/cells/archive/pi-stage-dispatch/psd-2.json, .bee/cells/archive/pi-stage-dispatch/psd-3.json, .bee/cells/archive/pi-stage-dispatch/psd-4.json, .bee/cells/archive/pi-stage-dispatch/psd-5.json, .bee/cells/archive/pi-stage-dispatch/psd-6.json, .bee/cells/archive/pi-stage-dispatch/psd-7.json, .bee/cells/archive/pi-stage-dispatch/psd-8.json, .bee/cells/archive/pi-stage-dispatch/psd-9.json, .bee/cells/archive/pi-stage-dispatch/psd-10.json, .bee/cells/archive/pi-stage-dispatch/psd-11.json, .bee/cells/archive/pi-stage-dispatch/psd-12.json, .bee/cells/archive/pi-stage-dispatch/psd-13.json, .bee/cells/archive/pi-stage-dispatch/psd-14.json]
---

# pi-stage-dispatch — Delivery

## What shipped

- **psd-1** — dispatch prepare and advisor-ref record/show serve main's store from a granted worktree; non-cell dispatch there defaults to the worktree feature (3 file(s) changed)
- **psd-2** — herding run --seat writes seat into inbox marker (write_inbox_marker) and JSON envelope (result_envelope); Pi drain renderResultInjection pushes seat row after job_id; PermissionRequest comment says Pi 0.84–0.85; release manifest refreshed (3 file(s) changed)
- **psd-3** — Pi note names PI_SESSION_ID, foreground, report_path (prepare.rs HERDING_DETACHED_DELIVERY_PI); pi non-cell role adds --seat and hat-* caps --ceiling at 600 (prepare.rs herding arm, HAT_WAVE_CEILING_SECONDS); advisor.md seat block via prompt_body_for seat arg; tests in drivers/tests.rs (5 file(s) changed)
- **psd-4** — pi joins both marker label lists (templates.rs RENDER_RUNTIMES, skill_trees.rs MARKER_RUNTIMES); onboard render.rs render_skill_bytes keeps pi blocks for the codex agents-root render only; claude/opencode roots and devtools plugin trees strip pi; sidecar schema unchanged (4 file(s) changed)
- **psd-5** — Pi dispatch and collection blocks added to four stage skills, rendered into the agents tree only (11 file(s) changed)
- **psd-6** — Pi advisor, hat, reviewer, and cell dispatch plus a detached drain round trip proven in pi_plugin_contracts.rs (1 file(s) changed)
- **psd-7** — Pi dispatch facts synced: config-reference (0.84–0.85 label, --seat and hat ceiling cap paragraph, PI_SESSION_ID token, drain seat field); hook-runtime B9 (token source, seat row); model-roles B15a (hat seat block, prepare and advisor-ref from granted worktree) (3 file(s) changed)
- **psd-8** — Live Pi hat wave mapped in pi-hat-wave.md with its README row (2 file(s) changed)
- **psd-9** — Unconfigured hat seat keeps its own name on --seat, Pi 600s ceiling, and prompt seat block while its model resolves through the advisor slot (prepare.rs seat_name; test an_unconfigured_hat_seat_keeps_its_own_seat_when_it_falls_through) (2 file(s) changed)
- **psd-10** — Pinned the psd-1 plan action span in KNOWN_HISTORICAL_EXCEPTIONS (cli_shape.rs, array length 3->4); documented_invocations test green (1 file(s) changed)
- **psd-11** — run() in herding/run.rs detaches an inbox-session run: should_detach gates it, spawn_detached_runner re-launches bee in process_group(0) with the task on a stdin pipe and --job-id, the launcher prints detached_envelope and exits 0 (1 file(s) changed)
- **psd-12** — resolve_split_parent takes caller_is_worker (Options.caller_is_worker, read from BEE_HERDING_WORKER at parse): a worker caller splits its own pane down under the same width guard and fresh-tab fallback; top-level choice unchanged (1 file(s) changed)
- **psd-13** — Knowledge doc states nested worker column rule and detached runner; config-reference Pi note says the detached command returns at once (2 file(s) changed)
- **psd-14** — Recorded the second live Pi hat wave in .bee/verify/verify-app/features/pi-hat-wave.md: detached launches, hat panes in the worker column, all panes closed, evidence paths kept (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **psd-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- drivers:: advisor_ref` — cell verify over touched prepare.rs, advisor_ref.rs and their tests (422 passed)
- **psd-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee herding::run && node --check .pi/extensions/bee-guard.ts && .bee/bin/bee dev release-manifest --write && .bee/bin/bee dev release-manifest --check` — 217 herding::run tests incl. parse_options_reads_the_seat_flag_and_defaults_to_none, seat_flag_writes_seat_into_the_inbox_marker_and_absent_leaves_it_out, seat_flag_writes_seat_into_the_result_envelo…
- **psd-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- drivers:: prompts && .bee/bin/bee dev release-manifest --write && .bee/bin/bee dev release-manifest --check` — touched prepare.rs, advisor.md, and drivers tests; 410 passed, manifest 376 files match
- **psd-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- render skill_trees` — 140 passed; the cell verify filter covers the touched render and skill_trees code plus the new onboard pi test
- **psd-5** — `export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"; cargo build --release --manifest-path packages/bee-rs/Cargo.toml -p bee && CANDIDATE="$(cargo metadata --no-deps --format-version 1 --manifest-path packages/bee-rs/Cargo.toml | jq -r .target_directory)/release/bee" && "$CANDIDATE" dev regen && rg -q 'On Pi:' .agents/skills/bee-hive/references/gates-and-delegation.md && ! rg -q 'On Pi:' .claude/skills .opencode/skills .claude-plugin/skills .codex-plugin/skills && "$CANDIDATE" dev release-manifest --write && "$CANDIDATE" dev release-manifest --check` — docs-only skill text; each step run in order: regen 3/3 green, On Pi: only under .agents/skills, manifest 376 files match, claude/opencode/plugin trees show no diff
- **psd-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts` — the cell verify, the one test binary this cell touched (67 passed)
- **psd-7** — `rg -n 'seat' docs/config-reference.md docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md && ! rg -n 'Pi 0\.84\.x' docs/config-reference.md` — cell verify pointer check for a docs-only change
- **psd-8** — `rg -n 'seat: hat-facts-gaps' .bee/verify/verify-app/features/pi-hat-wave.md && rg -n 'advisor-ref record' .bee/verify/verify-app/features/pi-hat-wave.md && rg -n 'pi-hat-wave' .bee/verify/verify-app/features/README.md` — a real Pi 0.85.1 leader ran the wave; evidence re-read from /tmp/bee-verify/run/20260915-180023-56873/evidence/wave-log.md and the Pi session log 01a0a4be-5ed3-77fc-9a7f-928b7df1850a.jsonl with three…
- **psd-9** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- drivers::` — cell verify covers prepare.rs seat/ceiling/prompt tests, 406 passed
- **psd-10** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- hooks::cli_shape::documented_invocations` — the one test the pinned exception feeds; its dead-entry equality proves every pin is still refused
- **psd-11** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- herding::run` — cell verify over the touched herding run module, run with the absolute cargo path the PATH prefix resolves to (220 passed, 3 new tests seen by name)
- **psd-12** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- herding::run` — 223 passed incl. 3 new resolve_split_parent worker-caller tests; also green with BEE_HERDING_WORKER=1 set in the test env
- **psd-13** — `rg -n 'process group' docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md && rg -n 'worker column' docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md && rg -n 'returns at once' docs/config-reference.md` — docs-only cell, cell verify pointer check
- **psd-14** — `rg -n 'closed_pane' .bee/verify/verify-app/features/pi-hat-wave.md && rg -n 'worker column' .bee/verify/verify-app/features/pi-hat-wave.md && rg -n 'outcome detached' .bee/verify/verify-app/features/pi-hat-wave.md` — docs-only feature file; cell verify pointer check, facts cross-checked against evidence2 layout captures (cmp 00-before/99-after identical), job.json, result-1.json and dispatch.jsonl

## Deviations

- **psd-1** — dispatch prepare --claim still refuses inside a granted worktree — claim writes holds and reservations on the shared control plane, which this cell did not audit — found a better route
- **psd-1** — the worktree feature is passed as the --feature default in run_dispatch_prepare, not threaded as a new prepare_dispatch_wire parameter — avoids editing 28 test call sites; order stays flag, bound lane, worktree, state.json — found a better route
- **psd-1** — show_body in advisor_ref.rs became pub(crate) so the drivers test can reuse the git worktree fixture — the fixture is private to drivers tests — hit an unforeseen obstacle
- **psd-1** — sync-ack: skill text for the pi-stage-dispatch slice lands in its planned docs cells (psd-5, psd-7); psd-1 files list no skills
- **psd-2** — kept the run.rs seat flag, marker, envelope, and tests that an earlier worker on this cell left uncommitted in the worktree, after review against the cell action — found a better route
- **psd-2** — sync-ack: --seat is an additive herding flag; the dispatch payload that passes it is cell psd-3, and no plan cell names a skills/bee-herding file (affects_skills is empty)
- **psd-3** — Also synced the vendored copy .bee/bin/prompts/advisor.md (reserved first) — three drivers tests assert the vendored prompt matches the source, and the cell file list missed it — the plan was wrong about a fact
- **psd-3** — A Pi hat seat with no configured ceiling gets --ceiling 600 — the plan text min(configured, 600) did not name the absent case, and the wave budget needs a cap — found a better route
- **psd-4** — followed the plan
- **psd-4** — sync-ack: approved plan sets affects_skills empty for psd-4: this cell only adds the pi marker label and agents-root render rule; skill text that uses bee:only pi blocks lands in other pi-stage-dispatch cells
- **psd-5** — ran the verify chain as separate commands instead of one compound line — the worktree command guard refused the compound form — hit an unforeseen obstacle
- **psd-5** — capped through a scratchpad script file instead of an inline finish command — the command guard refused the verify text inside the report — hit an unforeseen obstacle
- **psd-5** — committed .bee/onboarding.json, which is not in the cell files — dev regen refreshed its advisor.md hash from the earlier psd-3 prompt change, and a dirty tracked file would block merge — something else had to be fixed first
- **psd-5** — left docs/config-reference.md and two docs/knowledge files uncommitted — they are psd-7 in-flight edits reserved by w-psd-7b — something else had to be fixed first
- **psd-6** — Token proof is a shell expansion of the note flag text plus a successful --dry-run, not a read of parsed options — the dry-run JSON and job.json carry no inbox_session field — the plan was wrong about a fact
- **psd-6** — The test config sets team.pi hat-facts-gaps — an unconfigured hat seat falls through to advisor (prepare.rs:1964-1965), so Pi emits --seat advisor, no 600 cap, and no seat block; the product fix is in prepare.rs, outside this cell — hit an unforeseen obstacle
- **psd-6** — Updated the_injected_header_carries_exactly_the_one_line_rows_the_contract_names to include seat — it was red on the base since psd-2 added the drain seat row — something else had to be fixed first
- **psd-6** — The test initializes a repo in its tempdir — herding run resolves the main root through the checkout — hit an unforeseen obstacle
- **psd-7** — followed the plan
- **psd-7** — sync-ack: the four skills/** paths are a sibling worker's uncommitted edits in the shared worktree (psd-5 skill blocks); commit 1ad9f2cd for psd-7 touches only the three docs files
- **psd-8** — the orchestrator started the live Pi leader through the herding transport in the sandbox worktree instead of this worker typing into a pane — the isolation guard refused pane prompts from this subagent, decision 74593eae — hit an unforeseen obstacle
- **psd-8** — the Pi shell tool default timeout is recorded as not observed — all 14 shell calls in the Pi log passed an explicit timeout — the plan was wrong about a fact
- **psd-9** — followed the plan
- **psd-10** — ran the verify with PATH=$HOME/.cargo/bin prefix instead of the CARGO_HOME-default form — the worktree shell guard refused the runtime-computed variable; CARGO_HOME is unset so the same cargo ran — hit an unforeseen obstacle
- **psd-11** — The runner gets --task-file - appended only when the task did not come inline through --task; an inline task already rides argv and its stdin is null, because a non-empty --task wins in parse_options and an unread pipe could block the launcher on a large task — found a better route
- **psd-11** — The detach check runs after the transport config check, so a bad herding.transport still refuses in the foreground where the caller sees it — found a better route
- **psd-11** — No live launcher-kill proof was driven in this cell; the live check belongs to the orchestrator live run cell psd-14 — something else
- **psd-11** — sync-ack: cell files name only herding/run.rs and affects_skills is empty; the run verb flags are unchanged, and the doc sync for the detached outcome is planned cell psd-13
- **psd-12** — read the BEE_HERDING_WORKER marker once at option parse into Options.caller_is_worker and pass it through split_worker_pane, not at the resolve_split_parent call site — reading it inside split_worker_pane/execute made 2 execute tests fail when cargo test runs inside a worker pane (BEE_HERDING_WORKER=1) — hit an unforeseen obstacle
- **psd-12** — sync-ack: cell files are run.rs only; the planned knowledge sync for this split rule is psd-13 (the-run-verb-and-worker-outcomes.md); skills/bee-herding carries no split-parent rule text to change
- **psd-13** — followed the plan
- **psd-14** — Proved hat pane close from the layout captures and result-1.json mtimes, not from a hat envelope closed_pane field; closed_pane true is shown only for the leader envelope — a detached runner sends its stdout to null, so no hat envelope exists — the plan was wrong about a fact
- **psd-14** — Replaced one gotcha (the & wait launcher timeout), not two — the feature file held only one gotcha tied to open panes — the plan was wrong about a fact

## Provenance

Proposed by `bee knowledge promote --work pi-stage-dispatch` from 14 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-stage-dispatch/CONTEXT.md`, `docs/history/pi-stage-dispatch/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "pi-stage-dispatch" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-15T12:58:17.366Z), the work item declares no bee.areas.

area bee-herding:
  - [psd-11] run() in herding/run.rs detaches an inbox-session run: should_detach gates it, spawn_detached_runner re-launches bee in process_group(0) with the task on a stdin pipe and --job-id, the launcher prints detached_envelope and exits 0 — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/archive/pi-stage-dispatch/psd-11.json)
  - [psd-12] resolve_split_parent takes caller_is_worker (Options.caller_is_worker, read from BEE_HERDING_WORKER at parse): a worker caller splits its own pane down under the same width guard and fresh-tab fallback; top-level choice unchanged — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/archive/pi-stage-dispatch/psd-12.json)

area hook-runtime:
  - [psd-11] run() in herding/run.rs detaches an inbox-session run: should_detach gates it, spawn_detached_runner re-launches bee in process_group(0) with the task on a stdin pipe and --job-id, the launcher prints detached_envelope and exits 0 — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/archive/pi-stage-dispatch/psd-11.json)
  - [psd-12] resolve_split_parent takes caller_is_worker (Options.caller_is_worker, read from BEE_HERDING_WORKER at parse): a worker caller splits its own pane down under the same width guard and fresh-tab fallback; top-level choice unchanged — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/archive/pi-stage-dispatch/psd-12.json)

area doctrine-layer:
  - [psd-11] run() in herding/run.rs detaches an inbox-session run: should_detach gates it, spawn_detached_runner re-launches bee in process_group(0) with the task on a stdin pipe and --job-id, the launcher prints detached_envelope and exits 0 — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/archive/pi-stage-dispatch/psd-11.json)
  - [psd-12] resolve_split_parent takes caller_is_worker (Options.caller_is_worker, read from BEE_HERDING_WORKER at parse): a worker caller splits its own pane down under the same width guard and fresh-tab fallback; top-level choice unchanged — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/archive/pi-stage-dispatch/psd-12.json)

area okf-profile:
  - [psd-11] run() in herding/run.rs detaches an inbox-session run: should_detach gates it, spawn_detached_runner re-launches bee in process_group(0) with the task on a stdin pipe and --job-id, the launcher prints detached_envelope and exits 0 — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/archive/pi-stage-dispatch/psd-11.json)
  - [psd-12] resolve_split_parent takes caller_is_worker (Options.caller_is_worker, read from BEE_HERDING_WORKER at parse): a worker caller splits its own pane down under the same width guard and fresh-tab fallback; top-level choice unchanged — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/archive/pi-stage-dispatch/psd-12.json)

area advisor-protocol:
  - [psd-11] run() in herding/run.rs detaches an inbox-session run: should_detach gates it, spawn_detached_runner re-launches bee in process_group(0) with the task on a stdin pipe and --job-id, the launcher prints detached_envelope and exits 0 — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/archive/pi-stage-dispatch/psd-11.json)
  - [psd-12] resolve_split_parent takes caller_is_worker (Options.caller_is_worker, read from BEE_HERDING_WORKER at parse): a worker caller splits its own pane down under the same width guard and fresh-tab fallback; top-level choice unchanged — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/archive/pi-stage-dispatch/psd-12.json)

area worktree-parallelism:
  - [psd-11] run() in herding/run.rs detaches an inbox-session run: should_detach gates it, spawn_detached_runner re-launches bee in process_group(0) with the task on a stdin pipe and --job-id, the launcher prints detached_envelope and exits 0 — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/archive/pi-stage-dispatch/psd-11.json)
  - [psd-12] resolve_split_parent takes caller_is_worker (Options.caller_is_worker, read from BEE_HERDING_WORKER at parse): a worker caller splits its own pane down under the same width guard and fresh-tab fallback; top-level choice unchanged — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/archive/pi-stage-dispatch/psd-12.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell psd-1 — save as docs/knowledge/patterns/pi-stage-dispatch-psd-1-pitfall.md

---
type: bee.pattern
title: pi-stage-dispatch cell psd-1 — pitfall candidate
description: "Pitfall candidate mined from cell psd-1's capped trace: dispatch prepare --claim still refuses inside a granted worktree — claim writes holds and reservations on the shared control plane, which this cell did not aud…"
timestamp: 2026-09-15
bee:
  id: pi-stage-dispatch-psd-1-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime, doctrine-layer, okf-profile, advisor-protocol, worktree-parallelism]
  sources: [.bee/cells/archive/pi-stage-dispatch/psd-1.json]
  polarity: pitfall
---

# pi-stage-dispatch cell psd-1 — pitfall candidate

## What the cell did

dispatch prepare and advisor-ref record/show serve main's store from a granted worktree; non-cell dispatch there defaults to the worktree feature

## Recorded evidence (verbatim from .bee/cells/archive/pi-stage-dispatch/psd-1.json)

- **deviation** — dispatch prepare --claim still refuses inside a granted worktree — claim writes holds and reservations on the shared control plane, which this cell did not audit — found a better route
- **deviation** — the worktree feature is passed as the --feature default in run_dispatch_prepare, not threaded as a new prepare_dispatch_wire parameter — avoids editing 28 test call sites; order stays flag, bound lane, worktree, state.json — found a better route
- **deviation** — show_body in advisor_ref.rs became pub(crate) so the drivers test can reuse the git worktree fixture — the fixture is private to drivers tests — hit an unforeseen obstacle
- **deviation** — sync-ack: skill text for the pi-stage-dispatch slice lands in its planned docs cells (psd-5, psd-7); psd-1 files list no skills

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell psd-2 — save as docs/knowledge/patterns/pi-stage-dispatch-psd-2-pitfall.md

---
type: bee.pattern
title: pi-stage-dispatch cell psd-2 — pitfall candidate
description: "Pitfall candidate mined from cell psd-2's capped trace: kept the run.rs seat flag, marker, envelope, and tests that an earlier worker on this cell left uncommitted in the worktree, after review against the cell acti…"
timestamp: 2026-09-15
bee:
  id: pi-stage-dispatch-psd-2-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime, doctrine-layer, okf-profile, advisor-protocol, worktree-parallelism]
  sources: [.bee/cells/archive/pi-stage-dispatch/psd-2.json]
  polarity: pitfall
---

# pi-stage-dispatch cell psd-2 — pitfall candidate

## What the cell did

herding run --seat writes seat into inbox marker (write_inbox_marker) and JSON envelope (result_envelope); Pi drain renderResultInjection pushes seat row after job_id; PermissionRequest comment says Pi 0.84–0.85; release manifest refreshed

## Recorded evidence (verbatim from .bee/cells/archive/pi-stage-dispatch/psd-2.json)

- **deviation** — kept the run.rs seat flag, marker, envelope, and tests that an earlier worker on this cell left uncommitted in the worktree, after review against the cell action — found a better route
- **deviation** — sync-ack: --seat is an additive herding flag; the dispatch payload that passes it is cell psd-3, and no plan cell names a skills/bee-herding file (affects_skills is empty)

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell psd-3 — save as docs/knowledge/patterns/pi-stage-dispatch-psd-3-pitfall.md

---
type: bee.pattern
title: pi-stage-dispatch cell psd-3 — pitfall candidate
description: "Pitfall candidate mined from cell psd-3's capped trace: Also synced the vendored copy .bee/bin/prompts/advisor.md (reserved first) — three drivers tests assert the vendored prompt matches the source, and the cell fi…"
timestamp: 2026-09-15
bee:
  id: pi-stage-dispatch-psd-3-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime, doctrine-layer, okf-profile, advisor-protocol, worktree-parallelism]
  sources: [.bee/cells/archive/pi-stage-dispatch/psd-3.json]
  polarity: pitfall
---

# pi-stage-dispatch cell psd-3 — pitfall candidate

## What the cell did

Pi note names PI_SESSION_ID, foreground, report_path (prepare.rs HERDING_DETACHED_DELIVERY_PI); pi non-cell role adds --seat and hat-* caps --ceiling at 600 (prepare.rs herding arm, HAT_WAVE_CEILING_SECONDS); advisor.md seat block via prompt_body_for seat arg; tests in drivers/tests.rs

## Recorded evidence (verbatim from .bee/cells/archive/pi-stage-dispatch/psd-3.json)

- **deviation** — Also synced the vendored copy .bee/bin/prompts/advisor.md (reserved first) — three drivers tests assert the vendored prompt matches the source, and the cell file list missed it — the plan was wrong about a fact
- **deviation** — A Pi hat seat with no configured ceiling gets --ceiling 600 — the plan text min(configured, 600) did not name the absent case, and the wave budget needs a cap — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell psd-4 — save as docs/knowledge/patterns/pi-stage-dispatch-psd-4-pitfall.md

---
type: bee.pattern
title: pi-stage-dispatch cell psd-4 — pitfall candidate
description: "Pitfall candidate mined from cell psd-4's capped trace: followed the plan"
timestamp: 2026-09-15
bee:
  id: pi-stage-dispatch-psd-4-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime, doctrine-layer, okf-profile, advisor-protocol, worktree-parallelism]
  sources: [.bee/cells/archive/pi-stage-dispatch/psd-4.json]
  polarity: pitfall
---

# pi-stage-dispatch cell psd-4 — pitfall candidate

## What the cell did

pi joins both marker label lists (templates.rs RENDER_RUNTIMES, skill_trees.rs MARKER_RUNTIMES); onboard render.rs render_skill_bytes keeps pi blocks for the codex agents-root render only; claude/opencode roots and devtools plugin trees strip pi; sidecar schema unchanged

## Recorded evidence (verbatim from .bee/cells/archive/pi-stage-dispatch/psd-4.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: approved plan sets affects_skills empty for psd-4: this cell only adds the pi marker label and agents-root render rule; skill text that uses bee:only pi blocks lands in other pi-stage-dispatch cells

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell psd-5 — save as docs/knowledge/patterns/pi-stage-dispatch-psd-5-pitfall.md

---
type: bee.pattern
title: pi-stage-dispatch cell psd-5 — pitfall candidate
description: "Pitfall candidate mined from cell psd-5's capped trace: ran the verify chain as separate commands instead of one compound line — the worktree command guard refused the compound form — hit an unforeseen obstacle"
timestamp: 2026-09-15
bee:
  id: pi-stage-dispatch-psd-5-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime, doctrine-layer, okf-profile, advisor-protocol, worktree-parallelism]
  sources: [.bee/cells/archive/pi-stage-dispatch/psd-5.json]
  polarity: pitfall
---

# pi-stage-dispatch cell psd-5 — pitfall candidate

## What the cell did

Pi dispatch and collection blocks added to four stage skills, rendered into the agents tree only

## Recorded evidence (verbatim from .bee/cells/archive/pi-stage-dispatch/psd-5.json)

- **deviation** — ran the verify chain as separate commands instead of one compound line — the worktree command guard refused the compound form — hit an unforeseen obstacle
- **deviation** — capped through a scratchpad script file instead of an inline finish command — the command guard refused the verify text inside the report — hit an unforeseen obstacle
- **deviation** — committed .bee/onboarding.json, which is not in the cell files — dev regen refreshed its advisor.md hash from the earlier psd-3 prompt change, and a dirty tracked file would block merge — something else had to be fixed first
- **deviation** — left docs/config-reference.md and two docs/knowledge files uncommitted — they are psd-7 in-flight edits reserved by w-psd-7b — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell psd-6 — save as docs/knowledge/patterns/pi-stage-dispatch-psd-6-pitfall.md

---
type: bee.pattern
title: pi-stage-dispatch cell psd-6 — pitfall candidate
description: "Pitfall candidate mined from cell psd-6's capped trace: Token proof is a shell expansion of the note flag text plus a successful --dry-run, not a read of parsed options — the dry-run JSON and job.json carry no inbox…"
timestamp: 2026-09-15
bee:
  id: pi-stage-dispatch-psd-6-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime, doctrine-layer, okf-profile, advisor-protocol, worktree-parallelism]
  sources: [.bee/cells/archive/pi-stage-dispatch/psd-6.json]
  polarity: pitfall
---

# pi-stage-dispatch cell psd-6 — pitfall candidate

## What the cell did

Pi advisor, hat, reviewer, and cell dispatch plus a detached drain round trip proven in pi_plugin_contracts.rs

## Recorded evidence (verbatim from .bee/cells/archive/pi-stage-dispatch/psd-6.json)

- **deviation** — Token proof is a shell expansion of the note flag text plus a successful --dry-run, not a read of parsed options — the dry-run JSON and job.json carry no inbox_session field — the plan was wrong about a fact
- **deviation** — The test config sets team.pi hat-facts-gaps — an unconfigured hat seat falls through to advisor (prepare.rs:1964-1965), so Pi emits --seat advisor, no 600 cap, and no seat block; the product fix is in prepare.rs, outside this cell — hit an unforeseen obstacle
- **deviation** — Updated the_injected_header_carries_exactly_the_one_line_rows_the_contract_names to include seat — it was red on the base since psd-2 added the drain seat row — something else had to be fixed first
- **deviation** — The test initializes a repo in its tempdir — herding run resolves the main root through the checkout — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell psd-7 — save as docs/knowledge/patterns/pi-stage-dispatch-psd-7-pitfall.md

---
type: bee.pattern
title: pi-stage-dispatch cell psd-7 — pitfall candidate
description: "Pitfall candidate mined from cell psd-7's capped trace: followed the plan"
timestamp: 2026-09-15
bee:
  id: pi-stage-dispatch-psd-7-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime, doctrine-layer, okf-profile, advisor-protocol, worktree-parallelism]
  sources: [.bee/cells/archive/pi-stage-dispatch/psd-7.json]
  polarity: pitfall
---

# pi-stage-dispatch cell psd-7 — pitfall candidate

## What the cell did

Pi dispatch facts synced: config-reference (0.84–0.85 label, --seat and hat ceiling cap paragraph, PI_SESSION_ID token, drain seat field); hook-runtime B9 (token source, seat row); model-roles B15a (hat seat block, prepare and advisor-ref from granted worktree)

## Recorded evidence (verbatim from .bee/cells/archive/pi-stage-dispatch/psd-7.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: the four skills/** paths are a sibling worker's uncommitted edits in the shared worktree (psd-5 skill blocks); commit 1ad9f2cd for psd-7 touches only the three docs files

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell psd-8 — save as docs/knowledge/patterns/pi-stage-dispatch-psd-8-pitfall.md

---
type: bee.pattern
title: pi-stage-dispatch cell psd-8 — pitfall candidate
description: "Pitfall candidate mined from cell psd-8's capped trace: the orchestrator started the live Pi leader through the herding transport in the sandbox worktree instead of this worker typing into a pane — the isolation gua…"
timestamp: 2026-09-15
bee:
  id: pi-stage-dispatch-psd-8-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime, doctrine-layer, okf-profile, advisor-protocol, worktree-parallelism]
  sources: [.bee/cells/archive/pi-stage-dispatch/psd-8.json]
  polarity: pitfall
---

# pi-stage-dispatch cell psd-8 — pitfall candidate

## What the cell did

Live Pi hat wave mapped in pi-hat-wave.md with its README row

## Recorded evidence (verbatim from .bee/cells/archive/pi-stage-dispatch/psd-8.json)

- **deviation** — the orchestrator started the live Pi leader through the herding transport in the sandbox worktree instead of this worker typing into a pane — the isolation guard refused pane prompts from this subagent, decision 74593eae — hit an unforeseen obstacle
- **deviation** — the Pi shell tool default timeout is recorded as not observed — all 14 shell calls in the Pi log passed an explicit timeout — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell psd-9 — save as docs/knowledge/patterns/pi-stage-dispatch-psd-9-pitfall.md

---
type: bee.pattern
title: pi-stage-dispatch cell psd-9 — pitfall candidate
description: "Pitfall candidate mined from cell psd-9's capped trace: followed the plan"
timestamp: 2026-09-15
bee:
  id: pi-stage-dispatch-psd-9-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime, doctrine-layer, okf-profile, advisor-protocol, worktree-parallelism]
  sources: [.bee/cells/archive/pi-stage-dispatch/psd-9.json]
  polarity: pitfall
---

# pi-stage-dispatch cell psd-9 — pitfall candidate

## What the cell did

Unconfigured hat seat keeps its own name on --seat, Pi 600s ceiling, and prompt seat block while its model resolves through the advisor slot (prepare.rs seat_name; test an_unconfigured_hat_seat_keeps_its_own_seat_when_it_falls_through)

## Recorded evidence (verbatim from .bee/cells/archive/pi-stage-dispatch/psd-9.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell psd-10 — save as docs/knowledge/patterns/pi-stage-dispatch-psd-10-pitfall.md

---
type: bee.pattern
title: pi-stage-dispatch cell psd-10 — pitfall candidate
description: "Pitfall candidate mined from cell psd-10's capped trace: ran the verify with PATH=$HOME/.cargo/bin prefix instead of the CARGO_HOME-default form — the worktree shell guard refused the runtime-computed variable; CARGO…"
timestamp: 2026-09-15
bee:
  id: pi-stage-dispatch-psd-10-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime, doctrine-layer, okf-profile, advisor-protocol, worktree-parallelism]
  sources: [.bee/cells/archive/pi-stage-dispatch/psd-10.json]
  polarity: pitfall
---

# pi-stage-dispatch cell psd-10 — pitfall candidate

## What the cell did

Pinned the psd-1 plan action span in KNOWN_HISTORICAL_EXCEPTIONS (cli_shape.rs, array length 3->4); documented_invocations test green

## Recorded evidence (verbatim from .bee/cells/archive/pi-stage-dispatch/psd-10.json)

- **deviation** — ran the verify with PATH=$HOME/.cargo/bin prefix instead of the CARGO_HOME-default form — the worktree shell guard refused the runtime-computed variable; CARGO_HOME is unset so the same cargo ran — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell psd-11 — save as docs/knowledge/patterns/pi-stage-dispatch-psd-11-pitfall.md

---
type: bee.pattern
title: pi-stage-dispatch cell psd-11 — pitfall candidate
description: "Pitfall candidate mined from cell psd-11's capped trace: The runner gets --task-file - appended only when the task did not come inline through --task; an inline task already rides argv and its stdin is null, because …"
timestamp: 2026-09-15
bee:
  id: pi-stage-dispatch-psd-11-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime, doctrine-layer, okf-profile, advisor-protocol, worktree-parallelism]
  sources: [.bee/cells/archive/pi-stage-dispatch/psd-11.json]
  polarity: pitfall
---

# pi-stage-dispatch cell psd-11 — pitfall candidate

## What the cell did

run() in herding/run.rs detaches an inbox-session run: should_detach gates it, spawn_detached_runner re-launches bee in process_group(0) with the task on a stdin pipe and --job-id, the launcher prints detached_envelope and exits 0

## Recorded evidence (verbatim from .bee/cells/archive/pi-stage-dispatch/psd-11.json)

- **deviation** — The runner gets --task-file - appended only when the task did not come inline through --task; an inline task already rides argv and its stdin is null, because a non-empty --task wins in parse_options and an unread pipe could block the launcher on a large task — found a better route
- **deviation** — The detach check runs after the transport config check, so a bad herding.transport still refuses in the foreground where the caller sees it — found a better route
- **deviation** — No live launcher-kill proof was driven in this cell; the live check belongs to the orchestrator live run cell psd-14 — something else
- **deviation** — sync-ack: cell files name only herding/run.rs and affects_skills is empty; the run verb flags are unchanged, and the doc sync for the detached outcome is planned cell psd-13

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell psd-12 — save as docs/knowledge/patterns/pi-stage-dispatch-psd-12-pitfall.md

---
type: bee.pattern
title: pi-stage-dispatch cell psd-12 — pitfall candidate
description: "Pitfall candidate mined from cell psd-12's capped trace: read the BEE_HERDING_WORKER marker once at option parse into Options.caller_is_worker and pass it through split_worker_pane, not at the resolve_split_parent ca…"
timestamp: 2026-09-15
bee:
  id: pi-stage-dispatch-psd-12-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime, doctrine-layer, okf-profile, advisor-protocol, worktree-parallelism]
  sources: [.bee/cells/archive/pi-stage-dispatch/psd-12.json]
  polarity: pitfall
---

# pi-stage-dispatch cell psd-12 — pitfall candidate

## What the cell did

resolve_split_parent takes caller_is_worker (Options.caller_is_worker, read from BEE_HERDING_WORKER at parse): a worker caller splits its own pane down under the same width guard and fresh-tab fallback; top-level choice unchanged

## Recorded evidence (verbatim from .bee/cells/archive/pi-stage-dispatch/psd-12.json)

- **deviation** — read the BEE_HERDING_WORKER marker once at option parse into Options.caller_is_worker and pass it through split_worker_pane, not at the resolve_split_parent call site — reading it inside split_worker_pane/execute made 2 execute tests fail when cargo test runs inside a worker pane (BEE_HERDING_WORKER=1) — hit an unforeseen obstacle
- **deviation** — sync-ack: cell files are run.rs only; the planned knowledge sync for this split rule is psd-13 (the-run-verb-and-worker-outcomes.md); skills/bee-herding carries no split-parent rule text to change

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell psd-13 — save as docs/knowledge/patterns/pi-stage-dispatch-psd-13-pitfall.md

---
type: bee.pattern
title: pi-stage-dispatch cell psd-13 — pitfall candidate
description: "Pitfall candidate mined from cell psd-13's capped trace: followed the plan"
timestamp: 2026-09-15
bee:
  id: pi-stage-dispatch-psd-13-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime, doctrine-layer, okf-profile, advisor-protocol, worktree-parallelism]
  sources: [.bee/cells/archive/pi-stage-dispatch/psd-13.json]
  polarity: pitfall
---

# pi-stage-dispatch cell psd-13 — pitfall candidate

## What the cell did

Knowledge doc states nested worker column rule and detached runner; config-reference Pi note says the detached command returns at once

## Recorded evidence (verbatim from .bee/cells/archive/pi-stage-dispatch/psd-13.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell psd-14 — save as docs/knowledge/patterns/pi-stage-dispatch-psd-14-pitfall.md

---
type: bee.pattern
title: pi-stage-dispatch cell psd-14 — pitfall candidate
description: "Pitfall candidate mined from cell psd-14's capped trace: Proved hat pane close from the layout captures and result-1.json mtimes, not from a hat envelope closed_pane field; closed_pane true is shown only for the lead…"
timestamp: 2026-09-15
bee:
  id: pi-stage-dispatch-psd-14-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime, doctrine-layer, okf-profile, advisor-protocol, worktree-parallelism]
  sources: [.bee/cells/archive/pi-stage-dispatch/psd-14.json]
  polarity: pitfall
---

# pi-stage-dispatch cell psd-14 — pitfall candidate

## What the cell did

Recorded the second live Pi hat wave in .bee/verify/verify-app/features/pi-hat-wave.md: detached launches, hat panes in the worker column, all panes closed, evidence paths kept

## Recorded evidence (verbatim from .bee/cells/archive/pi-stage-dispatch/psd-14.json)

- **deviation** — Proved hat pane close from the layout captures and result-1.json mtimes, not from a hat envelope closed_pane field; closed_pane true is shown only for the leader envelope — a detached runner sends its stdout to null, so no hat envelope exists — the plan was wrong about a fact
- **deviation** — Replaced one gotcha (the & wait launcher timeout), not two — the feature file held only one gotcha tied to open panes — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 14 capped cell(s) mined, 1 delivery draft, 12 area bullet(s), 14 pattern candidate(s), 0 file(s) written.