# tsc-6 — knowledge sync and tsc-5 parity

## Knowledge concepts updated in place

| Concept | Paragraph | Decisions cited |
|---|---|---|
| docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md | the prompt-context handoff paragraph rewritten (300000 default, directive line, merge next action); a new paragraph for the knowledge pack budget | token-spend-cuts D1, D4 |
| docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md | new B25 after B24: role-model test and design cells, plan-time escalation, stack-neutral role text | token-spend-cuts D2 |
| docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md | the judge-debt door paragraph and B32 gain `trace.changed_lines` and the standard-lane skip under 30 lines | token-spend-cuts D5 |

Each concept's frontmatter `decisions` list carries the token-spend-cuts D-numbers, `sources` names the history context, and `anchors` gained the code files the new text describes (phases.rs, prepare.rs, templates.rs, close.rs, and skills/bee-planning/SKILL.md).

## tsc-5 parity: each must_have against the files

1. Rule agents-context-handoff-65 — met.
   - Source: packages/bee/playbooks/handoff.md:16-30 (marker line 16; 300000 and the next-clean-stop rule at 17-20; fresh leader session after every green merge at 19-21; never mid-cell at 21).
   - Rendered: .bee/playbooks/handoff.md:15-20 holds the same text; sha256 of both files is d495eab0…ebd9, identical.
2. Skill lines match the rule — met for the six listed files, gap beyond them.
   - skills/bee-hive/SKILL.md:23
   - skills/bee-swarming/SKILL.md:167
   - skills/bee-hive/references/go-mode.md:115 and :122
   - skills/bee-swarming/references/execute.md:91-92
   - skills/bee-swarming/references/swarming-reference.md:712 and :741-743
   - skills/bee-hive/references/routing-and-contracts.md:72-74
   - Rendered copies of bee-swarming and bee-hive carry the same lines under .agents/skills, .claude/skills, .claude-plugin/skills, .codex-plugin/skills and .opencode/skills.
   - GAP A: skills/bee-planning/playbooks/session/pause-safely.md:8 still says "a context mark never stops work, so when compaction is the only trigger, write the note in step 4 to a file and keep working". The must_have says every skill line that said this now matches the rule; the file is outside the six listed ones and tsc-5 did not touch it. The same line is rendered in .agents, .claude, .claude-plugin, .codex-plugin and .opencode skills. README.md:526 also says bee "never stops for" the context size (outside the skill tree).
   - GAP B: skills/bee-swarming/SKILL.md:167 writes `(rule: agents-context-handoff-65, \`.bee/playbooks/handoff.md\`)`. The knowledge check reads the id as `agents-context-handoff-65,` and reports `ERROR [unknown_rule_ref] skills/bee-swarming/SKILL.md: rule reference "(rule: agents-context-handoff-65,)" matches no homed rule marker`. This is the only error of the verify. The other rule references in the six files use `(rule: agents-context-handoff-65)` or `(rule: \`agents-context-handoff-65\`)`.
3. Read rule in the six templates — met. packages/bee/agents/bee-worker.md.tmpl:15, bee-comb.md.tmpl:15, bee-guard.md.tmpl:13, bee-forager.md.tmpl:13, bee-scout.md.tmpl:13, bee-elder.md.tmpl:13. Rendered at .claude/agents/bee-worker.md:15, bee-comb.md:15, bee-guard.md:13, bee-forager.md:13, bee-scout.md:13, bee-elder.md:13, and the Read rule line of each is identical to its template.
   - Note: .codex/agents holds five other agents (bee-build, bee-design, bee-extract, bee-gather, bee-review as .toml) with no read rule. They are not among the six templates and nothing in the plan renders them from these templates, so this is not counted as a gap.
4. skills/bee-planning/SKILL.md plan-time escalation — met. skills/bee-planning/SKILL.md:173-176 ("Test and design cells run on their role model. Small design, such as polish and tiny UI fixes, stays on the design role. A hard test cell or a substantial design cell is marked with `bee cells escalate` at plan time."). Rendered at lines 173-174 of the bee-planning SKILL.md in .agents, .claude, .claude-plugin, .codex-plugin and .opencode.
5. Judge tier table and judge paragraph — met.
   - skills/bee-hive/references/gates-and-delegation.md:394 (standard row: under 30 lines owes no verdict; the tester returns the judge block; the leader records it with `bee cells judge-record`).
   - skills/bee-swarming/SKILL.md:96-100.
   - Rendered in .agents, .claude, .claude-plugin, .codex-plugin and .opencode copies of both files.
6. bee dev regen ran and its rendered trees are committed — met. Commit 34ca8718e carries the rendered .agents, .claude, .claude-plugin, .codex-plugin and .opencode trees, .bee/playbooks/handoff.md and docs/history/codex-harness-hardening/release-manifest.json. The manifest entry for packages/bee/playbooks/handoff.md is at release-manifest.json:5100-5104 and its sha256 equals the file's current sha256. I did not re-run `bee dev release-manifest --check`: the brief allows only the cell verify.

## Verify

`./.bee/bin/bee knowledge check` on this tree after the three concept edits:

```
ERROR [unknown_rule_ref] skills/bee-swarming/SKILL.md: rule reference "(rule: agents-context-handoff-65,)" matches no homed rule marker in the bundle or AGENTS.md (D4)
knowledge check: 330 concept(s) in 351 file(s), 0 OKF error(s), 1 profile error(s), 607 profile warning(s) — FAIL
```

The one error is GAP B in a skill file, which this cell may not edit. None of the three concepts raises an error. The 607 warnings are stale-anchor and similar; I did not compare that count with the count before this cell.

## Fix for the leader

- GAP B: in skills/bee-swarming/SKILL.md:167 write `(rule: agents-context-handoff-65; \`.bee/playbooks/handoff.md\`)` or drop the comma, then run `bee dev regen` so the five rendered copies follow.
- GAP A: in skills/bee-planning/playbooks/session/pause-safely.md:8 replace the "a context mark never stops work" clause with a pointer to rule agents-context-handoff-65, then regen. README.md:526 needs the same change if the owner wants it.
