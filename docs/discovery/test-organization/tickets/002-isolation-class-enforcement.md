---
type: research
status: closed
claimed-by: none
blocked-by: none
---

## Question

For each of the four stacks, what is the idiomatic runner-enforced
mechanism for the three draft isolation classes — `pure`, `tmpdir`,
`global` — and specifically: how do you force the `global` set serial
WITHOUT serializing the whole suite? Also: any prior art for a declared
isolation class per test file.

## Answer

Findings: ../research/002-findings.md

Every stack can express `tmpdir` cleanly and three of four can express
`global` without serializing everything:

- **Go** has the safest model — `t.Parallel()` is opt-in, so serial is
  already the default; `t.Setenv()` is *confirmed* incompatible with
  `t.Parallel()`.
- **pytest** uses `@pytest.mark.xdist_group` with `--dist loadgroup`,
  plus `filelock` when the shared resource crosses worker processes.
- **Rust** uses `serial_test`'s `#[serial(key)]`, or nextest
  `test-groups` with `max-threads = 1`.
- **Vitest** needs a separate workspace project with
  `poolOptions.threads.singleThread` — a community pattern, not a
  documented one.
- **Jest is the weak row**: no first-party way to make ONE file serial
  while others stay parallel. `--runInBand` serializes the whole run.
  The convention must say so plainly rather than inventing a lever.

Confirmed and directly relevant to bee's own suite: Rust 2024 made
`std::env::set_var` / `remove_var` `unsafe`, because mutating the
process environment in a multithreaded program is unsound. That is
exactly what `verbs/status_full/tests.rs` does in 11 places.

**Prior art** is JUnit 5's `@ResourceLock` / `@Isolated` — the only
existing runner-enforced declared-isolation convention found, and it
carries a known composition bug (junit5#2605) where `@Isolated` fails
to dominate a `@ResourceLock`-only test.

**The finding that shapes the design**: no stack declares a three-tier
class. Every mechanism found expresses only the `global` tier as
named-lock grouping; `pure` and `tmpdir` are implicit defaults
everywhere. So the three-tier taxonomy is genuinely new — and its risk
is that a declared class nothing enforces is documentation, not a
guard. Ticket 004 must decide whether the header is comment-only,
attribute-backed, or both.
