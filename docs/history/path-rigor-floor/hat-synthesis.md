# Hat wave synthesis — path-rigor-floor

Seats: hat-facts-gaps (two runs, conf 85 each), hat-alternatives (conf 72),
hat-user-impact (conf 80). Raw findings: `.bee/tmp/hat-wave/path-rigor-floor.md`
in the main checkout.

## Applied to plan.md

| Finding | Seat | Change |
|---|---|---|
| prf-2 is one file, serial on prf-1 | alternatives | prf-2 merged into prf-1; prf-3 stays parallel |
| Matcher choice open | alternatives, facts-gaps | write_guard `glob_tokens` / `glob_match` through the `crate::hooks::write_guard` re-export, chars collected first |
| `**/*.sql` catches fixtures; misses Rails, Alembic, Prisma | user-impact | built-in rows are schema-shaped; tests for both sides; decision touches f9ba8d88 |
| A wrong built-in row cannot be turned off | user-impact | no exempt list: it would let a repo lower a floor (D1); recorded in the same decision |
| GATE_OWNER_FIRST must tell the leader to stop | user-impact | message says stop, no retry, no `--actor user`, set waiting-on gate, end the turn |
| A route raised after an auto gate lets an owner-first cell in (G1) | facts-gaps, user-impact | cells add/update refuse an owner-first hit when the gate actor is auto |
| Re-check after the lock (W1) | facts-gaps | gate check runs before and after acquire_mutation_locks |
| Route fallback to state.json (W2) | facts-gaps | prf-1 reads the route the way feature_route does |
| Regen writes rendered trees; manifest is exclusive (W3, G6) | facts-gaps | prf-3 reserves the manifest and lists regen output in its cap |
| Remedy needs `--files` (W4) | facts-gaps | remedy line carries `--files` |
| No code field on refusals | facts-gaps | in-message tokens in the REGEN_OBLIGATION style |
| Row source in the message | user-impact | message names built-in or repo row |
| Module export, private lane list, feature arg (G3, G4, G7) | facts-gaps | stated in the prf-1 action |
| release-binaries.yml matches `release*` | facts-gaps | risk map row, accepted |

## Not applied

- Nothing else. No locked decision changed, so no supersede decision is needed.
