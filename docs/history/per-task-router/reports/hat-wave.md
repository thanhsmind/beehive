# per-task-router — hat wave synthesis

Three seats ran on the first plan.md draft, each as a read-only herded
advisor. Full digests:

- hat-facts-gaps: `.bee/mailbox/job-1794669953669-2533589-1/report-1.md` (confidence 85)
- hat-alternatives: `.bee/mailbox/job-1794669957388-2534045-1/report-1.md` (confidence 70)
- hat-user-impact: `.bee/mailbox/job-1794669960939-2535202-1/report-1.md` (confidence 70)

No seat dropped. All three recommended re-shaping before the gate.

## Findings and what the plan did

| Finding | Seat | Plan change |
|---|---|---|
| Existing preamble tests assert Route with no session and no lanes; gating on `source == "lane"` reds them | facts-gaps F1 | Route shows when the session is bound by either path or the repo has no lane records |
| Two bind paths; `source == "lane"` hides `session_binding` sessions | facts-gaps F6, user-impact 5 | one shared helper covers both paths |
| A new cap field breaks every `CapFlags` literal, including herding and knowledge tests | facts-gaps F2 | ptr-4 owns every literal site |
| A mandatory principle answer breaks the herded cap | facts-gaps F3 | required only in the principles arm, on both paths |
| The 3-value vocabulary cannot express red-then-green; the floor table contradicts feature.md | facts-gaps F4, alternatives A5, user-impact 1 | `green:red-first` added; floors per cell; feature.md's `green:live` stays feature-level |
| Implementer cells cannot cap green when their change flips tests they do not own | alternatives A2, facts-gaps F2 | each implementer owns the test edits its change forces |
| Pairing refusal at `cells add` breaks fixtures and later single cells | alternatives A3, facts-gaps F8 | stamp and warn at add, refuse at close and merge |
| Same-worker identity check has no stored basis and fires falsely under herding | alternatives A4 | dropped; structural separation holds D1 |
| Merge ptr-4 and ptr-5 | alternatives A1 | merged into ptr-4 |
| No red-tester path | user-impact 3 | tester returns BLOCKED; leader reopens the implementer with a fresh dispatch |
| Upgrade breaks in-flight cells | user-impact 2 | unstamped cells exempt; principle answer only in the principles arm |
| No hint when principles disappear | user-impact 4 | one hint line naming `bee orient --class` |
| Holdout can split a pair | user-impact 7 | tester hashes its implementer's id |
| Checker cells declare files they should not write | user-impact 8 | checker cells write only their own report |
| Projection rule superseded needlessly | user-impact 9 | kept; playbook steps seed the todo list as a projection |
| Proof commands lack the manifest path; two test filters | facts-gaps F7 | one manifest path for all; one filter per proof |
| Parallel Rust cells share one crate | facts-gaps F2 | Rust cells run as one serial chain |
| Docs-only undefined | facts-gaps F5 | every path is `.md` or under `docs/` |

Decisions: `58d7b2a7` records the choices above; `0ada9257` records how
the new doors narrow older decisions `412e9b3a` and `d02a6bc6`.

Not taken: the optional two-slice split (alternatives A6). The doctrine
cell must land with the pairing refusal, and gate bypass makes a second
gate cost nothing the split would save.
