# role-edge-hardening — CONTEXT

**Route.** class `bugfix` · lane `standard` · flag `covered-contract-change` ·
5 product files. Asked: the user said "tiếp tục"; this lane closes every
remaining review-filed finding on the role surface.

## What was asked

Five items, each verified at HEAD by the r2/r3 reviews:

1. **P3** — a well-formed `models.<rt>.ceiling` key was accepted, ignored,
   and never mentioned; with it present, `dispatch prepare` stamped a
   `[bee-tier: ceiling]` marker beside a model param — the pair the guard
   denies. Decision 0015 forbids the key; nothing enforced it.
2. **P3** — a mis-cased `"Advisor"` key split the two doors the null fix had
   just unified: the marker door folds case, `role_is_declarable`'s advisor
   arm and `resolve_advisor` matched exactly.
3. **P3** — a role-keyed fallback chain naming a role no runtime configures
   was accepted, published, and could never fire — silently.
4. **P2** — the `host_opted_into_roles` null boundary had no test: the r2
   mutation probe narrowed `.is_some()` to non-null and the whole suite
   stayed green, though `.bee/config-sample.json` ships exactly the
   null-valued shape the boundary decides.
5. **P3** — the backfill's `already_roled`/`unmapped` counts came from the
   unlocked scan while the doc claimed all counts described what the writes
   saw.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Close at the config and proof layers, never by adding a membership check to cell roles: the validator names a `ceiling` key (teaching `bee cells escalate`); the advisor identity folds case at both doors; a dead chain key warns; the null boundary is pinned as an OPT-IN (a written key shuts the window); mutation counts fold from the locked pass, and `already_roled` counts a role gained during the scan window from the fresh reading. | D7 locks cell-role validation to presence and shape — the ceiling fix lands where the decision forbidding it lives. Recounting the whole store under the lock would hold it across a full re-scan, the exact refusal-for-a-second the P1-B design avoids; so the split is stated, and the one direction the count can drift is folded fresh. |

## What was done

Red-first on the validator (a well-formed `ceiling` key produced zero
problems); the null-boundary tooth proven by applying the r2 mutation and
watching the new test fail exactly at its named assertion, then reverting.
The dead-key logic extracted as `dead_chain_keys` so it is pinned by test
rather than by stderr. One named deviation from the cell text: the
counts-under-lock must-have was narrowed to "the one direction the count can
move folds fresh, the doc states the split" — full recounting would undo
P1-B's short-hold lock design.
