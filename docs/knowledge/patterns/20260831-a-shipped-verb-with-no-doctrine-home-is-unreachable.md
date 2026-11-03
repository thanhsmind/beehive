---
type: bee.pattern
title: A shipped verb with no doctrine home is unreachable — 104 of 191 bee commands are named nowhere an agent reads
description: letter-reflection shipped `bee mailbox reflect` with doctrine as its only trigger, the doctrine line was never written, and the verb recorded zero entries; a sweep of the whole command surface shows it was one of 104 orphans
tags: [cli, docs, skills, capture, onboarding]
timestamp: 2026-08-31
bee:
  id: pattern-20260831-orphaned-verb-no-doctrine-home
  lifecycle: active
  areas: [human-mailbox, workflow-state]
  sources: ["reflection-becomes-lesson, 2026-08-31 — CONTEXT.md and the plan-step hat wave", "sweep of `bee --help --all` against skills/, AGENTS.md, CLAUDE.md, packages/bee/AGENTS.block.md and packages/bee/prompts/, 2026-08-31", "decision 662647d1, 2026-08-06 — the same shape found for the promote-apply loop"]
  polarity: pitfall
  evidence: measured
---

# A shipped verb with no doctrine home is unreachable

`letter-reflection` shipped `bee mailbox reflect` on 2026-08-30: the verb,
its two-part refusal, the letter section, the tests. Its only trigger was
doctrine — an agent was supposed to know to call it. The doctrine line was
never written. Zero reflection entries existed a day later, and the store
had never held one.

Nothing was broken. Every test was green. The feature was simply
**unreachable**: no file an agent reads at context-load time named the verb,
so no agent ever called it.

## This is not a one-off

Sweeping the whole surface on 2026-08-31 — every command from
`bee --help --all` against `skills/`, `AGENTS.md`, `CLAUDE.md`,
`packages/bee/AGENTS.block.md` and `packages/bee/prompts/`:

**104 of 191 commands are named nowhere in the instruction layer.**

Some are legitimately internal (`dev *`, `perf *`, `rs-info`, and the
`state *` spellings that already ship a flow alias). Many are not:
`capture count`, `feedback digest`, `knowledge promote`, `mailbox mark`,
`models show`, `decisions search`, `discovery list`, `triggers list`,
`reviews list`, and the whole `supervisor *` group.

The same shape was already found once, on 2026-08-06, for the promote-apply
loop (decision `662647d1`): *"zero hits for promote-proposals or knowledge
promote across skills/, AGENTS.md and CLAUDE.md, so a compounding run
re-mined the raw traces instead."* It was fixed for that one verb, as prose,
and the class went unfixed.

## Why it survives review

A verb's tests prove the verb works. Nothing tests that anyone will call it.
The gap sits exactly between two things that both look complete: a green
suite, and a shipped command. Neither is wrong; the join is missing.

The failure is also silent in the worst way — not an error, an absence.
`letter-reflection` was reported as delivered, correctly, and the delivery
record is accurate. What no record said was that the feature had no reader.

## The rule

**A cell that ships a new verb ships its doctrine line in the same commit,
or it has not shipped.** The line goes wherever the readers are, and the
homes do not derive from one another: this repo's `AGENTS.md`, the host-repo
block `packages/bee/AGENTS.block.md`, and the rendered worker prompt
`packages/bee/prompts/worker-cell.md` reach three disjoint reader classes,
with no parity test syncing them (`packages/bee-rs/crates/bee/tests/pointer_integrity.rs:35`).
A verb that reaches only the first is invisible to every host repo and every
dispatched worker.

## The durable owner

Prose has now failed this twice, so the escalation is a check, not another
paragraph: a test that walks the command index against the instruction
layer and refuses a command that appears in neither, with an explicit,
justified allowlist for the genuinely internal ones. The allowlist is the
point — it turns "nobody documented this" into a named decision someone had
to write down.

Filed as a backlog item with the 104-verb baseline as its starting evidence.

## The tell

If a feature's only trigger is "the agent will know to do this", ask where
that knowledge is written and open the file. If the answer is a plan, a
decision record, or a delivery note, the feature is unreachable — none of
those load into an agent's context when the moment arrives.
