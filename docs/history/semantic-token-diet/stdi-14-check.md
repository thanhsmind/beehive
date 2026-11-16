# stdi-14 check: the stdi-13 playbook and knowledge-upkeep fixes

Checked commits: dd1901a39 and the restore 637bc8988 (both trailer `cell: stdi-13`), against CONTEXT.md D1 and D2.

## Truths

- PASS: `rg 'cells finish --help' skills/bee-planning/playbooks` exits 1 and `rg -- '--cleanup' skills/bee-planning/playbooks` exits 1, so neither finds a hit. The class indexes now read "names it when it refuses" (skills/bee-planning/playbooks/feature.md:12, skills/bee-planning/playbooks/bugfix.md:11, and the same line in the other seven class indexes).
- PASS: every Situations row still parses. The diff changes no `` - `name` `` row (`git show dd1901a39 -U0 | rg '^[-+]- \`'` exits 1). All nine class indexes keep `## Situations`. class_playbook_parity passes 9 of 9 (its parser is at packages/bee-rs/crates/bee/tests/class_playbook_parity.rs:171).
- PASS: the step-2 rank order is unchanged. The diff leaves step 2 alone, and it still reads "wrong count, then orphan, then suspect-and-loaded, then loaded-never-used" (skills/bee-knowledge-upkeep/SKILL.md:62-63). The restore 637bc8988 touches only the step-4 bullet.

## Moved or cut rules and their homes

- The "Proof floor" line in 14 situation files was cut, and the dead `bee cells finish --help` pointer was dropped from the 9 class indexes. Home: the class index line (skills/bee-planning/playbooks/feature.md:12), the floor table in code (packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:111), and the refusal that names the floor (packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:462).
- `--cleanup` was dropped from autopilot-full, autopilot-stack and shipping. No rule moved: cleanup now runs by default and the flag is a no-op (packages/bee-rs/crates/bee/src/generated/registry_payload.json, the `cleanup` parameter of `worktree merge`).
- bug-fix step 2, the red-before-green rule. Home: skills/bee-planning/playbooks/bugfix/bug-fix.md:23 names `bee-principle-red-before-green` (skills/bee-principle-red-before-green/SKILL.md). The "craft, enforced by review" sentence stays at skills/bee-swarming/references/worker-details.md:40.
- The knowledge-upkeep signal table. Home: the reasons in packages/bee-rs/crates/bee/src/verbs/upkeep_group.rs:148-197, the 5-load threshold at upkeep_group.rs:15, and the pointer at skills/bee-knowledge-upkeep/SKILL.md:49-51.
- "Starting defaults, change them here" was cut as stale (D2), because the thresholds live in the binary (skills/bee-knowledge-upkeep/SKILL.md:50).
- "A run that was never recorded counts as due." Home: skills/bee-knowledge-upkeep/SKILL.md:51.
- "Loaded-never-used never makes a run due alone." Home: it goes to `info`, not `reasons` (upkeep_group.rs:191-197), and the `upkeep due` help says "never make the run due" (registry_payload.json).
- "It is trimmed or retired only after the source reader confirms it is dead weight" and its why "a concept may only be unreported". Home: skills/bee-knowledge-upkeep/SKILL.md:83-85, restored by 637bc8988. The earlier defect from this check is fixed. The qualifier "while the report's silent cells outnumber its answered ones" was not restored; the restored why-clause carries its meaning, so this is not a defect.
- The "When knowledge is read, written and revised" table. Homes: the read row is the worker brief's Learned context block (packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs:3380), the write row is packages/bee/prompts/worker-cell.md:81 and skills/bee-swarming/references/execute.md:52, the revise and report rows are skills/bee-capturing/SKILL.md:110-113, and the audit row is the Pass at skills/bee-knowledge-upkeep/SKILL.md:69-86.

## Do-not-touch list (C-playbooks.md section 4)

- No listed line was removed. The diff does not touch babysit, pause-safely, session-pickup, worktree-cleanup, eval, multi-phase-plan or reproduce-first. In feature/feature.md and investigation.md it removes only the Proof floor line. No situation file loses its `Proof line:` (`git show dd1901a39 -U0 | rg '^-Proof line'` exits 1).

## Verify

Re-run after 637bc8988, `cargo test --release --test class_playbook_parity --test route_class_parity --test pointer_integrity`: class_playbook_parity 9 passed, 0 failed; pointer_integrity 10 passed, 0 failed; route_class_parity 2 passed, 0 failed.
