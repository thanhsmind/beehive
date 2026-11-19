# Close the mistake loop end to end — Context

**Feature slug:** mistake-loop-closed
**Date:** 2026-11-19
**Backlog item:** p-59a26c99
**Shaping session:** complete under gate bypass full (owner asked to build the whole loop: mistake → recorded → layered → backlog row → guard built → guard proven → no recurrence)
**Scope:** Standard
**Domain types:** RUN | READ

## Feature Boundary

The loop has seven steps. bee runs steps 1 to 4 today and stops: a
check or architecture mistake becomes one P3 finding row and nothing
pulls it further. Measured on 2026-11-19 in this checkout:

1. 343 fix-at finding rows (299 check, 44 architecture); one key has a
   landed guard (the clippy strip_prefix rule, fix-at-loop D5).
2. The mistakes door clears on one answer of the closing run, for every
   capped cell of the feature (verbs/drivers/close.rs:1605).
3. A check or architecture reflection from a run that never closed a
   feature is never filed (decision d1d83be7).
4. A doctrine or none label is never re-judged, even when the same
   mistake repeats across runs.
5. Nothing shows the debt: no preamble line, no orient blocker, no
   doctor row. The trigger that asked for a doctor row
   (more-than-20-backlog-rows-with-layer-fix__6e79d14e) is manual tier,
   and manual triggers never auto-fire by law (verbs/triggers/mod.rs:42).
6. Closing a fix-at row is a free-text backlog-closed row; nothing
   checks that the named guard exists or ever went red.
7. Recurrence is caught only when the new title equals the closed title
   after normalize_title.
8. The open/closed rule lives twice: feedback.rs fix_at_load and
   backlog.rs current_epoch_finding_feature.
9. knowledge check never reads bee.evidence_ref.

This feature closes every step. It changes the backlog store readers,
the mistakes door, the reflect verb, session takeup, the preamble,
orient, status, doctor, upkeep due, knowledge check, and the doctrine
that names the loop.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | One fix-at ledger. A new module verbs/fix_at.rs folds backlog rows into groups keyed by (layer, normalize_title(title)) for layers fix-at:check, fix-at:architecture and fix-at:rejudge. Each group has one state from a closed set: Open, Closed, Recurred (a finding newer than the latest close). It carries the finding count, the oldest open ts, the latest close row and its guard command. The fold is a pure function over rows; one IO wrapper reads the store. feedback.rs fix_at_load and backlog.rs current_epoch_finding_feature become callers of it and their duplicated rule goes. | One fact, one home. Every reader below needs the same open/closed answer. |
| D2 | The mistakes door clears only the cells an answer names. bee mailbox reflect gains --cell <id>[,<id>...]; the entry stores a cells list (additive key, written only when non-empty). A closing-run answer clears a debt cell only when its cells list names that cell. The refusal prints the exact reflect and no-mistakes commands with the unanswered ids filled in. An answer with no --cell is still recorded and still reaches the letter; it clears nothing. | One blanket sentence must not erase the answer owed for N cells. The remedy stays printable, so rbl-1's dead-end concern holds. |
| D3 | Session takeup files what close never saw. Beside compose_digests_for_finished_periods, one sweep reads every run's entries. A check or architecture reflection is filed as a P3 finding row, feature "unattributed", when its group has no current-epoch finding and its entry is newer than the group's latest close. The sweep is idempotent through the ledger. | d1d83be7 forbids inventing a feature; it does not forbid filing under an explicit unattributed owner. |
| D4 | Repeated soft labels are re-judged. The same sweep groups doctrine and none reflections by the layer-free key (first four normalized words of the wrong text). A key seen in two or more distinct runs, with no current-epoch fix-at row of any layer for that key, files one P3 finding row with layer fix-at:rejudge. | The miner already flags repeats as lessons; nothing asks whether the soft label was right. A layer the author never named is not invented: rejudge says only "decide again". |
| D5 | Wording drift is caught. When close or the sweep files a mistake, its layer-free four-word key is compared with every group's title key. A match on a Closed group files under that group's exact title, so the group turns Recurred; the detail keeps the original wording. A match on an Open group counts as a skip. | Exact-title matching misses the same mistake in new words, so a failed guard looks like success. |
| D6 | Debt is visible. The ledger exposes open counts per layer, the recurred count and the oldest open age in days. The session preamble shows one block "### Fix-at debt" after the capture queue; orient adds one blocker line; bee status --json carries fix_at_debt; bee doctor adds a non-voting advisory that warns when more than 20 open groups are older than 30 days. The doctor advisory is what trigger 6e79d14e asked for; this feature resolves that trigger with the advisory named as its outcome. The manual-tier law stays. | A debt nobody sees is never paid. The trigger's own condition becomes a standing check instead of a one-time flag. |
| D7 | Closing a fix-at group needs proof. New verb bee backlog fix-at close --layer <l> --title <t> --guard <what stops it> --guard-cmd <command> --red-proof <text> [--commit <sha>]. It refuses when the group is not Open or Recurred, when guard, guard-cmd or red-proof is empty, or when guard-cmd does not exit 0 run from the repo root. On success it appends a backlog-closed row carrying guard, guard_cmd, red_proof and commit. A fix-at:rejudge group may close instead with --kept <reason> and no command. bee backlog add refuses --type backlog-closed on a fix-at layer and names the new verb. | A close without a runnable guard is the same free text that let 343 rows sit. The red proof is evidence the build already emitted. |
| D8 | Closed guards are re-run. bee backlog fix-at verify runs the guard_cmd of every Closed group. A command that now fails appends a finding row with the same layer and title, so the group turns Recurred, and the verb exits non-zero. | A guard that was deleted or broke is a recurrence that no mistake has reported yet. |
| D9 | The loop has a puller. bee backlog fix-at next prints the top group: Recurred first, then the most findings, then the oldest. bee backlog fix-at promote files one proposed PBI for that group, title "Guard: <title>", and refuses while another fix-at PBI is proposed or in flight (work-in-progress limit one). Session takeup runs promote when open debt exists and no fix-at PBI is open, so the existing dispatch flow, attended or herded, picks it up. bee upkeep due gains kind fix-at, due when a Closed group exists and no fix-at-upkeep audit row is younger than 7 days; its run line names the commands bee backlog fix-at verify and bee backlog fix-at promote, and verify logs the fix-at-upkeep audit row itself. No new skill: upkeep skills are hub-served, and a verb pair needs no skill. | Every earlier loop step waited for a human pull. A PBI enters the one flow bee already drives to done. |
| D10 | knowledge check reads evidence_ref. It splits the value on whitespace, semicolons and parentheses and warns evidence_ref_missing_path for each token that has a slash and a file extension and exists under none of three bases: the repo root, packages/bee-rs/crates/bee/src, packages/bee-rs/crates/bee. It warns evidence_without_ref when bee.evidence is wired or exercised with no evidence_ref. Both are warnings; --strict makes them fail as every warning does. | A wired rung that names nothing is the same unproven claim D7 removes from the backlog. |
| D11 | The doctrine names the whole loop. AGENTS.block.md and LEADER.md's mistake bullet gain --cell; skills/bee-capturing/references/promotion.md and expertise/static-rules.md name fix-at close with its proof flags in place of the hand-written backlog-closed row; the owning concepts (human-mailbox overview, feedback-digest ranking, okf conformance check, token economy of injected context) sync in the same change. | A verb no doctrine names is unreachable. |

