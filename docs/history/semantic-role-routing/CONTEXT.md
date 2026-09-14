# Semantic Role Routing — Context

**Feature slug:** semantic-role-routing
**Date:** 2026-09-14
**Shaping session:** complete
**Scope:** small

## Feature Boundary

Make role assignment a formal planning-time output: the planner reads the
workflow stages the request needs and the complete team role descriptions,
assigns each stage and job to the semantically fitting role before execution,
and execution dispatches must follow that approved assignment. A newly
discovered stage during execution requires an explicit re-route record, never
an ad hoc role choice.

This changes planning workflow (how plan.md carries role assignments), dispatch
discipline (dispatches follow the approved plan), and adds a re-route record
format. The open set of roles, their descriptions, fall-through, escalation,
and transport selection are all unchanged — only the *assignment* step moves
from dispatch-time leader prose to planning-time structured output.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Role assignment is a planning output at task start. The planner inspects workflow stages and team role descriptions, assigns each stage and job to the semantically fitting role before execution. Execution dispatches follow that approved assignment. A newly discovered stage requires an explicit re-route record, never an ad hoc role choice. | The user identified that prose-only guidance lets the leader ignore the roster and choose at dispatch time. Planning-time assignments make role choice reviewable and prevent transport or habit from becoming the selector. |

*Source: decision fc2bb09a, logged 2026-09-14T06:25:25Z*

## Existing Code Context

### Reusable Assets

- `.bee/expertise/planning.md` — current planning-flow guidance
- `docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md` — role open set, fall-through, escalation, descriptions
- `.agents/skills/bee-planning/` — planning skill, references, templates
- `docs/history/<feature>/plan.md` — plan structure per feature (template at `.agents/skills/bee-planning/references/`)

### Established Patterns

- Cells carry a `role` field that resolves to a model through config (model-roles B3/B4)
- dispatch prepare resolves roles by name through config
- The planner already writes plan.md with section structure

### Integration Points

- Planning flow (plan.md sections or instructions need a role-assignment step)
- dispatch prepare (may need a plan-check assertion)
- Decision log (re-route records)

## Outstanding Questions

### Deferred To Planning

- [ ] Where does the role assignment live in plan.md? — the technical shape of the assignment block
- [ ] How does dispatch prepare validate against the plan assignment? — enforcement mechanism
- [ ] What is the re-route record format? — decision log entry shape

## Deferred Ideas

None.

## Handoff Note

CONTEXT.md is the source of truth. Decision D1 is fixed. Planning reads
this context, the model-roles-and-escalation area spec, and the existing
plan templates.