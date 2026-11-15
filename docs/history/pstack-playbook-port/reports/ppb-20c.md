# ppb-20c — bee-recall checked against its pstack source

Scope: what ppb-20 wrote in commit `d5246e882`: `skills/bee-recall/`
(`SKILL.md`, `LICENSE.pstack`) and the port notes
`docs/history/pstack-playbook-port/port-notes/skill-bee-recall.md`. They were
checked against the pstack skill `pstack/skills/recall/SKILL.md`. The source has
no reference files. `SKILL.md` was compared line by line, and the licence with
`diff`. The five rendered copies (`.claude-plugin`, `.codex-plugin`, `.claude`,
`.agents`, `.opencode`) were compared with `diff -rq`, and all of them match
`skills/bee-recall/`.

Verdict: no source step is missing. All six source steps, the two-records framing
(now three records), the default-on rule for the shared-record sweep and its one
exemption, and the whole output contract are present. Every removed or swapped
item has a port-notes entry. `LICENSE.pstack` is identical to the pstack
`LICENSE`. The pstack-only tool check is clean. No model and no subagent type is
picked by hand. The description is 277 characters and says when not to use the
skill. There is one finding, P3, about the workspace scope. It does not change
any source step.

Labels: **present** means the step is in the new file. **swapped (noted)** means
it is present with a bee tool in place of the pstack tool, and the port notes name
the swap. **dropped (noted)** means it is gone and the port notes name it.
**dropped (not noted)** means it is gone and the notes say nothing; D2
(`063a21e4`) counts that as missing. **missing** means a method step is gone with
no reason.

## `SKILL.md`

| Source step or rule (source line) | New home (bee-recall line) | Status |
|---|---|---|
| Frontmatter `name: recall` (2) | `name: bee-recall` (2) | present |
| Frontmatter description (3) | 4, rewritten: what it does, when, when not | present |
| Frontmatter `disable-model-invocation: true` (4) | removed | dropped (noted: frontmatter paragraph) |
| Opening promise: rebuild context, hand back a capsule; trigger phrases (9) | 18 | present |
| Keep it tight; heavy reading fans out; main thread keeps findings only (11) | 20, "parallel workers" | present |
| Two records: own chat history and the shared record (13) | 22, three records: bee's own records first | present, extended (noted: "Added" first bullet) |
| The shared record is what the why skill searches, with its source list (13) | 22, `bee-why` over the decision log, source control, feature history, knowledge bundle, tracker | swapped (noted: "Tool swaps", second bullet) |
| Don't rebuild a bug-tail feature from transcripts alone (13) | 22 | present |
| Transcript path recipe under `~/.cursor/projects/<slug>/...`, slug rule, one message per line (15) | 38, `bee state session list --json`, field `transcript_path` | swapped (noted: "Tool swaps", third bullet; "Dropped steps", second bullet) |
| Step 1: classify, route a single chat to `session-pickup` (17) | 26, with the playbook's repo path | swapped (noted: "Tool swaps", sixth bullet) |
| Step 1: habits to a durable skill go to `automate-me` (17) | removed; a settled rule routes to `bee-capturing` | dropped (noted: "Dropped steps", first bullet) |
| Step 1: a human-readable summary is a different task (17) | 26 | present |
| Step 1: a full state capsule from the user, so skip the mining (17) | 26 | present |
| Step 2: lock the scope: window (default 7 days), topic, workspace; never another project's transcripts unasked (18) | 28 | present |
| Step 2: state the scope back; never turn "all" into "recent N" (18) | 28 | present |
| (no source step) bee's records first: `bee orient`, `bee status`, handoff, `bee decisions search` | 30-36, new step 3 | added (noted: "Added" first bullet) |
| Step 3: parallel subagents on a fast, cheap model, one per slice (19) | 38, one `bee dispatch prepare --runtime claude --kind gather --role extraction --json` per slice, same turn, in the background | swapped (noted: "Tool swaps", first bullet) |
| Step 3: order by real modification time (`ls -t`), never by UUID name (19) | 38, "never by file name" | present |
| Step 3: grep the topic first, read only matching chats and regions (19) | 38 | present |
| Step 3: skip the current chat and noise (subagent, eval, test chats) (19) | 38, "worker, eval and test sessions" | present (noted: "Added" third bullet) |
| Step 3: one schema per chat: topic, goal, decisions, open threads, struggles and corrections, artifacts; cite the chat UUID (19) | 38, adds cell ids, cites the session id | swapped (noted: "Tool swaps", last bullet) |
| Step 3: one or two chats, so search directly (19) | 38 | present |
| Step 3: raw transcripts stay in the subagents (19) | 38 | present |
| Step 4: sweep the shared record when a feature, file, subsystem, area or bug is named; default, and "my work on X" is no exemption (20) | 40 | present |
| Step 4: hand off to the why skill's investigators, steered to current state, what was tried, what users report (20) | 40, `bee-why`'s Provenance sweep (`bee-researching/references/trace-and-provenance.md`) | swapped (noted: "Tool swaps", second bullet) |
| Step 4: reuse its per-source playbooks (20) | 40, "per-source rows" | swapped (noted) |
| Step 4: run in parallel with the chat mining (20) | 40 | present |
| Step 4: posture: one investigator per source, null results are findings, skip an unavailable MCP and say so (20) | 40, "one worker per row that needs one", UNSWEPT with the reason | swapped (noted: "Tool swaps", second bullet) |
| Step 4: fold the results into the brief (20) | 40 | present |
| Step 4: the only exemption is pure activity recall with no named target (20) | 40 | present |
| Step 5: transcripts and stale tickets are history; check PRs, branches and tickets with `git` and `gh` (21) | 42, adds cells and `bee status`, cites `bee-principle-verify-before-reporting` | swapped (noted: "Tool swaps", fourth bullet) |
| Step 5: read the full transcript, not a trimmed local copy, when the answer hinges on what an agent did (21) | 42, "not just a trimmed copy" | present; the cloud/local split is dropped (noted: "Dropped steps", second bullet) |
| Step 6: write the brief to the contract, group by thread, stay on topic (22) | 44 | present |
| Output order: capsule, threads, problems, next move; detail below or cut (26) | 48 | present |
| Capsule: at most 5 bullets (28) | 50 | present |
| Threads: one line each, exactly one of six status tags; untagged means not done (29) | 51, the same six tags | present |
| Problems: at most 5, user symptoms, shipped-then-reverted fixes (30) | 52 | present |
| Next move: one concrete action (31) | 53 | present |
| Adjacent work stays out unless it blocks; cut detail before threads (33) | 55 | present |
| Write through the unslop skill (33) | 55, `bee-unslop` | swapped (noted: "Tool swaps", fifth bullet) |
| Cite chat findings by UUID and shared-record findings by PR, ticket, chat permalink, error-tracker issue (33) | 55, session id; PR, ticket, decision id, commit | swapped (noted: "Tool swaps", last bullet; "Dropped steps", third bullet) |
| Sanitize private context before public output (33) | 55 | present |
| Reply: the brief, to the contract (35) | 59, plus "Read-only: write no brief file, change no state" | present (noted: "Added" last bullet) |