### Agent's Discretion

- Names of private helpers and the exact wording of refusals and preamble lines.
- The guard-cmd timeout value, provided it is stated in the concept.

## Terms

- **Group:** all fix-at rows that share (layer, normalized title).
- **Layer-free key:** the first four normalized words of a mistake's wrong text, without its layer.
- **Guard:** the lint, helper, test, hook or doctor row that stops a mistake; guard-cmd is the command that runs it.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/human-mailbox/overview.md | suspect | Rules for reflections, the mistakes door and close-time filing (D2, D3, D4, D5). |
| docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md | unverified | Rank, recurred and backlog-closed semantics (D1, D7, D8). |
| docs/knowledge/areas/okf-profile/conformance-check.md | suspect | Evidence ladder warnings (D10). |
| docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md | suspect | Preamble budget for the new block (D6). |
| docs/knowledge/areas/decision-memory/audit-log.md | confirmed | Audit kinds for fix-at-upkeep (D9). |

### Code

| Path | What it decides |
|------|-----------------|
| packages/bee-rs/crates/bee/src/verbs/feedback.rs:1705-1778 | fix_at_load, close_stamps, the open/closed rule (D1) |
| packages/bee-rs/crates/bee/src/verbs/backlog.rs:407-454 | finding row builder, current_epoch_finding_feature (D1, D7) |
| packages/bee-rs/crates/bee/src/verbs/backlog.rs:818-926 | bee backlog add refusals (D7) |
| packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:274-426 | mistakes_debt, closing_run_answered_mistakes, filing (D2, D5) |
| packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1600-1626 | the mistakes door (D2) |
| packages/bee-rs/crates/bee/src/verbs/mailbox.rs:675-829 | Entry and its additive keys (D2) |
| packages/bee-rs/crates/bee/src/verbs/mailbox.rs:2910-3103 | reflect parse and run_reflect (D2) |
| packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs:737-762 | fix_at_or_none, reflection_key (D4, D5) |
| packages/bee-rs/crates/bee/src/verbs/work.rs:300-316 | session takeup call site (D3, D4, D9) |
| packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:868-883 | preamble block insertion point (D6) |
| packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:523-556 | orient report-only blockers (D6) |
| packages/bee-rs/crates/bee/src/doctor.rs:1510-1547 | non-voting advisories (D6) |
| packages/bee-rs/crates/bee/src/verbs/upkeep_group.rs:20-304 | upkeep kinds (D9) |
| packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs:746-761 | evidence validation (D10) |

## Existing Code Context

### Reusable Assets

- normalize_title (feedback.rs:2158), backlog_finding_row (backlog.rs:407), runs_with_entries (mailbox.rs:1037), reflection_key (mailbox_digest.rs:755), audit::rows (upkeep last-run).

### Established Patterns

- Additive optional keys on mailbox entries: written only when present, read as absent by older readers (mailbox.rs:713-715).
- Fail-open writes on side paths: a backlog write error warns and never fails close or takeup.
- Report-only orient blockers and non-voting doctor advisories.

### Integration Points

- Session takeup (work.rs:311) for D3, D4, D9 auto-promote.
- bee backlog subcommands, the catalog and the generated registry for D7, D8, D9.

## Canonical References

- docs/history/mistake-fix-at/CONTEXT.md (D2, D5 of that feature: one row per mechanizable mistake, promotion reads fix-at).
- docs/history/fix-at-loop/CONTEXT.md (D1, D8, D10: global identity, backlog-closed as the resolution home, close epochs).
- docs/history/reflection-becomes-lesson/CONTEXT.md and promote-proposals.md (the remedy path of the mistakes door).

## Outstanding Questions

None. Every gap the measurement found has a locked decision above.

## Deferred Ideas

- A count predicate for triggers. Not built: D6 turns the one trigger that needed it into a standing doctor advisory, and the manual-tier law stays.

## Handoff Note

Plan the ledger (D1) first: every other decision reads it.
