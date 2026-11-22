# Stale promote-proposal audit (read-only), round 1

Read in full: the 8 proposal files under docs/history/<feature>/promote-proposals.md (sections a, b, c; (a) ignored).
Searched: docs/knowledge/areas/{workflow-state,okf-profile,doctrine-layer}/ and docs/knowledge/patterns/ (rg -n -i, 2-4 terms per bullet). Every (b) bullet's area was found in those three areas.
Totals: 21 (b) bullets (16 PRESENT, 5 ABSENT, 0 CONFLICT); 35 (c) deviation/failure lines (25 ROUTINE, 10 LESSON: 4 COVERED, 6 UNCOVERED).

### knowledge-outcome-measure
b: workflow-state | kom-1 | PRESENT workflow-state/dispatch.md:396-401 | holdout arm drops Learned context block
b: workflow-state | kom-2 | ABSENT | outcomes section, comparable at 30/arm (report text lives in okf-profile only)
b: okf-profile | kom-1 | PRESENT okf-profile/critical-bar.md:73 | prepare record carries knowledge_arm (record field only)
b: okf-profile | kom-2 | PRESENT okf-profile/critical-bar.md:70-79 | outcomes section, comparable at 30/arm
c: none

### verified-claims-triage
b: none
c: vct-1 | ROUTINE | sync-ack: new LEADER.md rule block | —
c: vct-2 | ROUTINE | sync-ack: one sentence in verify-before-agreeing block | —

### protocol-init-from-project
b: none (no capped behavior_change cell)
c: pi-1 | ROUTINE | sync-ack: skill side is pi-2's onboarding reference | —

### workspace-protocol
b: none (no capped behavior_change cell)
c: wsp-3 | ROUTINE | sync-ack: LEADER.md paragraph; applied_at text unchanged | —

### windows-ci-1107
b: none (no bee.areas declared)
c: wc7-1 | ROUTINE | sync-ack: fixture-only change | —
c: wc7-3 | ROUTINE | sync-ack: fixture-only change | —
c: wc7-4 | LESSON | failure signature 0223bfb925ff, text not recorded; unix/non-unix spill split | COVERED docs/knowledge/patterns/20260708-windows-git-bash-tmp-is-invisible-to-node.md (partial: /tmp only)

### knowledge-freshness-mechanic
b: okf-profile | kfm-1 | PRESENT okf-profile/freshness-and-document-homes.md:53-55, 64, 74-79 | grade from anchors and git; seed and confirm
b: okf-profile | kfm-2 | PRESENT okf-profile/freshness-and-document-homes.md:56 | grade printed wherever a concept is read
b: okf-profile | kfm-3 | PRESENT okf-profile/freshness-and-document-homes.md:63-69 | cap refuses suspect or orphan; unverified passes
b: okf-profile | kfm-4 | PRESENT okf-profile/context-and-promote.md:97, 279 | delivery draft retired; work tree moved to knowledge-work
b: okf-profile | kfm-5 | PRESENT okf-profile/freshness-and-document-homes.md:83-85 | per-feature pull-rate table in knowledge report
b: workflow-state | kfm-1 | ABSENT (grade rule only cross-referenced at cells-completion-judge-and-archive.md:483-484) | grade from anchors and git; seed and confirm
b: workflow-state | kfm-2 | ABSENT | grade printed wherever a concept is read
b: workflow-state | kfm-3 | PRESENT workflow-state/cells-completion-judge-and-archive.md:479-482 | cap door refuses suspect or orphan anchored concepts
b: workflow-state | kfm-4 | ABSENT | delivery draft retired; work tree moved
b: workflow-state | kfm-5 | ABSENT | per-feature pull-rate table
c: kfm-1 | ROUTINE | owner-directed window rule change to vc..HEAD | —
c: kfm-1 | ROUTINE | run ceiling raised 1800 s to 3600 s | —
c: kfm-1 | LESSON | worker stalled idle about 3 h, cancelled, re-dispatched | UNCOVERED
c: kfm-1 | ROUTINE | four worker rounds; judges found history gaps | —
c: kfm-1 | LESSON | first-parent-off vc grades ancestors suspect (freshness.rs:169-177, 206-209) | UNCOVERED (nearest 20260901-a-stated-ancestry-direction-is-a-coin-flip-until-it-is-measured.md: prose ordering only)
c: kfm-1 | LESSON | unrelated-history vcs make merge-base exit 1; grade breaks | UNCOVERED
c: kfm-1 | LESSON | evil merge edits an anchor; git log without -m/--cc lists no files | UNCOVERED
c: kfm-2 | ROUTINE | sync_door.rs: one doc-comment line deleted (comment_baseline) | —
c: kfm-2 | ROUTINE | sync-ack: sync_door.rs touched only for that deletion | —
c: kfm-3 | ROUTINE | run ceiling raised 1800 s to 3600 s for release build | —
c: kfm-3 | LESSON | first rework judge idle twice, cancelled, re-run | UNCOVERED
c: kfm-3 | ROUTINE | sync-ack: re-confirm step landed in first round | —
c: kfm-3 | LESSON | cap door matches exact paths; grade treats directories as covering | COVERED docs/knowledge/patterns/20260819-the-join-between-two-tested-parts-is-what-nobody-tests.md (partial: untested join between two tested parts)
c: kfm-6 | LESSON | installed binary is a symlink into main; rebuild after merge | COVERED docs/knowledge/patterns/20260805-source-that-ships-without-reinstalling-the-binary-the-hooks-call-is-inert.md
c: kfm-6 | ROUTINE | sync-ack: seed wrote frontmatter on three concepts | —

