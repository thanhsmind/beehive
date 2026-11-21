# tsc-8 — parity of doctrine, rendered agents and concepts

Tree checked: `wt/token-spend-cuts` at d82a9d478. Every anchor below came from
`rg -n` on that tree. "Rendered" means the copy under `.claude/agents`,
`.claude/skills` or `.bee/playbooks`. Concepts under `docs/knowledge` have no
rendered copy, so only the source location is listed for them.

## Verify

The cell verify, run once:

```
PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- pinned_types agent_file_drift && ./.bee/bin/bee dev release-manifest --check && ./.bee/bin/bee knowledge check
```

Result lines:

- cargo test: `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 5570 filtered out; finished in 0.00s`
  (`pinned_types_match_the_rendered_bee_agents`, `agent_file_drift_reads_the_shared_role_table`,
  `opencode_agent_file_drift_findings`, `agent_file_drift_findings`)
- release manifest: `release_manifest --check: 990 file(s) match stored manifest, 1 unhashed artifact(s) present`
- knowledge check: `knowledge check: 330 concept(s) in 351 file(s), 0 OKF error(s), 0 profile error(s), 607 profile warning(s) — OK`
- chain exit code: 0

The `unknown_rule_ref` error that tsc-6 reported is gone (0 profile errors).
The 607 warnings are the same count tsc-6 saw and are stale-anchor and
missing-read-when warnings; this cell did not sort them. The manifest check
names one unhashed artifact; the check passes with it, and this cell did not
look up which file it is.

## Read rule: rendered agents against templates

Each template line was taken from the template and matched as a whole line
(`rg -F -x`) in its rendered agent. All six are byte-identical.

| Agent | Template | Rendered | Identical |
|---|---|---|---|
| bee-worker | packages/bee/agents/bee-worker.md.tmpl:15 | .claude/agents/bee-worker.md:15 | yes |
| bee-comb | packages/bee/agents/bee-comb.md.tmpl:15 | .claude/agents/bee-comb.md:15 | yes |
| bee-guard | packages/bee/agents/bee-guard.md.tmpl:13 | .claude/agents/bee-guard.md:13 | yes |
| bee-forager | packages/bee/agents/bee-forager.md.tmpl:13 | .claude/agents/bee-forager.md:13 | yes |
| bee-scout | packages/bee/agents/bee-scout.md.tmpl:13 | .claude/agents/bee-scout.md:13 | yes |
| bee-elder | packages/bee/agents/bee-elder.md.tmpl:13 | .claude/agents/bee-elder.md:13 | yes |

The line in every file: ``- Read rule: locate with `rg -n` first (the Grep tool where you have no shell), then Read a range with offset and limit; never dump a file with `cat`, `head`, `tail` or `sed`; read a whole file only when the work edits most of it or it is short.``

## tsc-5 must_haves

