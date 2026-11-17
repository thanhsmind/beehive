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
  decisions: ["knowledge-first-lean-decisions D3, D4, D5, D6 (docs/history/knowledge-first-lean-decisions/plan.md, 2026-11-07)", "outcome-scorecard D2 (docs/history/outcome-scorecard/CONTEXT.md, 2026-11-17 — every gate set appends a gate audit row; gate-bypass stays for auto)"]
  sources: ["knowledge-first-lean-decisions cells kbc-2, kbc-3, kbc-4 (traces in .bee/cells/, 2026-11-07)", "outcome-scorecard cells osc-2/osc-2t (docs/history/outcome-scorecard/plan.md, 2026-11-17)", packages/bee-rs/crates/bee/src/audit.rs, packages/bee-rs/crates/bee/src/verbs/audit.rs, packages/bee/LEADER.md (§ Capture what settles)]
  authoritative_for: "decision-memory: what belongs in the audit log and what stays a decision"
  anchors: [packages/bee/LEADER.md, packages/bee-rs/crates/bee/src/audit.rs, packages/bee-rs/crates/bee/src/verbs/upkeep_group.rs, packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs]
  verified_at: 2026-11-17
  verified_commit: 993b9548c8f1ec83c3b7d0e846b7930e662ae316
---

# Decision Memory — the audit log for bookkeeping

Read when: deciding whether an event belongs in `.bee/audit.jsonl` or `.bee/decisions.jsonl`, or reading and writing gate and bookkeeping audit rows.

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
`gate`, `gate-bypass`, `close`, `merge`, `release`, `scribing`, `promote`, `handoff`
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

**B6 — Two kinds are last-run records, not bookkeeping about a feature**
(upkeep-due-signal D3). `knowledge-upkeep` and `verify-upkeep` carry the outcome
word first in the note, and the newest row of each kind is the `last_run` that
`bee upkeep due` reads. They do not end in `-deferral`, so no door reads them as
an escape. A run that logs no row stays due; find the reasons with
`bee upkeep due --json`.

**B7 — Every gate set leaves a `gate` row, manual or auto** (outcome-scorecard
D2). Without it nobody can count how often a plan was reopened, because only an
auto set wrote a row (`gate-bypass`). The `gate` row is written for each gate a
set touched, so a merged shape+execution set writes two; `gate-bypass` stays
beside it for auto. A refused set writes nothing. Find the writer by searching
`"gate"` in `verbs/state_group/set_gate.rs`.

## Open Gaps

- Untagged bookkeeping written with `bee decisions log` is not refused; only the
  doctrine line steers it to the audit log.
