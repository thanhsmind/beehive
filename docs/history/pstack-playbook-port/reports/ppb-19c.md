# ppb-19c — bee-reflect checked against its pstack source

Scope: what ppb-19 wrote in commit `7d06966ab`. That covers `skills/bee-reflect/`
(`SKILL.md`, `LICENSE.pstack`, `references/judgment-reviewer.md`,
`references/tooling-reviewer.md`, `references/divergent-reviewer.md`,
`references/synthesizer.md`) and the port notes
`docs/history/pstack-playbook-port/port-notes/skill-bee-reflect.md`. They were
checked against the pstack skill `pstack/skills/reflect/` (`SKILL.md` and the same
four reference files). `SKILL.md` was compared line by line. The four references
and the licence were compared with `diff`. The five rendered copies
(`.claude-plugin`, `.codex-plugin`, `.claude`, `.agents`, `.opencode`) were
compared with `diff -rq` and all match the source tree.

Verdict: no source step is missing. All six process steps, the "When to invoke"
list, the skip rule, the three lenses with their scan lists, the
untrusted-transcript paragraphs, the skill-was-used scope rule, the synthesizer's
eight criteria and its Accepted / Rejected / Backlog format are present. Every
removed or swapped item has a port-notes entry. `LICENSE.pstack` is identical to
the pstack `LICENSE`. The pstack-only tool check is clean. No model and no
subagent type is picked by hand. The description is 282 characters and says when
not to use the skill. There is one finding, P3, on wording in the reference
prompts. It does not change any source step.

Labels: **present** means the step is in the new file. **swapped (noted)** means
it is present with a bee tool in place of the pstack tool, and the port notes name
the swap. **dropped (noted)** means it is gone and the port notes name it.
**dropped (not noted)** means it is gone and the notes say nothing; D2
(`063a21e4`) counts that as missing. **missing** means a method step is gone with
no reason.

## `SKILL.md`

