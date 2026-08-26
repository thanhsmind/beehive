---
type: bee.delivery
title: models-show-verb — delivery
description: "Delivery record proposed by bee knowledge promote for work item models-show-verb: 3 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-08-26
bee:
  id: models-show-verb-delivery
  lifecycle: active
  required_context: [docs/history/models-show-verb/CONTEXT.md, docs/history/models-show-verb/plan.md]
  sources: [docs/history/models-show-verb/CONTEXT.md, docs/history/models-show-verb/plan.md, .bee/cells/archive/models-show-verb/archive/models-show-verb/ms-1.json, .bee/cells/archive/models-show-verb/archive/models-show-verb/ms-2.json, .bee/cells/archive/models-show-verb/archive/models-show-verb/ms-3.json]
---

# models-show-verb — Delivery

## What shipped

- **ms-1** — bee models show prints the raw models role table, descriptions intact, each row marked configured or default (5 file(s) changed)
- **ms-2** — bee status --json models section merges each raw slot description onto the normalized slot for display; resolution keeps the stripped map (2 file(s) changed)
- **ms-3** — Fresh installs seed a described role table; both role doors send the author to bee models show (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ms-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee models_group — green with the new tests named in the output; bee dev regen green; bee dev release-manifest --check green`
- **ms-2** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee status_full — green with the new tests named in the output`
- **ms-3** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee onboard cells — green with updated seed and refusal tests named in the output; bee dev regen green`

## Deviations

- **ms-1** — registry_payload.json was hand-edited instead of regenerated — the repo has no generator for it (bee dev regen renders skill trees, applies onboarding, writes the release manifest, and never touches the payload); every earlier verb addition declared its entry by hand the same way, and registry_contracts + registry_dispatch are the drift gate — the plan was wrong about a fact
- **ms-1** — bee dev regen also deleted three retired .claude/agents/*.md files and bumped .bee/onboarding.json; I restored them and kept them out of the commit — unrelated repo-vs-config drift, and a sibling worker shares this worktree — something else had to be fixed first
- **ms-2** — Waited on a background build poll before the verify run — the sibling worker had verbs/models_group.rs mid-edit and the crate would not compile — hit an unforeseen obstacle
- **ms-2** — Test expectation for a whitespace-only description was wrong: a raw {model, description:" "} slot normalizes to the OBJECT {model}, not to a bare string, so it asserts {model:"haiku"} and a separate undescribed string slot covers the no-widen case — the plan was wrong about a fact
- **ms-3** — Also edited packages/bee-rs/crates/bee/src/verbs/cells/tests.rs (reserved first) — two tests pin the missing-role refusal string verbatim, so changing the refusal without them is a guaranteed red — something else had to be fixed first
- **ms-3** — Compared resolve_role answers instead of the whole normalized map — the {model} leaf is not byte-equal to the bare string it replaces, so map equality would fail on a shape change that resolution is blind to — the plan was wrong about a fact
- **ms-3** — sync-ack: The cell declares affects_skills [] on purpose: this changes the onboarding seed and two CLI strings (the missing-role refusal, cells add --help), and both now teach bee models show at the point of use. No bee-planning/swarming/reviewing/capturing text states the seed shape or quotes either string, so there is nothing there to keep in sync.

## Provenance

Proposed by `bee knowledge promote --work models-show-verb` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/models-show-verb/CONTEXT.md`, `docs/history/models-show-verb/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

