# Port notes — skill bee-reflect, cell ppb-19

Source: pstack skill `reflect` (`SKILL.md` and four reference files:
`judgment-reviewer.md`, `tooling-reviewer.md`, `divergent-reviewer.md`,
`synthesizer.md`) at commit `b9ddc83`, MIT, copyright 2026 Lauren Tan. The notice
is `skills/bee-reflect/LICENSE.pstack`, a verbatim copy of the pstack `LICENSE`.
This file is the only place a pstack-only tool is named; no ported file names one.

Name: `bee-reflect` (the pstack name with the `bee-` prefix every ported skill
carries).

Kept as the base, with the swaps below: "When invoke" list and the skip rule, the
six steps (locate the transcript, three parallel reviewers, synthesize, structural
enforcement check, apply, summarize), the three lenses (judgment, tooling,
divergent) with their scan lists, the agent-self-sufficiency addition in the
tooling lens, the untrusted-transcript paragraph, the skill-was-used scope rule,
the synthesizer's nine criteria and its Accepted / Rejected / Backlog output.

Frontmatter follows `bee-how`: `name`, a description of 282 characters (what it
does, when to use it, what it is not for) and `metadata` with `ecosystem: bee`
and a `bee-cli` dependency that degrades to the leader applying each lens itself.
The source's `disable-model-invocation: true` is dropped, as in `bee-architect`,
`bee-interrogate`, `bee-fanout` and `bee-tdd`: the description names the explicit
trigger and its limits.

## Tool swaps

- Subagent spawning ("three `Task` calls, `subagent_type: generalPurpose`") became
  one `bee dispatch prepare --runtime claude --kind reviewer --json` per reviewer,
  all three in the same turn, each returned payload run in the background and used
  exactly as returned. The synthesizer is one more `--kind reviewer` payload. No
  model or subagent type is named.
- The model-per-lens table (reflect-judgment and reflect-tooling model settings
  with named default model slugs) became the review role of the bee team table.
  The three default slugs are dropped. The skill says to note in the summary when
  all payloads name one model, so a reader weighs agreement across the lenses
  accordingly, as `bee-interrogate` does.
- "Agent mode (`readonly: false`)" and its reason (reviewers need MCP access, and
  readonly strips MCPs) are dropped. Read-only and tool access are properties of
  the dispatch payload, so the reviewer prompts now say "use any tool your payload
  gives you" where the source said "any MCP tool available in your environment".
- "Backlog items file to whatever devex / backlog tracker your team uses" became
  `bee backlog add`, in Step 5, in the summary and in the synthesizer's Backlog
  section.
- Cursor's built-in `create-skill` skill (substantive edit, description tuning,
  new skill) became `bee-writing-skills`: the routing strings are
  `tune description: <skill path>` and `new skill via bee-writing-skills:
  <kebab-name>`.
- The "user said /reflect" trigger became `/bee-reflect`, plus the bee names for
  the neighbouring doors: a single mistake goes to `bee mailbox reflect`, a settled
  rule to `bee-capturing`.
- Skill-path examples in the three scope sections (workspace, user-level and
  plugin skill directories of the source host) became the repo's `skills/` and its
  rendered skill directories, user-level skill directories and plugin-installed
  paths. "`Task` prompts that name a skill path" became "Dispatch prompts".
- The synthesizer's drift example naming a review bot became "a review flagged
  regex backtracking on May 2".
- The structural-enforcement step cites `bee-principle-encode-lessons-in-structure`,
  where the source said "the **encode-lessons-in-structure** principle skill".

## Added, not in the source

- A "Record" block in Step 5: a mistake the session made goes through
  `bee mailbox reflect --wrong --better --fix-at <architecture|check|doctrine|none>`,
  a rule or value that settled goes through `bee decisions log`. Sources: the
  reflection and capture doctrine in `.bee/LEADER.md`.
- The framework paragraph in Step 5: in the bee repo a learning about how agents
  work changes the framework through the bee chain (`bee-shaping`, `bee-planning`,
  `bee-swarming`) and is released, rule `agents-feedback-changes-framework`.
- The single re-dispatch for a return that breaks the finding schema, and the
  uncovered-lens line, from `bee-interrogate` and `bee-hive/references/gates-and-delegation.md`.
- A "Recorded" line in the user summary, and a `Handoff` section ("Invoke bee-hive
  skill"), as `bee-interrogate` and `bee-tdd` have.

## Pointers kept as plain sentences

None. Every skill and principle the skill names exists in `skills/`.

## Dropped steps

- The `ls -t <agent-transcripts>/*.jsonl ...` command and the three transcript
  layouts (legacy flat, current nested, subagent), and the instruction to take the
  directory from the system prompt: they describe one host's on-disk transcript
  store. The step now says to use the path the host names for the active session,
  never to glob across other projects' directories, and to fall back to a digest.
  The first-line check and the digest fallback are kept.
- The Cursor-only note that the parent hands skill work to the host's skill
  creator: replaced by `bee-writing-skills`, above.
- Graphite and the Cursor loop: the source cites neither, so nothing was dropped
  for them. The poteto-agent, arena, architect, how, why and show-me-your-work are
  not cited in the source either.

## Trigger fixture

Five briefs added to `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json`:
three expect (one Vietnamese) and two near (expect `bee-capturing` and
`bee-reviewing`), none naming the slug.
