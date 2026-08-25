# exclusive-create-atomic — locked context

## The defect

Two sites elect a single winner with `create_new(true)` (O_EXCL) and then write
the winner's record in a **separate** step:

```rust
OpenOptions::new().write(true).create_new(true).open(&file)
    .and_then(|mut f| f.write_all(body.as_bytes()))
```

Between those two operations the path **exists and is empty**. A loser's own
`create_new` fails instantly with `AlreadyExists`, and both losers then read
that same path to find out who won — landing inside the gap.

| site | loser's read | failure |
|---|---|---|
| `packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:735` | `read_claim` on the same path | **Lies.** Reports `"no session (sessionless claim)"` and `(no expiry)` as the winner's identity. |
| `packages/bee-rs/crates/bee/src/verbs/reservations/reserve.rs:317` | `read_to_string` on the lease file | **Goes anonymous.** Unparseable read takes `Err(_) => None`, so `conflict_out` returns an EMPTY conflicts vector — a correct "there is a conflict" naming nobody. |

The other ten `create_new(true)` sites in `packages/bee-rs/crates/bee/src` are
**correct as written** and are out of scope: their losers never read the
winner's content, existence is the entire signal. Spot-verified directly:
`verbs/state_group/store.rs:497` refuses with a static string and reads only on
the winner path; `lease_store.rs:370` returns `Ok(false)`.

## Evidence

`one_claimant_wins_the_cell_and_every_loser_is_a_typed_claimed_refusal`
(`packages/bee-rs/crates/bee/tests/concurrency.rs:509`) catches the `claims.rs`
site:

```
a losing claimant's refusal must name the actual winner's session "sess-3";
got: already claimed by session "no session (sessionless claim)" (no expiry).
bee: could not parse JSON at .../.bee/claims/race-a.json — invalid JSON
```

Measured rates, three trees, two of them never carrying the commit originally
blamed for it:

- **solo: ~4 in 5 fail** — `101,0,101,101,101` at `492f8fa9~1`, `101×5` at HEAD,
  `FAILED,FAILED,ok,FAILED,FAILED` on a peer's independent branch.
- **full suite: ~1 in 8 fail** — one failure across eight logged suite runs here,
  at least one in four on the peer's tree.

The race is **pre-existing**; an earlier attribution to `492f8fa9` was retracted
after re-testing with repetition. Solo is the efficient reproducer because the
claimants interleave tightly; load lowers the rate but does **not** prevent it.

## Locked decisions

1. **Both sites publish their record atomically, by `link(2)`.** Write the
   complete body to a temp file, then hard-link it to the target. `link` fails
   with `EEXIST` when a winner already exists, so exclusivity is preserved
   exactly as `O_EXCL` provided it — and the target name never exists without
   its full content, so a partial read stops being possible rather than
   becoming unlikely.
2. **The temp file lives in the SAME directory as its target.** A hard link
   cannot cross a filesystem, and a temp elsewhere would fail on any setup
   where the store sits on its own mount.
3. **The temp is removed on every path, including `EEXIST`.** A lost race must
   not leak. At this repo's default concurrency most claims and most
   reservations lose, so a leak here would accumulate fast — the reservation
   path more than the claim path.
4. **The temp name must not collide between racers.** Two losers racing the
   same target must not stage onto one another's temp.
5. **No retry, no sleep, no widened read.** The remedy removes the window; it
   does not make readers tolerant of it.

## The invariant that must survive

At `reserve.rs`, `expired` is computed **from a successful parse**: an
unparseable lease yields `parsed = None`, `expired = false`, and the takeover
branch is skipped. That is precisely what stops a partial read from deciding a
**live** lease is stale and removing it.

Keep that derivation. This fix removes the partial read, so the `None` branch
stops being reachable from a race — but the predicate must stay conservative
regardless. If a later cleanup makes an unreadable lease count as expired, a
mid-write lease becomes takeable and two agents can hold the same path. That is
the line between today's cosmetic defect and a data-loss one, and both reviewing
sessions independently landed on it.

## Acceptance

- `one_claimant_wins_the_cell_and_every_loser_is_a_typed_claimed_refusal` run
  **solo, 20 consecutive times: 0 failures.** This is the proof. A green full
  suite is NOT — at ~1 in 8 it would pass by luck routinely.
- One full-suite run recorded green, so the load path is visibly checked rather
  than assumed.
- No temp file remains in the claims or reservations store directories after
  the solo runs.
- The `expired`-from-successful-parse derivation at `reserve.rs` is unchanged.
- Every existing test passes unedited; the reservation conflict message still
  names its holder, as it does today when the race is not hit.

## Out of scope

- The other ten `create_new(true)` sites.
- Making readers retry or tolerate partial content.
- Anything about the claim record's schema, TTL, fencing or adoption.
