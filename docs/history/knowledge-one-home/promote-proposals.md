promote proposal for work item "knowledge-one-home" (work/knowledge-one-home/work-item.md) — 11 capped cell(s): koh-1, koh-2, koh-3, koh-4, koh-5, koh-6, koh-7, koh-8, koh-9, koh-10, koh-12
anchor: work-item — docs/knowledge/work/knowledge-one-home/work-item.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/knowledge-one-home/delivery.md

---
type: bee.delivery
title: "knowledge-one-home — one home per rule, and the two doors that hold it — delivery"
description: "Delivery record proposed by bee knowledge promote for work item knowledge-one-home: 11 capped cell(s), 19 recorded deviation(s)."
tags: [knowledge, rule-homes, ownership, cap-door, gate-door, standard]
timestamp: 2026-08-22
bee:
  id: knowledge-one-home-delivery
  lifecycle: active
  areas: [okf-profile, workflow-state, decision-memory, doctrine-layer]
  required_context: [work/knowledge-one-home/work-item.md]
  decisions: [D1, D2, D3, D4, D5]
  sources: [docs/knowledge/work/knowledge-one-home/work-item.md, .bee/cells/koh-1.json, .bee/cells/koh-2.json, .bee/cells/koh-3.json, .bee/cells/koh-4.json, .bee/cells/koh-5.json, .bee/cells/koh-6.json, .bee/cells/koh-7.json, .bee/cells/koh-8.json, .bee/cells/koh-9.json, .bee/cells/koh-10.json, .bee/cells/koh-12.json]
  lane: standard
---

# knowledge-one-home — one home per rule, and the two doors that hold it — Delivery

## What shipped

