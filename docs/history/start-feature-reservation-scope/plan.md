# start-feature-reservation-scope — plan

## Problem

`start_default` (packages/bee-rs/crates/bee/src/verbs/state_group/policy.rs) refuses
`bee state start-feature` when ANY active reservation exists anywhere in the repo —
unscoped by feature, worktree, session, or path overlap. Its FIX line tells the caller
to run `bee reservations release`, which would release a live peer's holds — the exact
move `docs/knowledge/patterns/20260710-never-release-another-agents-reservations-on-a-stall.md`
forbids.

The correctly-scoped shape already exists on the lane path a few lines above
(`start_lane` branch (d)): it refuses only on DECLARED-PATH OVERLAP.

## Shape (decision start-feature-reservation-scope D1)

1. `start_default` grows a `paths: &[String]` parameter, mirroring `start_lane`.
   `run_start_feature` already parses `--paths`; it currently hands them only to
   `start_lane`. Pass them to `start_default` too.
2. Replace the blanket reservation refusal with two scoped ones:
   - **Same-session hold** — an active reservation whose `session` equals the acting
     `session_id`. This is the caller's own leftover state and is the original hygiene
     intent of the check. FIX names `bee reservations release --agent <agent>` with the
     caller's OWN agent.
   - **Peer path overlap** — an active reservation held by a DIFFERENT session whose
     path overlaps a path this start declares via `--paths`. FIX mirrors the lane
     wording: wait for release/expiry, or start over non-overlapping paths. It never
     says "release them".
3. Everything else in `start_default` is untouched: the claimed-cells precondition
   below stays the guard for real in-flight work; the `--as-lane --paths` road stays
   byte-identical.

Net effect: with no `--paths` and no same-session hold, a peer's reservation in another
worktree over unrelated paths refuses nothing.

## Slice

- `sfrs-1` — the whole change plus its tests (one file + the state_group test module).

## Proof

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
