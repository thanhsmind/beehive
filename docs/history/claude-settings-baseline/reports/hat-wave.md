# Plan-step hat wave — claude-settings-baseline

Date: 2026-11-10. Five seats opened (high-risk).

## Seats

| Seat | Outcome | What came back |
|---|---|---|
| hat-facts-gaps | done | claims table 8 of 8 match; 5 should-fix, 1 note |
| hat-risks | done | no blocker; 4 should-fix |
| hat-value | done | no findings |
| hat-alternatives | dropped | hit the 30-minute ceiling; its partial log suggested a data file in place of a constant set |
| hat-user-impact | done | no blocker; 4 should-fix, 2 notes |

## Accepted findings (plan changed)

1. The opt-out did not survive a bare re-run, and bee's own fix lines tell users to run a bare `bee onboard --apply` (risks, user-impact). Added D5: a durable `"claude_settings": false` preference read like `"statusline": false`.
2. The permission-mode switch was invisible: install.sh prints only notices and a status word (user-impact). Added D6: one notice on every plan or apply of the full baseline, naming kept keys, allowed-then-denied tools and the opt-outs.
3. Bypass mode is written even on an install with `--no-hooks`, so the accepted cost's premise fails there (risks, facts-gaps). D7, a named guess: keep it unconditional as asked, and say so in the notice and the Safety text.
4. The upgrade path was unproved: every already-onboarded host gains the baseline on its next run (risks). csb-1 adds an existing-host-shape test; csb-2's Safety text names the upgrade.
5. Two odd-shape cases were undefined: the missing-check must return false for every shape the merge refuses, and deny is read only under the full baseline (facts-gaps). Stated in csb-1 with a non-array-deny test.
6. The existing tests were miscounted: four in hooks_wiring.rs and six end-to-end tests in onboard/tests.rs pin the item name (facts-gaps). csb-1 names all ten.
7. Denying AskUserQuestion turns host gate questions into plain chat text (user-impact). Added to the accepted costs; csb-3 states it in the knowledge.

## Dismissed

- A data file in place of the constant set (alternatives, partial): the worktree-tool projection already uses a constant, and the request is one fixed block.