1. Rule agents-context-handoff-65 says: 300000 default, next clean stop, a
   fresh leader session after every green merge, never mid-cell. **Met.**
   - Source: packages/bee/playbooks/handoff.md:16 (marker), :17-18 (300000,
     next clean stop), :20-21 (HANDOFF.json, fresh leader session through the
     host's leader door, never stops mid-cell).
   - Rendered: .bee/playbooks/handoff.md:16-21, the same lines. `cmp` finds
     the two files identical.
2. Every skill line that said a mark never stops work, or a handoff only on
   the user's word or at feature close, matches the rule. **Met.**
   - skills/bee-hive/SKILL.md:23; rendered .claude/skills/bee-hive/SKILL.md:23
   - skills/bee-swarming/SKILL.md:167; rendered .claude/skills/bee-swarming/SKILL.md:163
   - skills/bee-hive/references/go-mode.md:115, :122; rendered at the same lines
   - skills/bee-swarming/references/execute.md:92; rendered at :92
   - skills/bee-swarming/references/swarming-reference.md:712; rendered at :666
   - skills/bee-hive/references/routing-and-contracts.md:72; rendered at :72
   - tsc-6 GAP A is closed: skills/bee-planning/playbooks/session/pause-safely.md:8-10
     now points to agents-context-handoff-65 for the next clean stop, and :31
     names the rule. The rendered copy has the same lines.
   - tsc-6 GAP B is closed: `rg -n "rule: agents-context-handoff-65,"` finds
     no match in skills, .claude/skills, packages/bee or .bee/playbooks; line
     167 now reads `(rule: agents-context-handoff-65; see ...)`.
   - A search for `never stops work`, `mark never stops`, `only on the user's word`
     and `only at feature close` finds nothing in skills, .claude/skills,
     packages/bee or .bee/playbooks. The only hits for "never stops" say "a
     context mark never stops a cell", which is the new rule.
   - Outside the skill tree: README.md:526 still says bee "shows the context
     size but never stops for it" and that the next session "waits". This is
     not a skill line, so it is not counted against the must_have. It is left
     for the leader (see Gaps).
3. Each of the six templates carries one read rule line. **Met.** See the
   table above.
4. skills/bee-planning/SKILL.md: test and design cells run on their role
   model, small design stays on the design role, hard ones are escalated at
   plan time. **Met.** skills/bee-planning/SKILL.md:173-176; rendered
   .claude/skills/bee-planning/SKILL.md:173-176.
5. Judge tier table and the bee-swarming judge paragraph: at standard, under
   30 lines owes no verdict; the tester returns the judge block; the leader
   records it with `bee cells judge-record`. **Met.**
   - skills/bee-hive/references/gates-and-delegation.md:394; rendered
     .claude/skills/bee-hive/references/gates-and-delegation.md:383
   - skills/bee-swarming/SKILL.md:96-99; rendered .claude/skills/bee-swarming/SKILL.md:92-95
6. bee dev regen ran with a binary built from this worktree and its rendered
   trees are committed. **Met.** Commits 34ca8718e and a5a15f285 carry the
   rendered trees. `git status` was clean before this report. The
   `agent_file_drift` and `pinned_types` tests and `release-manifest --check`
   pass in the verify above.

## tsc-6 must_haves

1. token-economy-of-injected-context.md states the 300000 default, the
   directive line and the merge next action, citing D1 and D4. **Met.**
   docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md:122
   (heading cites D1), :127 (300000; 0 turns it off), :128 (directive line
   text), :134 (merge next action), :139 (D4 paragraph); frontmatter :11
   lists token-spend-cuts D1 and D4.
2. model-roles-and-escalation.md states plan-time escalation for hard test and
   substantial design cells, citing D2. **Met.**
   docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md:379-383
   (B25); frontmatter :11 lists token-spend-cuts D2.
3. cells-completion-judge-and-archive.md states `trace.changed_lines` and the
   standard-lane skip under 30 lines, citing D5. **Met.**
   docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md:78-82,
   :370; frontmatter :11 lists token-spend-cuts D5.
4. The tsc-6 report lists each tsc-5 must_have with file:line or names the
   gap. **Met.** docs/history/token-spend-cuts/reports/tsc-6.md lists all six
   and names GAP A and GAP B. Both gaps are now fixed (see tsc-5 item 2).

## tsc-7 must_haves

1. One paragraph under Behaviors & Operations states the read rule and cites
   D3. **Met.**
   docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md:162-169.
   The section runs from `## Behaviors & Operations` at :27 to
   `## Business Rules` at :180, so the paragraph is inside it. Its wording
   matches the template rule.
2. Frontmatter decisions carry D3, and Pointers name
   packages/bee/agents/bee-worker.md.tmpl. **Met.** Frontmatter :11 has
   `token-spend-cuts D3 (...)`; Pointers (from :216) name the template at :261.

## Gaps

No must_have of tsc-5, tsc-6 or tsc-7 is unmet.

One item outside the must_haves: README.md:526 still says bee never stops
for the context size and that the next session waits. That contradicts D1.
README is not a skill or a template, and this cell may not edit it. It needs
a one-sentence edit if the owner wants the README to match.
