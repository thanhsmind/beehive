# Hat wave synthesis — pi-1-0-upgrade

Five seats critiqued the first harness-native draft of `plan.md` on
2026-10-02. All five returned inside the ceiling; none was dropped. The
leader synthesized; each finding below names where it landed.

## Blockers

| Seat | Finding | Landed |
|---|---|---|
| risks | Codemode scripts call tools whatever the active set is (Pi `docs/mcp.md:204`), so hiding `edit`/`write` cannot stop a leader write. | D8 amended: the rule is enforced in write-guard; the loadout is the signal. Claim row 7. |
| risks | `FULL_TOOL_SET` (`stage_tools.rs:16-17`) lacks `codemode`/`tool_search`, and the belt keeps only allowed names (`bee-guard.ts:2913`), so narrowing strips the D2 unblock. | p1u-1 now edits `stage_tools.rs`. Claim rows 5-6. |
| risks | No worker signal reaches stage-tools; a worker pane would lose writes with no unattended escape. | D8: workers and unknown roles keep every tool. Slice-2 proof row. |
| risks | The belt ships in the binary (`doctor.rs:47`); rollback needs a release. | D11 kill switch. |
| facts-gaps | Claim row 2 bytes dropped indentation. | Row 2 re-quoted with indentation. |
| facts-gaps | The mechanism/prose map and the D3 premise had no rows. | Rows 9 and 17. |

## Warnings and shape notes

| Seat | Finding | Landed |
|---|---|---|
| facts-gaps | `/reload` question partly answered by `CHANGELOG.md:64`. | Row 12; open question narrowed. |
| facts-gaps | Row 8 "relocate rides it" had no anchor; omp claims had no row. | Rows 14 and 23. |
| facts-gaps | Release count wrong in CONTEXT; D4 wording differed between files. | Fixed: eight releases; one D4 wording. |
| facts-gaps | p1u-1 had no rebuild/doctor step; key_links empty. | p1u-1 action and key_links. |
| risks | Continuation loop when bee cannot record; key must carry the cell id; no forced turn in workers. | D7. |
| risks | `bee_dispatch` must run only the bee binary, argv form, detached. | D6. |
| risks | A denied codemode script leaves earlier writes in place (`codemode.md:17`). | Risk map; p1u-3 audit row. |
| value | `bee_advisor` is nearly free once `bee_dispatch` exists. | D9: ships in slice 2. |
| value | The virtual-model router does not make dispatch or the advisor fire. | D10: backlog row. |
| value | Herding was covered only implicitly. | D6 names `bee_dispatch` as the herding path; cockpit out of scope. |
| alternatives | SMALLER PATH: PASS for slice 1. Build obligations over `session-close`'s finder, not a new engine; `bee_advisor` as a thin alias. | D7, D9. |
| user-impact | Leader edit block, forced turn, background worker and auto hat wave each need on-screen text and an escape. | Shape: SEE mocks and named defaults; slice 2-3 test rows. |

## Not adopted

- alternatives: enforce only in the tool loadout. Rejected by the risks seat's codemode finding; enforcement moved to write-guard.

## Named deviation — slice 4 added after the wave

The user added mid-run input on 2026-10-02, after the five seats returned.
Slice 4 (D12-D14) is a headline with no cells, so the wave is not re-run for
the whole feature (once per feature). Its cells get the hat-risks and
hat-user-impact check at slice-4 drafting, before they persist.
