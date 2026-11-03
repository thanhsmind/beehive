# xia: pstack — what bee should take

> **Read `pstack-xia.md` first.** That study is deeper and better sourced: it
> read pstack's own tree at pinned commit `b9ddc83`, where this one read only
> Flavio Copes' article about it. This document was written without checking
> `docs/history/research/` for prior work and duplicates part of it. It is kept
> because the four items it ranks became the `pstack-adoption` feature, and
> because its `class`-enum evidence (nothing branches on `route.class`) is not
> in the other study. Where the two disagree, `pstack-xia.md` wins — it already
> did once, on verbatim playbooks (decision `132551fb`, which SUPERSEDED
> `cc87b3c4`'s "copies verbatim" wording; both are settled and the feature
> shipped under the superseding one, so this citation needs no revisit).

Source: `docs/history/research/pstack-source.md` (Flavio Copes' write-up of Cursor's
pstack plugin by Lauren Tan / @poteto). Local path, no ref/SHA — the
source is an article about the plugin, not the plugin tree. Scope:
philosophy, skills, flows, playbooks. Mode: `xia` (distill, ends in
discussion). Evidence labels: `Local` = proven in this repo,
`Upstream` = stated by the article, `Inference` = concluded.

## Bottom line

bee already has ~70% of pstack, usually in a stronger form (gates,
worktrees, decision log, model-role dispatch, verification skills that
ship to hosts). The real gap is **one idea**: pstack binds the *kind*
of task to a *named, verbatim-copied playbook*, and bee does not.

bee already has the taxonomy. `bee route --set --class` is a validated
enum — `feature`, `bugfix`, `docs`, `refactor`, `research`, `release`,
`spike` (`Local` `skills/bee-hive/references/scout-and-ticks.md:34`,
`packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs`). Free
prose is refused. And **no code branches on `route.class`.** The
code-touching check that looked like a consumer reads the *lane*, not the
class — `workflows.rs:827` binds `lane_class` from
`route_object.get("lane")` and hands it to `is_code_touching_lane`, whose
body tests only `docs` and `tiny` (`Local`
`workflows.rs:587-595, 827, 915`). The variable name is misleading; the
value is a lane. No skill reads the class either (`Local`: zero hits for
a class branch across `skills/`).

The class *vocabulary* is not entirely unread, though — a lane record's
`mode` field usually carries a workflow class, and two call sites exist to
stop that being misread as a lane (`Local`
`packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:395-406`,
`uat.rs:137-147`). One of them works around the private constant by
duplicating the lane list. So the taxonomy already leaks, unguarded, into a
field that was never meant to hold it — which strengthens the case for
giving `class` a real reader rather than leaving it decorative.

That makes the port unusually cheap: the enum, the refusal, the
recording law, and the preamble display already exist. Only the
playbook bodies are missing.

Four things are worth porting. Everything else bee has, or should not want.

## Dependency matrix

| pstack component | bee today | Verdict | Evidence |
|---|---|---|---|
| `/poteto-mode` router | `bee-hive` + `bee orient` | EXISTS | `Local` `skills/bee-hive/SKILL.md:26` |
| 22 task playbooks, copied verbatim, skipped steps keep their reason | lanes scale ceremony by size; `class` is a live enum no procedure branches on | **NEW** | `Local` `scout-and-ticks.md:34` |
| Bug fix: reproduce before fixing | stated once as craft — "for a bugfix watch the repro fail before the fix … not by flags" | NEW (partial) | `Local` `skills/bee-swarming/references/worker-details.md:33` |
| Refactor: characterization check green first | `refactor` is a class value; nothing behind it | **NEW** | `Local` |
| Perf: baseline trace vs after | not even a class value; `bee perf` times bee itself, not the product | **NEW** | `Local` `scout-and-ticks.md:34` |
| Investigation playbook (read-only, no diff) | AGENTS.md "A question is a question" — a rule with no flow | **NEW** | `Local` `AGENTS.md` |
| `/how` — trace the runtime, split 2-4 explorers, one account | `bee knowledge search` is lookup, not a trace | **NEW** | `Local` |
| `/why` — 7 evidence categories, empty searches reported | `bee decisions search` + git only | NEW (partial) | `Local` |
| `/arena` — competing candidates, cross-model judge, fold in the best | blind lanes and convergence | EXISTS | `Local` `skills/bee-hive/references/gates-and-delegation.md:152` |
| `/swarm` — split for coverage, one report | `bee-swarming` + `bee dispatch wave` | EXISTS | `Local` |
| `/interrogate` — multi-model review, four buckets incl. **dismissed shown** | `bee-reviewing` has severity findings; no dismissed bucket surfaced | NEW (small) | `Local` |
| Model per role (`/setup-pstack`) | `bee dispatch prepare` role table, open set | EXISTS (stronger) | `Local` preamble |
| `/create-verification-skill`, `/maintain-verification-skill` | `bee-verifying`, `bee-verify-upkeep` | EXISTS | `Local` `skills/` |
| Autonomous run needs a checkable finish condition | `bee-herding` control loop runs; no required done-condition field | NEW (small) | `Local` |
| Decision log TSV per iteration | `bee decisions log` (1741 active) | EXISTS (stronger) | `Local` `bee orient` |
| Session pickup / Pause safely | `bee state handoff write/adopt`, 65% rule | EXISTS | `Local` `AGENTS.md` |
| `/reflect` — transcript to skill proposals, human approves | `bee mailbox reflect` + `bee-evolving` | EXISTS (stronger) | `Local` |
| Isolated worktree per parallel writer | worktree-first is a boundary rule with hook teeth | EXISTS (stronger) | `Local` `AGENTS.md` |
| 21 named principles, index read first, invocable by name mid-run | `docs/knowledge/patterns/` (59 critical) + AGENTS.md prose | CONFLICT | `Local` |
| `/automate-me`, `/teach`, `/bro`, Graphite stacking, Benny | — | SKIP | `Inference` |

