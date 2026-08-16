---
type: bee.pattern
title: A derivation the tooling already computes is worthless where doctrine forbids it
description: "The impact registry already maps every source file to the suites that consume it, but a cell's verify is doctrinally forbidden from using it — so sibling-suite selection fell back to human memory, and two cells editing the same file made opposite guesses."
tags: [verification, cells, impact-registry, coverage, derived-ground-truth, validation-diet]
timestamp: 2026-07-28
bee:
  id: pattern-20260728-a-derivation-the-tooling-computes-but-doctrine-forbids-where-it-is-needed
  lifecycle: active
  sources: ["validation-diet cell vd-2 (verify omitted test_misc.mjs while editing lib/state.mjs, trace .bee/cells/vd-2.json, commit a7fab75d, 2026-07-28)", "validation-diet cell vd-1 (same file, verify INCLUDED test_misc.mjs, trace .bee/cells/vd-1.json, commit 4a03006e)", "validation-diet cell vd-5 (the repair, commit db11b709)", "removed 2026-08-16: skills/bee-planning/references/planning-reference.md no longer states the cited rule at any line — the impact-registry-derived per-cell verify doctrine it named is superseded; planning-reference.md's Test scoping section now states commands.verify is retired and cells run commands.test, the project's one declared test command, instead", "skills/bee-swarming/references/worker-details.md (formerly the executing skill's copy, cited at its then line 166)", docs/history/learnings/20260728-validation-diet.md L1]
  polarity: pitfall
  critical: false
---

# A derivation the tooling already computes is worthless where doctrine forbids it

`vd-2` exported a new helper from `packages/bee/lib/state.mjs`. Its verify —
`test_guards.mjs && test_bee_write_guard_hook.mjs` — passed. Meanwhile
`test_misc.mjs`'s exact-set export census went red and stayed red until a
whole repair cell (`vd-5`) was authored to un-export the helper.

The registry knew. `node scripts/impact_registry.mjs --query packages/bee/lib/state.mjs`
names `packages/bee/tests/test_misc.mjs` as a **direct** edge, and
`scripts/run_verify.mjs:813-909` already implements `--impacted-from-git`
against that same registry. The derivation was computed, committed, and
sitting on disk the entire time.

It was unreachable from the one place it mattered.
`skills/bee-planning/references/planning-reference.md:162` rules the impacted
run *"still never authored as a per-cell `verify`"*, and the wave-level
fallback that used to cover the gap was retired the same day by
`main-verifies` (`worker-details.md:166`). So the only unscoped check in the
feature's entire lifecycle was its final step, and every intermediate
sibling-suite decision fell back to whatever the cell's author happened to
remember.

The proof that it is memory and not method: `vd-1` and `vd-2` edited the same
file, `packages/bee/lib/state.mjs`, in the same slice, hours apart. `vd-1`
listed `test_misc.mjs` in its verify. `vd-2` did not. Same file, same
registry edge, opposite guesses, and nothing in between them to notice.

**Rule.** When a project already derives a relationship mechanically, the
question is never "is the derivation correct" — it is "is it reachable from
every decision that depends on it." A doctrine that narrows a check for good
reasons (cost, scope discipline, worker autonomy) must name what now supplies
the coverage it removed, and if the answer is "the author remembers," the
coverage is gone. Wire the derivation into the door rather than the
instructions: the cap door can run `queryRegistry(registry, cell.files, {level:1})`
and require every returned suite to appear in `cell.verify`, reusing
`scripts/impact_registry.mjs:449-486` with no new logic. A deferred
regeneration obligation (`regen_obligation_ack: wave-barrier`) has the same
shape and needs the same treatment — the **next claim**, not the eventual
close, is where a stale-artifact check belongs, because that is the last
moment the obligation is still cheap.

See also [[pattern-20260727-an-impacted-run-computed-after-the-commit-selects-nothing]]:
that pattern is about the derivation running at the wrong *moment*; this one
is about the derivation being forbidden at the right moment and replaced by
recall.
