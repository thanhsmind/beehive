# diet-8 — bee-reviewing thin-body migration

**Status:** [DONE]
**Outcome:** `skills/bee-reviewing/SKILL.md` rewritten to the D7 thin-body doctrine:
22,303 → **8,190 bytes** (≤ 8,192). All exiled text landed in
`skills/bee-reviewing/references/reviewing-reference.md` (new sections: Scope
Resolution in full, Scope Freeze in full, Frozen-Judge Flags, Gate 4 Bypass
Mechanics, Lane Scaling in full, Required Inputs and Delegation, Headless in
full — alongside the pre-existing Specialist Dispatch, Conditional Reviewers,
Finding Schema, Review Cells and Backlog Routing, Session Record Checklist,
Delta Re-Review Protocol, Human UAT, Finishing Checklist, Red Flags); new
`skills/bee-reviewing/references/provenance.md` maps every body rule to its
decision IDs. Baseline lowered to 8190, `bee-reviewing` appended to
`migrated[]` (provenance grep now active — 0 hits across all 18 skills),
`notes["bee-reviewing"]` deleted. Regen obligation ran in-cell: plugin mirror
render, onboard `--apply`, release manifest `--write`/`--check`. Full verify
chain green.

**Files:** `skills/bee-reviewing/SKILL.md`,
`skills/bee-reviewing/references/reviewing-reference.md`,
`skills/bee-reviewing/references/provenance.md` (new),
`scripts/skill-body-budget.json`,
`docs/history/codex-harness-hardening/release-manifest.json` + regenerated
mirror trees (`.claude/skills`, `.claude-plugin/skills`, `.codex-plugin/skills`,
`.agents/skills`, `.bee-render.json` stamps, `.bee/onboarding.json`).

Full trace and verify output: `.bee/cells/diet-8.json`.

## Side-by-side behavior checks

### s1 — trigger law (user-invoked-only, 565e68d0) survives verbatim

Before (`Trigger — explicit user intent only`):

> Dispatch this skill only when the user names one of these intents (R1):
> - "review this / review this feature"
> - "review all of today's work"
> - "review feature A and B" (or any named list)
> - "review the diff from X to Y"
> - "review everything unreviewed before release"
> None of the following are triggers, no matter how tempting the alignment
> feels: a cell, slice, feature, or working day finishing — verification
> completing is not a review request; the words "merge", "ship", or "release"
> on their own (7.4/A9): when the user asks to merge/ship/release while
> unreviewed or stale work exists, report the count and risk level (`node
> .bee/bin/bee.mjs reviews status`), then ask exactly ONE question — does the
> user want a review session for that scope? Only an explicit yes starts a
> session; silence or a non-answer means no dispatch...

After (`Trigger — explicit user intent only`):

> - "review this / review this feature"
> - "review all of today's work"
> - "review feature A and B" (or any named list)
> - "review the diff from X to Y"
> - "review everything unreviewed before release"
> Never a trigger: a cell/slice/feature/day finishing · "merge"/"ship"/"release"
> alone (report count+risk via `reviews status`, ask exactly ONE yes/no
> question, silence stays `unreviewed`) · gate bypass being on.

The five trigger phrasings are byte-identical; the three non-trigger
conditions (finished work, bare merge/ship/release, bypass alone) all survive
with the same mechanics (one boundary-risk report, exactly one yes/no
question, silence = no dispatch, stays `unreviewed`). The decision hash
`565e68d0-327f-404e-b49e-d1c61ba81bfd` and the `R1`/`7.4/A9` citations moved to
`references/provenance.md` (D8 provenance exile) — rule meaning unchanged. A
"review feature A and B" request still dispatches identically; a bare "let's
ship this" still gets the one-question, count+risk protocol.

### s2 — Gate 4 verbatim questions and P1-blocks-merge law survive intact

Before (`Gate 4 (wording is fixed) — lives only inside a session`):

> Then verbatim:
> - P1 > 0 → "P1 findings block merge. Fix before proceeding?"
> - P1 = 0 → "Review complete. Approve merge?"
> Never continue past open P1s without explicit user acknowledgment. Silence
> is not acknowledgment. A session stays `blocked` (A11) until every P1's fix
> and delta re-review (§6) pass.

After (`Gate 4 (wording fixed) — lives only inside a session`):

> Then verbatim:
> - P1 > 0 → "P1 findings block merge. Fix before proceeding?"
> - P1 = 0 → "Review complete. Approve merge?"
> Never continue past open P1s without explicit acknowledgment — silence isn't
> acknowledgment; session stays `blocked` until every P1's fix + delta
> re-review pass.

The two Gate 4 question strings are byte-identical in both the P1>0 and
P1=0 forms; the P1-blocks-merge law (never continue past an open P1, silence
is not acknowledgment, session stays `blocked` until every P1's fix and delta
re-review pass) is unchanged in meaning. The `A11` citation moved to
`references/provenance.md`. A scope with 2 open P1s still gets asked the
exact P1>0 question; a clean scope still gets asked the exact P1=0 question.

### s3 — severity vocabulary and P1-blocks-merge scoring survive intact

Before (`2. Severity and Synthesis`):

> - **P1** — security breach, data loss, breaking change, production blocker.
>   Blocks session approval.
> - **P2** — real performance, architecture, reliability, or important test
>   gap.
> - **P3** — cleanup, docs, future debt.

After (`1-2. Review & Synthesis`):

> - **P1** — security breach, data loss, breaking change, production blocker.
>   Blocks approval.
> - **P2** — real performance, architecture, reliability, or important test
>   gap.
> - **P3** — cleanup, docs, future debt.

The three severity definitions are unchanged word-for-word (P1's "Blocks
session approval" compressed to "Blocks approval" — same meaning, no scope
change: only a session ever reaches Gate 4). A security breach still scores
P1 and blocks merge identically before and after.

## Notes

- Provenance exile (D8): zero `\((D\d|AO\d|decision [0-9a-f]|hardening-\d|plan \d)` matches in the body (`skill_budget_fence.mjs` bare run confirms — 18 skills checked, 0 findings); the rule → decision-ID map is `references/provenance.md`, covering all citations found in the pre-migration body (565e68d0, R1, R4, R5, R6/A7, R8, R9/A12, A6, A10, A11, A12, P12/decision 0018, P16/decision 0021, AO12/B1, AO14, decision 0009, decision 0010 boundary, spec §7.3/§7.4/§7.5/§11.1, goal 1, goal 5, critical pattern 20260711).
- All quoted headings in the body resolve to real headings in `references/reviewing-reference.md` (`skill_lint.mjs` anchor-integrity check green) — several pointers use a shortened substring of the full reference heading (e.g. body `("Scope Resolution")` resolves to reference `## Scope Resolution in full`), which the lint's substring match accepts and which keeps the body pointer shorter without breaking navigation.
- `okf_instructions_fence.mjs` stayed green after the move — bee-reviewing carries no `docs/specs`/`docs/knowledge` bundle-branch lines, so the move had nothing to keep line-local.
- `references/reviewing-reference.md` keeps its own decision-ID citations (R5, A6, A10, etc.) — the D8 provenance grep only fences `skills/*/SKILL.md` bodies, not references, matching the pattern already established in `bee-planning`'s and `bee-hive`'s reference files.
- Deviation (recorded in cell trace): none — every body rule found a home either as a load-bearing invariant (Trigger, Gate 4, severity vocabulary, Red Flags) or a reference pointer; no content was dropped, only relocated and compressed.