### knowledge-docs-in-the-flow
b: okf-profile | kdf-1 | PRESENT okf-profile/freshness-and-document-homes.md:98-100, 105 | read set parsed; approval refused without it
b: okf-profile | kdf-2 | PRESENT okf-profile/freshness-and-document-homes.md:146-151 | unlinked area concept errors missing_decision_link
b: okf-profile | kdf-3 | PRESENT okf-profile/freshness-and-document-homes.md:113-133 | template question, deletion lines, cut area template
c: kdf-1 | ROUTINE | undeclared e2e fixture needed no-read-set marker | —
c: kdf-1 | ROUTINE | advisor consult not run; fixes fully specified | —
c: kdf-1 | ROUTINE | sync-ack: skill side landed in kdf-3 | —
c: kdf-1 | LESSON | read_set_refusal checks paths against caller root, not worktree root | UNCOVERED (nearest 20260819-inside-a-worktree-the-tracked-copy-of-a-runtime-store-answers-stale.md: different trap)
c: kdf-2 | ROUTINE | sync-ack: one profile error branch and three tests | —
c: kdf-3 | ROUTINE | sync-ack: template sections in six skill references | —
c: kdf-4 | LESSON | worker committed and stopped before cap; leader ran verify and capped | COVERED docs/knowledge/patterns/20261027-a-dispatched-stage-that-fails-preflight-gets-run-inline-by-the-leader-and-the-preflight-never-gets-fixed.md (partial: different stage)
c: kdf-4 | ROUTINE | rendered skill copies outside declared globs; regen writes them | —
c: kdf-4 | ROUTINE | one-line judge fix ran inline in leader | —
c: kdf-4 | ROUTINE | sync-ack: one templates-rule paragraph in bee-capturing | —
c: kdf-4 | ROUTINE | failure: freshness-and-document-homes.md:142 said strict-only; now :151 says every mode | —

### tiny-bound-delegation
b: workflow-state | tbdel-1 | PRESENT workflow-state/cells-completion-judge-and-archive.md:331-345 | inline tiny over one counted file refused; route refuses
b: workflow-state | tbdel-2 | PRESENT workflow-state/cells-completion-judge-and-archive.md:336-339 | tiny is one hand-authored file; inline reason
b: doctrine-layer | tbdel-1 | PRESENT doctrine-layer/lane-and-working-discipline.md:43-44 | inline tiny refused at cap; route refuses N>1
b: doctrine-layer | tbdel-2 | PRESENT doctrine-layer/lane-and-working-discipline.md:41-42 | tiny is one hand-authored file; delegation reason
c: tbdel-1 | ROUTINE | sync-ack: all 22 concepts already suspect before cell | —
c: tbdel-2 | ROUTINE | sync-ack: LEADER.md edit changes one rule only | —

## Notes
- ABSENT (5): kom-2 in workflow-state; kfm-1, kfm-2, kfm-4, kfm-5 in workflow-state. Each of these is stated in okf-profile, not workflow-state.
- kfm-1 window rule (vc..HEAD) is not restated in the bundle; freshness-and-document-homes.md:55 cites plan.md. No CONFLICT.
- UNCOVERED LESSONs (6): kfm-1 stall; kfm-1 three git-graph traps (first-parent, unrelated histories, evil merge); kfm-3 idle judge; kdf-1 caller-root vs worktree-root. The git merge-graph traps have no pattern at all.
- The wc7-4 signature has no recorded text; its LESSON call is from the cell title only.