Additions with no source counterpart, each named in the port notes ("Added, not in
the source"): the `metadata` block with the `bee-cli` dependency (frontmatter
paragraph); step 3, bee's records first (lines 30-36); the `bee-capturing` route
(line 26); cell ids in the worker schema and "worker" sessions in the noise list
(line 38); the read-only line in Reply (line 59); the Handoff section (lines
61-63). Each named path and skill exists: `skills/bee-planning/playbooks/session/session-pickup.md`,
`skills/bee-researching/references/trace-and-provenance.md` (it has the
"Provenance sweep" section and the UNSWEPT rule at line 115), `bee-why`,
`bee-unslop`, `bee-capturing`, `bee-hive` and
`bee-principle-verify-before-reporting` under `skills/`. The bee verbs the skill
names were run read-only in the store checkout. `bee state session list --json`
returns records with a `transcript_path` field. `bee state handoff show --json`
runs and prints `null` when there is no handoff. `bee decisions search` takes
`--text`. `bee dispatch prepare` takes `--kind gather` and `--role`, and its help
says a gather with `--role extraction` is extraction-headed.

## `LICENSE.pstack`

`diff` against the pstack `LICENSE` prints nothing.

## Pstack-only tool names

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-recall/` prints
nothing and exits 1. The only pstack-specific names in the source are the Cursor
transcript path (`~/.cursor/projects/.../agent-transcripts/`), the `automate-me`
skill and "MCP" in the why posture. All three are swapped or dropped, and the
notes record each one. The source does not cite Graphite, Bugbot, `poteto-agent`,
`/loop`, arena, architect or show-me-your-work, and the notes say the same.

## Hand-picked model or subagent type

None. `rg -n -i 'subagent_type|model' skills/bee-recall/` matches only
`SKILL.md:38`, which is the rule never to pick a model or a subagent type by hand.
Every dispatch goes through `bee dispatch prepare`. The source's wish for "a fast,
cheap model" became `--role extraction`, which is a seat in the team table, not a
model name.

## Description

The folded `description:` value is 277 characters, which matches the port notes
and fits the 300-character limit. It says what the skill does, when to use it
("the user types /bee-recall or asks to catch up or where they left off") and when
not to: "Not for resuming one session (session-pickup) or a why (bee-why)."

## Findings

1. **P3: "the active workspace" has no stated filter on the session records.** The
   source scoped its transcripts by directory: the active workspace's slug
   directory held only that project's chats. The port reads paths from
   `bee state session list --json`. In the store checkout those records carry a
   `workspace_id` (for example `"main"`) and an `activity.cwd`, so one store can
   list sessions from several worktrees. Step 2 still says "default the active
   one", but step 4 does not tell the workers which field selects it. Whether
   worktree sessions should count as the active workspace is a guess I could not
   settle from the records. Suggested fix: in step 4, name the field that selects
   the scope (`workspace_id` or `activity.cwd`). That change is outside this cell's
   scope, so the leader decides. It does not change the step table.

## Verify

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-recall/ ; test $? -eq 1`
printed nothing, and `test` exited 0.
