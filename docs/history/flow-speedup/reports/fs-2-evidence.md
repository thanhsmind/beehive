# fs-2 — Delta validation: hash-anchored evidence cache

Spec #77 **P1 only**. P2/P3/P4/P6 untouched; P5 was rejected by the owner and nothing here
loosens evidence law, spike discipline, Gate 3 semantics, or advisor-consult enforcement.

## The surface

Two verbs in the `state` group, deliberately shaped as twins of `advisor-ref record`/`show`:

| Verb | Nature |
|---|---|
| `bee state validation-cache record --slice <n> --rows-file <f> [--feature <x>]` | Mutation. Writes `.bee/validation-cache.json` atomically under its own `validation-cache` lock. |
| `bee state validation-cache check [--feature <x>] [--outputs-file <f>] --json` | Pure read. Never writes, never executes a command, never throws, always exits 0. |

There is no `clear` verb, for the same reason `advisor-ref` has none: staleness makes a stale
entry inert on its own.

### File shape — `.bee/validation-cache.json`

```json
{
  "version": 1,
  "features": {
    "<feature>": {
      "slice": 1,
      "updated_at": "<audit only>",
      "anchors": { "feature": "...", "newest_decision_id": "...", "plan_sha256": "..." },
      "rows": [
        {
          "id": "row-a", "kind": "matrix", "claim": "...", "verdict": "PASS",
          "evidence": "proof-a.mjs:1", "proven_in_slice": 1, "recorded_at": "<audit only>",
          "sources": [
            { "type": "file",    "path": "proof-a.mjs", "sha256": "<64 hex>" },
            { "type": "command", "command": "node x.mjs", "output_sha": "<hex>" }
          ]
        }
      ]
    }
  }
}
```

`record` **hashes every file source itself** — a caller can never assert a hash it did not earn,
exactly as `advisor-ref record` stamps its own anchors. Recording replaces a feature's row set
rather than merging, so a row dropped from the input cannot linger as a fresh-looking survivor.
The same rows go into `docs/history/<feature>/reports/validation-<slice>.md` via the `Sources`
column added to the matrix format in `references/validation-reference.md`.

## Staleness inputs

The computation lives in `packages/bee/lib/state.mjs`, immediately beneath `advisorRefStale`, and
**reuses `advisorRefAnchors` verbatim** rather than defining a second staleness rule — AO13 already
settled which anchors a cached proof binds to, and two subtly different rules in one repo is how
these surfaces would drift apart. A row is stale when **any** of:

1. any cited source file's sha256 changed (or the file was deleted → absent sentinel);
2. the newest active decision id changed *(feature-level: stales every row at once)*;
3. `sha256(plan.md)` changed *(feature-level; the plan is frozen, so this catches re-shape events)*.

Command evidence carries forward **only** when `--outputs-file` supplies a freshly observed hash for
that command, because a read-only check cannot re-run it. No supplied hash → stale → re-proven.

**No TTL exists anywhere.** No age threshold, no wall-clock comparison, no "expires after". The only
timestamps written (`updated_at`, `recorded_at`) are audit fields that no predicate reads — proven by
the backdating test below.

## Degradation — the safety property

Every defect resolves to **more** validation, never less. Each path is demonstrated by its own test:

| Defect | Result |
|---|---|
| Cache file missing | `degraded: true`, `revalidate: "full"`, zero rows offered |
| Cache file unreadable (EISDIR probe — uid-independent, unlike a chmod trick) | degraded → full |
| Malformed JSON | degraded → full |
| Unknown `version` | degraded → full |
| Partially valid: feature entry has no `rows` array | degraded → full |
| Partially valid: feature entry records no `anchors` | degraded → full |
| No entry for this feature | degraded → full |
| Row storing no sources / no `sha256` | that row is **stale** (file itself is fine) |
| `--outputs-file` absent, unreadable, or not an object | command rows re-prove; never fatal |

A cache whose rows all went stale reports `revalidate: "full"`, because that is what it is.
Cached evidence remains Accepted Evidence held to the identical bar — plausibility language still
auto-fails, stated in both SKILL.md and the reference.

## Falsifiability — mutation-tested, not just asserted

A cache of evidence makes hollow proof cheap to reuse, so the tests were checked against deliberate
bugs rather than trusted because they were green:

- Disabling the file-sha comparison (`if (false && current !== source.sha256)`) → **2 tests failed**
  (`touching ONE cited source…`, `a deleted source stales its row`). Reverted.
- Disabling the degradation branch (`if (false && read.degraded)`) → **4 tests failed**
  (missing / unreadable / malformed / unknown-version). Reverted.

Both reverts re-confirmed green. The demonstration the cell required — a row flipping stale on a
one-line touch — is a real one-line `appendFileSync` against a recorded source, asserting that the
touched row goes stale, the *untouched* row still carries forward, and the reason names the file.

## Verification

`node scripts/ledger_parity.mjs --check && node scripts/release_manifest.mjs --check && node scripts/run_verify.mjs`

```
PASS run_verify: 108 suite(s), concurrency=5, wall=73537ms   EXIT=0
```

15 new tests in `packages/bee/tests/test_bee_cli.mjs` (312 passed, 0 failed in that suite).

Two pre-existing invariant guards legitimately caught this work and were satisfied rather than
weakened — no test was deleted or loosened:

- `scripts/tests/test_state_projection_race.mjs` demands every `state` verb be classified. Both new
  verbs went into `NON_PROJECTION_VERBS`: neither writes `.bee/state.json`, and `record` takes its
  own lock, never the `state` lock.
- `packages/bee/tests/test_misc.mjs` pins `lib/state.mjs`'s export surface. Rather than widen it by
  nine names, the module's public surface was kept to four (`validationCacheCheck`,
  `writeValidationCache`, and the two constants); hashing, row normalization, row-level staleness
  and the file reader stay module-private, mirroring how `advisorPlanPath` stays private for the
  advisor precedent.

## Files

- `packages/bee/lib/state.mjs` — staleness core (+ vendored `.bee/bin/lib/state.mjs`)
- `packages/bee/bee.mjs` — the two handlers (+ vendored `.bee/bin/bee.mjs`)
- `packages/bee/lib/command-registry.mjs` — both registry entries (+ vendored copy)
- `packages/bee/scripts/onboard_bee.mjs` + `.gitignore` — cache is machine-local, never tracked
- `skills/bee-validating/SKILL.md` — delta rule as replaced contract lines, not an appended section
- `skills/bee-validating/references/validation-reference.md` — `Sources` column in the matrix format
- `packages/bee/tests/test_bee_cli.mjs`, `packages/bee/tests/test_misc.mjs`,
  `packages/bee/scripts/test_onboard_bee.mjs`, `scripts/tests/test_state_projection_race.mjs`
