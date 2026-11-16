# Hat wave synthesis — team-global-config (plan step, one round)

Seats: hat-facts-gaps (opus), hat-alternatives (agy-flash), hat-user-impact (opus), hat-risks (fable), hat-value (agy-flash). All five returned within the ceiling; none dropped.
Seat reports (main checkout): `.bee/mailbox/job-1794822852449-3459682-1/report-1.md`, `job-1794822853411-3459935-1`, `job-1794822853520-3459987-1`, `job-1794822853630-3460010-1`, `job-1794822854970-3460793-1`.

Verdicts: facts-gaps not-ready; alternatives ready; user-impact ready-with-fixes; risks ready-with-fixes; value ready-with-fixes.

## Applied

| Finding | Seat | Where it landed |
|---|---|---|
| Herding reads herding.agents / agent_command from config.json alone (wave.rs:803, job_verbs.rs:890, herding.rs:1461/1543) — BLOCKER | facts-gaps | new cell tgc-2, claims 4c-4e |
| Further direct value readers (herding.rs:659/689/702/1676, control_loop.rs:575, broker.rs:316, paseo_ws.rs:32, report.rs:113, detector.rs:608, status_full/store.rs:487/496) | facts-gaps, risks | tgc-2 |
| Claim 4 false: a byte-copy of the merge in session_preamble/state.rs:24 | facts-gaps | claim 4b; tgc-1 deletes the copy |
| Corrupt policy is three rows, not two; advisor strip differs | facts-gaps | tgc-1 truth table |
| Per-layer fold changes today's result for mixed models/team | facts-gaps | D13: fold once after merge; tgc-1 test |
| Worktree bootstrap copies config.json only (registry.rs:974) | facts-gaps | tgc-2 |
| A global team value is always shadowed by the seeded table — BLOCKER | user-impact | owner chose 1A: D11; tgc-4 --drop-global, tgc-5 seed |
| Agents can change every project through bee config set --global | user-impact | owner chose: only waggledance writes (D12); tgc-3, tgc-4 |
| Half-commit after split-team | user-impact | tgc-4 output and tgc-5 notice say commit both |
| Show which keys come from global | user-impact | validate lists global-supplied keys (D7, tgc-4) |
| Corrupt global file is silent everywhere | user-impact | tgc-1 warning names the path; tgc-6 drives it |
| tgc-6 proof files never written | user-impact | tgc-7 no longer splits; the leader step saves both outputs |
| Guard comparison leaves out the global layer, so an ADD shadows a global slot — BLOCKER | risks | D13; tgc-3 |
| Rollout: old binary on a split config — BLOCKER | risks | beehive split moved to a leader step after reinstall; undo = git revert |
| split-team write order | risks, value | team.json first (tgc-4); guard test pins add-then-remove (tgc-3) |
| Unparseable team.json passes the guard | risks | absent layer → removal → refused (tgc-3) |
| BEEHIVE_CONFIG_DIR / XDG_CONFIG_HOME assignment by an agent | risks | refused in Bash (D12, tgc-3) |
| Existing tests read the developer's real global file | risks | default resolver returns none under cfg(test) (tgc-1) |
| config_merged_text must not grow a second merge loop | alternatives | tgc-3 delegates to config_layers |
| verbs/config.rs local fn merged | alternatives | tgc-4 deletes it |
| Seed team.json by splitting default_config() | alternatives | tgc-5 |
| tgc-5 and tgc-6 overlap tests and builds | value | tgc-6 adds tests only for gaps, builds once |
| Keep budget.rs legacy warning on config.json only | value | tgc-1 no longer touches budget.rs |
| Match only the resolved global path | value | tgc-3 |

## Dismissed

- facts-gaps F7 (`config_governed` filter may match nothing) — kept: the filter is an extra; each code cell must quote its matched count above zero, and the full suite runs once per feature.
- alternatives NOTE (tgc-4 dep on tgc-3 is a build edge only) — no change needed; the seat agreed.
- user-impact F4 optional preamble line when the global file is non-empty — not taken: validate already lists global-supplied keys; a preamble line adds always-loaded prose.
- risks F5 `split-team --undo` — not taken: git revert of the split commit is the undo, documented in delivery.md.

## Open questions

None. Both owner questions were answered on 2026-11-16 (D11, D12).
