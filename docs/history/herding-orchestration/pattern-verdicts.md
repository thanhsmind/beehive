# herding-orchestration — pattern-check verdicts (D6 gate)

Scope: `git diff main..HEAD` for this feature — the `fleet` crate
(`packages/bee-rs/crates/fleet/`), the `bee` crate's herding module
(`packages/bee-rs/crates/bee/src/herding/`), and `skills/bee-herding/`.
33 commits (`ae2b11cd`..`4055cd22`), merge-base `d1ffa3d5`.

Note on sourcing: the four patterns named in the task exist on `main`
(post-branch-point commits this worktree's branch predates) but not in
this worktree's checked-out `docs/knowledge/patterns/`, since this
branch diverged from `main` before they landed. Read via
`git show main:<path>`; the first pattern's exact file is already
present in this worktree
(`20260805-source-that-ships-without-reinstalling-the-binary-the-hooks-call-is-inert.md`)
under a slightly different filename than the task's paraphrase.

---

## 1. Source that ships without reinstalling the binary the hooks call is inert

**Verdict: respected.**

This is exactly the shape this feature could have hit: commits
`5f2afeaa`..`a13977f7` move the cockpit's control loop from a bash
script (`skills/bee-herding/scripts/control-loop.sh`, deleted) into
Rust source merged into the `bee` crate
(`packages/bee-rs/crates/bee/src/herding/control_loop.rs`). Every
caller reaches it only through the installed, gitignored
`$MAIN_ROOT/.bee/bin/bee` binary
(`skills/bee-herding/scripts/bootstrap-cockpit.sh:154,231`,
`skills/bee-herding/references/role-dispatch.md:75,202,249,405,444,448,452,457`)
— never `cargo run`. If that binary were not rebuilt after the merge,
the cockpit would keep running old logic while the source tree,
tests, and PR all read green — the precise failure the pattern names.

The feature caught itself doing this and fixed it, on the record. Commit
`a13977f7` ("Point the cockpit at the Rust control loop, retire
control-loop.sh"), body: *"Confirmed the handoff actually works:
rebuilt the worktree's stale `.bee/bin/bee` (it predated ho-13's verb
— `bee herding control-loop --help` refused before the rebuild,
succeeded after); ran `bootstrap-cockpit.sh --dry-run` ... and ran
that exact printed command line through `bash -c`"*. That is the
pattern's prescribed remedy verbatim: reinstall, then verify with the
same command against the installed path, not `cargo run`.

Independently, a structural net already exists on `main` from before
this feature (decision `dbf-1`, 2026-08-11):
`packages/bee-rs/crates/bee/src/doctor.rs:264` carries a
`binary_freshness_row` check whose doc comment cites this exact
pattern by name. This feature's own diff does not touch that check;
it neither disables nor evades it.

Residual, forward-looking note (not a violation of this diff): the
rebuild-and-verify above was done against this feature's own worktree
checkout during cell `ho-14`. The same reinstall step still needs to
happen against `main`'s `.bee/bin/bee` once this feature actually
merges — a merge-time obligation, not something the diff itself can
discharge.

## 2. A green count is not evidence that your new test ran

**Verdict: respected.**

No merge commits exist on this branch (`git log --merges main..HEAD`
is empty), so the pattern's literal insertion-point scenario (a test
block re-anchored one brace too high during conflict resolution)
cannot have happened here. But the diff repeatedly practices the
pattern's actual remedy — proving new tests ran by name or by an
exact count delta, not a suite total — rather than being merely
not-applicable.

Concrete instance: commit `1130f5bc` ("Make control-loop test doubles
observe what they receive") applies a hardcoded-argv mutation and
reports the *named* tests it kills, then: *"suite is green again at
1937 passed (23 original + 5 new tests, all green; no assertion
deleted or loosened)"* — an explicit count-delta check (1932 → 1937,
delta 5, matching the 5 tests added), not a bare "suite passed."

Same discipline recurs at every `fleet`/herdr-backend rework: commit
`2aaffc0e` re-runs seven judge-found mutations against
`cargo test -p fleet --lib --test choreography --test manifest_boundary`
and names the specific test that dies for each; `cc1799c0` and
`da530467` do the same, each naming the exact test function that
failed under a manual mutation before reverting it.

## 3. A "never do X twice" constraint is invisible to a suite that only asserts outcomes

**Verdict: respected.**

This feature has multiple exactly-once constraints (a wave-ledger row
per wave, a canonically-deduped worker sent-to exactly once), and each
is pinned by call-count or independent-artifact-count instrumentation,
not outcome-only assertions.

