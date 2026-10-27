---
type: bee.delivery
title: pstack-0-15-port — delivery
description: "Delivery record proposed by bee knowledge promote for work item pstack-0-15-port: 3 capped cell(s), 4 recorded deviation(s)."
timestamp: 2026-10-16
bee:
  id: pstack-0-15-port-delivery
  lifecycle: active
  areas: [doctrine-layer, workflow-state, worktree-parallelism, human-mailbox]
  required_context: [docs/history/pstack-0-15-port/CONTEXT.md, docs/history/pstack-0-15-port/plan.md]
  sources: [docs/history/pstack-0-15-port/CONTEXT.md, docs/history/pstack-0-15-port/plan.md, .bee/cells/p15-1.json, .bee/cells/p15-2.json, .bee/cells/p15-3.json]
---

# pstack-0-15-port — Delivery

## What shipped

- **p15-1** — D2 landed: bee worktree merge reports proof_carried (JSON + printed text) for a cap whose patch-id survives a rebase, keeps proof_stale for a plain merge, and never refuses; new rebase test seen red then green (commit c7daa1bb8). (5 file(s) changed)
- **p15-2** — Seven reason-less spoken lines in the rule index got an em-dash reason placed last; 36 reason-less playbook steps got a because/so that/em-dash clause; bugfix.md got the D5 always-on-signal sentence; regen copies and release manifest committed (11 file(s) changed)
- **p15-3** — Judge-tier table gains the narrowed re-verify row; Interview craft opens with goal playback; human-mailbox overview owns the twice rule with one-sentence pointers from bee-evolving and bee-capturing (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **p15-1** — ``PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee verbs::worktree::tests` is green, AND the recorded red run of the new rebase test failed before the fix. Both outputs are in the cap evidence. `.bee/bin/bee dev comment-baseline --check` is clean. `rg -n proof_carried docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md` returns at least one line.` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee verbs::worktree::tests — green:unit — 199 passed, 0 failed (whole crate: 4306 passe…
- **p15-2** — ``rg -n '^  spoken:' docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md | rg -v '—' | wc -l` prints 0. `rg -n '^[0-9]+\.' skills/bee-planning/playbooks/*.md | rg -v '—|because|so that' | wc -l` prints 0 or names only steps that point at a home (list them on the cap). `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity` is green. `.bee/bin/bee knowledge check --json` reports no new duplicate_rule_home or unknown_rule_ref finding. `rg -c 'always-on signal' skills/bee-planning/playbooks/bugfix.md` prints 1. `.bee/bin/bee dev regen` runs clean and `.bee/bin/bee dev release-manifest --check` is clean.` — rule index and playbooks edited: spoken rg prints 0; playbook rg lists only bugfix.md step 2 (points at worker-details.md); rule_index_parity 3 passed; knowledge check has no duplicate_rule_home or u…
- **p15-3** — ``rg -n 'under 30 changed lines' skills/bee-hive/references/gates-and-delegation.md` returns one line inside the judge-tier table and `rg -c 'lines since cap' skills/bee-hive/references/gates-and-delegation.md` prints 1. `rg -n 'Play the goal back first' skills/bee-shaping/references/shaping-reference.md` returns one line above `One question per turn`. `rg -n 'A mistake seen twice' docs/knowledge/areas/human-mailbox/overview.md` returns one heading line. `rg -c 'seen twice' skills/bee-evolving/SKILL.md skills/bee-capturing/SKILL.md` prints 1 for each file. `.bee/bin/bee knowledge check --json` reports no new finding. `.bee/bin/bee dev regen` runs clean and `.bee/bin/bee dev release-manifest --check` is clean.` — docs-only doctrine edits; every rg count matched, regen 3/3 green, manifest 419 files match, knowledge check has 0 errors and only the pre-existing not_canonical frontmatter warnings

## Deviations

- **p15-2** — Six of the seven touched spoken lines stay over 140 characters (139 to 185) — the base text of most is already 115 to 148 characters, so appending alone cannot meet the bound; I shortened the reasons and left the rule text as is — the plan was wrong about a fact
- **p15-2** — Steps whose reason sat on a continuation line (docs 2, release 2 and 4, refactor 7, research 5, spike 5, content 1 and 3) were reworded so the reason clause starts on the numbered line the verify rg reads — found a better route
- **p15-2** — Left .bee/onboarding.json (regen timestamp only) and packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs (a sibling worker diff) out of the path-scoped commit — they are not this cell's files — hit an unforeseen obstacle
- **p15-3** — pointer sentences lead with repeated mistake / repeated reflection instead of seen twice — the heading name already carries seen twice and verify wants a count of 1 per file — the plan was wrong about a fact

## Provenance

Proposed by `bee knowledge promote --work pstack-0-15-port` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pstack-0-15-port/CONTEXT.md`, `docs/history/pstack-0-15-port/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
