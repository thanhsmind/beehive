# Port notes — skill bee-interrogate, cell ppb-15

Source: pstack skill `interrogate` (`SKILL.md` and four reference files:
`rubric.md`, `code-quality-review.md`, `reviewer-prompt.md`, `lead-judgment.md`)
at commit `b9ddc83`, MIT, copyright 2026 Lauren Tan. The notice is
`skills/bee-interrogate/LICENSE.pstack`, a verbatim copy of the pstack `LICENSE`.
This file is the only place a pstack-only tool is named; no file under
`skills/bee-interrogate/` names one.

Kept as the base, with the swaps below: the five steps (scope, intent, reviewers,
synthesize, lead judgment), the verdict shape (Intent, Reviewers, Act On,
Consider, Noted, Dismissed, Agreement Map), `rubric.md` and the Core Prompt and
seven dimensions of `code-quality-review.md` unchanged, the reviewer template and
the lead-judgment framework.

Frontmatter follows `bee-how`: `name`, a description of 276 characters (what it
does, when to use it, what it is not for) and `metadata` with `ecosystem: bee`
and a `bee-cli` dependency that degrades to the leader reviewing the change
itself. The source's `disable-model-invocation: true` is dropped, as in
`bee-architect` and `bee-arena`: the description names the explicit trigger.

## Tool swaps

- Subagent spawning ("launch all reviewers in a single message using the Task
  tool", `subagent_type: generalPurpose`, `readonly: true`) became one
  `bee dispatch prepare --runtime claude --kind reviewer --json` per reviewer,
  all issued in the same turn with no `--role`, each returned payload run in the
  background and used exactly as returned. No model or subagent type is named.
  Read-only is the reviewer kind's own property, so the `readonly` flag is gone.
- The `interrogate reviewers` list in the Cursor rules file and the four-model
  default table became the review role of the bee team table. The four model
  slugs and the Reviewer A to D labels bound to them are dropped. Three
  reviewers is the default and two for a small change; labels are generic.
- "Model diversity" as the adversarial signal became "independence": the reviewer
  kind runs on one role, so diversity is a team setting. The verdict now says
  when all payloads name one model, and Lead Judgment says agreement on one model
  is weaker corroboration. This follows `bee-reviewing`, where corroboration
  counts only across different models or disjoint lenses.
- Severity `critical | warning | nit` became `P1 | P2 | P3` in the reviewer
  template, with the `bee-reviewing` definitions. A P1 blocks: the verdict puts
  every surviving P1 under Act On and says it blocks approval; Lead Judgment gains
  a short "A P1 Blocks" section. The reviewer template's Evidence item takes the
  `bee-reviewing` rule for a behavior finding (command, output, required output,
  or file:line).
- The Approval Bar in `code-quality-review.md` gains one sentence: structural
  blockers are filed as P2 unless they also break behavior. Without it, "presumptive
  blockers" and a P1 that blocks would collide.
- The brief rule (no verdict, leaning or expected findings in a reviewer brief)
  and the one re-dispatch for a return that breaks the finding schema come from
  `bee-reviewing` and `bee-hive/references/gates-and-delegation.md`; the pstack
  source has neither.
- "Reproduce or trace a behavior finding before it reaches the verdict" is the
  `bee-principle-verify-before-reporting` method, written as a sentence in Step 5
  and in `lead-judgment.md` ("Hypothetical vs. Actual"). The pstack text already
  said to trace the call site.
- The skill states it is advice before a gate and that the review gate and its
  record stay `bee-reviewing`'s. There is no `bee reviews record` call from this
  skill.
- A `Handoff` section ("Invoke bee-hive skill") is added, as in `bee-architect`
  and `bee-arena`.
- Smaller wording swaps: "PR description" became "pull request description",
  "a real PR" became "a real merge", "the next PR in the stack" became "the next
  planned cell or change", "a PR push a file" became "a change push a file".

## Pointers kept as plain sentences

None. The source names no principle or skill from the pstack set.

## Dropped steps

- The slug-rejection fallback ("if a model slug is rejected as unresolvable, pick
  the closest equivalent, spawn with the valid slug, and open a separate PR to
  update the configured value or default table") and the `inherit-parent` or
  `auto` alias note. Both exist only because pstack names model slugs in a Cursor
  rules file. bee's dispatch payload carries the model and an unconfigured role
  fails by name, so there is no counterpart.
- Graphite and the Cursor loop: the source cites neither, so nothing was dropped
  for them. bugbot, the poteto-agent, arena, architect, how, why and
  show-me-your-work are not cited in the source either.

## Trigger fixture

Six briefs added to `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json`:
four expect (one Vietnamese) and two near (expect `bee-reviewing` and `bee-how`),
none naming the slug.