## Cross-cutting sweep

Where a playbook port would have to wire in, outside any one skill:

- `bee route --set --class` — the enum exists, is validated and is
  written; adding the FIRST procedural reader is additive, no schema
  change. `perf` would be the one new enum value, and
  `ROUTE_CLASS_VALUES: [&str; 7]` carries its own arity in the type
  (`Local` `workflows.rs:287-288`).
- `bee-hive` § Route — the router table would gain a class dimension
  beside the existing flow dimension (`Local` `skills/bee-hive/SKILL.md:38`).
- `bee-planning` lane classification — lane (size) and class (kind)
  become two axes, not one (`Local`).
- Cell templates — a bugfix playbook's "reproduce first" is a cell, so
  cell generation is touched. Cells already carry their own
  `change_class` enum (`formatting`, `bugfix`, `behavior`, `api`,
  `security`, `migration`, `refactor`, `test`) — a second taxonomy the
  port must line up with, not duplicate (`Local`
  `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs`).
- `bee-hive/references/scout-and-ticks.md:42` prints `class=` in the
  preamble already — the display surface is free (`Local`).

Nothing else. The playbook idea lands on an open extension point, which
is why it is cheap.

## The one CONFLICT worth naming

pstack's principles index is a **short list read at the start of every
multi-step run**, and the user can steer mid-run by name ("apply prove
it works"), with the rule that the reply must name the decision the
principle changed.

bee's equivalent is 59 critical patterns plus AGENTS.md prose. That is
more knowledge and less leverage: the patterns are ranked and digested
per feature, but no user can say "apply never-build-on-red" and get a
named decision back. This is not a port — bee should not add a 22nd
document. It is a *shape* worth stealing: give the existing rule ids
(`agents-never-build-on-red`, `agents-capture-line-at-close`, …) a
spoken form, and require the answer to name what changed.

## Recommendation — ranked

Ladder position: rung 3, **Adapt** — bee has the extension points
(rung 1 reuse fails because `class` has no reader; rung 2 has no
built-in). Build-from-scratch is not needed for any item.

1. **Class playbooks.** Give the existing `class` enum a procedure.
   Start with the three that carry real proof discipline: `bugfix`
   (reproduce before fixing, verify the original reproduction after —
   promoting today's craft line into a step), `refactor`
   (characterization check recorded green first), and a new `perf`
   value (baseline vs after, never "feels faster"). The verbatim-copy
   rule maps exactly onto bee's existing "named deviation" rule — a
   skipped step stays visible with its reason. Highest value, lowest
   wiring cost, because the taxonomy and its refusals already exist.
2. **An investigation route.** "A question is a question" is a rule in
   AGENTS.md with no flow behind it. A read-only class that produces a
   traced explanation and refuses to write source gives the rule teeth.
   This is also pstack's `/how` in the shape bee can afford.
3. **Dismissed findings shown.** `bee-reviewing` should print what the
   lead rejected and why, not a filtered list. Cheap, and it is the
   same honesty rule as pstack's empty-search reporting.
4. **Finish condition required for an autonomous run.** `bee-herding`'s
   control loop should refuse a run whose done-condition is not
   checkable. "Work for four hours" measures motion.

Skip: `/automate-me`, `/teach`, `/bro`, Graphite stacking, the Benny
automation pack. bee has the first three in other forms and the last two
assume a toolchain this repo does not have.

## What would change the answer

If `class` turns out to have a consumer somewhere unread, item 1 shrinks
to filling in playbook bodies. The sweep above found none, but it read
`packages/bee-rs/src` and `skills/` only — a hook or a rendered template
could still hold one.
