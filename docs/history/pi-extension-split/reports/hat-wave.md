# Hat wave: pi-extension-split

Five seats ran on plan revision 1 (commit `1b242017c`). All five returned
SHIP-WITH-CHANGES. No seat was dropped.

| Seat | Model | Verdict |
|---|---|---|
| hat-facts-gaps | opus | SHIP-WITH-CHANGES (1 blocker) |
| hat-risks | fable | SHIP-WITH-CHANGES (2 blockers) |
| hat-alternatives | opus | SHIP-WITH-CHANGES |
| hat-value | gemini-3.8-flash-high | SHIP-WITH-CHANGES |
| hat-user-impact | gemini-3.8-flash-high | SHIP-WITH-CHANGES |

## Accepted, and where it landed

| Finding | Seat | Plan change |
|---|---|---|
| The agent_settled slice test ends at the session_before_compact handler; split event files break it | facts-gaps (blocker) | all `pi.on` handlers stay in one `events.ts` in order (D6, claim 15) |
| Deleting the file in a release-manifest root owes a manifest regen | risks (blocker) | pes-1 lists the manifest and runs `release-manifest --write` and `--check` (claims 16-17) |
| A crash between copy and remove leaves two guards | risks (blocker) | legacy `remove_pi_extension` planned before every copy; failed remove is reported (pes-3) |
| Prune can delete user files | risks, value, user-impact | prune only unshipped `.ts`, no symlink follow, scoped remove arm (D6, pes-3) |
| Old plugin cache plus new binary removes the guard with nothing to replace it | facts-gaps | legacy removal only when the source folder ships files (pes-3) |
| pes-1 holds all the risk | alternatives, value | full-folder doctor compare moved to pes-2 |
| Three walks decide which files ship | alternatives | one rule: every `.ts` file in the folder, sorted by name (D6) |
| Copy the existing expertise vendor-and-prune shape | alternatives | pes-3 action names it (claim 18) |
| Register functions must keep the parameter name `pi` | facts-gaps | pes-1 action |
| Join modules with a newline; each ends with a newline | facts-gaps | pes-1 action |
| Nested folders not proven safe on Pi 1.0 | facts-gaps | flat folder, `tool-*.ts` names (D6) |
| Doctor text must name the fix and the count | risks, user-impact | pes-1 and pes-2 actions |
| Rust doc comments still name the old path | risks, facts-gaps | pes-1 files |
| pes-3 docs check: `rg` exit code, `docs/knowledge/work`, `.opencode` copies | facts-gaps, value | `! rg`, work records left alone, all four copies listed |
| Downgrade leaves two guards | risks | rollback note with the manual step |

## Rejected, with reason

| Finding | Seat | Reason |
|---|---|---|
| Static Rust array instead of build.rs | value | D4 locks one source of truth |
| Per-domain state with accessors | value | D2 locked; read-only live bindings make it riskier |
| Point tests at single modules instead of a joined source | value | D5 locked; markers cross modules |
| Debug-mode cell proof | value | CI runs `--release`; debug has a known unrelated red |
| Startup duplicate check in `index.ts` | user-impact | behavior change; onboard ordering removes the case |
| `.bak` copy of a drifted legacy file | user-impact | onboard already overwrites drifted vendored files with no backup |
| Module-map comment in `index.ts` | user-impact | no new code comments; the map lives in the knowledge concept |
| Merge everything into 6-8 modules | value | `events.ts` merge taken; the rest keeps one concern per file (D1) |
