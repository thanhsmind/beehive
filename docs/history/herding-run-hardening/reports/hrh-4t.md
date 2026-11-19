# hrh-4t — Check the one-home worker cap (hrh-4, commit b6474208)

**Verdict: RED.** Two of the three truths fail. hrh-4's cap report was not read.

## Check 1 — no literal worker count in skills/bee-herding or skills/bee-swarming: FAIL

Command:

    rg -n -i "(cap is [0-9]|cap of (4|four|[0-9])|>= ?4\b|four[- ]slot|four (concurrent|working|workers|agents|runtime)|(4|four) slots|up to (4|four))" skills/bee-herding skills/bee-swarming

Output:

    skills/bee-herding/scripts/bootstrap-cockpit.sh:24:# a runtime tab (one pane to start, filled up to four by the dispatch loop
    skills/bee-herding/scripts/bootstrap-cockpit.sh:288:# main-root), filled up to D5's cap of four by the dispatch loop later.
    skills/bee-herding/references/dispatch-dry-run.md:95:`occupied_count`. `occupied_count = 0`. A slot is free (well under the cap of 4).
    skills/bee-herding/references/role-dispatch-spawn.md:219:   lets the four-slot cap be walked past silently. Report it loudly —
    skills/bee-herding/references/role-bootstrap.md:96:ready, or all four runtime slots are busy), and stop the dispatch loop with
    skills/bee-herding/references/route-prompt.md:107:### 3. Check occupied runtime slots against the four-slot cap
    skills/bee-herding/references/route-prompt.md:116:A reviewer counts against the four-slot cap like any other worker.
    skills/bee-herding/references/route-prompt.md:119:- `source: "live"`: Read `count` as the live occupied count. The cap is 4.
    skills/bee-herding/references/route-prompt.md:120:  If `count >= 4`, no slot is free. Announce the refusal once to
    skills/bee-herding/references/route-prompt.md:254:Count the reviewer pane against the four-slot cap like any other worker.
    skills/bee-herding/references/operational-invariants.md:633:4. **The four-slot cap and the stop file** — bound concurrency and give the
    skills/bee-herding/references/role-dispatch.md:104:  and hold it against the cap exactly as before: the cap is 4; at `>= 4` no

hrh-4 fixed only `skills/bee-herding/README.md` (lines 14, 90, 134, 143), the
sites D4 names. `skills/bee-swarming` is clean: SKILL.md:42 and
references/swarming-reference.md:32 cite `herding.limits.concurrency`. The
reference files and the bootstrap script under `skills/bee-herding` still carry
the literal cap of four. route-prompt.md:119-120 and role-dispatch.md:104 state
the number as an operating rule, so an agent that follows them caps at 4 no
matter what `herding.limits.concurrency` says.

## Check 2 — default-cap test goes red when default_concurrency grows: FAIL

Mutation in `packages/bee-rs/crates/bee/src/herding/wave.rs:1148`:

    -    (cpus as u64 / 2).max(1)
    +    (cpus as u64 / 2).max(1) + 100

Cell verify under the mutation:

    test herding::wave::tests::a_fresh_run_at_the_default_cap_is_refused_with_limits_concurrency ... ok
    test herding::wave::tests::default_concurrency_with_fallback_occupancy_does_not_refuse ... FAILED
    test herding::wave::tests::unset_concurrency_defaults_to_cpu_count_over_two_never_below_one ... FAILED
    test result: FAILED. 94 passed; 2 failed; 0 ignored; 0 measured; 5415 filtered out; finished in 0.02s

The new test `a_fresh_run_at_the_default_cap_is_refused_with_limits_concurrency`
stays green. It computes `cap = default_concurrency(16)` and feeds
`Occupancy::Live(cap)` back in, so the expected value follows the code under
test and the test cannot notice a wrong default. Two older tests catch the
mutation because they pin literal values (`Some(8)` at wave.rs:2939 and
`Live(8)` at wave.rs:2990). To fix it, pin the occupancy to a literal value
(`Live(8)` for 16 CPUs) or assert `limit == 8`, not `limit == cap`.

Restored (the mutation was reverted with sed, and `git status --short` was empty):

    test herding::wave::tests::a_fresh_run_at_the_default_cap_is_refused_with_limits_concurrency ... ok
    test result: ok. 96 passed; 0 failed; 0 ignored; 0 measured; 5415 filtered out; finished in 0.01s

## Check 3 — the report quotes each check: PASS (this file)

## Proof

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- herding::wave`
— green:unit on the restored tree (96 passed). Under the mutation it was red,
but the new test was not one of the failures.
