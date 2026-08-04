# hook-teeth — CONTEXT

Batch B of the prose-rule-audit: six prose rules gain mechanical
enforcement. Parent decision: e1e41ec8. Route deviation: 399d72e1.
Sequencing law (D7) applies to every flip.

## Locked decisions

- **D1 — plan.md freezes mechanically.** The write-guard denies
  Edit/Write to `docs/history/<feature>/plan.md` when that feature's
  `approved_gates.shape` is true (lane-aware: resolve the feature from
  the path segment, then its lane record or default state). The refusal
  names `bee state plan-rev bump` as the stamp path and "unapprove the
  gate" as the redraft path.
- **D2 — no claim on a red base.** `cells claim` refuses when the last
  recorded test run (`.bee/logs/test-results.json`) is red, unless the
  claim carries `--fix-first <reason>`, which is stored on the claim
  trace. A missing results file stays a warning (cannot know), never a
  refusal.
- **D3 — no cells before the gate.** `cells add` refuses while the
  target feature's phase is gated (planning/exploring) and
  `approved_gates.execution` is not true. Docs-lane features and cells
  added in swarming/idle flow are untouched.
- **D4 — adoption only at the fresh boundary.** `state handoff adopt`
  refuses when the calling session's recorded start source is
  resume/compact (session-init already knows the source; if it is not
  yet persisted on the session record, persisting it is part of this
  cell). A missing source stays a warning (older records), never a
  refusal.
- **D5 — re-lane transitions validated.** `route --set` over an existing
  route record enforces: demotion moves down the ladder only
  (standard→small→tiny), at most one demotion per feature, `high-risk`
  never demotes, and any hard-gate flag blocks demotion. Promotion is
  always allowed. The refusal names the violated rule.
- **D6 — the cell commit trailer is checked.** `cells finish` verifies a
  commit whose trailer names the finishing cell id exists on the
  feature's branch (the granted worktree's HEAD history, else main's)
  when `files_changed` is non-empty; `--commit-pending <reason>` escapes
  and is stored on the trace. A cell with no file changes is exempt.
- **D7 — sequencing law.** Per flip: a test proving the
  counter/condition computes correctly lands before the refusal wires
  in, red-first, same cell. No flip ships with a known false positive.

## Open questions

None blocking.

## Out of scope

- Batch C (doctrine diet), config keys for any threshold, reinstalling
  the live binary (separate coordination with the exec-speed session).
