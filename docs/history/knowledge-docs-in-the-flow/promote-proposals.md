promote proposal for work item "knowledge-docs-in-the-flow" (docs/history/knowledge-docs-in-the-flow/CONTEXT.md + docs/history/knowledge-docs-in-the-flow/plan.md) — 4 capped cell(s): kdf-1, kdf-2, kdf-3, kdf-4
anchor: history — docs/history/knowledge-docs-in-the-flow/CONTEXT.md, docs/history/knowledge-docs-in-the-flow/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

This proposal covers (b) area updates and (c) pattern candidates only; it proposes no delivery record.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "knowledge-docs-in-the-flow" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-08T09:50:12.206Z), the work item declares no bee.areas.

area okf-profile:
  - [kdf-1] parse_read_set + read_set_refusal wired once at the pre-lock peek after plan_claims_refusal; 11 read_set tests green; 12 existing approval fixtures and one e2e test carry the bee:no-read-set marker; commit 239680ffb — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/kdf-1.json)
  - [kdf-2] knowledge check now errors with missing_decision_link on an unlinked area concept once the rule is armed (one NOTE otherwise); three tests added; the five area concepts linked, 0 profile errors; commit 9fcd11c1a — feature-wide sync per the scribing stamp, 7 file(s) changed (trace .bee/cells/kdf-2.json)
  - [kdf-3] Added Read set, section questions, deletion lines, rule-bullet tail and Cut area template to the templates, ran bee dev regen, committed 2425b28e7 — feature-wide sync per the scribing stamp, 14 file(s) changed (trace .bee/cells/kdf-3.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell kdf-1 — save as docs/knowledge/patterns/knowledge-docs-in-the-flow-kdf-1-pitfall.md

---
type: bee.pattern
title: knowledge-docs-in-the-flow cell kdf-1 — pitfall candidate
description: "Pitfall candidate mined from cell kdf-1's capped trace: undeclared-file pi_plugin_contracts.rs — e2e shape approval fixture needed the bee:no-read-set marker (one line from round 239680ffb, kept)"
timestamp: 2026-11-08
bee:
  id: knowledge-docs-in-the-flow-kdf-1-pitfall
  lifecycle: draft
  areas: [okf-profile]
  sources: [.bee/cells/kdf-1.json]
  polarity: pitfall
---

# knowledge-docs-in-the-flow cell kdf-1 — pitfall candidate

## What the cell did

parse_read_set + read_set_refusal wired once at the pre-lock peek after plan_claims_refusal; 11 read_set tests green; 12 existing approval fixtures and one e2e test carry the bee:no-read-set marker; commit 239680ffb

## Recorded evidence (verbatim from .bee/cells/kdf-1.json)

- **deviation** — undeclared-file pi_plugin_contracts.rs — e2e shape approval fixture needed the bee:no-read-set marker (one line from round 239680ffb, kept)
- **deviation** — advisor consult not run: the three judge fixes were fully specified
- **deviation** — sync-ack: the skill side of this door landed in kdf-3 (commit 2425b28e7); kdf-1 adds only the refusal and its tests
- **failure_signature** — read_set_refusal checks knowledge paths against the caller root while CONTEXT.md resolves to the worktree root

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell kdf-2 — save as docs/knowledge/patterns/knowledge-docs-in-the-flow-kdf-2-pitfall.md

---
type: bee.pattern
title: knowledge-docs-in-the-flow cell kdf-2 — pitfall candidate
description: "Pitfall candidate mined from cell kdf-2's capped trace: sync-ack: kdf-2 added one profile error branch to check.rs and three tests; the concepts anchored to check.rs and knowledge/tests.rs were already suspect from …"
timestamp: 2026-11-08
bee:
  id: knowledge-docs-in-the-flow-kdf-2-pitfall
  lifecycle: draft
  areas: [okf-profile]
  sources: [.bee/cells/kdf-2.json]
  polarity: pitfall
---

# knowledge-docs-in-the-flow cell kdf-2 — pitfall candidate

## What the cell did

knowledge check now errors with missing_decision_link on an unlinked area concept once the rule is armed (one NOTE otherwise); three tests added; the five area concepts linked, 0 profile errors; commit 9fcd11c1a

## Recorded evidence (verbatim from .bee/cells/kdf-2.json)

- **deviation** — sync-ack: kdf-2 added one profile error branch to check.rs and three tests; the concepts anchored to check.rs and knowledge/tests.rs were already suspect from earlier commits and describe checks this cell did not change; kdf-4 (writer) extends and re-confirms the okf-profile concept that documents this error, and the content cut (PBI p-3a756890) owns the rest.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell kdf-3 — save as docs/knowledge/patterns/knowledge-docs-in-the-flow-kdf-3-pitfall.md

---
type: bee.pattern
title: knowledge-docs-in-the-flow cell kdf-3 — pitfall candidate
description: "Pitfall candidate mined from cell kdf-3's capped trace: sync-ack: kdf-3 added template sections to six skill references and their rendered trees; the anchored concepts describe behaviors these edits do not change, a…"
timestamp: 2026-11-08
bee:
  id: knowledge-docs-in-the-flow-kdf-3-pitfall
  lifecycle: draft
  areas: [okf-profile]
  sources: [.bee/cells/kdf-3.json]
  polarity: pitfall
---

# knowledge-docs-in-the-flow cell kdf-3 — pitfall candidate

## What the cell did

Added Read set, section questions, deletion lines, rule-bullet tail and Cut area template to the templates, ran bee dev regen, committed 2425b28e7

## Recorded evidence (verbatim from .bee/cells/kdf-3.json)

- **deviation** — sync-ack: kdf-3 added template sections to six skill references and their rendered trees; the anchored concepts describe behaviors these edits do not change, and every one of them was already suspect from earlier commits — re-confirming them here would assert a review this cell did not do. Their re-confirmation belongs to the content cut (PBI p-3a756890).

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell kdf-4 — save as docs/knowledge/patterns/knowledge-docs-in-the-flow-kdf-4-pitfall.md

---
type: bee.pattern
title: knowledge-docs-in-the-flow cell kdf-4 — pitfall candidate
description: "Pitfall candidate mined from cell kdf-4's capped trace: the worker committed 54805b017 and stopped before cap; the leader ran the verify and capped"
timestamp: 2026-11-08
bee:
  id: knowledge-docs-in-the-flow-kdf-4-pitfall
  lifecycle: draft
  areas: [okf-profile]
  sources: [.bee/cells/kdf-4.json]
  polarity: pitfall
---

# knowledge-docs-in-the-flow cell kdf-4 — pitfall candidate

## What the cell did

Record the read set, the templates rule and the decision-link error in the OKF profile

## Recorded evidence (verbatim from .bee/cells/kdf-4.json)

- **deviation** — the worker committed 54805b017 and stopped before cap; the leader ran the verify and capped
- **deviation** — rendered skill copies under .claude-plugin, .codex-plugin and .opencode are outside the declared globs; bee dev regen writes them with the others
- **deviation** — the one-line judge fix ran inline in the leader, not through a worker: a single doc sentence
- **deviation** — sync-ack: the only skill change is one templates-rule paragraph in bee-capturing plus its regenerated copies; the anchoring concepts cover the skill trees as a whole and none states the bee-capturing templates rule, which okf-profile/freshness-and-document-homes.md (in this cell) now owns
- **failure_signature** — freshness-and-document-homes.md:142 misstates missing_decision_link exit behaviour as strict-only; code fails in both modes

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 3 area bullet(s), 4 pattern candidate(s), 0 file(s) written.