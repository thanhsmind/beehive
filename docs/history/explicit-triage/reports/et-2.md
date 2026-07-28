# et-2 — Triage law: count THEN record, re-lane updates in place (D4)

[DONE]

Hive body (`skills/bee-hive/SKILL.md`) now mandates recording the route the
same turn as the mode-gate flag count, via `state route --set`, citing the
verbatim `Route: class=<c> | lane=<l> | flags=<n> [<names>] | files=<n>`
format. Body trimmed elsewhere in the Lanes section to land at 8180/8183
bytes (3 under the recorded budget — net-negative). `routing-and-contracts.md`
gained a new "Route record" subsection (full protocol: shape, enum
refusals, status/preamble surfacing, claim-warning-as-safety-net) plus a
note in the Re-lane checkpoint section that a demotion rewrites the same
route record in place, never a second one. `provenance.md` got one new row
citing explicit-triage D1-D4. No vendored regen run (wave-barrier ack).

Files touched:
- `skills/bee-hive/SKILL.md`
- `skills/bee-hive/references/routing-and-contracts.md`
- `skills/bee-hive/references/provenance.md`

Full trace/evidence: `.bee/cells/et-2.json`.
