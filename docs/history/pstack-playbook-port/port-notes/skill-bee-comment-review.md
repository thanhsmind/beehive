# Port notes — skill bee-comment-review, cell ppb-22

Source: pstack skill `no-comments` (`SKILL.md`, no reference files) and the pstack
agent `Comment Sicko` (`agents/comment-sicko.md`) at commit `b9ddc83`, MIT,
copyright 2026 Lauren Tan. The notice is `skills/bee-comment-review/LICENSE.pstack`,
a verbatim copy of the pstack `LICENSE`. This file is the only place a pstack-only
tool is named; no file under `skills/bee-comment-review/` names one.

Name: `bee-comment-review` (the cell's name; the pstack skill is `no-comments`).

Kept as the base, with the swaps below: the six steps (spawn, inspect, fix trivial
flags, root-cause fix, constraint comments, report), "Authoring agents defend
comments", the scope rule, every rejection clause of step 2, and in the agent text
the opening line, the keep list, the suppression and scent paragraphs, the
"doubt is meat" rule and the report-only close.

Frontmatter follows `bee-how`: `name`, a description of 276 characters (what it
does, when to use it, what it is not for) and `metadata` with `ecosystem: bee` and
a `bee-cli` dependency that degrades to the leader reading the scoped comments
itself. The source's `disable-model-invocation: true` is dropped, as in
`bee-interrogate` and `bee-blast-radius`: the description names the explicit
trigger and its limits.

## Tool swaps

- Step 1, "Spawn `Task` with `subagent_type: "Comment Sicko"`", became
  `bee dispatch prepare --runtime claude --kind reviewer --json` with no `--role`,
  the returned payload run exactly as returned. No model or subagent type is named.
  The agent file became `references/comment-reviewer.md`, the brief the lead sends
  with the scope. The agent name and its definition file are pstack-only and are
  not carried.
- "Do not restate its rules" kept: the skill points at the reference and the brief
  carries no verdict, leaning or expected findings, as `bee-reviewing` requires.
- `/how` and `/why` became `bee-how` and `bee-why`, in step 2 and in the reviewer
  text.
- `/architect` became `bee-architect`.
- `/no-comments` in "fail `/no-comments`" became `bee-comment-review`.
- The two pstack principles of step 4: `principle-fix-root-causes` became
  `bee-principle-crash-site-versus-fault-site` (fix at the origin) with
  `bee-principle-subtract-before-you-add`; `principle-redesign-from-first-principles`
  has no bee skill yet, so its method stays a plain sentence ("build the fix as if
  the requirement had always existed"). When that principle lands, the sentence
  becomes a citation.
- The long-run decision trail named in step 6 is `bee-show-me-your-work`; the
  source had no such line, see Added.

## Departures

- The reviewer is read-only. The pstack agent deletes comments itself and the lead
  inspects the diff afterwards, restoring wrongly deleted text. bee's reviewer kind
  never edits the working tree, so the reviewer reports each comment to delete as
  `file:line` with its verbatim text and the lead applies the accepted deletions.
  Step 2 changed with it: "Revert and rerun one rejected report" became "Re-dispatch
  one rejected report", "Restore deletions only with exact exceptions" became
  "Refuse a deletion only with the exact exception that protects the text", and the
  step 6 count "restored comments" became "comments not applied".
- The keep list. pstack's list (license header, forced external behavior, formatter
  ignore, public API doc comments, issue links) conflicts with bee's rule, where a
  public item's description lives in the owning concept and three exceptions only
  survive (`#!`, `SAFETY:`, a license header). The skill does not restate either:
  it names the marked rule `agents-no-code-comments` in `.bee/LEADER.md`, the
  `no_code_comments` key and the comment baseline as the leash. The reviewer
  reference keeps the pstack list as the base for a host where the key is off, and
  says that where the key is on the rule's exceptions are the only keeps and the
  why goes to a `docs/knowledge` concept or `bee decisions log`.
- `// prettier-ignore` became "formatter-ignore directives", and the suppression
  examples gained `#[allow(...)]` and `# noqa`, since bee hosts are not only
  TypeScript.
- Step 5's "Wait for interactive approval. Unattended and eval require caller
  pre-approval" became "An unattended run requires the caller's pre-approval". bee
  has no eval mode.

## Added, not in the source

- The leash paragraph under the title, tying the skill to the rule and the baseline.
- A `Reply` line and a `Handoff` section ("Invoke bee-hive skill"), as
  `bee-blast-radius`, `bee-why` and `bee-tdd` have.
- The `bee-show-me-your-work` pointer for a long or unattended run.
- In step 5, where the rule is on, the why a deleted comment carried goes to a
  concept or the decision log, never a new comment.

## Pointers kept as plain sentences

- `principle-redesign-from-first-principles`: no skill in `skills/` yet; named above.

## Dropped steps

- Cursor `/loop`, Graphite, bugbot and poteto-agent: the source cites none of them,
  so nothing was dropped for them.
- The pstack `eval` run mode: no bee counterpart, dropped from step 5.
- The agent's `name`/`description` frontmatter and its Cursor agent registration:
  bee reviewers are reached through `bee dispatch prepare`, never by agent name.

## Trigger fixture

Five briefs added to `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json`:
three expect (one Vietnamese) and two near (expect `bee-reviewing` and `bee-why`),
none naming the slug.
