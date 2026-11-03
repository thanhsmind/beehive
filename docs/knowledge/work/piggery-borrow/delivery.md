---
type: bee.delivery
title: piggery-borrow — delivery
description: "Delivery record proposed by bee knowledge promote for work item piggery-borrow: 7 capped cell(s), 23 recorded deviation(s)."
timestamp: 2026-10-07
bee:
  id: piggery-borrow-delivery
  lifecycle: active
  areas: [bee-herding, hook-runtime]
  required_context: [docs/history/piggery-borrow/CONTEXT.md, docs/history/piggery-borrow/plan.md]
  sources: [docs/history/piggery-borrow/CONTEXT.md, docs/history/piggery-borrow/plan.md, .bee/cells/pbr-1.json, .bee/cells/pbr-2.json, .bee/cells/pbr-3.json, .bee/cells/pbr-4.json, .bee/cells/pbr-5.json, .bee/cells/pbr-6.json, .bee/cells/pbr-7.json]
---

# piggery-borrow — Delivery

## What shipped

- **pbr-1** — Aborted or errored leader turns requeue and hold their result claims until the next user input; completed or outcome-less turns delete; drain skips while a UI prompt is open (4 file(s) changed)
- **pbr-2** — Belt state, claim sets, prompt depths and settle outcomes sit on globalThis[Symbol.for("bee.pi.state")]; the Paseo leader keeps its id in the slot, strips it from process.env, and every belt bee/paseo spawn gets it back via beltEnv() (11 file(s) changed)
- **pbr-3** — Belts send belt_contract at session-init; the binary returns one fix line on mismatch that pi notifies and opencode toasts; steer spawn gets beltEnv() (8 file(s) changed)
- **pbr-4** — herding cancel kills the worker process tree on unix (TERM, 2 s, KILL) behind a ProcessTree seam; help text updated (2 file(s) changed)
- **pbr-5** — herding status reports ctx_tokens and turns per job from the transcript_path the activity hook now records (4 file(s) changed)
- **pbr-6** — herding.limits refused at the door; --explain; rework keeps help and CLI-shape tests green (8 file(s) changed)
- **pbr-7** — bee doctor prints one advisory row per runtime with a fix line; verdict untouched; BELT_CONTRACT shared const added (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pbr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts && .bee/bin/bee dev release-manifest --check` — the cell verify; 118 passed in the pi belt contract suite, manifest 419 files match; the rest of the workspace suite was not run
- **pbr-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test pi_paseo_heartbeat_contracts --test pi_worker_guard_contracts && .bee/bin/bee dev release-manifest --check` — cell verify run verbatim (PATH=$HOME/.cargo/bin form for the worktree guard): 119+26+7 passed, manifest 419 files match; the three new tests were red before the fix
- **pbr-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test opencode_plugin_contracts && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee hooks::session_init && .bee/bin/bee dev release-manifest --check` — cell verify run with PATH=$HOME/.cargo/bin (pi 121 ok, opencode 9 ok, session_init 23 ok, manifest 419 match); plus --bin bee doctor 62 ok because doctor.rs changed; full suite not run
- **pbr-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee herding::job_verbs` — cell verify (run with CARGO_HOME unset so PATH=$HOME/.cargo/bin, the same expansion), 20 passed incl. kill_tree_stops_a_detached_setsid_child (real ps + signals) and kill_tree_terms_then_kills_descen…
- **pbr-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- herding:: hooks::activity devtools::statusline` — the cell verify over the three touched modules, 756 passed; full suite not run
- **pbr-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- verbs::drivers::prepare herding::run herding::wave` — cell filter plus verbs::help hooks::cli_shape catalog, 498 passed after the rework; the full suite had found two reds (help text --json, frozen plan span) that the rework fixed
- **pbr-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee doctor` — the cell's verify, filtered to doctor tests (62 passed, 4103 filtered out); I also ran the rebuilt binary with bee doctor --runtime claude and --runtime pi in this worktree and saw all five runtime r…

## Deviations

- **pbr-1** — The hold clears only on an input event whose source is not extension — pi 1.0.0 sendUserMessage emits input with source extension, so a belt nudge would otherwise clear the hold — the plan was wrong about a fact
- **pbr-1** — Reserved and regenerated docs/history/codex-harness-hardening/release-manifest.json (two hashes) — leader note assigned the regen to this cell — something else had to be fixed first
- **pbr-1** — heldClaims is also cleared at startResultDrain beside inFlightClaims — a session boundary resets every drain latch — found a better route
- **pbr-2** — The slot also holds a worker's PASEO_AGENT_ID (paseoAgentId), but only the leader deletes it from process.env — the worker readers at events.ts:80 and index.ts:141 must read the slot too, and they need the worker's id — the plan was wrong about a fact
- **pbr-2** — events.ts spawns of session-close, the heartbeat broker tick and the paseo heartbeat runners also pass beltEnv(), beyond the four files D5 names — D5 says every belt spawn of bee or paseo, and those are such spawns — found a better route
- **pbr-2** — tool-steer.ts:102 (bee herding steer, leader-only) still spawns with process.env and so gets no PASEO_AGENT_ID; it is outside this cell's files and is left as a scope question for the orchestrator — something else had to be fixed first
- **pbr-2** — docs/knowledge/work/pi-result-mailbox/delivery.md (affects_specs) is not edited here; it is not in the cell files and is left for the scribe step — something else had to be fixed first
- **pbr-3** — Made doctor.rs belt_contract_check pub(crate) and take Option<u32> instead of the source list, and reserved doctor.rs — the binary_freshness remedy strings are private to doctor.rs and copying them would drift — found a better route
- **pbr-3** — hooks.ts unchanged: runAdvisoryHook already passes the payload through, so belt_contract rides the payload from events.ts — found a better route
- **pbr-4** — edited packages/bee-rs/crates/bee/src/generated/registry_payload.json (reserved first) — the cancel help text lives only in that hand-maintained registry payload, not in job_verbs.rs — the plan was wrong about a fact
- **pbr-4** — the setsid test is cfg(target_os = linux) instead of cfg(unix) — macOS ships no setsid command and CI tests run on ubuntu — hit an unforeseen obstacle
- **pbr-4** — moved cancel_with_transport and cancel_with_transport_and_timeout into the test module and wired them to NoopTree — they had no production callers, so the no-op seam cannot leak into the real path — found a better route
- **pbr-4** — red-first shown as a compile red (test written against kill_tree before it existed), not a runtime red against the old path — the old path never signals the tree, so the setsid sleep survives by construction — something else had to be fixed first
- **pbr-4** — sync-ack: plan sets affects_skills to none for pbr-4; the bee-herding skill does not describe cancel internals, and the affected spec docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md is the scribe step
- **pbr-5** — made devtools/mod.rs declare statusline pub(crate) — the parser cannot be shared without it and the cell said expose it rather than copy it — the plan was wrong about a fact
- **pbr-5** — the knowledge doc docs/knowledge/areas/bee-herding/waves-and-occupancy.md (affects_specs) is not updated — it is outside the cell files; left for scribe — the plan was wrong about a fact
- **pbr-5** — sync-ack: cell affects_skills is empty; the new status fields are additive JSON and a plain suffix, the area doc sync is left to scribe
- **pbr-6** — leader reworked the cell inline after the full suite went red: help text says JSON instead of --json, and the frozen plan span is pinned in KNOWN_HISTORICAL_EXCEPTIONS (13 to 14) — the worker ran only the cell filter, which did not reach verbs::help or hooks::cli_shape — something else had to be fixed first
- **pbr-6** — sync-ack: no owned skill describes herding.limits; the knowledge doc waves-and-occupancy.md carries it
- **pbr-7** — The paseo row checks for a paseo agent under herding.agents (or herding.paseo) and its fix line says to add one to .bee/config.json, not bee onboard --apply — onboard never writes paseo config and this repo configures paseo per agent, so bee onboard --apply would be a wrong fix — the plan was wrong about a fact
- **pbr-7** — The opencode row counts the plugin as current when the file is present, then checks the belt contract — bee embeds no opencode plugin text to compare bytes against — the plan was wrong about a fact
- **pbr-7** — I moved the wiring check into wiring_row() and the PATH lookup into locate_on_path(), and split doctor_report() out of run_doctor() — the runtime rows reuse those checks without starting the paseo or bee binaries, and the verdict test can call doctor_report() directly — found a better route
- **pbr-7** — I wrote the code before the tests, so I did not see the tests fail first — I edited doctor.rs first, then wrote all five tests — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work piggery-borrow` from 7 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/piggery-borrow/CONTEXT.md`, `docs/history/piggery-borrow/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
