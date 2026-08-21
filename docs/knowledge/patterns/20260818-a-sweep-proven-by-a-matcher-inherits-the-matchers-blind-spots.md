---
type: bee.pattern
title: A sweep proven by a matcher inherits the matcher's blind spots
description: A sweep proven by a matcher inherits the matcher's blind spots
tags: [failure, verification, docs, search]
timestamp: 2026-08-18
bee:
  id: pattern-20260818-a-sweep-proven-by-a-matcher-inherits-the-matchers-blind-spots
  lifecycle: active
  areas: [workflow-state]
  sources: ["test-doctrine-text-sweep cell tdt-3, four rounds and three rejected verdicts, 2026-08-18"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "run the sweep's own pattern against the pre-fix content first — a pattern that cannot match the text you are removing proves nothing when it later matches nothing"
---

# A sweep proven by a matcher inherits the matcher's blind spots

A doctrine sweep had to remove every claim that two commands still ran the test
suite. Its proof was a grep, and the grep came back clean. It was clean because
the surviving sentence wrapped a newline and the pattern was single-line — two
whole files still taught the retired rule.

That was round one. Round two fixed those files but rewrote three passages by
following the shape of the old sentence instead of reading the code, so the
sweep produced *new* false claims. Round three found a third instance by
reading. Round four removed a relic sitting inside a parenthetical that no
pattern in any round would have matched.

Four rounds, and **every defect was found by reading — not once by matching**.

**The rule:** a sweep's proof must be shown to catch the thing it is removing
before an empty result means anything. Run the pattern against the pre-fix
content and watch it match; only then does a later empty result carry
information. State the coverage honestly in both directions — a proof line that
says "clean" while the pattern demonstrably cannot see one of the fixed cases is
the same false-clean mistake wearing a receipt.

**The corollary that costs more:** matching finds instances of a phrasing;
reading finds instances of a claim. Prose says the same wrong thing in a
sentence, a table cell, a heading, a diagram label, a tree annotation, and a
parenthetical aside — six shapes, one pattern, at best two hits. When the target
is a *claim*, budget for reading the file and checking each assertion against
the source, and treat the matcher as a way to find the first ones, never as the
proof that you found the last one.

And when rewriting: the replacement is only as true as the reading behind it.
Editing toward the shape of the old sentence carries its retired assumptions
into the new one.
