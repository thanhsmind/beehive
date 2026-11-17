---
name: bee-formal-verifying
description: >-
  Generate a project-local formal verification skill under .bee/verify/ for critical invariants, races, or state machines and prove it fails on a broken variant. Use when tests cannot reach bad step orders or every input. Not for updating an existing check (bee-formal-upkeep).
disable-model-invocation: true
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: The generated skill is source under `.bee/verify/`; the vendored bee binary is what renders it into every runtime skill home. Absent, the generated skill still exists and still runs, but only the runtime whose home was written by hand can see it.
---

# Formal verifying — generate a project-local formal check

Unit tests check the scenarios an author thought to write. Formal verification checks every case within stated bounds: every interleaving of steps in a state model, every input up to a size limit, or every possible input. This skill picks the part of the codebase where that exhaustive coverage matters most, builds and runs one check, and writes a project-local skill so future agents can verify changes against that promise.

Write the generated skill for an agent reading it cold mid-change. A reader must be able to trust the result and act on it without knowing the verification language.

## 1. Pick the target from the code

Inspect the codebase first; ask the user only for facts you cannot observe. Start in the area or bug the user named, if any.

A strong target is a promise the code makes whose failure harms users and whose failure modes tests rarely hit (concurrent state transitions, lock/claim handoffs, retry loops, parsers, or invariants across boundaries). Pick the highest-risk promise in the repo rather than the easiest one to model.

## 2. Let the question pick the tool

Choose the lowest-cost tool that answers the exact question:

- **Step order and concurrency:** use a model checker over a small model of the design. TLA+ with TLC is the default for event-order questions in any language.
- **Every-input code behavior:** use a code-level verifier (for memory safety, integer overflow, or data-structure invariants) when one exists for the language.
- **No verifier on the language:** say so explicitly and test the real code with generated inputs instead of modeling the function by hand.
- **Unbounded size:** use a proof assistant only when a bound cannot settle the property.

Prefer a tool the repo's developers can install and run reproducibly, and explain why you chose it in the handoff.

## 3. What makes the check trustworthy

- **True to the code.** Every model action or proof element maps to code a reviewer can open. One indivisible step in the verification corresponds to one indivisible step (such as one lock block) in the code. Use the repo's own names.
- **Able to fail.** A check that has never failed proves nothing. Show each rule failing on a broken variant (for example, a removed guard or a reordered step) before trusting a pass.
- **Findings stay findings.** If current code violates a rule, report the counterexample in the code's words. Never weaken a rule or model to force a pass, and never edit product code unless the user asks.
- **Honest about scope.** Record bounds, assumptions, and exclusions. Passing means the property holds within those bounds, not across the entire system.
- **Readable without the tool.** State each rule and its harm in one plain sentence. Write counterexamples as short step sequences in the code's own terms.
- **Cheap to rerun.** Provide one command, pin tool versions, and keep the run fast enough for agents to run on every relevant change.
- **Kept in step with the code.** Record the covered files, covered functions, and the sync commit where the verification was last confirmed to match the code.

## 4. Generate the skill under `.bee/verify/<name>/`

Create `.bee/verify/<name>/` (never `bee-` prefixed, because bee prunes unowned `bee-*` skills). Include:

1. **`SKILL.md` with YAML frontmatter:** set `name: <name>` and a `description` naming the covered area and when to open the skill (without frontmatter, the skill does not register). From `SKILL.md` alone, a cold agent must be able to:
   - tell whether a change touches covered code;
   - run the check with one command;
   - distinguish a real code bug from a modeling error;
   - update the model when a change is intended, moving the sync commit forward only after confirming the verification matches the new code.
2. **One-command check script:** place an executable script (for example, `check`) under `.bee/verify/<name>/` and document its invocation as `bash .bee/verify/<name>/check ...`. Because bee's render copies files into runtime skill homes without the executable bit (`0644`), invoking with `bash` ensures both source and rendered copies run.
3. **Coverage record and sync commit:** list each model action or check alongside the exact file and function it represents, state what changes trigger the check, and include the `git diff <sync-commit> -- <covered-files>` command.
4. **Broken variants:** keep explicit broken configurations (or a `--variants` flag on the check script) so any future update can prove every rule still catches failures.
5. **Reading TLC results:** when the check uses TLA+ and TLC, point to [`references/statechart.md`](references/statechart.md) for drawing a Mermaid statechart and replaying counterexamples against real code. Copy none of another skill's internal commands or options into the generated skill.

See `.bee/verify/formal-cell-lifecycle` (`SKILL.md`, `check`, `CellLifecycle.tla`, `OwnerBlindClear.cfg`, and `NoOwnershipCheck.cfg`) as the worked example in this repo.

Re-render the generated source into every runtime skill home (`.claude/skills/`, `.codex/skills/`, `.agents/skills/`, `.opencode/skills/`) using `bee onboard --verify-only --apply` (no bee checkout needed). Never hand-edit the rendered copies; they are overwritten on the next apply.

## 5. Prove it on a broken variant before handoff

Follow the generated skill once as a cold agent would:

1. Run the check as-is and confirm it passes (or fails only on the findings you are reporting).
2. Introduce a scratch change in the real code (or run the broken variant) that violates the guarded property, and confirm the check catches it.
3. Drop the scratch change and confirm the check returns to its baseline result.
4. Run `bee onboard --verify-only --apply` so every runtime home has the proven skill.

A generated verify skill that has not been run and shown to fail on a break is a draft, not a deliverable.

## 6. Hand off

Report to the user:
- what promise was verified, which tool was chosen and why, and the bounds and assumptions;
- what the check found in plain words and how to run it;
- what remains out of scope;
- `bee-formal-upkeep` as the maintenance loop for bringing the check back in line after changes that skipped it.
