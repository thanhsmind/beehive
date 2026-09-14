---
artifact_contract: bee-plan/v1
mode: small
---

# Plan: Semantic Role Routing

## Summary
Make role assignment a formal planning-time output: the planner inspects workflow stages and team role descriptions, assigns each to the semantically fitting role before execution, and dispatches follow the approved assignment. A newly discovered stage gets a re-route record.

## Requirements (from CONTEXT.md)
- D1 (fc2bb09a): Role assignment is a planning output at task start. The planner reads workflow stages and team role descriptions, assigns each stage and job to the semantically fitting role before execution. Execution dispatches follow that approved assignment. A newly discovered stage requires an explicit re-route record, never an ad hoc role choice.

## Load-bearing claims
Every claim is `read` — the files below exist and are the ones to change.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | bee-planning skill exists | read | .agents/skills/bee-planning/SKILL.md | always present |
| 2 | planning-reference exists | read | .agents/skills/bee-planning/references/planning-reference.md | always present |
| 3 | swarming-reference exists | read | .agents/skills/bee-swarming/references/swarming-reference.md | always present |
| 4 | AGENTS.md exists | read | AGENTS.md | always present |
| 5 | model-roles-and-escalation.md exists | read | docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md | always present |

## Approach
**Recommended path:** docs-only updates to four files (planning skill, planning reference, AGENTS.md, swarming reference) to implement D1, then sync knowledge model-roles-and-escalation.md with B18. No code change — the cell role field already exists.

**Rejected alternatives:** code-level dispatch-prepare enforcement (costs more, no new capability since the cell role field already constrains dispatches); new dispatch kind (unnecessary, role overrides already handle it).

## Shape

### Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|----|-------|-------|------|---------|-------|
```json
[
  {"id":"slr-1","feature":"semantic-role-routing","role":"docs","lane":"small","title":"Add role assignment as a planning step and dispatch-follows-plan rule","action":"Update planning documentation and dispatch discipline to implement D1 (fc2bb09a): (1) Add role assignment to bee-planning/SKILL.md Shape step — after locked decisions are cited, planner reads team role descriptions via bee team show, assigns each stage/job to the semantically fitting role, and records the assignment in plan.md or the scoping synthesis. (2) Add Role assignments section to planning-reference.md plan.md template. (3) Add dispatch-follows-plan rule to AGENTS.md — dispatches must follow the cell's recorded role; a re-route requires a logged decision with tag role-reroute. (4) Update swarming-reference.md to cite cell role as the plan-approved assignment.","verify":"grep -q 'role assignment' .agents/skills/bee-planning/SKILL.md && grep -q 're-route' AGENTS.md","read_first":[],"files":[".agents/skills/bee-planning/SKILL.md",".agents/skills/bee-planning/references/planning-reference.md","AGENTS.md",".agents/skills/bee-swarming/references/swarming-reference.md"],"deps":[],"decisions":["fc2bb09a"],"must_haves":{"truths":["Planning SKILL.md names role assignment as a required step after locked decisions are cited.","planning-reference.md plan.md template includes a Role assignments section.","AGENTS.md carries a rule that dispatches follow the approved plan role assignment; a re-route needs a logged decision (tag role-reroute).","swarming-reference.md cites the cell role as the plan-approved assignment."]},"affects_skills":["skills/bee-planning/SKILL.md","skills/bee-swarming/references/swarming-reference.md"],"affects_specs":["docs/specs/doctrine-layer.md"],"acceptance":"bee-planning names role assignment as planning step; AGENTS.md has dispatch-follows-plan rule; swarming-reference cites cell role as plan assignment; re-route format defined."},
  {"id":"slr-2","feature":"semantic-role-routing","role":"docs","lane":"small","title":"Sync doctrine-layer knowledge with B18: plan-time role assignment and re-route records","action":"Capture the settled behavior: add B18 to model-roles-and-escalation.md — plan-time role assignment is a planning output (citing D1 fc2bb09a), dispatches follow the plan, re-route records go in the decision log with tag role-reroute.","verify":"grep -q 'B18' docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md","read_first":[],"files":["docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md"],"deps":["slr-1"],"decisions":["fc2bb09a"],"must_haves":{"truths":["model-roles-and-escalation.md has B18 covering plan-time role assignment.","B18 cites D1 (fc2bb09a)."]},"affects_skills":[],"affects_specs":["docs/specs/doctrine-layer.md"],"acceptance":"model-roles-and-escalation.md carries B18 with plan-time role assignment, re-route discipline, and cites D1."}
]
```

| slr-1 | Add role assignment as a planning step and dispatch-follows-plan rule | .agents/skills/bee-planning/SKILL.md, .agents/skills/bee-planning/references/planning-reference.md, AGENTS.md, .agents/skills/bee-swarming/references/swarming-reference.md | — | Planning SKILL.md requires role assignment after locked decisions; AGENTS.md carries dispatch-follows-plan rule; re-route format defined | grep -q 'role assignment' .agents/skills/bee-planning/SKILL.md && grep -q 're-route' AGENTS.md |
| slr-2 | Sync doctrine-layer knowledge with B18 | docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md | slr-1 | model-roles-and-escalation.md has B18 covering plan-time role assignment | grep -q 'B18' docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md |

## Test matrix
Docs-only change. No runtime tests needed. Verify: `bee team show` still works, `bee dispatch prepare --help` still works, no regressions in skill rendering.

## Open Questions
None.

## Out of scope
dispatch-prepare code-level enforcement of plan role assignments; plan.md template rendering changes beyond the reference doc.