# et-4 — 3 test_misc census checks lost their prose anchors after thin-body migrations

**Status:** [DONE]
**Worker:** exec-et4

## Outcome

`skill-token-diet` diet-4 thinned `bee-swarming/SKILL.md`'s body, moving the
detailed contract prose those 3 census checks grepped into
`skills/bee-swarming/references/swarming-reference.md` (and, for one check,
consolidating a formerly-duplicated `SKILL.md` copy away entirely). Fixed by
re-anchoring the checks to the new homes with meaning fully preserved — no
check weakened or deleted, no skill body grown, no source prose edited.

## Per-check disposition

1. **AO14 "Single execution worker"** — `SKILL.md`'s old
   `## Single execution worker (tiny/small lanes)` heading moved to
   `swarming-reference.md` as `## Single execution worker in full`. Census
   now checks the reference for that heading and `SKILL.md` for its pointer
   to it; "no workers are spawned" negative control now checked on both.
2. **Native Codex ordered-wait contract** — `SKILL.md`'s copy of the full
   `wait_agent`/`list_agents` paragraph was dropped entirely (not just
   moved) by diet-4; the sole remaining bee-swarming copy is
   `swarming-reference.md`'s `### Native Codex timeout interval`. Removed
   `SKILL.md` from `writableContractSurfaces` and — since diet-4's regen
   chain synced `.agents/**` too, so that mirror also dropped its copy —
   removed the matching stale entry from `readOnlyCodexProjectionSurfaces`.
3. **Worktree dispatch contract** — the entire eligibility/attestation/typed-halt
   prose moved into `swarming-reference.md`'s "Native Worktree Integration
   Transaction" and "Threat model and protected attestation" sections; the
   check now extracts those sections and asserts against them (order-based
   check for the `worktree-isolation-1→2→3` sequence rather than a
   backtick/arrow-formatting-sensitive regex; widened proximity windows
   scoped to the smaller extracted sections rather than the whole file;
   `record` added as an accepted synonym for `attestation` in one clause,
   matching the reference's own section title). `SKILL.md` is now checked
   only for its pointer to the reference.

All three re-anchors were mutation-tested (temporarily broke the sequence
order, the "at least two workers" phrasing, the heading, and the
timeout/failure distinction) and confirmed to still fail red before being
reverted.

## Files touched

- `packages/bee/tests/test_misc.mjs`

## Verification

`node packages/bee/tests/test_misc.mjs` — 117 passed, 0 failed. Full
trace/evidence: `.bee/cells/et-4.json`.

## Deviations

None — no skills/references content needed restoring; every check re-anchored
cleanly to prose that already existed at its new home.

## Friction

None.
