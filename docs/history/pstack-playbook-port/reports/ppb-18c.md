# ppb-18c — bee-tdd checked against its pstack source

Scope: what ppb-18 wrote in commit `1e9958527`. That is `skills/bee-tdd/`
(`SKILL.md`, `LICENSE.pstack`) and the port notes
`docs/history/pstack-playbook-port/port-notes/skill-bee-tdd.md`, checked
against the pstack skill `pstack/skills/tdd/SKILL.md` (no reference files) at
pstack commit `b9ddc83`. The licence was compared with `diff`. The five
rendered copies (`.claude-plugin`, `.codex-plugin`, `.claude`, `.agents`,
`.opencode`) of `SKILL.md` were compared with `diff -q` and match the source
tree.

Verdict: no source step is missing. Every workflow step, paragraph, guardrail
and final-response line of the source is present in the pstack wording. The one
removed item, `disable-model-invocation: true`, has a port-notes entry.
`LICENSE.pstack` is identical to the pstack `LICENSE`. The pstack-only tool
check is clean. No model and no subagent type is picked by hand. The
description is 285 characters and says when not to use the skill. There is one
finding, P2, on an added sentence; it does not touch a source step.

Labels: **present** means the step is in the new file. **dropped (noted)**
means it is gone and the port notes name it. **dropped (not noted)** means it
is gone and the notes say nothing, which D2 (`063a21e4`) counts as missing.
**missing** means a method step is gone with no reason.

## `SKILL.md`

| Source step or rule (source line) | New home (bee-tdd line) | Status |
|---|---|---|
| Frontmatter `name: tdd` (2) | `name: bee-tdd` (2) | present |
| Frontmatter description (3) | 4, rewritten: what it does, when, when not | present |
| Frontmatter `disable-model-invocation: true` (4) | removed | dropped (noted: frontmatter paragraph) |
| Intro: make the broken behavior executable first; regression test fails before, passes after (9) | 12, verbatim plus a pointer to `bee-principle-red-before-green` | present |
| Do not force an impractical test; list of costs; use closest verification (11) | 14, verbatim | present |
| Step 1 Understand the bug (15) | 20 | present |
| Step 2 Choose the narrowest executable check; do not invent a test path (16) | 21 | present |
| Step 3 Write the failing test first; encode intended behavior (17) | 22, plus `bee-principle-test-behavior-not-structure` | present |
| Step 4 Run the new test before fixing; fails for the intended reason (18) | 23 | present |
| Step 5 Fix the bug, smallest change (19) | 24 | present |
| Step 6 Rerun the regression test (20) | 25 | present |
| Step 7 Run nearby validation (21) | 26 | present |
| Impractical: do not silently skip; explain why; closest executable check, with examples (25) | 30, verbatim plus "that check is the red", citing `bee-principle-reproduce-first` | present |
| Prefer no new test over a bad test; what a bad test is (27) | 32, verbatim | present |
| Guardrail: do not change tests to match a wrong implementation (31) | 36 | present |
| Guardrail: do not weaken existing assertions without reason (32) | 37 | present |
| Guardrail: keep the regression test focused (33) | 38 | present |
| Guardrail: no tests on weak signal; scripted check and say why (34) | 39 | present |
| Guardrail: flaky bug, make it deterministic (35) | 40 | present |
| Guardrail: broader class, land the focused path first (36) | 41 | present |
| Final Response: report the evidence, not just the outcome (40) | 45, plus the "green only beside a run you just made" line | present |
| Final Response: name the failing-before check and its failure (42) | 47 | present |
| Final Response: name the passing-after run and nearby validation (43) | 48 | present |
| Final Response: if no failing-before evidence, say why and the closest check (44) | 49 | present |

Additions with no source counterpart, each named in the port notes ("Added, not
in the source"): the `metadata` block (frontmatter paragraph); the Tester
contract paragraph at line 16; the "green only beside a run you just made"
sentence at line 45; the Handoff section at lines 51-53. The principle pointers
at lines 12, 22 and 30 are listed under "Tool swaps".

## `LICENSE.pstack`

`diff` against the pstack `LICENSE` prints nothing.

## Pstack-only tool names

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-tdd/` prints
nothing and exits 1. The source names no pstack tool either (no subagent,
model, loop, stacked-PR tool or review bot), which matches the port notes'
"no tool was swapped".

Every bee name the skill cites resolves to a directory under `skills/`:
`bee-principle-red-before-green`, `bee-principle-test-behavior-not-structure`,
`bee-principle-reproduce-first`, `bee-swarming` (section "Tester contract",
`skills/bee-swarming/SKILL.md:263`) and `bee-hive`.

## Hand-picked model or subagent type

None. `grep -i 'subagent_type\|model'` over `skills/bee-tdd/SKILL.md` prints
nothing. The skill dispatches nothing.

## Description

285 characters (the folded `description:` value), matching the port notes. It
says what the skill does, when to use it (the user asks for TDD, a failing test
or a regression test, or the bug has a cheap local test target) and when not
to: "Not when the test path is unclear or costly, or for new-feature tests." It
fits the 300-character limit. The source's "integration-heavy" skip is not in
the description; the body keeps it (line 14, "slow end-to-end
infrastructure").

## Findings

1. **P2, wrong claim in an added sentence.** `skills/bee-tdd/SKILL.md:16` says
   the skill is "the method for a worker who holds both halves, outside a cell
   or in a `tiny` lane". `skills/bee-planning/SKILL.md:99` says a `tiny` lane
   is "one cell, plus its tester cell when it touches code", and
   `skills/bee-swarming/SKILL.md:38` dispatches every code cell, `tiny`
   included, through `bee dispatch prepare`. So a `tiny` code cell also splits
   implementer and tester; no worker holds both halves there. The port notes
   repeat the claim ("Tool swaps", last bullet). Fix: drop "or in a `tiny`
   lane" from line 16 and from the port notes, leaving "outside a cell". This
   is not a source step, so it does not change the step table.

## Verify

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-tdd/ ; test $? -eq 1`
— no output, `test` exits 0.
