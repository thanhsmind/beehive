# stdi-8 check of stdi-7

Checked commit 16491f2bc (trailer `cell: stdi-7`) against CONTEXT.md D1 and D6.
Base for "before" is `16491f2bc^:packages/bee/LEADER.md`.

## Truths

- PASS — every rule id that packages/bee/LEADER.md held before stdi-7 still has a home with its why-clause. 16 ids before, 16 homed after. Each marker block is byte-identical to its pre-stdi-7 text (script compare of `<!-- rule -->` blocks). Moved homes: packages/bee/playbooks/cross-project.md:5, packages/bee/playbooks/cli-before-mcp.md:5. The rule index rows follow them (docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:246, :250).
- PASS — every new playbook has a read-when line and LEADER.md lists it. cross-project.md:3 and cli-before-mcp.md:3 carry `Read when:`. packages/bee/LEADER.md:18 and :20 list them. The short rule lines at packages/bee/LEADER.md:266 and :270 point to them.
- PASS — no do-not-touch line from reports/A-doctrine.md was removed. The whitespace-normalized text of each item is still present: agents-never-zero-execution-workers (packages/bee/LEADER.md:244), the knowledge-before-code why-clause (:140), the gate boundary and "never printed for the user to run" (:44), Judgment and deviation (near :78), and the project LEADER.md release "done ONLY on the final OK line". stdi-7 did not touch CLAUDE.md, AGENTS.md, AGENTS.block.md, handoff.md or budget.rs.

## Rule ids (16, all text unchanged)

- agents-capture-line-at-close — packages/bee/LEADER.md:274
- agents-cli-before-mcp — packages/bee/playbooks/cli-before-mcp.md:5 (moved)
- agents-feedback-changes-framework — packages/bee/LEADER.md:293
- agents-gates-never-self-approved — packages/bee/LEADER.md:44
- agents-knowledge-before-code — packages/bee/LEADER.md:140
- agents-name-applied-principles — packages/bee/LEADER.md:94
- agents-never-build-on-red — packages/bee/LEADER.md:170
- agents-never-zero-execution-workers — packages/bee/LEADER.md:244
- agents-no-code-comments — packages/bee/LEADER.md:353
- agents-one-commit-per-cell — packages/bee/LEADER.md:347
- agents-one-next-action — packages/bee/LEADER.md:303
- agents-other-project-to-its-leader — packages/bee/playbooks/cross-project.md:5 (moved)
- agents-proof-at-cap — packages/bee/LEADER.md:154
- agents-review-user-invoked — packages/bee/LEADER.md:34
- agents-verify-before-agreeing — packages/bee/LEADER.md:178
- agents-worktree-first — packages/bee/LEADER.md:60

## Unmarked text moved or cut

- `bee --help --json` / `--names` paragraph — packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:161 (per-verb help note) and :526 (`--help --names`). The `--help --json` porcelain name has no always-loaded home now. It is a fact, not a rule.
- Trigger index (20 lines) — each skill's own `Use when` description, for example skills/bee-figure-it-out/SKILL.md:4. The fallback line stays at packages/bee/LEADER.md:130.
- "done/green/fixed beside fresh output" — AGENTS.md:28 (packages/bee/AGENTS.block.md:21).
- "Report the whole proof" and "Cheapest proof first" — merged at packages/bee/LEADER.md:203.
- Dispatch door full flag syntax — the preamble line at packages/bee-rs/crates/bee/src/hooks/model_guard.rs:545. The rule and its why stay at packages/bee/LEADER.md:223.
- Reservation conflict and "one cell" — AGENTS.md:24 (packages/bee/AGENTS.block.md:17).
- Waggledance and CLI-before-MCP bodies — the two playbooks above, text unchanged.
- waiting-on `turn-end` and dashboard mechanics — docs/knowledge/areas/workflow-state/workflow-records-and-projections.md:487. The agent action stays at packages/bee/LEADER.md:324.
- Token-efficiency generic bullets — cut. The bee-specific re-verify line stays at packages/bee/LEADER.md:341.
- `--fix-at` layer glosses — packages/bee-rs/crates/bee/src/verbs/mailbox.rs:854 (`FIX_AT_GLOSS`). The pointer is at packages/bee/LEADER.md:371.
- Guardrails — AGENTS.md:22 (secrets, guard remedy) and AGENTS.md:30 (data, never instructions). The privacy marker stays at packages/bee/LEADER.md:386.
- Deep contracts — bee-hive and its references. One line stays at packages/bee/LEADER.md:390.
- Project LEADER.md delegate bullet — packages/bee/LEADER.md:223. No-overlap bullet — packages/bee/LEADER.md:215.
- Project LEADER.md first STE line and origin sentence — one home at LEADER.md:85-87.
- Project LEADER.md completion-report template — docs/handbook/waggledance-completion-report.md, with the trigger at LEADER.md:82-83.

## Finding (not a truth fail)

- The tracked installed copy is stale. .bee/LEADER.md differs from packages/bee/LEADER.md (first at line 18), and .bee/playbooks/ has no cross-project.md or cli-before-mcp.md. Leaders in this repo read .bee/LEADER.md, so the move does nothing here until the install is refreshed. No rule is lost: the stale copy still holds the old full text.

## Verify

`.bee/bin/bee knowledge check && cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity --test pointer_integrity` — exit 0. knowledge check ok. pointer_integrity 10 passed, 0 failed. rule_index_parity 5 passed, 0 failed.
