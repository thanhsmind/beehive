---
type: bee.delivery
title: herding-prompt-path — delivery
description: "Delivery record for work item herding-prompt-path: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: herding-prompt-path-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/herding-prompt-path.json, docs/history/herding-prompt-path/promote-proposals.md]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/herding-prompt-path.json, docs/history/herding-prompt-path/promote-proposals.md, .bee/cells/archive/herding-prompt-path/hpp-1.json]
---

# herding-prompt-path — Delivery

## What shipped

- **hpp-1** — `read_prompt_file` searches five skill roots in order (`skills`, `.claude/skills`, `.agents/skills`, `.opencode/skills`, `.codex/skills`), `skills` first, and names every path tried on failure (1 file(s) changed)

## Verify

- **hpp-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml control_loop` — green.

## Deviations

- **hpp-1** — followed the plan.
- **hpp-1** — sync-ack: the bee-herding skill text is unchanged by design — the prompt still lives at `bee-herding/references/<role>-prompt.md`; only the root bee searches under widens from `skills/` to the installed runtime prefixes. The cell declares `affects_skills: []`.

## Provenance

Mined from 1 capped cell trace in `.bee/cells/`. Already reflected in `docs/knowledge/areas/bee-herding/overview.md` (the `read_prompt_file` candidate-root walk is documented there) — this record adds no further area edit.
