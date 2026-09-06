---
type: bee.pattern
title: A scan scope set from assumption passes green while hiding the very bug it was built to catch
description: "Three times in one session a hand-chosen scan scope was one directory too narrow and reported clean. The scope is the finding, not the hits — derive it by measurement before you trust a green."
tags: [process, census, gates, scope, measurement]
timestamp: 2026-07-23
bee:
  id: pattern-20260723-a-scan-scope-set-from-assumption-passes-green
  lifecycle: active
  areas: [verify-pipeline]
  sources: ["okf-integration-close-f4 (an instruction-layer audit whose own grep omitted hooks/, where the P1 was; trace in `.bee/cells/`, 2026-07-22)", "judge-record-tags cells jrt-1, jrt-2 (a tagless-decision census scoped to lib/** that passed green while bee.mjs carried a live instance; traces in `.bee/cells/`, 2026-07-23)"]
  polarity: pitfall
  critical: true
---

# A scan scope set from assumption passes green while hiding the very bug it was built to catch

A census, fence, or audit is two things: a **rule** and a **scope**. Review attention goes almost
entirely to the rule, because that is where the thinking looks like it happened. The scope is
usually typed once, from memory of where the relevant code lives — and a scope that is one directory
too narrow does not fail loudly. It reports **clean**, which is the most reassuring output a check
can produce.

**Three instances in a single session, all by the same author, all the same shape:**

- An audit command written to sweep the instruction surfaces silently omitted `hooks/` — which is
  exactly where the P1 was. The subagent running it noticed the omission and measured that directory
  separately; had it simply obeyed, the audit would have reported the gap closed.
- A census guarding a write-time rule was scoped to `lib/**` from memory. It passed green on a tree
  that still contained a live instance of the exact bug it existed to prevent, in a sibling file one
  level up.
- The same session's first audit read files for meaning rather than measuring counts, and missed
  two files that a later `grep -c` found immediately.

**The rule: derive the scope by measurement before you trust the check, and treat the scope as the
finding.** Concretely — run the unrestricted search first (repo-wide, minus vendor and generated
trees), look at what it returns, and only then narrow, with each exclusion carrying a stated reason.
"I know where those live" is the assumption that produces all three instances above. A check whose
scope was never measured has proven nothing when it passes; it has only proven something about the
subset someone remembered.

**Corollary — exclusions are claims that need their own evidence.** Every path a scope leaves out is
an assertion that nothing there can carry the defect. Legitimate exclusions exist (a fixture's
untagged call is its subject matter; a legacy anchor citation resolves through a stub), but each is a
claim, and each belongs in the code with its reason next to it rather than in the author's head.

**Corollary — the check must not flag the legitimate case.** In the census above, one caller
correctly forwards a *user's* tags and must never be reported as an offender: a check that flags it
pushes the next author to fabricate a value that was not theirs to choose. A scope that is too wide
fails differently from one too narrow, and worse in one way — it teaches people to satisfy it
dishonestly.