| Source step or rule (source line) | New home (bee-reflect line) | Status |
|---|---|---|
| Frontmatter `name: reflect` (2) | `name: bee-reflect` (2) | present |
| Frontmatter description (3) | 4, rewritten: what it does, when, when not | present |
| Frontmatter `disable-model-invocation: true` (4) | removed | dropped (noted: frontmatter paragraph) |
| Intro: mine the conversation, route into skill edits (9) | 18, "recorded edits", plus "Nothing is applied before the user approves it" | present |
| When: user said "reflect" or "/reflect" (13) | 22, `/bee-reflect` | swapped (noted) |
| When: complex task (5+ tool calls) landed cleanly (14) | 23 | present |
| When: dead ends, then a working path that generalizes (15) | 24 | present |
| When: user corrected the approach mid-task (16) | 25 | present |
| When: non-trivial workflow not captured anywhere (17) | 26 | present |
| Skip rule: trivial, off-topic, already covered; one-offs are not learnings (19) | 28 | present |
| Step 1: parent finds its own transcript before fanning out (25) | 36 | present |
| Step 1: take the directory from the system prompt's `agent-transcripts/` (25) | 36, "the path the host or the system prompt names" | dropped (noted: "Dropped steps", first bullet), generalized |
| Step 1: never glob across other projects; cross-workspace privacy reason (25) | 36 | present |
| Step 1: `ls -t <agent-transcripts>/...` command (27-29) | removed | dropped (noted: "Dropped steps", first bullet) |
| Step 1: three transcript layouts (31) | removed | dropped (noted: "Dropped steps", first bullet) |
| Step 1: first-line check against the opening prompt; take the match (33) | 38, "read the first line" instead of `message.content[0].text` | present (the host-specific JSON path is covered by the same notes bullet) |
| Step 1: no path resolves, so pass a digest (33) | 38 | present |
| Step 2: one message, three parallel reviewers (37) | 42, three `bee dispatch prepare` calls in one turn, run in the background | swapped (noted: "Tool swaps", first bullet) |
| Step 2: `Task`, `subagent_type: generalPurpose`, explicit `model:` (37) | 42, "Never pick a model or a subagent type by hand" | swapped (noted) |
| Step 2: agent mode `readonly: false`, because reviewers need MCP and readonly strips it (37) | 50, "uses whatever tools its payload gives it" | dropped (noted: "Tool swaps", third bullet) |
| Step 2: the prompt forbids file writes; the parent applies edits (37) | 50 | present |
| Step 2: lens table with a model per lens and default slugs (39-43) | 44-48, lens and template only; the model comes from the team table's review role (43) | swapped (noted: "Tool swaps", second bullet) |
| Step 2: pass each template verbatim, substitute path or digest (45) | 50 | present |
| Step 2: reviewers return findings in the response (45) | 50 | present |
| Step 3: one synthesizer call, judgment model, `readonly: false` (49) | 58, one `--kind reviewer` payload run as is | swapped (noted: "Tool swaps", first and third bullets) |
| Step 3: quality check spot-verifies citations (49) | 58 | present |
| Step 3: use `synthesizer.md` verbatim with reviewer outputs inlined (49) | 58 | present |
| Step 3: returns Accepted / Rejected / Backlog (49) | 58 | present |
| Step 4: move mechanism-enforceable items from Accepted to Backlog (53) | 62 | present |
| Step 4: the synthesizer already applies it; final pass (53) | 62 | present |
| Step 4: see the encode-lessons-in-structure principle skill (53) | 62, `bee-principle-encode-lessons-in-structure` (exists in `skills/`) | swapped (noted: last "Tool swaps" bullet) |
| Step 5: present the full output, wait for explicit approval; user picks the subset and may redirect; never auto-apply (57) | 66 | present |
| Step 5: Backlog files to the team's tracker automatically; tracker items, not skill edits; only Accepted waits (59) | 68, `bee backlog add` | swapped (noted: "Tool swaps", fourth bullet) |
| Step 5: follow the Routing field exactly (61) | 70 | present |
| Step 5: trivial edit, the parent does it directly (63) | 72 | present |
| Step 5: substantive edit, hand to `create-skill` draft / test / iterate (64) | 73, `bee-writing-skills` draft, pressure-test, iterate | swapped (noted: "Tool swaps", fifth bullet) |
| Step 5: `tune description:`, hand to `create-skill` description loop (65) | 74, `bee-writing-skills` | swapped (noted) |
| Step 5: `new skill via create-skill:`, never invent the shape ad hoc (66) | 75, `new skill via bee-writing-skills:` | swapped (noted) |
| Step 5: run the SKILL.md validator if the environment ships one (68) | 77 | present |
| Step 6: short list, no preamble (72) | 88 | present |
| Step 6: Edits applied line (74) | 90 | present |
| Step 6: New skills created line (75) | 91 | present |
| Step 6: Backlog filed to the devex tracker line (76) | 93, "Backlog filed" | swapped (noted) |
| Step 6: Dropped line, one per rejected finding plus reason (77) | 94 | present |

