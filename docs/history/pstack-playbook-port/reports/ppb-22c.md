# ppb-22c — bee-comment-review checked against its pstack source

Scope: what ppb-22 wrote in commit `071e12824`: `skills/bee-comment-review/`
(`SKILL.md`, `references/comment-reviewer.md`, `LICENSE.pstack`) and the port notes
`docs/history/pstack-playbook-port/port-notes/skill-bee-comment-review.md`. They
were checked against the pstack skill `pstack/skills/no-comments/SKILL.md` and the
pstack agent `pstack/agents/comment-sicko.md` at pstack commit `b9ddc83`. The skill
has no reference files of its own. Both files were compared line by line, and the
licence with `cmp`. The five rendered copies (`.claude-plugin`, `.codex-plugin`,
`.claude`, `.agents`, `.opencode`) were compared with `diff -rq`, and all of them
match `skills/bee-comment-review/`.

Verdict: no source step is missing. Every step and rule of the skill and the agent
is present, swapped with a port-notes entry, or dropped with a port-notes entry.
The largest change is that the reviewer is read-only, so the lead applies
deletions instead of restoring them. The port notes record it under "Departures",
and the change follows from bee's reviewer kind, which never edits the working
tree. `LICENSE.pstack` is identical to the pstack `LICENSE`. The pstack-only tool
check is clean. No model and no subagent type is picked by hand. The description
is 276 characters and says when not to use the skill. There are no findings.

Labels: **present** means the step is in the new file. **swapped (noted)** means
it is present with a bee tool or term in place of the pstack one, and the port
notes name the swap. **changed (noted)** means the step's mechanics changed and the
port notes record the change as a departure. **dropped (noted)** means it is gone
and the port notes name it. **missing** means a method step is gone with no reason.

## Skill: `no-comments/SKILL.md` → `bee-comment-review/SKILL.md`

<!-- bee:not-a-deferral: a record or description of work already done, never a promise to act -->
| Source step or rule (source line) | New home (bee-comment-review line) | Status |
|---|---|---|
| Frontmatter `name: no-comments` (2) | `name: bee-comment-review` (2) | swapped (noted: "Name") |
| Frontmatter description (3) | 3-4, rewritten: what it does, when, when not | present |
| Frontmatter `disable-model-invocation: true` (4) | removed | dropped (noted: frontmatter paragraph) |
| Spawn the reviewer, act on accepted findings (9) | 18 | present |
| Authoring agents defend comments; defer to the fresh perspective (11) | 20 | present |
| Scope: caller's files or diff, else the diff against base `main` with the working tree (15) | 26 | present |
| Step 1: spawn `Task` with `subagent_type: "Comment Sicko"` (19) | 30, `bee dispatch prepare --runtime claude --kind reviewer --json`, no `--role` | swapped (noted: "Tool swaps", first bullet) |
| Step 1: pass the scope, do not restate its rules (19) | 30, brief is the reference plus the scope, no verdict | present (noted: "Tool swaps", second bullet) |
| Step 2: inspect report and diff (20) | 31, "report against the files" | changed (noted: "Departures", read-only reviewer) |
| Step 2: reject application-code edits, scope escapes, exception-protected deletions, misstated `MUST KILL` reasons, flags against kept intentional code (20) | 31 | present |
| Step 2: reshape flags on our-code surprises stay actionable; do not restore those comments (20) | 31, "Do not keep those comments" | changed (noted: "Departures") |
| Step 2: a keep survives only with proof it is about something we cannot change (20) | 31 | present |
| Step 2: audit missed scoped lint and TypeScript suppressions (20) | 31, "lint and type-checker suppressions" | present |
| Step 2: correctness or safety suppressions stay actionable `MUST KILL`s (20) | 31 | present |
| Step 2: restore deletions only with exact exceptions and scoped proof (20) | 31, "Refuse a deletion only with the exact exception ... and scoped proof" | changed (noted: "Departures") |
| Step 2: run `/how` or `/why` before accepting thin `IMPORTANT`/`do not remove` kills or keeps (20) | 31, `bee-how` or `bee-why` | swapped (noted: "Tool swaps", third bullet) |
| Step 2: ambiguous kill not restored; refuted or ambiguous keep deleted (20) | 31, "do not apply it" | present |
| Step 2: revert and rerun one rejected report; reject a second, report open, fail `/no-comments` (20) | 31, "Re-dispatch", fail `bee-comment-review` | changed (noted: "Departures"; "Tool swaps", fifth bullet) |
| Step 3: fix trivial flags directly (dead path, parameter, real API) (21) | 32 | present |
| Step 3: a shape-needing fix runs `/architect` once, stop at the sketch (21) | 32, `bee-architect` | swapped (noted: "Tool swaps", fourth bullet) |
| Step 4: smallest root-cause fix in scope, remove every named workaround (22) | 33 | present |
| Step 4: root cause out of scope → smallest in-scope fix, rest open (22) | 33 | present |
| Step 4: `principle-fix-root-causes` guides intent (22) | 33, `bee-principle-crash-site-versus-fault-site` and `bee-principle-subtract-before-you-add` | swapped (noted: "Tool swaps", sixth bullet) |
| Step 4: `principle-redesign-from-first-principles` guides intent (22) | 33, plain sentence "build the fix as if the requirement had always existed" | swapped (noted: "Tool swaps", sixth bullet; "Pointers kept as plain sentences") |
| Step 4: principles never widen the fence (22) | 33 | present |
| Step 5: constraint comment phrases (23) | 34 | present |
| Step 5: leave keeps about things we cannot change (23) | 34, "where the project's rule allows a keep" | present (noted: "Departures", keep list) |
| Step 5: offer the cheapest in-scope type, runtime, test or CI lint (23) | 34 | present |
| Step 5: wait for interactive approval (23) | 34 | present |
| Step 5: unattended and eval require caller pre-approval (23) | 34, unattended only | changed (noted: "Departures", fourth bullet; "Dropped steps", eval) |
| Step 5: approved → encode then delete; else delete, report open, sketch out-of-scope work (23) | 34 | present |
| Step 6: report deletions, restored comments, reruns, sketch, fixes, offers, encodings, unenforced constraints, open work (24) | 35, "comments not applied", "re-dispatches" | changed (noted: "Departures") |
<!-- /bee:not-a-deferral -->

