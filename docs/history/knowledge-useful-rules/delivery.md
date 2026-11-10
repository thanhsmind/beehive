---
type: bee.delivery
title: knowledge-useful-rules — delivery
description: "Delivery record for feature knowledge-useful-rules: five writing rules from one home, Read-when lines in pack, three check warnings, knowledge-used cap trace, and knowledge-used report."
timestamp: 2026-11-10
bee:
  id: knowledge-useful-rules-delivery
  lifecycle: active
  areas: [okf-profile, workflow-state, rust-runtime]
  required_context: [docs/history/knowledge-useful-rules/CONTEXT.md, docs/history/knowledge-useful-rules/plan.md]
  sources: [docs/history/knowledge-useful-rules/CONTEXT.md, docs/history/knowledge-useful-rules/plan.md]
---

# knowledge-useful-rules — Delivery

## Concepts changed

- **docs/knowledge/areas/okf-profile/context-and-promote.md** — Documents the consolidate rule list as the single home for knowledge writing rules rendered in worker briefs (D6), and the `Read when:` line rendered in place of title in learned context (D2).
- **docs/knowledge/areas/okf-profile/critical-bar.md** — Documents the knowledge-used section in `bee knowledge report` tracking concept uses from capped cells and reporting concepts loaded 5+ times with no use (D5).
- **docs/knowledge/areas/okf-profile/freshness-and-document-homes.md** — Documents the three machine-visible writing check warnings (`missing_read_when`, `directory_anchor`, `long_concept`) and the shared `read_when_line` function (D2, D4).
- **docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md** — Documents B51a recording which concepts changed a decision on cell cap via `--knowledge-used`, `--no-knowledge-used`, or worker report, persisted on trace without refusal (D5).
- **docs/knowledge/areas/rust-runtime/prompt-files-and-learned-context.md** — Documents the learned-context line showing `Read when:` and the worker prompt rendering the consolidate rule list (D2, D6).

## What shipped

- **kur-1** — Five knowledge writing rules in consolidate and prompt, and Read-when lines printed in learned-context pack lines (commits 6b4942a01, 92d62fa4e).
- **kur-2** — Warnings in `bee knowledge check` for missing Read-when line, directory anchors, and long concepts over 300 lines (commit 37320c710).
- **kur-3** — Recorded on each cap which concepts changed a decision via `--knowledge-used` / `--no-knowledge-used` and report array, reaching trace (commit 905f60495).
- **kur-4** — Reported concept use counts and loaded-never-used concepts in `bee knowledge report` text and JSON (commit 87f425cf6).
- **kur-5** — Taught the rules in skills and concept-authoring pointing at the one consolidate rule list, and regenerated skill trees (commits f8f3fc75c, 2c3260a6d).
- **kur-7** — Shared `read_when_line` helper between knowledge check and prepare learned-context lines (commit 2f0495d24).
- **kur-6** — Consolidated feature knowledge, added Read-when lines and code anchors to touched concepts, and wrote delivery record.
