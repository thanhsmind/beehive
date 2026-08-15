# waiting-on-doctrine — brief (docs-lane, short form)

## What was asked

The user runs an external kanban dashboard (waggledance) over `.bee/`.
When an agent ends a turn on a question, the terminal shows idle and the
board shows nothing — the `bee state waiting-on set` verb exists
(awaiting-human D1-D4) but no handbook surface tells agents to run it,
so the mark is never written. The user approved part (a): add the
doctrine rule on the bee side.

## What was found

- `rg "waiting-on|waiting_on" skills/ packages/bee/AGENTS.block.md`
  returns nothing — zero doctrine mentions.
- awaiting-human CONTEXT.md D1 already states the semantics: "The agent
  marks the wait when it asks; the mark ends when the answer lands."
  D2: the UserPromptSubmit hook auto-clears on the human's next message;
  stale expiry is the backstop. The rule only needs to reach the
  handbook surfaces agents actually read.

## What will be written

One sentence in `packages/bee/AGENTS.block.md` (Communication section),
the full rule in `skills/bee-hive/references/routing-and-contracts.md`
(Question Format), and one line in
`skills/bee-shaping/references/shaping-reference.md` (Interview craft).
Then `bee dev regen` propagates the vendored block and rendered skill
trees. No runtime code changes.
