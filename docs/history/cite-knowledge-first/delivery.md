# cite-knowledge-first — delivery

## What changed

- `packages/bee/LEADER.md`: the `docs/history/<feature>/CONTEXT.md` line now says to cite it for a locked decision and its reason, and names the `docs/knowledge` concept as the home of current behavior (D2). Rule `agents-knowledge-before-code` gains one sentence: the same search comes first when a turn answers how something works, before `docs/history` is opened (D2). Rule `agents-verify-before-agreeing` gains the source order after the Sources sentence (D1).
- `skills/bee-hive/references/routing-and-contracts.md`: a "Source order" paragraph after "Claim labels" with the D1 order, the decision-hit step and one example that cites a `docs/knowledge` path for a behavior claim and a `docs/history` CONTEXT.md D-number for a reason (D1).
- `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`: the spoken lines of both rules carry the new text.
- Rendered by `bee dev regen`, never by hand: `.bee/LEADER.md`, five copies of `routing-and-contracts.md`, the render manifests, `.bee/onboarding.json` and the release manifest.

No new rule id. No hook or guard change.

## Proof

- `.bee/LEADER.md` is byte-equal to `packages/bee/LEADER.md` after regen (`cmp`).
- The cell verify command: `rule_index_parity`, `pointer_integrity` and `instruction_laws` tests, `bee dev release-manifest --check`, `bee onboard --repo-root . --json`.

## Known limit

`bee decisions search` still names no concept that cites a decision. That part of p-eb824507 stays open on the backlog.
