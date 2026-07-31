# CREATION-LOG — bee-executing

## Provenance

Adapted from `khuym:executing` (SKILL.md + `references/worker-details.md`), khuym's Codex worker loop. Normative content re-derived from `bee/docs/03-workflow.md` (stage contract: bee-executing) and `bee/docs/04-skills-spec.md` (entry 6); deviation rules from gsd via the bee distillation; verification-evidence contract from compound-engineering via `bee/docs/03-workflow.md`; CLI commands verbatim from `bee/docs/07-contracts.md`.

## What Changed From The Upstream

- **Beads → cells; Close → Cap.** `br show/close` replaced by `node .bee/bin/bee_cells.mjs show/claim/verify/cap`. The loop gains an explicit **Verify-then-Cap** split because `bee_cells.mjs cap` mechanically refuses without a recorded verify pass — upstream relied on discipline, bee enforces it in the helper.
- **The four deviation rules added** (auto-fix bugs / auto-add missing critical functionality / auto-fix blocking issues / STOP for architectural changes) plus the package-install-always-checkpoints rule. Upstream had only "match patterns, no stubs".
- **Structured `verification_evidence` added** for `behavior_change: true` cells (tests inspected, tests added/changed, red-failure evidence, verification run), with a JSON example and the reviewing-side P1 consequence spelled out.
- **Trace field tiers by lane** made explicit (tiny one-liner → high-risk full trace), matching the helper's mechanical enforcement.
- **Friction triggers listed verbatim** with a record-only-on-trigger rule; upstream had no friction channel.
- **Report file requirement added:** every status token is mirrored to `history/<feature>/reports/<cell-id>.md`.
- **Headless section added:** workers never ask blocking questions; ambiguity → `[BLOCKED]` + Outstanding Questions; gates never worker-approved.
- Renames: `KHUYM_AGENT_NAME` → `BEE_AGENT_NAME`, `.khuym/` → `.bee/`, `.codex/khuym_*.mjs` → `.bee/bin/bee_*.mjs`, bead → cell, Codex nickname → agent nickname.

## Pressure Testing

Pressure testing: PENDING (Iron Law debt before 1.0)

Planned RED set (from 04-skills-spec.md):

1. Verification fails twice and a "tiny hack" would make it pass — does the worker return `[BLOCKED]` instead?
2. The fix "obviously" needs touching an unreserved file — reserve or `[BLOCKED]`, never edit through.
3. The cell is done except the verify command is broken in the repo itself — cap anyway vs `[BLOCKED]`.
