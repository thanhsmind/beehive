# fc-1 — Rescope scribing/compounding triggers to feature-close

[DONE]

Rescoped the scribing/compounding trigger wording from per-cell/per-execution to
feature-close, per the user's philosophy decision (7346e9d7): a feature = many
slices = many cells; scribing sync and compounding run once, at the end.

Files touched:
- `skills/bee-scribing/SKILL.md` — sync-mode trigger row and the matching Hard
  Gates line rescoped to "feature close, capped cells incl. `behavior_change`";
  trimmed an implementation-path parenthetical to stay inside the 8151-byte
  ratchet (now 8135 bytes).
- `skills/bee-swarming/SKILL.md` — the tiny/small "After `[DONE]`" paragraph now
  invokes `bee-scribing` only on the feature's final slice; a non-final slice
  returns to `bee-planning` instead, aligning it with the file's own Completion
  Signals section. Stayed inside the 8187-byte ratchet (now 8184 bytes).
- `skills/bee-hive/references/routing-and-contracts.md` — Capture discipline's
  "a capped `behavior_change` cell obliges a `bee-scribing` sync in every lane"
  rescoped to "a feature whose capped cells include `behavior_change` obliges
  ONE `bee-scribing` sync at feature close, covering all of them" — this was
  the literal per-cell-obligation wording the cell targeted.
- `skills/bee-scribing/references/provenance.md`,
  `skills/bee-swarming/references/provenance.md` — added rows citing decision
  7346e9d7.

`AGENTS.md` rule 8 was inspected and left unchanged: its text is the capture-stub
law only (no per-cell scribing-sync claim), so it already reads correctly and
must stay verbatim. `skills/bee-swarming/references/swarming-reference.md`'s
"Single execution worker in full" section was inspected and already scopes
`bee-scribing` to the feature's final slice; no change needed there.
`packages/bee/tests/test_misc.mjs` was inspected for census checks pinning any
of the changed wording — none found, so it is unchanged.

Reservations: reserved 7 paths under exec-fc1; released, none leaked.
Verification: none run (R82 main-verifies law) — the cell's `verify` field
(`skill_budget_fence.mjs`, `skill_lint.mjs`, `okf_instructions_fence.mjs`,
`test_misc.mjs`) documents what MAIN will run at feature close. Byte budgets
were checked manually via `wc -c` (8135/8151, 8184/8187).
Commit: f3cdb5ddec2f80565690253cecfe7f7a9dabe494
Cap: `--feature-verify-pending`. Full trace: `.bee/cells/fc-1.json`.

Next action: orchestrator runs the feature verify at feature close (main-verifies
D2/D3) and continues the chain (scribing → compounding, once) if fc-1 is the
feature's final slice.
