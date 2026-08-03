---
type: bee.area
title: "Doctrine Layer — the prompt-writing standard"
description: "The standard every edit to bee's instruction text is judged by: the four-question line filter, add-on-failure, one rule one home, verifiable-imperative style, the deterministic-backstop preference, and the standing record that no size ceiling exists or may be introduced."
tags: [doctrine-layer, instruction-text, context-budget]
timestamp: 2026-08-03
bee:
  id: doctrine-layer-prompt-writing-standard
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [areas/doctrine-layer/overview.md]
  decisions: ["prompt-diet D1-D7 (docs/history/prompt-diet/CONTEXT.md, 2026-08-03)", "8f63adb4 + budget-fence-removal D1/D6 (a size ceiling on instruction text is never a standing rule)"]
  sources: ["docs/history/prompt-diet/CONTEXT.md", "Lulla et al., arXiv 2601.20404", "Gloaguen et al. (ETH SRI), arXiv 2602.11988", "Khatri, arXiv 2607.27250", "Chatlatanagulchai et al. (NAIST), arXiv 2511.12884"]
  authoritative_for: "doctrine-layer: the prompt-writing standard"
---

## Purpose

Every line of instruction text bee ships — the AGENTS operating block, a
SKILL.md body, a procedure reference, an agent file — is loaded into a model's
context and paid for on every turn it is present. Four independent studies of
agent context files converge on the same finding: instruction text is not
free, and most of it does not steer. Named tools are invoked at roughly 100x
the rate of unnamed ones (the ETH SRI tool-mention counts: 1.6 vs <0.01
calls per instance); overview prose shows no measurable steering effect
(time-to-first-patch-file unchanged on both benchmarks tested); and every
line costs — context files added +20-23% inference cost across agents, while
developer-written files moved solve rates only +4% and LLM-generated ones
moved them -3%.

This concept is the standard any authored or edited instruction line is
judged by. The numbers above are evidence for the standard's shape — they are
never a gate, and no size threshold derives from them (see the last Business
Rule).

## Entry Points & Triggers

- **Authoring** — any new line proposed for the operating block, a SKILL.md,
  a reference, or an agent file passes the four-question filter before it
  lands.
- **Editing** — any diet, rewrite, or restructure of existing instruction
  text applies the one-rule-one-home and pinned-wording rules.
- **Review** — a reviewer of an instruction-text diff asks the same four
  questions of every added line.

## Behaviors & Operations

**The four-question line filter.** A line earns its place only if it survives
all four questions:

1. *Is it self-discoverable?* If the agent would find it anyway — by `ls`, by
   `rg`, by running `--help` — the line restates what the environment already
   teaches, and the research says such restatement does not steer (overview
   prose: no measurable effect on time-to-first-patch-file).
2. *What concrete costly behavior does its absence cause?* A line that cannot
   name the failure it prevents is narration. Name the failure or cut the
   line.
3. *Is it verifiable as written?* A reader — or a check — must be able to
   tell whether the line was obeyed. "Be careful" is not verifiable; "run X
   before Y" is.
4. *Does it conflict with another line?* Two lines pulling in different
   directions cost more than either alone; the conflict is resolved before
   either ships.

**Add-on-failure, never add-upfront.** Instruction text grows from observed
failure, not from anticipation. A line earns its place when one of three
things has happened: the same failure occurred twice; a review caught a
defect the text should have prevented; or the same correction was typed by
hand in two sessions. Anticipatory lines are exactly the ones the studies
found inert — they read as coverage while steering nothing.

**One rule, one home.** The duplication boundary settled for the router
(`router-triage-and-the-agents-md-duplication-boundary.md`, "Business Rules"
R4: a document may drop only what the operating block genuinely carries,
verified against the real document, never assumed) generalizes to every
instruction document. A boundary rule is stated in full exactly once — the
AGENTS operating block is the canonical home — and everywhere else it appears
as a one-line cite plus the local delta that document actually adds. A
near-verbatim restatement is paid for on every load of every document that
carries it. Presence-pinned strings are the one asymmetry: a pinned string
may gain occurrences (a presence test tolerates extras) but an existing
occurrence is never reworded.

**Verifiable-imperative style.** Commands with arguments over adjectives:
"run X" over "be careful about X", "cite the file and heading" over "cite
properly". An imperative with a concrete object can be obeyed, checked, and
diffed; an adjective can only be agreed with. This is the line-level form of
filter question 3.

**Deterministic backstop preference.** An absolute MUST or NEVER that is
structurally reachable — a file write, a command invocation, a state
transition — belongs in a hook or a permission, not in prose. Markdown
carries only what enforcement cannot reach: semantic rules, conversational
rules, judgment calls. The doctrine layer already records why the prose copy
still binds where no guard exists (`unenforced-obedience.md`, "Behaviors &
Operations" B5 — doctrine binds the assistant even where no mechanism
enforces it, and a guard's silence is not an approval); the preference here
is the complement: where a mechanism *can* enforce the rule, build the
mechanism, and let the prose shrink to the judgment the mechanism cannot
carry.

## Business Rules

- **R1** — Every added instruction line passes the four-question filter:
  self-discoverability, named costly failure, verifiability as written, and
  no conflict with an existing line.
- **R2** — A line is added on evidence of failure (twice-observed failure, a
  review catch, a twice-typed correction), never on anticipation.
- **R3** — A boundary rule has exactly one full statement, in the AGENTS
  operating block; every other occurrence is a one-line cite plus local
  delta. Pinned wording survives verbatim wherever it already stands.
- **R4** — Instruction prose is written as verifiable imperatives; a rule
  that cannot be checked as written is rewritten until it can be, or cut.
- **R5** — An absolute rule over a structurally reachable action is
  implemented as a hook or permission; markdown keeps only what enforcement
  cannot reach.
- **R6** — **No standing size ceiling exists or may be introduced** for any
  instruction text. This is the settled rule of budget-fence-removal, carried
  in `placement-and-anchoring.md` ("Business Rules" R5/R6): a ceiling makes
  an author fund a correct addition by cutting correct text, so density is
  judged per edit, by whether each line changes behavior — never against a
  recorded size. A diet is a legitimate one-off event that leaves no
  permanent gate behind, and the prompt-diet feature this standard emerged
  from is one such event. The research numbers in Purpose are evidence that
  dieting was worth doing once; they are not a budget.

## Pointers (implementation)

- `AGENTS.md` — the canonical home R3 names; its rendered master is
  `packages/bee/AGENTS.block.md`.
- `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`
  — the settled duplication boundary R3 generalizes.
- `docs/knowledge/areas/doctrine-layer/placement-and-anchoring.md` — the
  placement question, the anchor tests, and the budget-fence-removal record
  R6 cites.
- `docs/knowledge/areas/doctrine-layer/unenforced-obedience.md` — why the
  prose copy still binds where R5 leaves no mechanism.
- `packages/bee-rs/crates/bee/tests/instruction_laws.rs` — fails the build if
  a ceiling-shaped construct on instruction text reappears in shipped tooling
  (R6's enforcement).
- `packages/bee-rs/crates/bee/tests/pointer_integrity.rs` — every citation
  from an instruction document to a reference must resolve to a real file and
  a real heading (the check behind R3's cite-plus-delta form).
- `docs/history/prompt-diet/CONTEXT.md` — the locked decisions (D1-D7) this
  standard operationalizes.
