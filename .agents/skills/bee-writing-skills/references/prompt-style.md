# Prompt style — the instruction-spec standard

Write skill prose as an operating spec for a machine, never as
conversation. Seven laws; apply them to every SKILL.md and reference.

## The seven laws

1. **Imperative, declarative.** "Do X. Never Y." No hedging ("perhaps",
   "try to", "you might want to"), no filler ("please note", "keep in
   mind"), no throat-clearing intros. Every sentence is a rule or a
   trigger.
2. **Trigger framing.** The `description` keeps the shape: one purpose
   clause, "Use when ..." triggers, then "Not for ..." exclusions.
   Inside the body, a conditional line states its condition before its
   instruction.
3. **Contrastive examples.** A `Not: ... / Yes: ...` pair only where it
   replaces 3+ lines of abstract explanation. An example that restates
   an adjacent rule is bloat — cut it.
4. **One word, one meaning.** One fixed term per concept, repeated
   verbatim everywhere it renders. Never rotate synonyms; a second name
   for the same thing is a second thing to misread.
5. **Constraint first.** The rule leads; rationale follows only when
   its absence would cause misapplication. Decorative "why" dies.
6. **Progressive disclosure.** SKILL.md is the routing/rule layer;
   depth lives one level down in `references/` with a "when to load"
   table. Exception (already law in the checklist): per-turn rules
   never leave the always-loaded layer.
7. **Token economy.** A line earns its place only by changing agent
   behavior. Cut restatements, metaphors that carry no rule, and
   status narration. Clarity still beats brevity: a misread rule costs
   more than it saves.

Not: "It's usually a good idea to try to keep your changes small, since
large diffs can sometimes be harder to review."
Yes: "Keep the diff small. Split anything a reviewer cannot hold in one
pass."

## Hard guardrails when editing an existing skill

- Preserve the semantics of every rule — this standard rewrites form,
  never behavior. Unsure whether a cut changes meaning → keep the text.
- Headings are frozen once cited: pointer-integrity CI resolves
  citations like `references/x.md ("Heading")` and
  `bee-<skill> ("Heading")` against real headings. Never rename,
  re-level, or delete a cited heading.
- Protocol vocabulary is byte-for-byte: command names, flags, JSON
  keys, status tokens, markers, gate/lane/phase names, file paths,
  template placeholders.
- Never add numeric limits or ceiling-shaped language the text does not
  already carry (instruction-laws CI scans for these).
- Rendered or hash-pinned copies (generated skill trees, onboard
  templates, vendored prompts) change only through their regen chain,
  never by hand.
