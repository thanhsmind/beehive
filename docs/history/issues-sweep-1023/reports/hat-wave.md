# issues-sweep-1023 — hat wave synthesis (plan check)

Wave: 2026-10-23, three seats (hat-facts-gaps opus, hat-alternatives
opus, hat-user-impact opus), all returned; zero dropped. Leader
synthesis; the seats' digests are data.

PLAN CHECK
Work: slice 1 — iss-1 to iss-12

STRUCTURE
BLOCKERS (all applied before the gate):
- coverage, iss-11: phases.rs missing from files, and the exit
  contradicted itself. Leader ran the falsifier both seats named
  (`bash .bee/tmp/gitt.sh`): git refuses a merge over an untracked file
  the branch writes, identical bytes included. D10 superseded
  (17294ba4): refuse up front with WORKTREE_MERGE_MAIN_DIRTY, zero
  mutations. Cell rewritten; phases.rs added.
- completeness, iss-7: the no-version path of release.sh would lose its
  permit check. Cell now names all three paths and a named order test.
- coverage, iss-12: D11 says "the same selector"; mutation_lane_selector
  reads only `lane`. flags.rs added with a key parameter; `bee route`
  registry entry added beside `bee state route`.
- membership: rows 19-24 added (#119 commit, session adopt, discarded
  unwind, is_deployment, git experiment, lock::is_pid_alive).
WARNINGS:
- D12 issue closing now owned by the leader (Out of scope line).
- iss-2 success-without-session truth now has a named test.
- iss-10: no hand edit of this repo's .gitattributes (D9 superseded,
  5e66f40a); managed key in scope; affects_specs set.
- iss-4/iss-5: Windows truths provable only by Windows CI; #96 closes
  only on a green Windows run.

ALTERNATIVES (applied):
- iss-2: shadow `session` once after the claim file outcome; no
  claims.rs edit; dependency on iss-4 dropped.
- iss-4: reuse crate::lock::is_pid_alive (existing Windows arm), no new
  unsafe code.
- iss-10: reuse merge_gitignore_content / find_gitignore_markers.

USER IMPACT (applied):
- iss-2: the silent Fail::Delegate unwind and the store-mismatch check
  named as candidates (reporter saw no refusal line).
- iss-5: canonicalize change limited to cfg(windows) in path_identity.
- iss-9: leader runs the full declared suite once with
  BEE_HERDING_WORKER=1 after all caps.
- New user-visible text: iss-1 refusal names missing ids; iss-8 message
  says the agent was stopped; iss-12 new flag; iss-7 permit errors come
  after cheap checks; iss-10 hosts see a .gitattributes change on next
  onboarding and must commit it.

CELLS (reviewed: 12) — leader cold-pickup pass
CRITICAL FLAGS: none open.
MINOR FLAGS: iss-1 refusal still attaches to the first batch row
(accepted, recorded).
CLEAN CELLS: iss-1 to iss-12 after the edits above.

SUMMARY: Three structural blockers and three membership gaps were fixed
in the plan and two decisions were superseded on evidence. No blocker
is open, so no second pass is needed.
