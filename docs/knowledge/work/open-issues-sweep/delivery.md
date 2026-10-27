---
type: bee.delivery
title: open-issues-sweep — delivery
description: "Delivery record proposed by bee knowledge promote for work item open-issues-sweep: 11 capped cell(s), 11 recorded deviation(s)."
timestamp: 2026-10-18
bee:
  id: open-issues-sweep-delivery
  lifecycle: active
  required_context: [docs/history/open-issues-sweep/CONTEXT.md, docs/history/open-issues-sweep/plan.md]
  sources: [docs/history/open-issues-sweep/CONTEXT.md, docs/history/open-issues-sweep/plan.md, .bee/cells/ois-1.json, .bee/cells/ois-2.json, .bee/cells/ois-3.json, .bee/cells/ois-4.json, .bee/cells/ois-5.json, .bee/cells/ois-6.json, .bee/cells/ois-7.json, .bee/cells/ois-8.json, .bee/cells/ois-9.json, .bee/cells/ois-10.json, .bee/cells/ois-11.json]
---

# open-issues-sweep — Delivery

## What shipped

- **ois-1** — Confirmed the rescue fix (escalate check reads orig_role), removed the stray doc comment, proved the new v2-plan test fails without the fix and passes with it, verify command green (69 prepare tests), committed 4f16ce195 path-scoped with trailer cell: ois-1. (3 file(s) changed)
- **ois-2** — Stale claim gate is now broken on age/dead-pid/unparseable with a rename-aside race guard, release retries, so unclaim unblocks and reclaims; 4 new tests red-first, cells:: 392 green, commit 98da2c18b. (2 file(s) changed)
- **ois-3** — Checked and completed the inherited edits for ois-3, removed two forbidden /// comments and restored a lost #[test] attribute, confirmed red-first for all three fixes, verify green, committed bebce8934. (3 file(s) changed)
- **ois-4** — decisions_prelude_at and backlog's preamble_at now resolve through resolve_root_serving_granted, so both use the main checkout's store from a granted worktree; tests inverted/added and red-first proved; committed 7983338d3. (4 file(s) changed)
- **ois-5** — triggers add refuses a decision id the store does not hold (1 file(s) changed)
- **ois-6** — plan-freeze fallback now reads the control-root state.json (falling back to the handed map only when that file is unusable or names another feature), with two new tests; commit f45f90a63. (2 file(s) changed)
- **ois-7** — doc_deferral_scan_files now drops a shape-approved feature's plan.md and any file starting with KNOWLEDGE_INDEX_HEADER (with or without leading frontmatter); two red-first tests added; commit 32773ca3c (2 file(s) changed)
- **ois-8** — promote reminder clear command (8 file(s) changed)
- **ois-9** — onboard projection guard (3 file(s) changed)
- **ois-10** — cap report from a file (4 file(s) changed)
- **ois-11** — herded worker git guard (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ois-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee prepare` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee prepare — green:unit — every prepare-matching test (69 passed, 0 failed), which inc…
- **ois-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee cells::` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee cells:: — green:unit — 392 passed, 0 failed (incl. the 4 new gate tests); red-first…
- **ois-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding:: — green:unit — 772 passed, 0 failed (33.93s); herding::tests::child_env_k…
- **ois-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee decisions && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee backlog` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee decisions && ... -p bee backlog — green:unit — the cell's own two verify filters; 1…
- **ois-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee triggers` — 20 passed, 0 failed in the triggers module, incl. add_refuses_a_decision_id_the_store_does_not_know, red before the fix
- **ois-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee plan_freeze && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee plan_md` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee plan_freeze && ... -p bee plan_md — green:unit — 6 plan_freeze tests pass (2 new) a…
- **ois-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee doc_deferral` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee doc_deferral — green:unit — 26 passed, 0 failed (full doc_deferral filter incl. the…
- **ois-8** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee promote && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee deferred_queue && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee scribing` — promote 10, deferred_queue 11, scribing 20 passed, 0 failed (worker job-1792290701295-813243-1); also driven live against the built binary: a pre-queue feature clears and bee orient drops the line
- **ois-9** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee onboard` — onboard 216 passed, 0 failed (jobs job-1792288859066-594914-1 and job-1792292402764-1034186-1); red-first via a stubbed verify_host_edit; comment baseline no longer names onboard files
- **ois-10** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee cells:: && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee distinct_flag_vocabulary && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts --test registry_dispatch` — cells:: 395, distinct_flag_vocabulary 1, registry_contracts 12 + registry_dispatch 9 passed, 0 failed (jobs job-1792291672906-922066-1 and job-1792292405489-1034743-1); red-first report-file tests
- **ois-11** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee write_guard && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee hooks::` — write_guard 290 and hooks:: 722 passed, 0 failed (worker job-1792290734994-817991-1); herded_worker_guard_deny_reaches_the_real_hook_entry_point failed with the routing fix reverted; leader full-suit…

## Deviations

- **ois-2** — sync-ack: claim-gate internals only; no skill states the gate's stale rule
- **ois-3** — sync-ack: herding env and session release internals; the Paseo channel concept is synced at capture
- **ois-4** — sync-ack: store routing only; the control-plane topology concept is synced at capture
- **ois-6** — sync-ack: guard fallback source only; the gates concept is synced at capture
- **ois-7** — sync-ack: close door scan-set internals; no skill states the doc-deferral scan set
- **ois-8** — sync-ack: orient and preamble reminder text plus queue verb; the close concept is synced at capture
- **ois-9** — follow-up commit 755483bd4 removed three comment lines the first commit added
- **ois-9** — sync-ack: onboard verify projection internals; no skill states the projection refresh rule
- **ois-10** — follow-up commit c1b94a987 removed the pin comment the plan action asked for: the comment baseline forbids it
- **ois-10** — sync-ack: cap door flag; the cells-completion concept is synced at capture
- **ois-11** — sync-ack: hook routing under the herding marker; the Paseo channel concept is synced at capture

## Provenance

Proposed by `bee knowledge promote --work open-issues-sweep` from 11 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/open-issues-sweep/CONTEXT.md`, `docs/history/open-issues-sweep/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
