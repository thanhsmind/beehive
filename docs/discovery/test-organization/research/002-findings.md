# 002 — isolation-class enforcement per stack

Answers ticket `../tickets/002-isolation-class-enforcement.md`. Web
research, 2026-08-18, official docs preferred.

Draft classes under test: `pure` (no I/O, no shared state), `tmpdir`
(filesystem, but only inside a per-test temp directory), `global`
(mutates process- or machine-wide state; must be serialized against
other `global` tests).

## Vitest

`test.concurrent` / `describe.concurrent` parallelize *within* a file;
`describe.sequential` is deprecated, use `concurrent: false`. Scope is
file-local — other files are unaffected either way.

`isolate` defaults to `true` (own module registry and globals per
file). `isolate: false` shares state across files in a worker for
speed; it is the opposite of what a `global` file needs.

To make ONE file serial while the rest stay parallel: define a second
project in the workspace/`projects` config matching only that file,
with `poolOptions.threads.singleThread: true`. Community-confirmed
pattern rather than a dedicated doc page.
Sources: https://vitest.dev/api/describe ,
https://vitest.dev/config/isolate ,
https://github.com/vitest-dev/vitest/discussions/6438

## Jest

Weaker. `--runInBand` serializes the entire run, not one file. There is
**no first-party lever to force a single file serial while others stay
parallel**; the closest is routing a `testPathPattern` to a separately
configured `projects` entry, or a community serial runner. Treat as an
open weakness for the convention's Jest row.
Source: https://jestjs.io/docs/cli

## pytest + xdist

`@pytest.mark.xdist_group(name="...")` with `--dist loadgroup` pins all
tests of a group onto the same worker; a worker runs one test at a
time, so same-group tests are serialized against each other while other
groups keep distributing. Untagged tests fall back to `load`.

Cross-worker shared resources (one real database, a one-time expensive
setup) need real inter-process coordination — the official how-to uses
`filelock` over `tmp_path_factory.getbasetemp().parent` plus the
`worker_id` fixture.

`tmp_path` / `tmp_path_factory` give a unique per-test directory: the
`tmpdir` class, safe by construction.

`monkeypatch.setenv` — xdist workers are separate OS processes with
their own `os.environ`, and each runs one test at a time, so env
mutations do not leak across workers. Inferred from the process model;
not a directly quoted guarantee.
Sources:
https://github.com/pytest-dev/pytest-xdist/blob/master/docs/distribution.rst ,
https://pytest-xdist.readthedocs.io/en/stable/how-to.html

## Go

The safest default of all four: `t.Parallel()` is opt-in, so a test is
serial unless it asks not to be. `global` files simply never call it;
`pure` and `tmpdir` files do.

`t.TempDir()` gives an auto-cleaned unique directory — the `tmpdir`
class.

`t.Setenv()` **cannot** be combined with `t.Parallel()` — pkg.go.dev
states it "cannot be used in parallel tests or tests with parallel
ancestors". Secondary sources report the enforcement is a runtime
panic; the exact panic wording was not verified against `testing.go`
this pass.

`-p 1` serializes across packages, not within one — wrong lever for a
single serial test inside an otherwise-parallel package.
Source: https://pkg.go.dev/testing#T.Setenv

## Rust

`cargo test` threads tests within a binary; `--test-threads=1`
serializes the whole binary, too coarse for one file.

`serial_test`: `#[serial]` / `#[serial(key)]` runs annotated tests one
at a time, optionally scoped to named keys; `#[parallel]` /
`#[parallel(key)]` may run concurrently with each other but never with
a `#[serial]` of the same key. A `file_parallel` / file-lock variant
covers cases spanning separate test binaries, where an in-process mutex
cannot reach.

`tempfile::TempDir` is RAII and unique per instantiation — the `tmpdir`
class.

**Confirmed**: Rust 2024 made `std::env::set_var` and `remove_var`
`unsafe`. The edition guide states it "can be unsound to call
std::env::set_var or std::env::remove_var in a multithreaded program
due to safety limitations of the way the process environment is handled
on some platforms." This is exactly bee's `status_full/tests.rs`
situation.

`cargo-nextest` alternative: declare
`[test-groups] serial-integration = { max-threads = 1 }` in
`.config/nextest.toml` and bind it with a
`[[profile.default.overrides]]` filter. The docs call this equivalent
to `serial_test` or a global mutex.
Sources: https://docs.rs/serial_test/latest/serial_test/attr.serial.html ,
https://docs.rs/serial_test/latest/serial_test/attr.file_parallel.html ,
https://doc.rust-lang.org/edition-guide/rust-2024/newly-unsafe-functions.html ,
https://nexte.st/book/test-groups.html

## Class-to-mechanism table

| Class | Jest | Vitest | pytest + xdist | Go | Rust (cargo test) | Rust (nextest) |
|---|---|---|---|---|---|---|
| `pure` | default | default | default `load` | call `t.Parallel()` | default thread pool | default |
| `tmpdir` | default + own temp dir | default + own temp dir | `tmp_path` / `tmp_path_factory` | `t.TempDir()` | `tempfile::TempDir` | same |
| `global` | **no clean per-file lever**; `--runInBand` serializes everything | separate project with `poolOptions.threads.singleThread` | `@pytest.mark.xdist_group` + `--dist loadgroup`; `filelock` for cross-worker | omit `t.Parallel()`; never with `t.Setenv()` | `#[serial]` / `#[serial(key)]` | `test-group` with `max-threads = 1` |

## Prior art

Closest match: **JUnit 5 `@ResourceLock` / `@Isolated`**
(`org.junit.jupiter.api.parallel`). `@ResourceLock(value = "key", mode =
READ_WRITE|READ)` declares a named shared resource so the parallel
scheduler serializes only conflicting tests; `@Isolated` is shorthand
for a global exclusive lock. Class- and method-scoped rather than
file-scoped, but it is the existing "declared isolation class,
runner-enforced" convention.
Source: https://junit.org/junit5/docs/5.11.3/api/org.junit.jupiter.api/org/junit/jupiter/api/parallel/package-summary.html

Cautionary precedent: JUnit has a known composition bug where
`@Isolated` does not correctly dominate a concurrently-running
`@ResourceLock`-only test —
https://github.com/junit-team/junit5/issues/2605 . Any scheme mixing a
"global lock" tier with a "named lock" tier inherits this hazard.

**The gap worth naming**: no surveyed stack has a three-tier
`pure` / `tmpdir` / `global` taxonomy as a declared attribute. Every
existing mechanism — xdist_group, serial_test keys, nextest
test-groups, JUnit ResourceLock — expresses only the `global` tier as
named-lock grouping. `pure` and `tmpdir` are implicit defaults
everywhere, never declared. That is the actual novelty in this
convention, and also its main risk: a declared class nothing enforces
is documentation, not a guard.
