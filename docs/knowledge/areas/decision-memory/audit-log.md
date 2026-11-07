---
type: bee.area
title: Decision Memory — the audit log for bookkeeping
description: "Why gate auto-approvals, close/merge/release/scribing/promote/handoff bookkeeping and door deferrals live in a tracked audit log instead of the decision store, how bee audit log and the doors use it, and how legacy decision rows stay readable."
timestamp: 2026-11-07
bee:
  id: decision-memory-audit-log
  lifecycle: active
  areas: [decision-memory]
  required_context: [areas/decision-memory/overview.md]
  decisions: ["knowledge-first-lean-decisions D3, D4, D5, D6 (docs/history/knowledge-first-lean-decisions/plan.md, 2026-11-07)"]
  sources: ["knowledge-first-lean-decisions cells kbc-2, kbc-3, kbc-4 (traces in .bee/cells/, 2026-11-07)", "packages/bee-rs/crates/bee/src/audit.rs", "packages/bee-rs/crates/bee/src/verbs/audit.rs", "packages/bee/LEADER.md (§ Capture what settles)"]
  authoritative_for: "decision-memory: what belongs in the audit log and what stays a decision"
---

# Decision Memory — the audit log for bookkeeping

The decision store is read for rules, behavior and trade-offs. Over 30 days
about a fifth of its rows were gate auto-approval audit lines and another fifth
were close, merge, release, scribing, promote and handoff bookkeeping, while
writes outnumbered reads about 13 to 1. Bookkeeping crowded out the rules the
store exists to recall.

## Behaviors & Operations

**B1 — Gate auto-approval has one home** (plan D3). `bee gate --actor auto`
stamps the gate record and appends one `gate-bypass` row to the audit log
itself. The agent logs no decision for an auto-approval, and no doctrine, skill,
preamble or nudge text tells it to.

**B2 — Bookkeeping goes to `.bee/audit.jsonl`** (plan D4). The audit log is a
tracked file with `merge=union`, written by `bee audit log --kind <k> --feature
<f> --note "<text>"` and read by `bee audit list`. It is tracked on purpose:
`.bee/logs/` is git-ignored, so rows there would never reach main through a
worktree merge, and the close doors run on main. The kind set is closed:
`gate-bypass`, `close`, `merge`, `release`, `scribing`, `promote`, `handoff`
and nine `*-deferral` kinds (judge, uat, capture, knowledge-freshness, impact,
routing, doc, leader-check, dissent).

**B3 — Old rows stay where they are** (plan D5). Nothing rewrites
`decisions.jsonl`. The doors read the audit log first and then the legacy
decision rows, so a deferral logged either way satisfies its door. `decisions
active` and `search`, status and the preamble hide rows tagged with an audit
kind at read time unless the caller asks for that tag.

**B4 — The decision verb refuses audit kinds** (plan D6). `bee decisions log`
refuses a `--tags` value that is an audit kind and names `bee audit log` as the
remedy. Words like `release` or `merge` stay legal decision tags, because many
real rules carry them.

**B5 — Doctrine names the home where no check can** (plan D6). For
close/merge/release/scribing/promote/handoff bookkeeping with no tag to refuse,
`packages/bee/LEADER.md` § Capture what settles says it goes to the audit log.
That line is prose, not a check, and it is the named limit of this design.

## Open Gaps

- Untagged bookkeeping written with `bee decisions log` is not refused; only the
  doctrine line steers it to the audit log.
