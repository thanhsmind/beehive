---
name: prototype
description: "Build throwaway code in an isolated scratchpad to resolve design, layout, interaction, or behavioral decisions cheaply. Use for 'prototype this', 'try a few options', 'which layout', or when comparing implementation alternatives before planning."
disable-model-invocation: true
---

# Prototype

> Ported from pstack (`cursor/plugins`, `poteto-mode/playbooks/prototype.md`), adapted for bee.

**You own the design decision, not the code. The prototype is a throwaway instrument. The design decision is the deliverable and the code is throwaway. The real build follows `bee-planning`.**

This skill uses planning by code: speed over polish, code quality does not matter, and there is no formal planning during exploration. The rigor is in picking the right design cheaply. Propose variations the user did not ask for, throw an approach away, and try another.

## Prototyping steps

1. **Scope the decision the prototype exists to make.**
   Identify the exact question: which layout, which interaction, which density, or for an empirical fork which behavior, timing, or approach. If there is no open decision, make no prototype; route directly to `bee-planning`.

2. **Gather references when the design space is open.**
   Search for prior art, summarize a moodboard of themes, palettes, and layouts, and let the user pick directions before building. Skip this step when the direction is already set.

3. **Build throwaway in an isolated scratch directory.**
   Keep prototype code completely separate from production source. Build only inside the session scratchpad directory named in the session environment (such as `$SCRATCHPAD_DIR` or the active session scratchpad path; never use an unisolated temporary path outside the session scratchpad directory). For a visual decision, use vanilla HTML/CSS/JS or the lightest stack that renders the idea, CDN dependencies, and a dev server with hot reload. For a behavioral or timing decision, use the smallest script that exercises the question. Use no production framework, no tests, and no abstractions.

4. **When comparing alternatives, build them behind one switcher.**
   Put alternative options behind a single switcher (buttons or a keypress), with each variant clearly labeled. Explore multiple distinct variants across the design space cheaply before settling on one.

5. **Verify on the matching surface.**
   For a visual decision, take screenshots of each variant and drive the interaction. For this repository's CLI or application surfaces, drive the surface using `.claude/skills/verify-app/SKILL.md` via the `.bee/verify/verify-app/control-bee` helper script. For a behavioral or timing decision, observe the thing you are deciding by logging execution timing, printing output, or watching the render. The observation is the test here, not an automated assertion.

6. **Present alternatives, tradeoffs, and a recommendation.**
   The output is the decision plus the throwaway artifact, not shippable code. The prototype is throwaway and the decision is the deliverable. Hand the chosen direction to `bee-planning` (or to `architect` when the open question is the architecture or component shape rather than behavior) for the real build.

## Reply contract

**Reply:**
- The variants explored
- The evidence (screenshots for a visual decision, observed output or timing for a behavioral decision)
- Tradeoffs between options
- Your concrete recommendation
- The scratchpad path containing the prototype

State plainly in your response that the prototype is throwaway and the decision is the deliverable.
