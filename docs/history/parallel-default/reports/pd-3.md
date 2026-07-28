# pd-3 — Doctrine flip: parallel-by-default at the four mapped sites

[DONE]

Flipped serial-default doctrine to parallel-by-default (D1) plus the
wave-barrier regen protocol (D2) at all four mapped sites: bee-swarming
`SKILL.md` small-lane row, `swarming-reference.md` hardening-7 section
(rewritten in full), and `routing-and-contracts.md`'s lane row,
Small-lane serial doctrine section (retitled Parallel dispatch doctrine),
and execution-worker paragraph. Also updated the corresponding provenance
rows in both skills' `provenance.md` (thin-body law) — auto-add deviation,
not in the cell's declared `files` list but not a frozen-judge pattern.

Files modified: `skills/bee-swarming/SKILL.md`,
`skills/bee-swarming/references/swarming-reference.md`,
`skills/bee-hive/references/routing-and-contracts.md`,
`skills/bee-swarming/references/provenance.md`,
`skills/bee-hive/references/provenance.md`

Reservations: reserved all 5 paths under exec-pd3; released yes (5
reservations + 5 cross-worktree holds).

Verification: `node scripts/skill_budget_fence.mjs && node
scripts/skill_lint.mjs && node scripts/okf_instructions_fence.mjs` ->
passed. SKILL.md body net -12 bytes (8175/8187 budget, zero prior
headroom).

Commit: 4b79848e

Full trace/evidence: `.bee/cells/pd-3.json`

Next action: orchestrator's wave-close regen barrier (mirror render ->
onboard --apply -> manifest --write/--check) still owed once, per
`regen_obligation_ack: "wave-barrier"` on this cell.
