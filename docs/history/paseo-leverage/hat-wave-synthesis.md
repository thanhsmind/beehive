# Hat wave synthesis — paseo-leverage

Five seats ran once on the first draft of `plan.md` (2026-10-12): hat-facts-gaps
(opus), hat-risks (fable), hat-value (opus), hat-alternatives (opus),
hat-user-impact (opus). The leader merged the findings below into the current
`plan.md` and `CONTEXT.md`.

## PLAN CHECK

Work: current slice (pl-1 to pl-7)

### STRUCTURE

BLOCKERS (all fixed in the plan):
- Cell completeness: the pl-3 and pl-5 verify commands passed several name filters to one `cargo test`. Fixed: each cell chains one command per filter and uses `--test` for contract targets.
- Key links: verbs register in `generated/registry_payload.json`, not `catalog.rs`. Fixed: pl-4 and pl-5 list the payload and extend the catalog test.
- Risk: a read-only worker could not write its own ack and result files (Claude plan mode and the total write ban). Fixed: D2 revised (store `168b8037`): the job mailbox is the one write zone; no Paseo mode.
- Risk: the drafted allowlist had exec and output bypasses. Fixed: pl-3 reuses mode B's checker and lists each denied flag, with a test row per bypass.
- Risk: `sendLeaderMessage` cannot tell a skipped send from an idle turn. Fixed: pl-5 adds a three-way sibling and keeps the boolean contract.
- User impact: the brief forbids every bee command. Fixed: pl-5 names `bee herding notify` as the one exception.

WARNINGS (applied):
- Deps now carry every file overlap (pl-2 after pl-3; pl-5 after pl-2 and pl-4).
- pl-4 lists the five generated copies of `supervisor-prompt.md`.
- Reviewers leave the read-only set (they run tests).
- Codex and antigravity refuse read-only under Paseo (no guard belt).
- Unread messages use a per-job `messages.cursor`; Pi delivery only for inbox-session jobs; text fenced as data.
- Fork refuses a Working source without `--force` and a session outside the pi sessions root; prints undo and the model note.
- The Paseo home honors `--home` in the configured command.
- Parked names the permit command; read-only denials name the write zone.
- pl-2 reuses `parse_child_jsonl`'s usage mapping and extends `insert_usage`.
- D8 asserts `persistence.nativeHandle`; `ls -a` named.
- The D7 measurement is a leader task after merge, recorded in the decision log.

Refinements logged: store `6fc537f6`.

### CELLS (reviewed: 7)

CRITICAL FLAGS: none open.
MINOR FLAGS: pl-5 is the largest cell (verb, status, brief, drain); kept whole because its parts share one contract.
CLEAN CELLS: pl-1, pl-2, pl-3, pl-4, pl-6, pl-7

SUMMARY: The SMALLER PATH check passed (no cheaper shape honors D1–D8). The
two seats with blockers changed D2 and the D1 delivery rules; both changes are
logged decisions. No blocker is open.
