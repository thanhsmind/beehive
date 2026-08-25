# state-lock-lost-update — locked context

## The defect

`store_lock_survives_a_pre_seeded_stale_lock_without_wedging_or_double_entry`
(`packages/bee-rs/crates/bee/tests/concurrency.rs:863`) intermittently loses a
`state worker add` entry:

```
the store must contain EXACTLY the entries the racers reported writing —
a missing one is a lost update, an extra one is a phantom write
  left:  ["w1", "w2", "w3", "w4", "w5", "w6", "w7"]
 right:  ["w0", "w1", "w2", "w3", "w4", "w5", "w6", "w7"]
```

**This loses data.** Every other concurrency defect found today reported
something wrongly; this one destroys a write that was made and acknowledged.
Severity comes from that, not from the rate.

## What is measured

| tree | rate |
|---|---|
| here, unmodified base | 5 fail / 40 solo runs (~12.5%) |
| here, with the `link(2)` change | 2 / 28 |
| peer session, independent tree (no `492f8fa9`, no `link(2)`) | 3 / 32 (~9%) |

Same order on three trees, so the bug is real and pre-existing. **Nothing here
says the `link(2)` change moved the rate** — different mechanism, different
file — and no such claim is made.

Two facts from the peer's reproduction. **The first one was wrong — it is left
here, struck, because how it was wrong matters more than the fact did:**

1. ~~**The lost entry is `w0` — the FIRST racer**, not a random one.~~
   **DISPROVED by the captured run in this cell: the lost racer was `w5`.**
   This was `n=1` from a single peer reproduction, promoted into a rule and
   then written here under a heading that reads "What is measured" — the
   strongest available framing — and a refinement was built on top of it
   ("`w0` dies because it is the only entry already complete"). Both sessions
   held the same generalisation, so neither could catch it by review; only a
   second sample could, and did. The real rule is positional-independent: **the
   lost racer is whichever WON THE TAKEOVER** and was then clobbered by the
   plain acquirer that walked into its rename vacancy. In the peer's run that
   happened to be `w0`.
2. **`state.json` is well-formed.** The assertion compared cleanly parsed entry
   lists; seven entries, no truncation, no invalid JSON. This is **not** the
   partial-write signature fixed in `exclusive-create-atomic`. **This one
   held.**

The test's own comment names the hazard: *"the takeover must never let two
racers in at once either. Every racer starts against the SAME stale file, so
several attempt the takeover concurrently."*

## Two mechanisms already ELIMINATED — do not re-propose them

Both were proposed in review and both are dead on reading the source. Re-check
if you like, but do not spend the cell on them.

1. **"The read happens outside the lock."** It does not.
   `worker_mutate` (`verbs/state_group/workers.rs:40`) is:

   ```rust
   let guard = acquire_state_lock(root)?;
   let mut state = read_state_strict(root)?;   // ← under the lock
   ...
   write_state(root, &state)?;
   drop(guard);
   ```

   Lock first, then read. A stale snapshot cannot come from an unlocked read,
   so the fix is not "take the lock before reading" — that is already the
   shape. For `w0` to be clobbered, **two racers must both have believed they
   held the lock**: this is a mutual-exclusion failure, not a read-ordering
   one.

2. **"A freshly created empty lock file is judged stale."**
   `try_acquire` (`lock.rs:198`) *is* the same two-step this repo has been bitten
   by — `create_new` then a separate `write_all`, so the lock name is briefly
   visible and empty. But `judge_stale_takeover_eligibility` (`lock.rs:232`)
   checks `mtime` age **before** it reads the holder, and returns `None` while
   `age <= STALE_MS`. A just-created lock has age ≈ 0, so the empty window is
   not reachable through the staleness path.

   Note this does not prove the empty window is harmless — only that this
   particular route to it is closed. `read_holder` is called in three places;
   if instrumentation implicates one, that is a real finding.

## Where the remaining suspicion sits

The takeover protocol in `lock.rs`:

- `judge_stale_takeover_eligibility` → decides stale, returns the holder it saw
- re-verify `same_holder_identity(read_holder(lock_path), holder_before)`
- `rename_for_takeover` — `rename(lock_path, stale_path)`, atomic, exactly one
  racer can win it
- `settle_takeover` — keeps or restores, returns whether the takeover stands
- back in `acquire_store_lock` (`lock.rs:400`), a won takeover still requires a
  successful `try_acquire` before a `LockGuard` is returned

That chain looks defended on a careful read, which is exactly why it needs
evidence rather than another armchair pass. Something in it admits a second
holder about one time in ten.

## How to work this cell

**Diagnose before fixing. Do not ship a speculative fix.**

1. Reproduce solo with repetition — that is the efficient reproducer here, as
   it was for the claim race.
2. Instrument to catch the actual interleaving: which racer's write lands, what
   each racer believed about the lock, and whether two guards are ever live at
   once. Temporary instrumentation is fine and expected; it comes out before
   the cap.
3. Only then fix, and state the mechanism in the commit body.

If the evidence contradicts everything above, that is a legitimate and valuable
result — say so plainly rather than bending it to fit.

## Acceptance

- The mechanism is **named and evidenced**, not inferred.
- `store_lock_survives_a_pre_seeded_stale_lock_without_wedging_or_double_entry`
  passes **40 consecutive solo runs, 0 failures.** The pre-fix rate is ~1 in 10,
  so 20 would be too weak here — 40 is the bar.
- The sibling `store_lock_serializes_concurrent_state_mutators_with_no_lost_update`
  also passes 20 consecutive solo runs.
- One full-suite run recorded green — for the record, never as the proof.
- No temporary instrumentation remains in the shipped diff.
- A stale lock still never wedges the store: the takeover must keep working,
  and `completed >= 1` must still hold.

## Out of scope

- The `create_new`-then-`write_all` pattern at `lock.rs:198` unless
  instrumentation actually implicates it. It is a known shape, it is not
  currently reachable through the staleness path, and converting it
  speculatively would be a fix without a diagnosis.
- Anything in `verbs/state_group/` beyond what the evidence names.