`packages/bee-rs/crates/bee/src/herding/wave_ledger.rs:93` states the
constraint in a doc comment: *"One wave: the row this module appends
exactly once per wave."* The test that pins it,
`a_wave_run_appends_exactly_one_ledger_row_through_a_fake_backend`
(`packages/bee-rs/crates/bee/src/herding/wave.rs:771-816`), does not
just check the wave's outcome — it reads the ledger file back off disk
and asserts `raw.lines().count() == 1` (`wave.rs:802`, comment:
"exactly one row must be appended per wave run"), independent of any
counter the production code itself maintains, plus
`backend.send_call_count("alpha") == 1` (`wave.rs:815-816`) sourced
from `FakeBackend`'s own invocation counters
(`packages/bee-rs/crates/fleet/src/backend/fake.rs:220-246`,
`status_call_count`/`send_call_count`/`read_output_call_count`,
doc-linked to "herding-orchestration D15").

The same shape covers the dedupe boundary: `invariant_8_a_duplicate_target_name_is_sent_to_exactly_once`
(`packages/bee-rs/crates/fleet/tests/choreography.rs:366`) asserts
`status_call_count`/`send_call_count` deltas rather than final state.
This is the pattern's own prescribed fix — a counter instrumented
into a `#[cfg(test)]`-only seam — applied before the defect, not after
one was found the hard way.

## 4. A test that derives its fixture from the constant under test only proves the code agrees with itself

**Verdict: violated.**

`packages/bee-rs/crates/bee/src/herding/control_loop.rs:1206-1216`,
test `defaults_match_the_bash_script`:

```rust
fn defaults_match_the_bash_script() {
    let o = Options::parse(&["--role", "merge"]).unwrap();
    assert_eq!(o.role, Role::Merge);
    assert_eq!(o.interval, DEFAULT_INTERVAL);
    assert_eq!(o.timeout, DEFAULT_TIMEOUT);
    assert_eq!(o.max_iterations, DEFAULT_MAX_ITERATIONS);
    assert_eq!(o.max_consecutive_failures, DEFAULT_MAX_CONSECUTIVE_FAILURES);
    assert_eq!(o.turn_ceiling, DEFAULT_TURN_CEILING);
    assert!(!o.once);
}
```

Every expected value on the right of these `assert_eq!` calls is the
same named constant (`control_loop.rs:74-81`) that the production
defaulting path (`control_loop.rs:106-110`, `965-967`) assigns to
`o.*` when no flag is passed. Both sides move together under any edit
to a `DEFAULT_*` constant, so this test cannot fail from that class of
change — it only proves `Options::parse`'s defaulting code reads the
same symbol the test reads, never that the symbol's *value* still
matches the bash script the test's own name claims to check against.

This is exactly the pattern's shape: the failure scenario is concrete,
not hypothetical. `git show 5f2afeaa~1:skills/bee-herding/scripts/control-loop.sh`
shows the original bash values were `INTERVAL=60`, `TIMEOUT=900`,
`MAX_CONSECUTIVE_FAILURES=20`, `TURN_CEILING=50`,
`DEFAULT_MAX_ITERATIONS=10000` — and today's Rust constants do match
them. But nothing forces that continued match: if a future edit
changed `DEFAULT_TIMEOUT` from `900` to, say, `90` — a real behavior
change, silently dropping the timeout the bash script used to give
each iteration — `defaults_match_the_bash_script` would stay green,
because `o.timeout` and the assertion's right-hand side would both
read `90`. No other test in this diff pins the bash script's original
literal values (checked: no other reference to `60`, `900`, `20`,
`50`, or `10_000`/`10000` as bare literals anywhere in
`control_loop.rs` outside comments and this same constant block) — so
the parity claim the test's name makes has no test that can actually
falsify it. Per the pattern's own remedy, the fix is to assert against
the bash script's literal values (`60`, `900`, `10_000`, `20`, `50`),
not against the constants under test — this is a proof-quality gap,
not a behavior bug, and no production code changes are needed to close
it.

---

## Summary

| Pattern | Verdict |
|---|---|
| Source shipped without reinstalling the binary is inert | respected |
| A green count is not evidence that your new test ran | respected |
| A "never do X twice" constraint is invisible to an outcome-only suite | respected |
| A test that derives its fixture from the constant under test proves only self-agreement | **violated** — `control_loop.rs:1206-1216`, `defaults_match_the_bash_script` |
