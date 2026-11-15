# Hat wave synthesis — pstack-playbook-port

Three seats read the first draft of `plan.md` on 2026-11-15:
hat-facts-gaps (job-1794710028494-2194276-1), hat-alternatives
(job-1794710033576-2195279-1), hat-user-impact (job-1794710034937-2195443-1).
The full reports are in the main store's mailbox under those job ids.

## Accepted, and how the plan changed

| Finding | Seat | Change |
|---|---|---|
| A write-time check for `skills/bee-planning/playbooks/<class>/<name>.md` refuses in every host, which has no `skills/` tree at its root | facts-gaps F2, alternatives fact 3 | Dropped the route field and every new flag. orient reads situations from the repo copy, else a copy compiled into the binary (claims 14, 15). Decision `41bbba68` supersedes `0f6dae94`. |
| Port notes inside vendored files contradict the tool-name sweep | facts-gaps F1 | Port notes moved to `docs/history/pstack-playbook-port/port-notes.md`; the sweep covers `skills/bee-planning/`. |
| A class-only plan citing the menu passes, so the bugfix steps stop being enforced | user-impact F1 | The plan check now requires a situation citation when the routed class lists situations. |
| The bugfix index would name `bee-figure-it-out` before it ships | user-impact F2, facts-gaps F6 | No class index names a skill until slice 3 ships it. |
| The index grammar was undefined | user-impact F2 | One grammar, written into pp-1, pp-3 and pp-4. |
| Session situations were underspecified | facts-gaps F4 | `playbooks/session/index.md` lists them; the fence checks both directions. |
| Two deferred questions closed without answers | facts-gaps F5 | Decision `6ca3cd62`: 13 new principles, two merges, `bee-fanout` for swarm; claim 17 records the count. |
| The rendered plugin tree was missing from the docs cell | facts-gaps F7 | pp-3 owns the render; claim 16 shows the tree is a byte copy. |
| The no-class plan-check branch ignored situation paths | facts-gaps F8, alternatives fact 5 | pp-2 counts a situation path there. |
| No explicit SMALLER PATH line | alternatives | Added to Approach, with the rejected shapes. |
| JSON key presence unstated | user-impact F5 | `situations` is always present. |

## Rejected, with reason

| Finding | Seat | Reason |
|---|---|---|
| Docs-only selection (alternative A) | alternatives | The plan check could hold only the class menu; the Feature Boundary needs it to hold a situation. |
| Bare `bee orient` should show the routed situation | user-impact F3 | No situation is recorded on the route any more, so there is nothing to show. |
| Name validation and refusals for `--situation` | user-impact F4 | The flag no longer exists. |
| feature start root timing | facts-gaps F3 | The flag no longer exists. |

## Claims audit

facts-gaps confirmed rows 1 to 12 byte for byte; row 13 is out of repo and was
checked by the leader with the command it names.
