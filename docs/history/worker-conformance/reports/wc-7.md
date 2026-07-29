# wc-7 — Make the written law match the machine on every door

**Status:** [DONE] · lane `high-risk` · worker Norbert · full trace: `.bee/cells/wc-7.json`

## Outcome

Ten places where the doctrine overstated the machine are now stated as the code
computes them, each traced to a named line in `packages/bee/lib/cells.mjs` or
`state.mjs`. The blocking one first: `planning-reference.md:110` claimed the cap
helper refuses every `behavior_change` cell lacking both evidence and a
`red_failure_evidence` "before" — both halves are false (the evidence door is a
non-blocking warning at `cells.mjs:1969-1974`; the "before" door is tier-gated to
`red-first` at `:2118`/`:2150`), so a `standard`-lane behavior cell resolving to
`existing-targeted-green` (`:181-183`) is refused by neither.

The theme carried everywhere: four surviving refusals are gated on
`!pendingFeatureVerify` and the pending path is the **default**, so calling them
unconditional misled every worker taking the default. `worker-details.md` now
splits them honestly — the passing-verify requirement (`:1913`) moved out of the
"both paths" list it contradicted, and `new_suite_reason` (`:2013`), the ratio
refusal (`:2047-2053`) and red-first's D3 door (`:2150`) are each named as
deferred, never waived. Also corrected: the tiny-cell "silent marker" claim (the
`behavior_change` warning has no lane gate and prints on `tiny`); the trailing
test cell restated as the **coverage judgment** owed by the **feature**
(`testCellDebt(root, feature)`, `state.mjs:2570`), not authoring owed by the
slice; red-first's true scope in `bee-planning` (at `high-risk`,
`refactor`/`formatting` stay `suite-green` and `test` stays `targeted-green`);
and D13's two `testCellDebt` kinds, where only `missing` requires capped
code-touching cells while `not-green` fires from offenders alone.

Nothing was made more permissive: two near-misses the advisor caught were tightened
back before cap (see Consults).

## Verify

Not run by the worker — cell `verify_owner` is main at feature close; capped
`--feature-verify-pending`. Regen chain run in the cell's mandated order:
`render_plugin_skill_trees.mjs` → `onboard_bee.mjs --apply` →
`release_manifest.mjs --write`, then:

```
release_manifest --check: 448 file(s) match stored manifest
```

## Files + commit

`skills/bee-executing/references/worker-details.md`,
`skills/bee-planning/SKILL.md`,
`skills/bee-planning/references/planning-reference.md`, plus the regen chain's
outputs (`.claude-plugin/`, `.codex-plugin/`, `.claude/skills/`,
`.agents/skills/`, `.bee/onboarding.json`,
`docs/history/codex-harness-hardening/release-manifest.json`).

## Deviations

- The cell's action listed the pending/evidence exclusivity checks (`:1891`,
  `:1896`) as "ungated, fire on both paths". The code puts both inside
  `if (pendingFeatureVerify)` (`:1887-1900`), so they are the pending path's own
  rule. Wrote the narrower truth and had the advisor confirm it, per the cell's
  own instruction to verify the list against the code rather than take it on faith.

## Consults

1 consult — advisor `fable` (`docs/history/worker-conformance/reports/wc-7-consult.md`).
**Ask:** falsify the ten corrections against live code; flag anything made more
permissive. **Answer:** all seven asserted claims confirmed; four corrections
accepted — `spike` shares `tiny`'s unwarned gap, the `refactor`/`formatting`
new-test-file refusal is `diff_stats`-fail-open rather than truly ungated, "no
proof still caps" had to become "an asserted pass with nothing recorded" (`:1913`
still refuses otherwise), and "a pending-capped test cell clears the door" needed
"provided no failing verify was recorded" (`state.mjs:2605-2606`).
