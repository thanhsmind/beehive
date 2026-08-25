# role-surface-cleanup — CONTEXT

**Route.** class `bugfix` · lane `standard` · flag `covered-contract-change` ·
6 product files. Asked: the user said "tiếp tục" after the escalate-off-disarm
merge; this lane takes the next batch of review-verified findings.

## What was asked

Four findings from the r2/r3 reviews, each verified still present at HEAD,
each a one-site fix:

1. **P2** — `cell_role_list("code")` returned `["code","code","generation"]`,
   so the fall-through warn fired twice and its first copy named the very
   name that just failed (`prepare.rs:98`).
2. **P2** — `dispatch prepare`'s registry description still claimed kind
   `cell` "resolves the generation tier" — the surface agents read through
   `bee --help --json` taught the retired model.
3. **P3** — the worker-registry refusal opened with `invalid tier` for a
   value that records a role. The `--tier` flag and the persisted key keep
   the historical spelling on recorded rationale — not reopened.
4. **P2** — neither CI job passed `--no-fail-fast`, so a red target hid every
   target behind it; the declared `commands.test` had the same hole (patched
   directly on main, `.bee/config.json` being CLI-editable state).

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | All four land in one lane: the list never repeats a name (backstop stays reachable — as tail, or as head when the role *is* the backstop); the registry speaks role language; the refusal says role; both CI jobs and `commands.test` carry `--no-fail-fast`. | Each verified by review, each one site, each with an existing test surface; four lanes of ceremony for four one-line-class fixes serves nobody. |

## What was done

Red-first on the one behavior change: `the_ordered_role_list_never_repeats_a_name`
shown failing (`cell_role_list("code") repeats "code"`). The registry edited
as data with serialization round-trip check. Three wording assertions
retargeted with the fix. CI diff is exactly two lines; `set -o pipefail` and
the `tee` capture untouched.
