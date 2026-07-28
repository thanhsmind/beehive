# Provenance — bee-compounding body rules

The body states its rules bare (provenance exile, skill-token-diet D8). This table maps each
body rule to the decision(s) that authorize it and the rationale in one line. Long-form records:
`docs/decisions/`, `.bee/decisions.jsonl` (via `bee.mjs decisions search`), and
`docs/history/<feature>/CONTEXT.md` for the feature that locked each ID.

| Body rule | Decision IDs / labels | Rationale |
|---|---|---|
| §2: spawn each analyst as the runtime read-only type, never `general-purpose` | decision `040f8ef0` ("D1 — spawn read-only") | Dogfood-confirmed leak: "write no files" in a prompt is not a tool restriction; the ambient bee-reviewing "default/general subagent type" convention leaked full Edit/Write/Bash into analysts, producing an unrequested commit |
| §2: launch all three, end the turn, event-driven wait; a denied/errored dispatch is re-tried once, then synthesize from what returned, never loop | decision `040f8ef0` ("D2 — wait, don't hang") | Dogfood-confirmed a second gap: step 3 locked to three-of-three, refusing partial-return — an unbounded retry loop under a structural denial |
| §1 gather and §8 digest refresh delegate as extraction-tier I/O workers | Delegation contract (`bee-hive/references/routing-and-contracts.md`); decisions 0016, 0023 | Mechanical multi-file/read-only steps dispatch down-tier; decide-altitude (synthesis) never delegates |
| §6 / Guard the State Layer: learnings merge into the state layer, never a parallel notes pile; `bee-scribing` owns the write | decisions 0001 (state-layer), 0002 (bee-scribing BA skill, write ownership moved from bee-compounding to bee-scribing) | The state layer answers "what does this area do right now"; a parallel notes pile duplicates it and rots independently |
| §6 backlog done-flip fallback: identical per-clause CoS evidence gate as scribing, never looser | decision-propagation D1, decision `b9b9fee3` | Partial delivery must never silently flip a backlog row to `done`; the fallback path (scribing NOOPed) carries the same discipline as the primary path |
| §9 Sweep the Feature's Scratch: one of two scratch-sweep moments (feature close, session finish) | tree-hygiene D2 (`docs/history/tree-hygiene/CONTEXT.md`) | "Gitignored" answered the wrong half of the complaint — 153MB accumulated while `git status` stayed clean; deletion needs an explicit verb and an owning moment |
| §10 Commit the Close: commit before §11's state update; one commit, the close's own | GitHub issue #48 | `compounding-complete` claims the close is durable; setting the phase before the commit lands leaves that claim false if the tree is lost (crash, `git checkout`, worktree merge) |
| §10 review candidate registration at close | decision `565e68d0` (review-on-demand); SPEC review-on-demand R3, flow 7.1 step 6 | Execution closes `unreviewed` through scribing/compounding by design; the candidate registration is what lets a later user-invoked review find this head |
| §12 Suite Census: report suites/lines/delta in the run summary | decision `8a1271b7` (test-economy D4) | Suite count grows monotonically via auto-discovery with no counter-pressure; compounding's census is the visible ledger, `bee-grooming` test-prune is the counter-force |
