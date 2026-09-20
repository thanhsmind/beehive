---
type: bee.delivery
title: pi-relocation-delivery — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-relocation-delivery: 4 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-09-16
bee:
  id: pi-relocation-delivery-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/pi-relocation-delivery/CONTEXT.md, docs/history/pi-relocation-delivery/plan.md]
  sources: [docs/history/pi-relocation-delivery/CONTEXT.md, docs/history/pi-relocation-delivery/plan.md, .bee/cells/archive/pi-relocation-delivery/prd-1.json, .bee/cells/archive/pi-relocation-delivery/prd-2.json, .bee/cells/archive/pi-relocation-delivery/prd-4.json, .bee/cells/archive/pi-relocation-delivery/prd-5.json]
---

# pi-relocation-delivery — Delivery

## What shipped

- **prd-1** — Add the cells rebind-session verb over the cells-module claim rewriter (4 file(s) changed)
- **prd-2** — Carry the inbox token and rebind the claims across a Pi relocation, proven by a red-first contract case (3 file(s) changed)
- **prd-4** — Pi advisor transport block and the complete seven-row Pi spawn-mechanics table, with dev regen re-run (3 file(s) changed)
- **prd-5** — Run every relocation rebind from the main checkout, proven by a case that failed on the claim before the fix (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **prd-1** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee rebind` — the four new handler tests cover the scoped rewrite, the exact +1 fence epoch, the clean no-op and flag validation; leader re-ran the approved command and saw 4 passed, 0 failed
- **prd-2** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts && .bee/bin/bee dev release-manifest --check && .bee/bin/bee doctor --runtime pi --json` — leader re-ran the approved chain: 68 contract tests passed including the new relocation-with-a-job-in-flight case, manifest 376 files match, and doctor --runtime pi reported overall ready with wiring…
- **prd-4** — `rg -n 'bee:only pi' skills/bee-swarming/references/worker-details.md && rg -c 'Follow-up|Harness assist|Isolation guarantee|Subagent type' skills/bee-swarming/references/swarming-reference.md && .bee/bin/bee dev release-manifest --check` — docs-only cell: the pointer and parity checks plus the release-manifest check the regen obligation owes (376 files match)
- **prd-5** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts && .bee/bin/bee dev release-manifest --check && .bee/bin/bee doctor --runtime pi --json` — leader watched the case fail first on the claim (left sess-old-reloc, right sess-new-reloc), then re-ran the approved chain: 68 passed 0 failed, manifest 376 files match, doctor --runtime pi ready wi…

## Deviations

- **prd-1** — followed the plan
- **prd-1** — sync-ack: No skill text changes with this cell: cells rebind-session is internal plumbing the Pi belt calls in prd-2, and its one user-facing mention is the failure-path warning prd-2 adds, so the skill and knowledge wording lands with prd-2 and the capture step. The 244 changed lines carry their own tests inside handlers_write.rs (four new cases, re-run green by the leader), which the advisory's test-shaped-path heuristic does not see.
- **prd-2** — Rebuilt the release binary and refreshed the vendored .bee/bin/bee beyond the cell's file list — something else had to be fixed first: doctor's wiring_matches_binary compares the installed belt against the bytes the binary embeds, so the proof the cell owes cannot be produced without it
- **prd-4** — followed the plan
- **prd-4** — sync-ack: affects_skills predicted skills/bee-swarming/SKILL.md; the cell touched that skill's two reference files instead (worker-details.md, swarming-reference.md). Leader's prediction error at cell drafting, not a scope change — the touched files are exactly the ones the approved action names.
- **prd-5** — Taught the fixture's stub bee to model `cells rebind-session`, refusal included — why: the stub delegated cells verbs to the real binary, which refused under the fixture's worktree cwd, so the rebound-claim assertion rested on a call that rewrote nothing — the plan was wrong about a fact
- **prd-5** — Forced a rebuild by touching doctor.rs and re-vendored the newer artifact — why: the first vendored binary predated the belt fix, so doctor reported wiring_matches_binary not_ok — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work pi-relocation-delivery` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-relocation-delivery/CONTEXT.md`, `docs/history/pi-relocation-delivery/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
