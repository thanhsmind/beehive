# rust-port-2 — D5 benchmark instrument (queen-bench)

Status: [DONE]

## Outcome

Built the D5 acceptance instrument in `crates/queen-bench`: a fixture generator
(`--generate`) producing a host-real `.bee` store at the pinned minimum sizes
(decisions.jsonl >=700KB, reservations.json >=600KB, backlog.jsonl >=250KB,
>=250 cell files) that refuses to write anything if any requested size is
under its floor, plus a spawn-inclusive p95 runner (`--check`) gating
`queen-bee ping` against `--budget-ms` (default 5, dev gate; CI perf smoke
passes 15) over >=50 runs, always emitting a JSON report with queen-bee
percentiles alongside a `node -e ""` baseline. `--self-test` proves both
directions: all four sub-pinned-size refusal cases refuse, and a happy-path
generation measurably meets every pin on disk and passes a real
`node .bee/bin/bee.mjs status --json` sanity read against the generated store
(temp-dir root outside the repo tree). The queen-bee binary path resolves as
a sibling of queen-bench's own executable (the workspace release target
dir), overridable via `--bin-path`.

Measured on this WSL2 box: `queen-bee ping` p95 ~1.1–1.8 ms (well under the
5 ms budget, consistent with the earlier cold-exec spike), `node -e ""`
baseline p95 ~25–28 ms.

## Files touched

`crates/queen-bench/src/main.rs`, `crates/queen-bench/src/fixture.rs`,
`crates/queen-bench/src/bench.rs`, `crates/queen-bench/src/selftest.rs`.

Commit: `2528a24` — one commit, cell id `rust-port-2` in the message.

## Deviations

None. No package installs (`crates/queen-bench/Cargo.toml` keeps an empty
`[dependencies]` — the generator, timing, and JSON output are all hand-rolled
against `std`, matching rust-port-1's precedent of deferring new crate deps
to later slices).

Full cell definition/trace: `.bee/cells/rust-port-2.json`.
