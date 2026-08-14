# Retire Collation Guard — Context

**Feature slug:** retire-collation-guard
**Date:** 2026-08-14
**Shaping session:** complete
**Scope:** Quick
**Domain types:** RUN

## Feature Boundary

Three guards that kept locale-compare parity with a Node oracle deleted from
this build are retired, so the two CLI surfaces they silently disable start
working. This feature ends at those three functions and their tests — the
router message that masks such failures is deliberately out of scope.

## Feature Origin

`bee decisions render` refuses every documented argument shape, so
`docs/decisions/index.md` cannot be regenerated and still presents decisions
that have since been superseded twice. Diagnosis: the render groups by scope,
one stored scope is `feature:opencode-support`, and a colon falls outside
`collation_safe`'s alphabet (`is_ascii_alphanumeric() || ' ' | '_' | '-' | '.'`).
The guard returns `Ok(None)` (`decisions/render.rs:196`), which becomes
`Err(Err2::Ex)` (`:290`), which becomes a bare `None` at the dispatcher — and
the router, unable to see why the handler declined, reports "unsupported
argument shape" and blames the user's argv.

The guards exist because the Rust port had to match V8's `localeCompare` on
group keys, with Node as the oracle. Node was deleted from this build
(`backlog.rs:19`: `NOT BUILT (was: "still delegated to Node" — there is no
Node)`), so every delegate exit now has nowhere to go.

This is a known family, already recorded twice: decision `bda043af`
(`state route --set`) and `0248c1fd` (`bee close` for lane-carrying repos),
with `20969403` fixing one member (`set_gate`) by making its refusal state
itself.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Retire `collation_safe` in both copies — `decisions/render.rs:100` and `backlog.rs:1139`. They are separate functions with identical bodies, not one shared helper. | The oracle they preserve parity with no longer exists in this build; Rust's own sort is deterministic. Keeping them means every future group key containing a colon, a slash, or any other ordinary punctuation silently disables the command. User decision, 2026-08-14. |
| D2 | Retire `id_sort_safe` (`backlog.rs:214`) in the same pass. | Same dead-oracle guard under a different name, and proven live-broken: its alphabet is `^p-[0-9a-f]+$`, the store holds legacy ids `P72`/`P41`, and `bee backlog pbi list` (no `--status`) refuses today with the same misleading message. Included on the agent's call after the evidence appeared, named here so it can be objected to. |
| D3 | The router's misleading message is NOT changed. A handler returning `None` on a shape the registry declares valid keeps reporting "unsupported argument shape". | The user was offered that fix as a distinct option and chose the narrower scope. The masked-failure family stays open and is filed, not fixed. |
| D4 | The remaining sibling of this family — `run_supersede`'s ASCII guard (`decisions/mod.rs:31-32`) — is left in place and filed. | Not proven broken, and outside the two commands this feature repairs. |

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| parity guard | A check that refuses work whose ordering the Rust port could not prove identical to the Node oracle's. With the oracle deleted, a refusal it triggers is a dead end, not a fallback. |
| masked failure | A working command reported as a bad-argument error, because the dispatcher cannot see why the handler declined. |

## Existing Code Context

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/decisions/render.rs:100` — `collation_safe`; called at `:196` and `:230`, both `return Ok(None)`.
- `packages/bee-rs/crates/bee/src/verbs/backlog.rs:1139` — the second `collation_safe`; called once at `:1196`, `return None`.
- `packages/bee-rs/crates/bee/src/verbs/backlog.rs:214` — `id_sort_safe`; called at `:1046` (`pbi list`) and `:1348-1350` (`feature_backlog_rank`), both `None`.

### Tests that change

- `backlog.rs:1740` `locale_cmp_agrees_with_the_calibrated_probes` — direct asserts on `collation_safe` at `:1770-1772`.
- `backlog.rs:1786` `render_content_groups_by_weight_then_collated_id` — asserts the delegate branch at `:1819-1825`; the happy-path assertions stay.
- `backlog.rs:2061` `id_sort_guard_accepts_only_lowercase_hex_p_ids` — direct asserts on `id_sort_safe`.
- `backlog.rs:2148` `feature_backlog_rank_reads_the_feature_column_then_the_pbi_fold` — asserts `is_none()` for a legacy id at `:2194-2199`.
- `decisions/tests.rs:762` `locale_cmp_agrees_with_the_calibrated_probes` — the decisions copy.
- `decisions/tests.rs:865-871` — asserts `decision_index_content(...).is_none()` for scope `"café"`.

A test asserting that an exotic key DISABLES a command is asserting the defect.
Those assertions invert: the command now works and the ordering is whatever the
Rust sort produces, which is deterministic and is the new contract.

## Canonical References

- Decisions `bda043af`, `0248c1fd`, `20969403` — the masked-failure family and its one prior fix.
- `docs/decisions/index.md:1-5` — its own provenance header names `bee decisions render` as the only regeneration path.

## Outstanding Questions

### Deferred To Planning

- [ ] Does any ordering assertion elsewhere depend on the exact locale-compare
  order the guards were protecting? If so it pins behavior that was never
  reachable for exotic keys anyway, and needs reading before the guard goes.

## Deferred Ideas

- The router message for a handler that declines a registry-valid shape (D3).
- `run_supersede`'s ASCII guard (D4).

## Handoff Note

CONTEXT.md is the source of truth. D2 widened the user's stated scope on the
agent's call, with the evidence and the reason recorded; D3 records what was
deliberately left undone.
