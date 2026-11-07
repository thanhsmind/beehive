---
type: bee.delivery
title: pi-stage-dispatch — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-stage-dispatch: 14 capped cell(s), 33 recorded deviation(s)."
timestamp: 2026-09-15
bee:
  id: pi-stage-dispatch-delivery
  lifecycle: active
  areas: [bee-herding, hook-runtime, doctrine-layer]
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
- **psd-2** — herding::run tests plus `node --check .pi/extensions/bee-guard.ts` and the release-manifest write and check — 217 herding::run tests incl. the three seat tests
- **psd-3** — drivers:: and prompts tests plus the release-manifest write and check — 410 passed, manifest 376 files match
- **psd-4** — render and skill_trees tests — 140 passed; the cell verify filter covers the touched render and skill_trees code plus the new onboard pi test
- **psd-5** — candidate build, candidate dev regen, the On Pi: placement checks, and the release-manifest write and check through the candidate — regen 3/3 green, On Pi: only under .agents/skills, manifest 376 files match
- **psd-6** — `--test pi_plugin_contracts` — the cell verify, the one test binary this cell touched (67 passed)
- **psd-7** — seat pointer check across the config reference and two knowledge docs, plus the absence of the old Pi 0.84.x label — docs-only pointer check
- **psd-8** — pointer check on `.bee/verify/verify-app/features/pi-hat-wave.md` and its README row — a real Pi 0.85.1 leader ran the wave; evidence re-read from the first sandbox run's wave log and the leader Pi session log
- **psd-9** — `drivers::` tests — 406 passed, including the unconfigured-hat seat test
- **psd-10** — `hooks::cli_shape::documented_invocations` — the one test the pinned exception feeds; its dead-entry equality proves every pin is still refused
- **psd-11** — `herding::run` tests — 220 passed, 3 new tests seen by name
- **psd-12** — `herding::run` tests — 223 passed incl. 3 new resolve_split_parent worker-caller tests; also green with the worker marker set in the test env
- **psd-13** — pointer check on the herding knowledge doc and the config reference — docs-only
- **psd-14** — pointer check on the feature file — facts cross-checked against the second run's layout captures (before and after identical), job records, result files and dispatch log

Feature-wide: the full declared suite ran green on main after the merge (35 test binaries, 4015 passed, 0 failed).

## Deviations

- **psd-1** — dispatch prepare --claim still refuses inside a granted worktree — claim writes holds and reservations on the shared control plane, which this cell did not audit — found a better route
- **psd-1** — the worktree feature is passed as the --feature default in run_dispatch_prepare, not threaded as a new prepare_dispatch_wire parameter — avoids editing 28 test call sites — found a better route
- **psd-1** — show_body in advisor_ref.rs became pub(crate) so the drivers test can reuse the git worktree fixture — hit an unforeseen obstacle
- **psd-2** — kept the run.rs seat flag, marker, envelope, and tests that an earlier worker on this cell left uncommitted in the worktree, after review against the cell action — found a better route
- **psd-3** — also synced the vendored copy .bee/bin/prompts/advisor.md — three drivers tests assert the vendored prompt matches the source, and the cell file list missed it — the plan was wrong about a fact
- **psd-3** — a Pi hat seat with no configured ceiling gets --ceiling 600 — the plan text did not name the absent case — found a better route
- **psd-5** — committed .bee/onboarding.json — dev regen refreshed its advisor.md hash from the earlier psd-3 prompt change — something else had to be fixed first
- **psd-6** — token proof is a shell expansion of the note flag text plus a successful dry run — the dry-run output carries no inbox_session field — the plan was wrong about a fact
- **psd-6** — found that an unconfigured hat seat fell through to advisor and lost its seat name, ceiling cap and seat block; fixed by psd-9 — hit an unforeseen obstacle
- **psd-6** — updated the injected-header row test to include seat — it was red since psd-2 added the drain seat row — something else had to be fixed first
- **psd-8** — the orchestrator started the live Pi leader through the herding transport instead of the worker typing into a pane — the Claude isolation guard refused pane prompts from a subagent (decision 74593eae) — hit an unforeseen obstacle
- **psd-8** — the Pi shell tool default timeout is recorded as not observed — every shell call in the Pi log passed an explicit timeout — the plan was wrong about a fact
- **psd-11** — the runner gets the task-file sentinel only when the task did not come inline — an unread pipe could block the launcher on a large task — found a better route
- **psd-11** — the detach check runs after the transport config check, so a bad transport still refuses in the foreground — found a better route
- **psd-12** — the worker marker is read once at option parse, not at the split call site — reading it inside execute made 2 tests fail when cargo test runs inside a worker pane — hit an unforeseen obstacle
- **psd-14** — hat pane close is proven from the layout captures and result files, not a hat envelope — a detached runner sends its stdout to null — the plan was wrong about a fact

Cells psd-4, psd-7, psd-9 and psd-13 followed the plan. Every sync-ack deviation named the docs cell that carried the matching doc update.

## Provenance

Proposed by `bee knowledge promote --work pi-stage-dispatch` from 14 capped cell trace(s) and the anchor `docs/history/pi-stage-dispatch/CONTEXT.md`, `docs/history/pi-stage-dispatch/plan.md`. Reviewed at capture on 2026-09-15: sources point at the archived traces (`bee close` retired the cells), long verify commands are shortened to the proof they ran, and repeated sync-ack lines are folded into one sentence. Nothing was added that the traces do not carry, except the feature-wide suite line, which is the main-tree run recorded at close.
