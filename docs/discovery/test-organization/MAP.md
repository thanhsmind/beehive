# Test organization — discovery map

## Destination

A locked convention for how test files are organized — layout, naming,
size, isolation class, parallel posture — that holds in any project
regardless of language, plus the one bee skill that teaches it to
agents in two modes: **write** (author new tests to the convention)
and **audit** (report an existing suite's violations and propose the
splits, never auto-fix).

## Notes

- Origin: bee's own suite. `crates/bee/src/verbs/cells/tests.rs` is
  6042 lines / 178 tests; `hooks/write_guard/tests.rs` carries 194
  tests; 17 integration binaries live under `crates/bee/tests/`, the
  largest 81KB.
- **The speed premise needed correcting, and the correction is the
  map's central fact** (ticket 001, closed): splitting files does not
  universally buy parallelism, because each runner schedules a
  different unit. Jest and Vitest schedule the FILE — split is a real
  gain. Go schedules the PACKAGE. pytest is single-process until
  `pytest-xdist`, then schedules tests or files depending on `--dist`.
  Rust is the outlier: `cargo test` runs multiple test binaries
  **serially**, with concurrency only inside one binary's thread pool,
  so splitting buys zero and splitting into more integration files is
  actively worse. `cargo-nextest` (process per test) is Rust's only
  real lever.
- Isolation enforcement (ticket 002, closed): Go's `t.Parallel()`
  opt-in is the safest model; pytest has `xdist_group` +
  `--dist loadgroup`; Rust has `serial_test` keys or nextest
  `test-groups`. **Jest is the weak row** — no first-party way to make
  one file serial while others stay parallel. The convention must state
  that gap, not invent a lever.
- No runner declares a three-tier `pure`/`tmpdir`/`global` class. Every
  existing mechanism expresses only the `global` tier as named-lock
  grouping; JUnit 5's `@ResourceLock`/`@Isolated` is the closest prior
  art, and it carries a known composition bug (junit5#2605). The
  taxonomy is novel — and a declared class nothing enforces is
  documentation, not a guard.
- Therefore the convention has to be two-layered: universal laws on
  top, a per-stack mapping table underneath — the same shape as
  luongnv89's test-coverage skill, which detects stack from the
  manifest file and swaps in that stack's commands.
- Reference skill read in full:
  `/home/thanhsmind/projects/AI/luongnv89-skill/skills/test-coverage/SKILL.md`.
  It owns coverage *quantity* (find untested branches, prove the
  percentage moved). On organization it says one line — "Place test
  files alongside source or in the project's existing test directory".
  That gap is this map's subject; the two skills compose rather than
  overlap.
- Live counter-example for LAW 1: `verbs/status_full/tests.rs` mutates
  process-global state (`env::set_var`, `set_current_dir`) in 11
  places while `cargo test` runs that binary thread-parallel. The repo
  has no `serial_test`, no `#[serial]`, and no `cargo-nextest`. Rust
  2024 made `std::env::set_var` `unsafe` for exactly this unsoundness.
- Existing machinery LAW 2 must align with rather than duplicate: the
  test-economy D3 test-to-source ratio ceiling (>4 on standard and
  high-risk lanes) already refuses over-large test cells at cap.
- Locked upstream doctrine this map may not contradict: D-58ec9664
  (the agent owns test scope end to end; proof-per-change-type) and
  D-1f534837 (close/merge check a recorded proof line and run
  nothing).

## Decisions so far

- D-dab5f286: bee-owned skill, not a standalone package; scope is
  organization + isolation ONLY (coverage stays with luongnv89's
  test-coverage skill, the two compose); two modes, write and audit,
  audit never auto-fixes; first stack table covers TS/JS, Python, Go,
  Rust.
- D-90ce6d67: research constraints from tickets 001+002 — the parallel
  unit is stack-specific (cargo test serializes binaries); Jest cannot
  serialize one file alone; the three-tier isolation taxonomy has no
  prior art beyond JUnit's `@ResourceLock`, so enforcement must be
  designed, not assumed.
- D-588eecb5: two standing laws. **LAW 1 parallel-first** — concurrent
  is the default; a test that cannot run concurrently declares why in
  its file header, so serialization is a named cost, never a silent
  one. **LAW 2 enough, not overload** — tests are sized to what the
  change needs; a suite outgrowing its source is a defect, and the
  convention owes agents a stop rule.

## Not yet specified

- What the per-file header contract literally looks like — a comment
  block, a runner-native attribute, or both. Now graduated to ticket
  004: 001 and 002 have reported.
- Whether the convention says anything about test *data* and fixture
  sharing across files, which is where "one file, one behavior area"
  usually breaks in practice. (agent-suspected)
- Whether audit mode needs a machine-readable output for CI, or a
  human report is the whole job. (agent-suspected)

## Out of scope

- Coverage measurement, coverage targets, and gap-filling — owned by
  the existing test-coverage skill, by decision D-dab5f286.
- Assertion style, mocking policy, and fixture libraries.
- Changing bee's own CI command or the proof-line doctrine — locked by
  D-58ec9664 and D-1f534837.
