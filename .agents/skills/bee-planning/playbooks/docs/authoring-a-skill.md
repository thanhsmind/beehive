# authoring-a-skill

You own the skill's voice. Agent-facing prose has a higher bar than human
prose: an unhelpful sentence becomes an instruction, because the agent reading
it follows it.

1. Author the skill through `bee-writing-skills`. It holds the discipline for
   writing a skill and pressure-testing it, so this playbook does not restate
   it and the two cannot drift apart.
2. Validate the skill: the frontmatter has `name` and `description`, every
   file the skill references exists, and every cross-skill link resolves. A
   skill that points at a missing file fails silently in the session that
   loads it.
3. Add test cases when the skill is structural; skip them when it is
   subjective. A test pins behavior that can be checked, and prose tone cannot.
4. Land it through the bee close and worktree merge path: the work is capped,
   merged by `bee worktree merge`, and the `uat` acceptance at `bee close`
   stays the owner's, because a skill changes how every later agent behaves and
   the owner decides when that is accepted.

When in doubt, delete; prose earns its keep by changing a decision. Tell the
agent to do the thing and skip the reason; explain only when the rule is
confusing without one. Match tone to scope. Point at structural sources
(types, READMEs, config) because hardcoded details go stale
(`bee-principle-encode-lessons-in-structure`). Delegate to other skills by
path and do not restate them, so one fact keeps one home. A workflow you keep
hitting that no skill captures is a new skill to propose.

Reply: summary of the skill, key design decisions, validation notes.

Proof line: the validation that ran (frontmatter, referenced files, links) and
its output.
Proof floor: the cap refuses a result weaker than this class's floor and names it; the table is the cap's own (`bee cells finish --help`).