## Agent: `agents/comment-sicko.md` → `references/comment-reviewer.md`

| Source rule (source line) | New home (comment-reviewer line) | Status |
|---|---|---|
| Frontmatter `name: Comment Sicko`, description (2-3) | removed; the file is a brief, reached by dispatch | dropped (noted: "Dropped steps", third bullet) |
| First output is exactly the cackle (8-10) | 7-9 | present |
| Feed me the parent scope, else the diff against `main`; the list of targets (12) | 11 | present |
| Deletes comments itself (implied by "touched files", 32) | 13, read-only, reports each deletion | changed (noted: "Departures", first bullet) |
| Keep list: legal or license headers (16) | 23, for a host with `no_code_comments` off | present (noted: "Departures", keep list) |
| Keep list: forced external behavior; our-code surprises are meat, marked `MUST KILL` (17) | 24 | present |
| Keep list: `// prettier-ignore`; lint suppressions only for faulty, pedantic or style-only rules (18) | 25, "Formatter-ignore directives" | swapped (noted: "Departures", third bullet) |
| Keep list: public API doc comments (19) | 26 | present |
| Keep list: issue or RFC links (20) | 27 | present |
| The list is the only leash; doubt kills (22) | 29 | present |
| Suppressions stink; look up the rule; correctness or safety → kill and `MUST KILL` (24) | 31, plus `#[allow(...)]` and `# noqa` | present (noted: "Departures", third bullet) |
| Scent words; read nearby code; run `/how`, `/why` (26) | 33, `bee-how`, `bee-why` | swapped (noted: "Tool swaps", third bullet) |
| Only a foreign keep-list gotcha proven true today survives; doubt after the hunt is meat (26) | 33 | present |
| Long justification is a confession; never polish meat; `MUST KILL`; the kill ends there (28) | 35 | present |
| Every flag names in-scope code and tells the truth; never write application code (30) | 37, "I judge comments" for "I touch comments" | changed (noted: "Departures", first bullet) |
| Report only: touched files, deletion count, `MUST KILL` lines, skips (32) | 41, plus each deletion as `file:line` with verbatim text | changed (noted: "Departures", first bullet) |

Additions with no source counterpart, each named in the port notes ("Added, not in
the source"): the `metadata` block with the `bee-cli` dependency; the leash
paragraph (SKILL.md 22) and the reviewer's "The leash" section (reference 15-19);
the why-goes-to-a-concept-or-decision-log line in step 5; the
`bee-show-me-your-work` pointer in step 6; the Reply and Handoff sections
(SKILL.md 37-43); the lead-sends-nothing-else line (reference 3).

## Cross-references checked

Each name the new files cite exists in this worktree: `skills/bee-how`,
`skills/bee-why`, `skills/bee-architect`, `skills/bee-show-me-your-work`,
`skills/bee-reviewing`, `skills/bee-hive`,
`skills/bee-principle-crash-site-versus-fault-site`,
`skills/bee-principle-subtract-before-you-add`. The marked rule
`agents-no-code-comments` is at `.bee/LEADER.md:382`, and its text names the
comment baseline that reds the suite on any rising count
(`packages/bee-rs/crates/bee/src/devtools/comment_baseline.rs`). The
`no_code_comments` key is read by the write guard
(`packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:724`). The dispatch
form `bee dispatch prepare --kind reviewer` with no role matches
`skills/bee-reviewing/SKILL.md:42`. `principle-redesign-from-first-principles` has
no bee skill yet. The port notes say so and keep it as a plain sentence. D5
(`3ac421ec`) lists it for porting, so that sentence becomes a citation once the
principle lands. This is not a defect in ppb-22.

## pstack-only tool names

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-comment-review/` finds
nothing (exit 1). The source names none of these either. The pstack-only names it
does use (`Task`, `subagent_type: "Comment Sicko"`, `/how`, `/why`, `/architect`,
`/no-comments`, the two `principle-*` skills) are all swapped. The only place they
are still named is the port notes.

## Hand-picked model or subagent type

None. `rg -n -i 'model|subagent_type|opus|sonnet|haiku|fable|gpt'
skills/bee-comment-review/` matches only SKILL.md line 30. That line forbids
picking a model or subagent type by hand and routes the reviewer through
`bee dispatch prepare`.

## Description

276 characters, which fits in 300. It ends with "Not for the review gate
(bee-reviewing)", so it says when not to use the skill.

## Trigger fixture

`packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json` names
`bee-comment-review` five times, which matches the five briefs the port notes
claim.

## Licence

`cmp pstack/LICENSE skills/bee-comment-review/LICENSE.pstack` reports no
difference, so the two files are identical.

## Verify

```
$ rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-comment-review/ ; test $? -eq 1; echo $?
0
```
