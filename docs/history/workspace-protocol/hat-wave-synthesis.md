# Hat wave synthesis — workspace-protocol

Three seats ran on the drafted plan (herding jobs, read-only). The leader
took each finding below into CONTEXT.md and plan.md before Gate 2.

| Seat | Finding | Taken as |
|---|---|---|
| facts-gaps F1 (blocker) | `prompt_body` is not the text the payload carries | D7: hash the carried field (`prompt`, `stdin`, codex message), from the final payload; claim 4 re-anchored |
| facts-gaps F2 (blocker) | `lint_brief` also runs a four-section shape check, so every purpose would warn | D11: stem arm only; claim 12 added |
| facts-gaps F3 | hat purposes carry a perspective by design | D11: `--role hat-*` is exempt |
| facts-gaps F4 | registry payload: generated or hand-edited? | read gates.md:866: hand-edited; regen moves to wp-3, wp-2 no longer waits on wp-3 |
| facts-gaps F5 | dispatch log is per root | D8: verify reads the control root; refusal fix names it |
| facts-gaps F6 | appeared, absent, unrecorded untested | wp-2 tests added |
| facts-gaps F7 | non-UTF-8 prompt file | D8: mismatch; no second hasher |
| facts-gaps F8, user-impact F2 | an unfilled template checks valid | D2: placeholder values are an issue; template ships no placeholder; init prints a review line |
| facts-gaps F9, user-impact F4 | ok/warn/advisory wording; row hidden when missing | D5: row always prints; statuses `ok` and `advisory` |
| facts-gaps F10 | top-level router unknown | wp-1 wires it wherever it lives, as a named deviation |
| user-impact F1 (high) | boundary stop vs gate bypass undefined | D6: boundaries are named at Gate 2; they add no stop; bypass decides |
| user-impact F3 | no file for `--prompt-file` | D8: save the carried field with `jq -j`; round-trip test |
| user-impact F5 | duplicate keys, scalar list, list end | D1/D2: last wins plus issue; scalar is an issue; blank line ends a list |
| user-impact F6 | worktree vs main root | D7/D8: control root for both |
| user-impact F7 | warning not visible | D11: text line and record flag |
| alternatives | SMALLER PATH passes; optional trims A1 (wp-4 at capture), A2 (wave-barrier regen) | A2 superseded by moving regen into wp-3; A1 declined — the concept is new, a cell keeps it proven |

Reports: `.bee/mailbox/job-1794034876574-2199379-1/report-1.md` (facts-gaps),
`.bee/mailbox/job-1794034877929-2199500-1/report-1.md` (alternatives),
`.bee/mailbox/job-1794034879205-2200075-1/report-1.md` (user-impact), in the
main checkout.
