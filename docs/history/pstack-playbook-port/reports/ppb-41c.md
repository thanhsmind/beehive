# ppb-41c: check that ppb-41 closed the bee-how and bee-why gaps

Checked against commit `89b95c3a9` (ppb-41). Every row that `ppb-31c.md` and
`ppb-32c.md` marked missing was looked up in the current
`skills/bee-researching/references/trace-and-provenance.md` and
`skills/bee-how/SKILL.md`. No file other than this report was edited.

## Verdict

All 12 rows are present. None is still missing.

## Rows from ppb-31c (bee-how merge)

| Row | Rule | Status | Where |
|---|---|---|---|
| M1 | Spawn every explorer in one turn | present | Trace step 3: "every one spawned in the same turn" (`trace-and-provenance.md:25-26`) |
| M2 | The flow return carries the data passed between steps | present | Trace step 3 return: "what it calls next and the data it passes there" (`:35-36`) |
| M3 | How it works is the longest section | present | answer shape: "the core, and the longest section" (`:77`) |
| M4 | Trigger: walkthrough before a change | present | bee-how description: "or for a walkthrough before a change" (`skills/bee-how/SKILL.md:4`) |
| M5 | Fan out when an inline pass finds another entry point | present | Trace step 4: "When the inline pass turns up a second disjoint entry point, stop and fan out per step 3" (`:53-54`) |

## Rows from ppb-32c (bee-why merge)

| # | Rule | Status | Where |
|---|---|---|---|
| 1 | Follow links inside a row | present | worker brief: "A link inside your own row ... is followed to its end" (`:259-261`) |
| 2 | One record cited by several rows is one reference | present | Report rules (`:274-275`) |
| 3 | Duplicate and reopen or scope-drift history | present | row 7, external tracker (`:224-226`) |
| 4 | Several drafts: use the finalized or latest | present | Reading a source (`:338-339`) |
| 5 | A low count may be sampling | present | Reading a source (`:340-341`) |
| 6 | A step in volume may be new instrumentation | present | Reading a source (`:342-343`) |
| 7 | Never report from a source not confirmed to exist | present | Reading a source (`:344-345`) |

## Side checks

- bee-how description is 291 characters, under the 300 budget.
- To make room for M4, ppb-41 dropped "where its parts live" from the bee-how
  description. `merge-ppb-31.md` records this. The Scope line
  (`skills/bee-how/SKILL.md:28`, "where should this live") and the answer
  shape's Where things live section still cover placement questions, so no
  trigger was lost.
- None of the lines ppb-41 added contains a pstack-only tool name. A
  case-insensitive scan of the `+` lines for cursor, pstack, grok, mcp,
  subagent_type, linear, notion, slack, datadog, sentry, databricks, seer,
  fable, opus, sonnet and readonly found nothing.
- `trace-and-provenance.md` is byte-identical in `.agents`, `.claude-plugin`,
  `.codex-plugin` and `.opencode`.
- Both port notes have a "Closed by ppb-41" section that names every row and
  says where it went.
