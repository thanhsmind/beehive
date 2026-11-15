# cite-knowledge-first — context

Request (verbatim): "Sửa đi". It answers the evaluation given the same
day. The owner asked: "Cậy có thể luồng làm việc đang truy suất nguồn sai
thứ tự kién trúc knowledge xem xem có nên chỉnh lại không". The trigger
was an answer about the trust-layers lint offer. That answer cited
`docs/history/trust-layers/CONTEXT.md`, but
`docs/knowledge/areas/onboarding/host-project-artifacts.md:322` held the
current rule. Backlog item: p-eb824507.

Scope: doctrine prose only. This feature does not change `bee decisions
search` output. That part of p-eb824507 stays open on the backlog.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md | suspect | The rule index. It holds the spoken line and the applied_at list of `agents-verify-before-agreeing` and `agents-knowledge-before-code`. |
| docs/knowledge/areas/hook-runtime/advisories-and-turn-control.md | suspect | B16, the knowledge-first nudge. It points to the rule block and fires only on a code read. |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee/LEADER.md` | The source of the leader doctrine. The rendered copy is `.bee/LEADER.md`. |
| `skills/bee-hive/references/routing-and-contracts.md` | "Claim labels and pushback", the full form of `agents-verify-before-agreeing`. |

## Locked Decisions

| ID | Decision | Reason |
|---|---|---|
| D1 | Source order for a cited claim. A claim about how something works now cites the `docs/knowledge` concept that holds it, then code `file:line`. `docs/history` is cited only for why a decision was made, or for its locked wording. A decision hit leads to the concept that cites its D-number before history is opened. The short form goes in rule `agents-verify-before-agreeing`. The full form goes in "Claim labels" in `routing-and-contracts.md`. | The citation rule ranked no source. The decision-to-history path skipped knowledge without breaking a rule. |
| D2 | The `packages/bee/LEADER.md` line on `docs/history/<feature>/CONTEXT.md` scopes "cite them" to the locked decision and its reason, and names the knowledge concept as the home of current behavior. Rule `agents-knowledge-before-code` also covers a turn that answers how something works: search knowledge before opening `docs/history` for that answer. | The unscoped "cite them" read as a general order to cite history. |

## Out of scope

- A hook or guard that refuses a history citation. A guard cannot tell a rationale claim from a behavior claim.
- `bee decisions search` naming the concept that cites a decision (p-eb824507, second part).
- A release. The owner did not ask for one.