Additions with no source counterpart, each named in the port notes ("Added, not in
the source"): the `metadata` block (frontmatter paragraph); the neighbouring-doors
paragraph at line 30; the re-dispatch and uncovered-lens rule at line 52; the
same-model note at line 54 ("Tool swaps", second bullet); the Record block at
lines 79-82; the framework paragraph at line 84; the "Recorded" summary line at
line 92; the Handoff section at lines 96-98. The cited names exist:
`bee-capturing`, `bee-writing-skills`, `bee-shaping`, `bee-planning`,
`bee-swarming`, `bee-hive` and `bee-principle-encode-lessons-in-structure` under
`skills/`. The rule `agents-feedback-changes-framework` appears in
`packages/bee/LEADER.md` and `skills/bee-capturing/SKILL.md`. The
`bee mailbox reflect --wrong --better --fix-at` shape matches
`packages/bee/LEADER.md:395`.

## Reference files

`diff` between each source reference and the bee copy shows only the lines below.
Every other line, including every scan list, finding shape, criterion and output
template, is identical.

| File | Source line | Change | Status |
|---|---|---|---|
| `judgment-reviewer.md` | 3 | "any MCP tool available in your environment" became "any tool your payload gives you" | swapped (noted: "Tool swaps", third bullet) |
| `judgment-reviewer.md` | 22 | `.cursor/skills/` and plugin paths became the repo's `skills/`, rendered dirs, user-level and plugin paths | swapped (noted: "Tool swaps", seventh bullet) |
| `judgment-reviewer.md` | 23 | "`Task` prompts" became "Dispatch prompts" | swapped (noted) |
| `tooling-reviewer.md` | 3, 37, 38 | the same three swaps | swapped (noted) |
| `divergent-reviewer.md` | 5, 23, 24 | the same three swaps | swapped (noted) |
| `synthesizer.md` | 1 | "any MCP tool available" became "any tool your payload gives you" | swapped (noted) |
| `synthesizer.md` | 17, 44 | `create-skill` became `bee-writing-skills` | swapped (noted) |
| `synthesizer.md` | 27 | "Bugbot flagged ..." became "a review flagged ..." | swapped (noted: "Tool swaps", eighth bullet) |
| `synthesizer.md` | 56 | the team's backlog tracker became `bee backlog add` | swapped (noted) |

The source lists eight synthesizer criteria (durability, specificity,
existing-skill-first, convergence, decision-changing, structural-mechanism,
skill-was-used, already-covered), and all eight are present. The port notes say
"nine criteria". The likely reason is that the notes count the nine Rejected
reason codes. The miscount is in the notes only, and the skill is not affected.

## `LICENSE.pstack`

`diff` against the pstack `LICENSE` prints nothing.

## Pstack-only tool names

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-reflect/` prints
nothing and exits 1. The source names Cursor (`~/.cursor/projects`, `.cursor/skills/`,
Cursor's `create-skill`), the `Task` tool and Bugbot. All of them are swapped or
dropped, as noted. The source cites neither Graphite, `poteto-agent`, arena,
architect, how nor why, which matches the notes. "MCP" is still in the reference
prompts. It is a generic protocol name, not a pstack tool.

## Hand-picked model or subagent type

None. `rg -n -i 'subagent_type|model'` over `skills/bee-reflect/` finds three
lines: `SKILL.md:42` (the rule never to pick a model or subagent type by hand),
`SKILL.md:54` (the same-model note) and `synthesizer.md:28` (the verbatim
"drop this drifting detail" example `gpt-4`/`encodingForModel`, which is example
text, not a dispatch choice). Every dispatch goes through
`bee dispatch prepare --runtime claude --kind reviewer --json`, the same form
`bee-interrogate` and `bee-fanout` use.

## Description

The folded `description:` value is 282 characters, which matches the port notes
and fits the 300-character limit. It says what the skill does, when to use it
("the user types /bee-reflect or says reflect or retrospective") and when not to:
"Not for a mistake noticed mid-task (bee mailbox reflect) or a rule just settled
(bee-capturing)."

## Findings

1. **P3: the untrusted-transcript guard still reads "MCP" after the tool line was
   widened.** In all four references, the access line now says "Use any tool your
   payload gives you" (for example, `judgment-reviewer.md:3`). The guard paragraph
   right after it still says "Confine MCP lookups to context the transcript
   references" (`judgment-reviewer.md:5`, `tooling-reviewer.md:5`,
   `divergent-reviewer.md:7`, `synthesizer.md:3`). In the source these two lines
   matched, because MCP was the only lookup path. In the port, a reviewer could
   look things up through a non-MCP tool, such as a shell `gh` call, and read the
   guard as not covering that tool. Suggested fix: change "Confine MCP lookups" to
   "Confine lookups" in the four files. The change is outside this cell's scope,
   so the leader decides. It does not change the step table.

## Verify

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-reflect/ ; test $? -eq 1`
printed nothing, and `test` exited 0.
