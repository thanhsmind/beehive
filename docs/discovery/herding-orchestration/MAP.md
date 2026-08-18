# herding-orchestration — discovery map

## Destination

bee-herding runs ONE real coordination scenario end to end — several
agents opened, given work, waited on concurrently, results collected —
written in Rust, running on Linux and Windows. The coordination core is
generic; bee-herding is its first client, not its only possible one.

Map closed 2026-08-18 — 10 tickets, 11 decisions, no fog left. Ready
for bee-shaping's Lock, which consumes D01-D11 into the feature's
CONTEXT.md. The first thing the feature must repair is the dead spawn
line (D06).

## Notes

- The prompt that opened this effort: "script bằng Rust chạy Linux +
  Windows đầy đủ", "python script là flow phối hợp kịch bản hoàn chỉnh
  có thể là cái cần học", "phát triển bee-herding lên orchestration để
  sau phục vụ nhiều việc".
- **Windows is not a stretch goal here — it is already bee's stated
  primary platform.** `.github/workflows/windows.yml:4-5` says so in
  words, runs the full unexcluded `cargo test --release` suite on
  `windows-latest` on every push, and `release-binaries.yml:44-51`
  ships `x86_64-pc-windows-msvc` alongside `x86_64-unknown-linux-gnu`.
  The crate already carries `cfg(windows)`/`cfg(unix)` dependency
  splits (`windows-sys` / `libc`), a hand-written Win32 Git-Bash
  resolver (`src/shell.rs`), and Windows sharing-violation handling in
  `src/lock.rs:126-186`.
- herdr itself is cross-platform: its release notes record native
  Windows work — an app-local ConPTY runtime in the Windows archive,
  Windows `agent start`, Windows agent detection across Git-Bash `exec`
  boundaries, detached Windows servers surviving logout.
- The crate is **synchronous today**: no tokio, no async-std, no rayon,
  no crossbeam. **D09, the agent's call:** use `std::thread` plus
  `std::sync::mpsc`, not a new async runtime. A wave is a handful of
  workers (the cockpit's cap is four), each waiter is a blocking poll
  loop around a subprocess call, and a thread per worker is the
  cheapest correct thing. tokio would buy scale this workload will
  never need, and would be the crate's first async dependency on two
  platforms. Say so if you want it revisited.
- The bash `$tmpdir/$i.{baseline,out,err,code}` handoff layer in the
  source has no Rust counterpart and should not be recreated. It exists
  only because a bash subshell cannot return a structured value; a
  spawned thread returns one directly.
- The crate has **no test seam for shelling out to an external binary**.
  Every `git` call is a bare `std::process::Command`. The nearest thing
  is the `BEE_POSIX_SHELL` env override in `src/shell.rs:28,133-142`.
- Prior related work, all declined, none superseding this: P29 (headless
  outer loop), P40 (per-swarm-worker worktrees), P62 (five-target
  binaries).
- Standing constraint from `docs/knowledge/areas/bee-herding/overview.md`:
  merge stays a human gesture (R2), dispatch stays behind the owner
  interlock (R3), the permission-posture split holds (R4). An
  orchestration layer adds capability beside those; it never relaxes them.

## Decisions so far

- D01: destination is one real scenario running end to end, not a
  design document — tickets/001-destination.md
- D02: build a generic coordination core; bee-herding is its first
  client — tickets/002-scope-generic-core.md
- D03: the source Python/bash IS a complete choreography and the
  ordering is the thing worth learning — tickets/003-is-the-flow-worth-learning.md
- D04: Windows is already viable end to end (bee and herdr both) —
  tickets/004-windows-viability.md
- D05: own crate in the workspace, linked into the `bee` binary; the
  core never depends on the `bee` crate — tickets/005-where-does-the-code-live.md
- D06: first scenario is a spawn-and-brief wave with collection, and it
  forces the spawn-line repair — tickets/006-first-real-scenario.md
- D07: a worker-backend trait, herdr first; the trait doubles as the
  external-command test seam the crate lacks — tickets/007-backend-seam.md
- D08: `control-loop.sh` becomes Rust in this effort;
  `bootstrap-cockpit.sh` stays bash, recorded as a known gap —
  tickets/008-scripts-to-rust-when.md
- D09: concurrency is `std::thread` plus channels, not a new async
  runtime — agent's call, see Notes.
- D10: the core records nothing; bee keeps one append-only wave ledger,
  one row per wave, and that ledger is what occupancy reads instead of
  counting panes — tickets/009-where-do-results-land.md
- D11: a wave is a VALUE (a `Wave` struct with a failure-policy enum
  from day one), reached through a Rust API; no recipe file format in
  this effort — tickets/010-how-is-a-scenario-described.md

## Not yet specified

Nothing. Every open question this map could phrase has an answer.

## Out of scope

- Porting the source skill's Python or bash. Distilled and rejected in
  `docs/history/research/herdr-orchestrator-distill.md`; the discipline
  is taken, the code is not.
- Unattended merge. R2 holds — merge remains a human gesture.
- Making the working agents' permission posture narrower. R4 holds.
- Orchestrator succession — a wave surviving the orchestrator's own
  context running out. The source skill migrates the role to a fresh
  successor pane rather than pausing, and bee has no equivalent; worth
  building, but nothing in D01's destination depends on it. It returns
  as a fresh effort, not as a line on this map.
- A recipe file format for scenarios. D11 keeps it free to add later;
  it is not built here.
