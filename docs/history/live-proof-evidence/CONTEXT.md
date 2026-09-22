# live-proof-evidence

**Feature slug:** live-proof-evidence
**Date:** 2026-09-22
**Scope:** Standard
**Route:** class `feature` · lane `standard` · flags `public-contracts`,
`covered-contract-change` · product files 5

## Problem

A cap's proof line `<command> — <result> — <scope reason>` closes its
result segment over `green:live`, `green:unit`, `green:static`
(proof-strength-and-expiry D1, decision `cb7b14b7`). `green:live` means
the real product or command was driven and its result inspected. Nothing
asks WHERE that inspection can be reopened: the reason segment is free
text, so `manual inspection — green:live — checked` caps today.

Measured on 2026-09-22 in `.bee/cells/` (live plus archive):

- 39 caps carry `green:live`; 3 of them name a `control-bee` evidence
  dir.
- `control-bee` writes its evidence under `${TMPDIR:-/tmp}/bee-verify`,
  so the evidence of every earlier run is already gone.

The doctrine already says "evidence attached" (AGENTS.md
`agents-proof-at-cap`; `lane-and-working-discipline.md` R16c-a), but no
check reads it. A live proof nobody can reopen is prose.

## Decision

- **D1 — A `green:live` cap names an evidence locator in its scope
  reason** (decision `d56826ef`). The reason segment must carry at least
  one whitespace-delimited token that is a run URL (`http://` or
  `https://`), an absolute path (`/…`), or a path containing
  `evidence/`. The check lives in `parse_report_flag`
  (`verbs/cells/finish_support.rs`) beside the closed result vocabulary,
  on the WRITE path only; `feature_proof_check` and every read path stay
  tolerant of older caps (proof-strength-and-expiry D2 holds). The rule
  keys on the `green:live` value, for every cell, at every lane: bee has
  no user-facing flag on a cell, and adding one would be a second fact
  for the same meaning. The refusal names the three locator shapes and
  the `control-bee` evidence dir as the remedy.
- **D2 — `control-bee`'s evidence root leaves `/tmp`** (decision
  `bb51c581`). `VERIFY_HOME` defaults to
  `${XDG_STATE_HOME:-$HOME/.local/state}/bee-verify`; `TMPDIR` is no
  longer consulted. The script header, the SKILL.md paths table, and the
  rendered runtime copies (`bee dev regen`) follow.
- **D3 — The map and the doctrine name the rule** where they already
  describe the proof line: `features/cells-and-proof.md` gains the
  sub-feature and its drive step; R16c-a in
  `lane-and-working-discipline.md` gains the locator sentence;
  `bee-swarming/SKILL.md` step 5 says "evidence locator in the scope
  reason" instead of "evidence attached". `AGENTS.md` is not edited: its
  line already says "evidence attached", and R16c-a is the rule's home.
- **No hat wave** (decision `d33c2f89`): a clear ask, a shape the user
  picked, two disjoint cells.

## Scope

In: the write-path check and its tests; the two test fixtures that cap
`green:live` with a bare reason; the script default; the SKILL paths
table; the feature map entry; the two doctrine sentences.

Out: any read-path change; any new cell field; any change to
`green:unit` or `green:static`; migrating historical caps.

## Source

- `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:87-95`
  — `PROOF_RESULT_VALUES`, the closed vocabulary.
- `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:237-285`
  — `parse_report_flag`'s `tests` arms.
- `.bee/verify/verify-app/control-bee:43` — the `VERIFY_HOME` default.
- `docs/history/proof-strength-and-expiry/CONTEXT.md` — D1/D2 this
  feature builds on.