- **koh-1** — Accept and emit owns.* and applied_at frontmatter keys under bee: block (2 file(s) changed)
- **koh-2** — Grade owns, applied_at, and rule markers in knowledge check (7 file(s) changed)
- **koh-3** — Write ownership maps for all fifteen areas (20 file(s) changed)
- **koh-4** — Home three inventoried rules with ids and applied_at; pointer copies at every other site; skill trees re-rendered (14 file(s) changed)
- **koh-5** — Require affects_skills and affects_specs on every cell (6 file(s) changed)
- **koh-6** — Sync door (D3/D4): cap-time ownership/applied_at/prediction checks refuse, with --sync-ack escape and legacy-cell skip; docs and regen chain synced (11 file(s) changed)
- **koh-7** — Load rule homes and print update obligations from decisions log (7 file(s) changed)
- **koh-8** — state plan-conflicts derive/verdict land D5's plan-time conflict check on the workflow record (8 file(s) changed)
- **koh-9** — Merged/execution gate refuses a lane whose conflict review is absent, stale by plan_rev, or unverdicted; a conflicts verdict is surfaced, not refused (6 file(s) changed)
- **koh-10** — Rule markers and refs now scan code-stripped text; retired boundary-test help rewritten in the registry payload (4 file(s) changed)
- **koh-12** — Nine discipline rules homed in the operating block with ids, applied_at records, and one-line pointers everywhere else (24 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **koh-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee knowledge`
- **koh-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee knowledge && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts`
- **koh-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo run --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- knowledge check && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo run --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- knowledge index --check`
- **koh-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo run --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- knowledge check && .bee/bin/bee dev release-manifest --check`
- **koh-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee cells && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts && .bee/bin/bee dev release-manifest --check`
- **koh-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee cells && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts && .bee/bin/bee dev release-manifest --check`
- **koh-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee knowledge && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee decisions && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts`
- **koh-8** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee state_group && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts`
- **koh-9** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee state_group && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts && .bee/bin/bee dev release-manifest --check`
- **koh-10** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee knowledge && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts`
- **koh-12** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo run --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- knowledge check && .bee/bin/bee dev release-manifest --check`

## Deviations

- **koh-1** — worker commit lacked the cell trailer; orchestrator amended it in place
- **koh-3** — worker did not run finish; orchestrator capped from its result form
- **koh-4** — worker pane idled out after finishing the edits and the regen chain; orchestrator committed its tree and capped from inspection
- **koh-5** — worker did not run finish; orchestrator capped from its result form
- **koh-6** — fixed a pre-existing KNOWN RED: the legacy-cell deviation line was unconditional and broke 9 existing deviation-pipeline tests; scoped it to fire only when the touched set actually carries a skills/** path
- **koh-7** — worker did not run finish; orchestrator capped from its result form
- **koh-8** — Term-set normalization: terms are lowercased, punctuation-trimmed, filtered to length >= 4 and a 24-word stop list before scoring. Without it a title s the/a alone reach count_term_hits >= 2 against nearly every decision, so 0 conflicts becomes unreachable. The scorers themselves are untouched.
- **koh-8** — gates.md rules numbered R136/R137 (next free); the spec Pointers bullet records the hand-edited registry payload per decision 3358743e.
- **koh-9** — Tests authored in set_gate.rs own #[cfg(test)] module beside the advisor-precondition cases the cell cited as the pattern, not state_group/tests.rs — every gate test lives there and run_gate_body is private to that file
- **koh-9** — conflict_review_refusal reads the LIVE WORKFLOW RECORD (root + lane) instead of the passed lane record the cell sketched: koh-8 writes conflict_review to the workflow record and the lane projection never copies it down
- **koh-9** — catalog.rs PINNED_FLAG_COUNT stays 180 — koh-9 adds a refusal and an output field, no flag name; the audit comment says so explicitly
- **koh-9** — Regen output trees (.agents/.claude/.claude-plugin/.codex-plugin/.opencode skills + .bee/onboarding.json) reserved and committed beside the cell files — skills/ is a manifest root
- **koh-10** — Extended the payload fix past cells cap/finish to cells reopen and state handoff write: both carried the same retired claim that a cap or the finish door runs commands.test
- **koh-10** — Left docs/specs/test-simple.md and the two stale comments in verbs/cells/tests.rs alone — outside the cell files, exempt tree / comment-only
- **koh-10** — Pre-existing and untouched: skills/bee-planning/SKILL.md line 79 spells (rule: R138, ...), which no marker homes, so bee knowledge check still reports one unknown_rule_ref; fixing a skills/ file would pull in the regen chain this cell forbids
- **koh-12** — packages/bee/AGENTS.block.md edited by hand: it is the SOURCE the regen chain renders AGENTS.md FROM, not the other way round — the first regen reverted the AGENTS.md-only edits; path reserved before the write, same shape koh-4 used
- **koh-12** — router-triage carries no bee.applied_at frontmatter key: applied_at_unlinked resolves targets against rules the concept body homes, and these ten are homed in AGENTS.md, so the key would fail the check — the AGENTS.md rule homes section holds the records, as koh-4 did
- **koh-12** — koh-11 handoff fixed: skills/bee-capturing/SKILL.md wrapped (rule: workflow-state-capture-skill-answer) across a newline AND left an unbalanced backtick on the line, both of which blank the ref for extract_rule_refs — rewritten on one balanced line
- **koh-12** — agents-one-next-action marks the whole Communication opening paragraph, which also carries the progress-tick rules: splitting the semicolon list would have reworded pinned prose

## Provenance

Proposed by `bee knowledge promote --work knowledge-one-home` from 11 capped cell trace(s) in `.bee/cells/` and the work item `docs/knowledge/work/knowledge-one-home/work-item.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the work item's bee.areas.

area okf-profile:
  - [koh-2] Grade owns, applied_at, and rule markers in knowledge check — touched docs/knowledge/areas/okf-profile/conformance-check.md (trace .bee/cells/koh-2.json)
  - [koh-10] Rule markers and refs now scan code-stripped text; retired boundary-test help rewritten in the registry payload — touched docs/knowledge/areas/okf-profile/conformance-check.md (trace .bee/cells/koh-10.json)

area workflow-state:
  - [koh-5] Require affects_skills and affects_specs on every cell — touched docs/knowledge/areas/workflow-state/cells-authoring-and-revision.md (trace .bee/cells/koh-5.json)
  - [koh-6] Sync door (D3/D4): cap-time ownership/applied_at/prediction checks refuse, with --sync-ack escape and legacy-cell skip; docs and regen chain synced — touched docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md (trace .bee/cells/koh-6.json)
  - [koh-8] state plan-conflicts derive/verdict land D5's plan-time conflict check on the workflow record — touched docs/knowledge/areas/workflow-state/gates.md (trace .bee/cells/koh-8.json)
  - [koh-9] Merged/execution gate refuses a lane whose conflict review is absent, stale by plan_rev, or unverdicted; a conflicts verdict is surfaced, not refused — touched docs/knowledge/areas/workflow-state/gates.md (trace .bee/cells/koh-9.json)

area decision-memory:
  - [koh-7] Load rule homes and print update obligations from decisions log — touched docs/knowledge/areas/decision-memory/overview.md (trace .bee/cells/koh-7.json)

area doctrine-layer:
  - [koh-5] Require affects_skills and affects_specs on every cell — touched docs/knowledge/areas/workflow-state/cells-authoring-and-revision.md (trace .bee/cells/koh-5.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell koh-1 — save as docs/knowledge/patterns/knowledge-one-home-koh-1-pitfall.md

---
type: bee.pattern
title: knowledge-one-home cell koh-1 — pitfall candidate
description: "Pitfall candidate mined from cell koh-1's capped trace: worker commit lacked the cell trailer; orchestrator amended it in place"
timestamp: 2026-08-22
bee:
  id: knowledge-one-home-koh-1-pitfall
  lifecycle: draft
  areas: [okf-profile, workflow-state, decision-memory, doctrine-layer]
  sources: [.bee/cells/koh-1.json]
  polarity: pitfall
---

# knowledge-one-home cell koh-1 — pitfall candidate

## What the cell did

Accept and emit owns.* and applied_at frontmatter keys under bee: block

## Recorded evidence (verbatim from .bee/cells/koh-1.json)

- **deviation** — worker commit lacked the cell trailer; orchestrator amended it in place

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell koh-3 — save as docs/knowledge/patterns/knowledge-one-home-koh-3-pitfall.md

---
type: bee.pattern
title: knowledge-one-home cell koh-3 — pitfall candidate
description: "Pitfall candidate mined from cell koh-3's capped trace: worker did not run finish; orchestrator capped from its result form"
timestamp: 2026-08-22
bee:
  id: knowledge-one-home-koh-3-pitfall
  lifecycle: draft
  areas: [okf-profile, workflow-state, decision-memory, doctrine-layer]
  sources: [.bee/cells/koh-3.json]
  polarity: pitfall
---

# knowledge-one-home cell koh-3 — pitfall candidate

## What the cell did

Write ownership maps for all fifteen areas

## Recorded evidence (verbatim from .bee/cells/koh-3.json)

- **deviation** — worker did not run finish; orchestrator capped from its result form

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell koh-4 — save as docs/knowledge/patterns/knowledge-one-home-koh-4-pitfall.md

---
type: bee.pattern
title: knowledge-one-home cell koh-4 — pitfall candidate
description: "Pitfall candidate mined from cell koh-4's capped trace: worker pane idled out after finishing the edits and the regen chain; orchestrator committed its tree and capped from inspection"
timestamp: 2026-08-22
bee:
  id: knowledge-one-home-koh-4-pitfall
  lifecycle: draft
  areas: [okf-profile, workflow-state, decision-memory, doctrine-layer]
  sources: [.bee/cells/koh-4.json]
  polarity: pitfall
---

# knowledge-one-home cell koh-4 — pitfall candidate

## What the cell did

Home three inventoried rules with ids and applied_at; pointer copies at every other site; skill trees re-rendered

## Recorded evidence (verbatim from .bee/cells/koh-4.json)

- **deviation** — worker pane idled out after finishing the edits and the regen chain; orchestrator committed its tree and capped from inspection

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell koh-5 — save as docs/knowledge/patterns/knowledge-one-home-koh-5-pitfall.md

---
type: bee.pattern
title: knowledge-one-home cell koh-5 — pitfall candidate
description: "Pitfall candidate mined from cell koh-5's capped trace: worker did not run finish; orchestrator capped from its result form"
timestamp: 2026-08-22
bee:
  id: knowledge-one-home-koh-5-pitfall
  lifecycle: draft
  areas: [okf-profile, workflow-state, decision-memory, doctrine-layer]
  sources: [.bee/cells/koh-5.json]
  polarity: pitfall
---

# knowledge-one-home cell koh-5 — pitfall candidate

## What the cell did

Require affects_skills and affects_specs on every cell

## Recorded evidence (verbatim from .bee/cells/koh-5.json)

- **deviation** — worker did not run finish; orchestrator capped from its result form

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell koh-6 — save as docs/knowledge/patterns/knowledge-one-home-koh-6-pitfall.md

---
type: bee.pattern
title: knowledge-one-home cell koh-6 — pitfall candidate
description: "Pitfall candidate mined from cell koh-6's capped trace: fixed a pre-existing KNOWN RED: the legacy-cell deviation line was unconditional and broke 9 existing deviation-pipeline tests; scoped it to fire only when the…"
timestamp: 2026-08-22
bee:
  id: knowledge-one-home-koh-6-pitfall
  lifecycle: draft
  areas: [okf-profile, workflow-state, decision-memory, doctrine-layer]
  sources: [.bee/cells/koh-6.json]
  polarity: pitfall
---

# knowledge-one-home cell koh-6 — pitfall candidate

## What the cell did

Sync door (D3/D4): cap-time ownership/applied_at/prediction checks refuse, with --sync-ack escape and legacy-cell skip; docs and regen chain synced

## Recorded evidence (verbatim from .bee/cells/koh-6.json)

- **deviation** — fixed a pre-existing KNOWN RED: the legacy-cell deviation line was unconditional and broke 9 existing deviation-pipeline tests; scoped it to fire only when the touched set actually carries a skills/** path

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell koh-7 — save as docs/knowledge/patterns/knowledge-one-home-koh-7-pitfall.md

---
type: bee.pattern
title: knowledge-one-home cell koh-7 — pitfall candidate
description: "Pitfall candidate mined from cell koh-7's capped trace: worker did not run finish; orchestrator capped from its result form"
timestamp: 2026-08-22
bee:
  id: knowledge-one-home-koh-7-pitfall
  lifecycle: draft
  areas: [okf-profile, workflow-state, decision-memory, doctrine-layer]
  sources: [.bee/cells/koh-7.json]
  polarity: pitfall
---

# knowledge-one-home cell koh-7 — pitfall candidate

## What the cell did

Load rule homes and print update obligations from decisions log

## Recorded evidence (verbatim from .bee/cells/koh-7.json)

- **deviation** — worker did not run finish; orchestrator capped from its result form

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell koh-8 — save as docs/knowledge/patterns/knowledge-one-home-koh-8-pitfall.md

---
type: bee.pattern
title: knowledge-one-home cell koh-8 — pitfall candidate
description: "Pitfall candidate mined from cell koh-8's capped trace: Term-set normalization: terms are lowercased, punctuation-trimmed, filtered to length >= 4 and a 24-word stop list before scoring. Without it a title s the/a a…"
timestamp: 2026-08-22
bee:
  id: knowledge-one-home-koh-8-pitfall
  lifecycle: draft
  areas: [okf-profile, workflow-state, decision-memory, doctrine-layer]
  sources: [.bee/cells/koh-8.json]
  polarity: pitfall
---

# knowledge-one-home cell koh-8 — pitfall candidate

## What the cell did

state plan-conflicts derive/verdict land D5's plan-time conflict check on the workflow record

## Recorded evidence (verbatim from .bee/cells/koh-8.json)

- **deviation** — Term-set normalization: terms are lowercased, punctuation-trimmed, filtered to length >= 4 and a 24-word stop list before scoring. Without it a title s the/a alone reach count_term_hits >= 2 against nearly every decision, so 0 conflicts becomes unreachable. The scorers themselves are untouched.
- **deviation** — gates.md rules numbered R136/R137 (next free); the spec Pointers bullet records the hand-edited registry payload per decision 3358743e.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell koh-9 — save as docs/knowledge/patterns/knowledge-one-home-koh-9-pitfall.md

---
type: bee.pattern
title: knowledge-one-home cell koh-9 — pitfall candidate
description: "Pitfall candidate mined from cell koh-9's capped trace: Tests authored in set_gate.rs own #[cfg(test)] module beside the advisor-precondition cases the cell cited as the pattern, not state_group/tests.rs — every gat…"
timestamp: 2026-08-22
bee:
  id: knowledge-one-home-koh-9-pitfall
  lifecycle: draft
  areas: [okf-profile, workflow-state, decision-memory, doctrine-layer]
  sources: [.bee/cells/koh-9.json]
  polarity: pitfall
---

# knowledge-one-home cell koh-9 — pitfall candidate

## What the cell did

Merged/execution gate refuses a lane whose conflict review is absent, stale by plan_rev, or unverdicted; a conflicts verdict is surfaced, not refused

## Recorded evidence (verbatim from .bee/cells/koh-9.json)

- **deviation** — Tests authored in set_gate.rs own #[cfg(test)] module beside the advisor-precondition cases the cell cited as the pattern, not state_group/tests.rs — every gate test lives there and run_gate_body is private to that file
- **deviation** — conflict_review_refusal reads the LIVE WORKFLOW RECORD (root + lane) instead of the passed lane record the cell sketched: koh-8 writes conflict_review to the workflow record and the lane projection never copies it down
- **deviation** — catalog.rs PINNED_FLAG_COUNT stays 180 — koh-9 adds a refusal and an output field, no flag name; the audit comment says so explicitly
- **deviation** — Regen output trees (.agents/.claude/.claude-plugin/.codex-plugin/.opencode skills + .bee/onboarding.json) reserved and committed beside the cell files — skills/ is a manifest root

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell koh-10 — save as docs/knowledge/patterns/knowledge-one-home-koh-10-pitfall.md

---
type: bee.pattern
title: knowledge-one-home cell koh-10 — pitfall candidate
description: "Pitfall candidate mined from cell koh-10's capped trace: Extended the payload fix past cells cap/finish to cells reopen and state handoff write: both carried the same retired claim that a cap or the finish door runs …"
timestamp: 2026-08-22
bee:
  id: knowledge-one-home-koh-10-pitfall
  lifecycle: draft
  areas: [okf-profile, workflow-state, decision-memory, doctrine-layer]
  sources: [.bee/cells/koh-10.json]
  polarity: pitfall
---

# knowledge-one-home cell koh-10 — pitfall candidate

## What the cell did

Rule markers and refs now scan code-stripped text; retired boundary-test help rewritten in the registry payload

## Recorded evidence (verbatim from .bee/cells/koh-10.json)

- **deviation** — Extended the payload fix past cells cap/finish to cells reopen and state handoff write: both carried the same retired claim that a cap or the finish door runs commands.test
- **deviation** — Left docs/specs/test-simple.md and the two stale comments in verbs/cells/tests.rs alone — outside the cell files, exempt tree / comment-only
- **deviation** — Pre-existing and untouched: skills/bee-planning/SKILL.md line 79 spells (rule: R138, ...), which no marker homes, so bee knowledge check still reports one unknown_rule_ref; fixing a skills/ file would pull in the regen chain this cell forbids

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell koh-12 — save as docs/knowledge/patterns/knowledge-one-home-koh-12-pitfall.md

---
type: bee.pattern
title: knowledge-one-home cell koh-12 — pitfall candidate
description: "Pitfall candidate mined from cell koh-12's capped trace: packages/bee/AGENTS.block.md edited by hand: it is the SOURCE the regen chain renders AGENTS.md FROM, not the other way round — the first regen reverted the AG…"
timestamp: 2026-08-22
bee:
  id: knowledge-one-home-koh-12-pitfall
  lifecycle: draft
  areas: [okf-profile, workflow-state, decision-memory, doctrine-layer]
  sources: [.bee/cells/koh-12.json]
  polarity: pitfall
---

# knowledge-one-home cell koh-12 — pitfall candidate

## What the cell did

Nine discipline rules homed in the operating block with ids, applied_at records, and one-line pointers everywhere else

## Recorded evidence (verbatim from .bee/cells/koh-12.json)

- **deviation** — packages/bee/AGENTS.block.md edited by hand: it is the SOURCE the regen chain renders AGENTS.md FROM, not the other way round — the first regen reverted the AGENTS.md-only edits; path reserved before the write, same shape koh-4 used
- **deviation** — router-triage carries no bee.applied_at frontmatter key: applied_at_unlinked resolves targets against rules the concept body homes, and these ten are homed in AGENTS.md, so the key would fail the check — the AGENTS.md rule homes section holds the records, as koh-4 did
- **deviation** — koh-11 handoff fixed: skills/bee-capturing/SKILL.md wrapped (rule: workflow-state-capture-skill-answer) across a newline AND left an unbalanced backtick on the line, both of which blank the ref for extract_rule_refs — rewritten on one balanced line
- **deviation** — agents-one-next-action marks the whole Communication opening paragraph, which also carries the progress-tick rules: splitting the semicolon list would have reworded pinned prose

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 11 capped cell(s) mined, 1 delivery draft, 8 area bullet(s), 10 pattern candidate(s), 0 file(s) written.