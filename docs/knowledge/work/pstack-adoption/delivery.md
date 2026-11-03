---
type: bee.delivery
title: pstack-adoption — delivery
description: "Delivery record proposed by bee knowledge promote for work item pstack-adoption: 6 capped cell(s), 11 recorded deviation(s)."
timestamp: 2026-09-01
bee:
  id: pstack-adoption-delivery
  lifecycle: active
  required_context: [docs/history/pstack-adoption/CONTEXT.md, docs/history/pstack-adoption/plan.md]
  sources: [docs/history/pstack-adoption/CONTEXT.md, docs/history/pstack-adoption/plan.md, .bee/cells/archive/pstack-adoption/psa-1.json, .bee/cells/archive/pstack-adoption/psa-2.json, .bee/cells/archive/pstack-adoption/psa-3.json, .bee/cells/archive/pstack-adoption/psa-4.json, .bee/cells/archive/pstack-adoption/psa-5.json, .bee/cells/archive/pstack-adoption/psa-6.json]
---

# pstack-adoption — Delivery

## What shipped

- **psa-1** — perf added as the eighth route class value, with its pinned refusal updated and a happy-path test (2 file(s) changed)
- **psa-2** — New pure-filesystem fence pins the route-class vocabulary across all four source documents and freezes the class/lane collision set, reading both lists out of workflows.rs as text (1 file(s) changed)
- **psa-3** — All four documents naming the route class list perf as the eighth value, and the migration note D2 requires records the older-bee degradation (6 file(s) changed)
- **psa-4** — Added the ## Class playbooks section to planning-reference.md and one-line pointers in bee-planning/SKILL.md and scout-and-ticks.md (4 file(s) changed)
- **psa-5** — Key 2 of the herding lane-safety filter now refuses a candidate whose CoS names no command, path, or state an agent can evaluate (8 file(s) changed)
- **psa-6** — A review report now shows the findings it dismissed and why, and the reviewer is told to record each drop with its reason (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **psa-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast -p bee --manifest-path packages/bee-rs/Cargo.toml`
- **psa-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast -p bee --manifest-path packages/bee-rs/Cargo.toml`
- **psa-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check`
- **psa-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check`
- **psa-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check`
- **psa-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check && .bee/bin/bee onboard --repo-root . --json`

## Deviations

- **psa-1** — placed the safety-argument comment BELOW both consts instead of above them — close.rs:393-403 and uat.rs:139-171 cite ROUTE_CLASS_VALUES / ROUTE_LANE_VALUES by line number (287-288 / 289-290), and a comment above would have shifted every one of those citations — found a better route
- **psa-1** — sync-ack: the class enum change ships without a skill edit by cell design: psa-3 owns the four skill and doc sites that name the class values, and psa-2 writes the parity test that pins them; editing a skill here would collide with a sibling cell
- **psa-2** — Assertion 2 is implemented as no class value is a lane value BEYOND the pair workflows.rs itself grandfathers, with that pair parsed out of the source safety comment — taken literally the assertion is already false, because docs and spike sit in both vocabularies today, so a literal version could never pass as the acceptance requires — the plan was wrong about a fact
- **psa-2** — All FOUR documented sites list seven values, not three of four — scout-and-ticks.md:34 is stale too, matching the dispatch STATE OF THE TREE note rather than the acceptance wording — the plan was wrong about a fact
- **psa-3** — Updated all FOUR class sites, scout-and-ticks.md included, because the dispatch correction from psa-2 reported that site stale too and the fence confirmed it — the plan was wrong about a fact
- **psa-3** — Widened the Expected wording of row PLAN-03 in docs/product-description/verification/lifecycle.md to say the refusal names all eight values, ending in perf, because that row is an executable verification row and its expect text had to stay coherent with the enum it now lists — something else had to be fixed first
- **psa-3** — Put the migration note at docs/history/pstack-adoption/migration-note.md because this repo has NO dedicated home for contract-change notes (no CHANGELOG, no docs/migrations/, no upgrade guide); the nearest existing practice is a migration note inside the causing feature history folder, per docs/history/multisession-native/plan.md:64 — hit an unforeseen obstacle
- **psa-3** — The commit also carries bee dev regen output the cell files list does not name: the five rendered skill trees (.agents, .claude, .claude-plugin, .codex-plugin, .opencode) and a timestamp-only bump in .bee/onboarding.json, because the cell ordered the regen run and its acceptance requires that output committed — followed the plan
- **psa-4** — followed the plan
- **psa-5** — followed the plan
- **psa-6** — Also edited expertise/review.md, a path the cell did not name — the named .bee/expertise/review.md is an onboarding-installed copy that bee onboard --apply overwrote from that source — hit an unforeseen obstacle

## Provenance

Proposed by `bee knowledge promote --work pstack-adoption` from 6 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pstack-adoption/CONTEXT.md`, `docs/history/pstack-adoption/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
