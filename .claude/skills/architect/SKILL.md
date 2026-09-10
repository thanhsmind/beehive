---
name: architect
description: "Sketch types, signatures and module boundaries before code, synthesize across competing designs, then fill in against the chosen sketch and scrap it if implementation proves it wrong. Fires on non-trivial NEW code whose shape is still open, where jumping straight to code would lock in the wrong structure. Not for a change that has an existing pattern to follow, a bug fix, a rename, or anything a single obvious implementation already answers."
---

# Architect

Ported from pstack (cursor/plugins, architect), adapted for bee.

Design before implementing. Sketch types, function signatures, class shapes, and module boundaries with `not implemented` bodies and pseudocode. Synthesize across multiple model perspectives, then fill in code against the chosen sketch. If implementation proves the sketch wrong, throw it out and redesign.

## Start

Open a todolist with one entry per phase before starting.

1. Ground
2. Sketch
3. Agree
4. Implement
5. Scrap

## Phase A: Ground the problem

Build a real mental model of every system the new code touches. Run the **how** skill over the relevant subsystems.

Naming a file isn't grounding. Produce the traced model `how` prescribes. If the design redefines ownership or layering, also run the **why** skill on the existing shape so the rationale becomes a constraint, not a guess.

Skip Phase A only when the work is genuinely greenfield with no surrounding system to integrate.

## Phase B: Sketch

Run the **arena** skill with the design-sketch task and the Phase A grounding artifacts. Pass [references/runner-prompt.md](references/runner-prompt.md) as each runner's prompt. Each candidate produces a design package shaped per [references/rationale-template.md](references/rationale-template.md).

Candidate runners are prepared through the ONE bee door:
```bash
.bee/bin/bee dispatch prepare --runtime claude --kind advisor --purpose "architect design candidate" --json
```
Run this command for each candidate runner. The door picks the model and the skill never does. Run exactly the tool and payload the command returns. Never hand-pick a model name or subagent type.

Design it twice. Require at least two structurally distinct candidates before synthesis, even when the first looks sufficient, exploring multiple structurally distinct alternatives before settling on a design. Whole-shape alternatives, not point fixes inside one shape.

Screen every candidate against [references/design-red-flags.md](references/design-red-flags.md) before synthesis. Reject or revise shallow modules, information leakage, temporal decomposition, and pass-through methods.

Compare viable candidates on interface depth. Prefer the design that hides more complexity behind a smaller, simpler public surface. A rich interface can keep call chains short by concentrating capability instead of scattering it across layers.

Arena returns one synthesized design package. The synthesis decision populates the rationale's "Synthesis decision" section.

## Phase C: Agree (opt-in)

Default: proceed directly to implementation with the synthesized design. No human checkpoint.

Opt in to a checkpoint when the invoker explicitly asks: "/architect with checkpoint," "stop and show me before implementing," or similar. Then surface the synthesized design and pause for sign-off.

The synthesis can ship as its own commit either way, establishing the foundational scaffold and core interfaces first. Planned and scoped breakage during fill-in is fine when focused on reaching a verifiable working outcome.

If the human pushes back on the shape (in a checkpoint or after the fact), treat that as Phase A evidence. Re-ground and re-run Phase B before writing more code.

## Phase D: Implement against the sketch

Replace `not implemented` bodies with code, pseudocode with logic. The synthesized sketch is the contract.

Deviations from the sketch are signal worth surfacing, not friction to absorb silently. If a function needs a parameter the sketch didn't anticipate, ask whether the sketch was wrong, the requirement was missed, or the implementation is overreaching.

## Phase E: Scrap when the architecture is wrong

If implementation keeps producing friction the sketch can't absorb, throw the sketch out. Don't bolt fixes onto an unsound design; trace failures back to their structural origin and rebuild from first principles.

The signal is a *pattern*, not single instances. Tells:

- The same shape of workaround appearing repeatedly across unrelated code.
- Multiple unrelated edge cases that all need special-case branches.
- Types that need escape hatches (`any`, casts, optional fields always set in practice) to compile.
- The "we need a lock" reflex when the sketch said the state wasn't shared.
- Callers having to know the abstraction's internal rules to use it.
- Two or more independent Phase D deviations of the same shape across the implementation.

Use judgment. A few edge cases don't condemn an architecture. Some problems are legitimately complex. Complexity in the data is not complexity in the design.

When you scrap:

1. Re-run the **how** skill over what's been built.
2. Redesign as if the new constraints had been day-one assumptions, rebuilding from foundational constraints.
3. Subtract before adding: remove unnecessary complexity and dead structures before adding new ones. The new sketch should be smaller than the old one before it grows.
4. Return to Phase B and re-run arena.

## Outputs

The caller's usage is written first and the type sketch derived from it. One file with new types and signatures for small changes. Module map plus type definitions for larger work. The rationale ships alongside, shaped per [references/rationale-template.md](references/rationale-template.md), including the usage sketch and the synthesis decision.
