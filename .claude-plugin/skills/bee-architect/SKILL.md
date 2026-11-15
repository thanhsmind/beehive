---
name: bee-architect
description: >-
  Sketch types, signatures and module structure before code, design it twice through blind lanes, then fill in against the chosen sketch. Use when the user types /bee-architect or asks to architect or design non-trivial work. Not for cell planning (bee-planning) or locking decisions (bee-shaping).
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: The design lanes go out through the vendored bee binary's dispatch door and the dossier check is its blind verb. Absent, the leader sketches both candidates itself and the account says so.
---

# Architect

Design before implementing. Sketch types, function signatures, class shapes and module boundaries with `not implemented` bodies and pseudocode. Synthesize across several independent design candidates, then fill in code against the chosen sketch. If implementation proves the sketch wrong, throw it out and redesign.

## Start

Open a task list with one entry per phase before starting. An autonomous run without checkpoints needs the list to show phase position and to keep phases from silently disappearing.

1. Ground
2. Sketch
3. Agree
4. Implement
5. Scrap

## Phase A: Ground the problem

Build a real mental model of every system the new code touches. Run `bee-how` over the relevant subsystems. When existing structure is the constraint, or the design must push back on it, read the traced path with that question in hand: where does the current shape resist the new one?

Naming a file is not grounding. Produce the traced model `bee-how` prescribes. If the design redefines ownership or layering, also run `bee-why` on the existing shape, so the recorded rationale becomes a constraint and not a guess.

Skip Phase A only when the work is greenfield with no surrounding system to integrate.

## Phase B: Sketch

Fan the design out to blind lanes, with the task and the Phase A grounding artifacts as the lanes' read diet. The procedure is the one in `bee-hive/references/gates-and-delegation.md`, section "Blind lanes and convergence". It is the only fan-out mechanism here; never invent a second one.

- When the user invoked this skill, that is the order to open lanes. When a playbook or another skill reached it, open lanes only if the decision is both high-stakes and ambiguous, and log the reason with `bee decisions log` at open time. Otherwise write the two candidates yourself, one after the other, the second as a deliberate break from the first.
- Frame first. Write the rubric into the dossier before any lane runs: one gradeable criterion per screen in `references/design-red-flags.md`, plus interface depth and caller usage. A rubric written after the proposals arrive is fitted to them.
- One brief, the same bytes for every lane: the task, the grounding artifacts by path, and `references/runner-prompt.md` as the discipline each lane applies. A lane answers with one design package shaped per `references/rationale-template.md`: the caller's usage written first, then the type sketch, function signatures, module map and prose rationale derived from it.
- Lane runners go out as `bee dispatch prepare --runtime claude --kind advisor --role lane-1 --json`, then `--role lane-2`, and `--role lane-3` for a third. Use the payload it returns exactly as it returns it. Never pick a model or a subagent type by hand: each lane's model comes from the `lane-1` to `lane-3` roles of the bee team table, so a different model per lane is a team setting.
- Design it twice. Require at least two structurally distinct candidates before synthesis, even when the first looks sufficient: whole-shape alternatives, never point fixes inside one shape. This is exhausting the design space, made concrete.
- Screen every candidate against `references/design-red-flags.md` before synthesis. Reject or revise shallow modules, information leakage, temporal decomposition and pass-through methods.
- Compare viable candidates on interface depth. Prefer the design that hides more complexity behind a smaller, simpler public surface. A rich interface keeps call chains short by concentrating capability instead of scattering it across layers.
- Converge the way the procedure says: base, grafts, a dossier at `docs/history/<feature>/blind/<run-id>.md`, and `bee blind check --dossier <path>` green before `bee decisions log`. Lanes that converge ship the consensus; lanes that diverge wildly mean the frame was loose, so reframe the rubric and re-run, never average.
- A convergence that picks no answer hands the human the dossier unchanged and never resolves by coin flip.

The result is one synthesized design package. The dossier's `## Chosen` section, its base and grafts, populates the rationale's "Synthesis decision" section.

## Phase C: Agree (opt-in)

Default: proceed directly to implementation with the synthesized design. No human checkpoint.

Opt in to a checkpoint when the invoker explicitly asks: "/bee-architect with checkpoint", "stop and show me before implementing", or similar. Then surface the synthesized design and pause for sign-off. An unattended run never opts in.

The synthesis can ship as its own commit either way. That is the scaffold-first mode of foundational thinking: get the data shapes and the contract right first, so later commits read as filling in bodies against a stable contract. Planned and scoped breakage during fill-in is fine, because the outcome is what is held fixed, not every intermediate state. For adversarial pressure on the design before implementing, press on the synthesized sketch with the hardest objection you can build, and answer it in the rationale.

If the human pushes back on the shape, in a checkpoint or after the fact, treat that as Phase A evidence. Re-ground and re-run Phase B before writing more code.

## Phase D: Implement against the sketch

Replace `not implemented` bodies with code, pseudocode with logic. The synthesized sketch is the contract. When the work runs as cells, each cell's action names the part of the sketch it fills and the contract it must keep (`bee-principle-lock-the-contract`).

Deviations from the sketch are signal worth surfacing, not friction to absorb silently. If a function needs a parameter the sketch did not anticipate, ask whether the sketch was wrong, the requirement was missed, or the implementation is overreaching. Surface it; do not bolt it on.

## Phase E: Scrap when the architecture is wrong

If implementation keeps producing friction the sketch cannot absorb, throw the sketch out. Do not bolt fixes onto a wrong design: redesign from first principles, and fix the root cause instead of the symptom it shows.

The signal is a pattern, not single instances. Tells:

- The same shape of workaround appearing repeatedly across unrelated code.
- Multiple unrelated edge cases that all need special-case branches.
- Types that need escape hatches (`any`, casts, optional fields always set in practice) to compile.
- The "we need a lock" reflex when the sketch said the state was not shared.
- Callers having to know the abstraction's internal rules to use it.
- Two or more independent Phase D deviations of the same shape across the implementation. Surfacing deviations is Phase D's job; a repeated pattern of them is Phase E's trigger.

Use judgment. A few edge cases do not condemn an architecture. Some problems are legitimately complex; complexity in the data is not complexity in the design. The rewrite signal is repeated friction of the same shape, not single hard cases. When a second fix fails the same check, apply `bee-principle-attack-the-premise`.

When you scrap:

1. Re-run `bee-how` over what has been built. The implementation lessons enter the new design as inputs, not vibes.
2. Redesign as if the new constraints had been day-one assumptions.
3. Subtract before adding (`bee-principle-subtract-before-you-add`). The new sketch should be smaller than the old one before it grows.
4. Return to Phase B and open the lanes again.

## Outputs

The caller's usage is written first and the type sketch derived from it. One file with new types and signatures for small changes; a module map plus type definitions for larger work. The rationale ships alongside, shaped per `references/rationale-template.md`, including the usage sketch and the synthesis decision.

## Handoff

Sketch delivered, or the checkpoint reached. Invoke bee-hive skill.
