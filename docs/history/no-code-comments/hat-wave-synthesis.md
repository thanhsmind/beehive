# no-code-comments — plan-step hat wave synthesis

Three seats (hat-facts-gaps, hat-alternatives, hat-user-impact), opened
2026-09-22 under decision f02e89ab, over CONTEXT.md D1–D7 and plan.md
revision 1. Synthesized by the leader; plan.md revision 2 carries the
result. All three seats returned inside the ceiling; no seat dropped.

PLAN CHECK
Work: no-code-comments, single slice, cells ncc-1..ncc-4 (revision 2; revision 1 had three)

STRUCTURE
BLOCKERS (all resolved in revision 2):
- crate bee has no lib target, so tests/comment_fence.rs could never call comments::* / ratchet moved into devtools/comment_baseline.rs as a bin unit test
- the Edit arm read old_string/new_string fragments — no file line numbers, block state or file-top exceptions / reconstruct_target_text extracted from the config arm and shared
- heredoc bodies are removed by fence_heredocs and no reader exists; guards.rs was outside the cell / heredoc_writes added, guards.rs in ncc-3
- apply_patch has no hunk parser / apply_patch_added_lines added, detectors.rs in ncc-3
- the docs cell's edits can red rule_index_parity, pointer_integrity and instruction_laws, none in its verify / the bullet is a marked rule with an index row; all three targets in ncc-4's verify
WARNINGS (acted on):
- SOURCE_CHECKOUT_DEV_VERBS 5 → 6; preserve_order needs BTreeMap insertion; the ** sentinel; the config read is in the read branch — named in the actions
- no config fixture exists — tests reuse build_fixture + plain write + expect_done
- behavior_change on the switch-flipping cell — true
- CODE_ROOTS literal lost the crates/* glob — a predicate
- load-bearing prose without rows — claims 26–28, 35–38 added

CELLS (reviewed: 4, leader's cold-pickup pass)
CRITICAL FLAGS: none open
MINOR FLAGS: ncc-3 is the largest cell (5 files, three readers) — accepted because the arm, its readers and the shared helper are one compile unit
CLEAN CELLS: ncc-1, ncc-2, ncc-4

ALTERNATIVES ACTED ON
- ratchet as a bin unit test; walker copied from instruction_laws.rs with attribution; fsutil for JSON IO
- verify strings use --bin bee <module>:: (exact), not -p bee <substring>
- ncc-1 split into predicate (ncc-1) and verb+baseline+ratchet (ncc-2)

USER IMPACT ACTED ON
- five unmerged branches carry ~256 added comment lines — D2b 0add770d: seed over main plus unmerged branches; the ratchet still never raises
- the refusal names /// and //! and the home for a public item's description
- baseline keys /-normalized for the Windows lane
- #! is an exception on any line (heredoc-body shebangs)
- the concept's D6 sentence fenced as not-a-deferral
- the ungranted-worktree edge named in the concept
- a bee doctor row filed as a backlog proposal

DISMISSED
- alternatives: a #[ignore] test instead of a verb — D2 locks a verb
- user-impact: strip comments as a merge precondition for the five in-flight branches — the user chose stop-the-bleeding over disturbing in-flight work; the D2b seed covers it

SUMMARY: The claims table held on bytes (one claim, row 7 of revision 1,
drew the wrong conclusion from a true anchor and was rewritten). The
shape survived; what the wave changed is where the ratchet lives, how the
hook sees a whole file, two missing readers, the seed rule for in-flight
branches, and the doctrine landing as a spoken rule. Every locked decision
D1–D7 stands as written; D2b touches D2 without changing it.
