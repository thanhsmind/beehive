# Walkthrough Reference

Load when shaping's Brief move writes the post-ship walkthrough (post-Gate-3, `standard`/`high-risk` only). The walkthrough is the post-implementation summary that closes the loop: reconstructed from execution records, evidence-honest, findings-transparent.

Path: `docs/history/<feature>/walkthrough.md`. One file per feature, written once at Gate 3, updated in place if the feature is reopened. Never forked.

## The one rule that shapes everything

**Reconstruct from what shipped, not from what was planned.** `implement-plan.md` and `plan.md` are intent. The walkthrough's sources are the *execution records*:

| Section | Source (execution reality) | Never from |
|---|---|---|
| What shipped | capped cells' `outcome` + `files_changed` | `implement-plan.md` prose |
| How it was verified | the cells' recorded `verify` command output + the UAT record | "should pass" / the plan's Validation section |
| How to test it yourself | UAT record + SEE·CALL·RUN decisions | invented steps |
| Deviations from plan | cells' `deviations` field | (this section only exists because the plan and reality differ) |
| Known limitations / follow-ups | review P2/P3 findings + their backlog links + open gaps | omission to look clean |

Where the plan and the records disagree, the records win and the difference is named — silent divergence misleads the next reader.

## Template

```markdown
---
artifact_contract: bee-walkthrough/v1
feature: <feature-slug>
lane: standard | high-risk
shipped: YYYY-MM-DD
sources: [<cell-ids>, <review report>, <UAT record>]
---

# Walkthrough: <Feature>

<One-paragraph plain-language summary of what the user can now do that they
couldn't before. Present tense. From the capped cells, not the plan.>

## What shipped

<The observable outcome delivered, per capped cell. Each line ties to real
behavior a user or consumer can see now.>

- <capability> (`<cell-id>`)

## How it was verified

<The ACTUAL recorded evidence. Quote real verify outputs. State the coverage
honestly — what each check does and does NOT cover.>

- `<verify command>` → <recorded result> (`<cell-id>`)
- UAT: <what was walked through and its outcome, or "NOT performed — outstanding gap: <what to run>">

## How to test it yourself

<Manual steps a human runs to confirm the feature, from the UAT / SEE·CALL·RUN
decisions. Concrete: where to click / what to call / what to expect.>

1. <step> → <expected observable result>

## Deviations from plan

<What execution changed from the implement plan and why, from cell `deviations`.
"None" only if the cell traces genuinely record none.>

- <what changed> — <why> (`<cell-id>`)

## Known limitations / follow-ups

<Deferred P2/P3 findings with backlog links; open gaps. None block shipping,
but the reader deserves to know what is still open. Omitting these to look
polished is a violation, not a courtesy.>

- <finding> — deferred to backlog `<id>` (`<severity>`)
```

## Quiz (optional)

When presenting the walkthrough, offer: "Muốn kiểm tra nhanh 3–5 câu về thay đổi này trước khi chốt không?" On yes, append a `## Quiz` section — 3–5 questions whose answers come ONLY from the walkthrough's own sections (What shipped / How it was verified / Deviations / Known limitations). Good questions probe understanding the gate litmus cares about: "what does actor X observe now that they didn't before?", "which check was NOT run?", "what changed versus the plan, and why?". Grade conversationally, cite the section for each answer. A miss is a signal to walk that section together — never a blocker, never recorded as a failure.

## Rules

- **Evidence honesty (shared with the rest of bee).** Never claim broader verification than the evidence shows. Unit-level `verify` passes do not establish end-to-end behavior; a skipped or pending UAT is stated as an outstanding gap the human should close, never smoothed into "works end-to-end".
- **Findings-transparent.** Every deferred finding and every deviation appears, with its link. "Polished" means honest and well-organized, not curated to hide open edges. A vague "minor cleanup remains" is worse than either naming or omitting — it alarms without informing.
- **Lane-scaled.** Only `standard`/`high-risk` features get a walkthrough. `tiny`/`spike`/`small` close on the cap trace and commit; do not manufacture a walkthrough for them.
- **Closes the status.** After writing, set the feature's `implement-plan.md` frontmatter `status: Shipped`.
- **In place, never forked.** A reopened feature updates its existing walkthrough; no `-v2`/dated copies.
- Secrets and PII never appear.
