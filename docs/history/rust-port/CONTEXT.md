# Rust Port (queen-bee) — Context

**Feature slug:** rust-port
**Date:** 2026-07-26
**Exploring session:** complete
**Scope:** Deep
**Domain types:** CALL (CLI + hooks), RUN (verify/test system), ORGANIZE (repo layout, distribution)

## Feature Boundary

Freeze the existing `.mjs` mechanical layer of bee and port it to a single compiled Rust binary named `queen-bee` that runs everything a host project's *runtime mechanics* need — the full CLI surface as enumerated by `command-registry.mjs` (116 command defs across 19 group prefixes incl. `status` at scout time; the registry, not any hardcoded list, is the enumeration authority) and every lifecycle hook — while keeping every on-disk storage format under `.bee/` byte-compatible. The feature ends when a host project's bee **runtime** (CLI + hooks + statusline data path) requires no Node.js and the parity + conformance harnesses prove mjs↔rust equivalence. Install/onboarding may still use Node (per D10); host-facing skills text migration is in scope only as the final-flip rewrite of managed invocation strings (per D11); knowledge-bundle formats are out of scope.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | mjs mechanics are feature-frozen for the duration of the port: critical bugfixes only. The mirror artifact is mandatory and mechanical: every mjs bugfix lands with (a) a `rust-port`-tagged PBI or cell noting the behavior delta and (b) the affected parity fixture(s) updated in the same change | unmirrored fixes silently fork behavior; a named artifact makes the freeze checkable |
| D2 | One compiled binary, `queen-bee`, replaces `bee.mjs` **and all hooks** in host repos — hooks invoke the same binary (subcommand per lifecycle event), not separate scripts. End-state invocation is `.bee/bin/queen-bee` (repo-relative, not PATH-dependent) | user: "1 file queen-bee đã được biên dịch để chạy mọi thứ kể cả hooks" |
| D3 | Storage compatibility is a blanket contract: **every file under `.bee/`** (state.json, config.json, reservations.json, decisions.jsonl + decisions-archive.jsonl, backlog.jsonl, capture-queue.jsonl, review-candidates.jsonl, onboarding.json, cache/ incl. `cache/inject-cache.json` + `cache/manifest-hash.json` (legacy top-level `.inject-cache.json` fallback included), cells/ + cells/archive/, claims/, intent/, lanes/, reviews/, workers/, tmp/, spikes/, feedback-digest.json, logs/*.jsonl, locks/, runtime mailboxes, sessions/ stores) keeps its current schema and semantics. Zero migration; mjs and Rust interleave on the same store during the port. Unlisted ≠ changeable | user: "cấu trúc jsonl lưu trữ như hiện tại không gì đổi" |
| D4 | Rust workspace lives in this repo (`crates/` at repo root); host onboarding payload excludes all Rust source — host projects receive only the compiled binary | user: host users "sẽ không thấy" the Rust code |
| D5 | Performance target: p95 < 5 ms **spawn-inclusive wall time** per queen-bee invocation on the hot paths (hook events, `status`, preamble inject, statusline data), proven by a dedicated benchmark harness — **host-real fixture store with pinned minimum sizes taken from this repo (decisions.jsonl ≥ 700 KB, reservations.json ≥ 600 KB, backlog.jsonl ≥ 250 KB, ≥ 250 cell files)**, ≥50 runs per command, p95 over full process lifetime — runnable as a cell `verify` command on the dev machine (CI perf smoke: 15 ms budget for runner variance). Reaching 5 ms explicitly requires **work elimination, not just translation**: profiling shows `status` today spends ~97 ms in git subprocess spawns, ~37 ms transcript-tail reads, ~65 ms JSON reads — so the hot paths must run **zero subprocess spawns** (git via in-process library or cache) and bounded reads. If a planning spike proves 5 ms unreachable for a specific heavy command on the host-real fixture, that command gets an explicit per-command budget logged as a supersession — never a silently smaller fixture. The existing `timings.jsonl` is baseline color only (cold-start-exclusive, hooks never write it): NOT the acceptance instrument | measured node reality: hooks 90–160 ms end-to-end; `status` p95 486 ms startup-excluded — node cold start is only ~15% of `status` cost, so the target stands or falls on work elimination, and a small synthetic fixture would make green meaningless |
| D6 | Test system for the Rust project is graph-based: a dependency graph over crates/modules drives impacted-only local runs, executed in parallel. Done-bar for selection correctness: zero false negatives against a full run on a recorded probe set (mutation probes per crate). The existing mjs verify estate (105 suites at scout time: 82 discovered `test_*.mjs` + extras) stays mjs and serves as the parity oracle during the port (per D1 freeze); each is retired only when its command group flips with equivalent Rust-side coverage. Full suite stays CI-owned | user: graph-based, parallel; a coarser graph that silently under-selects would be a fake speedup |
| D7 | Port is incremental behind three harnesses, flipped registry-group-by-registry-group (the `command-registry.mjs` group prefixes are the flip units): (a) **CLI parity** — same command, same fixture store → mjs vs rust stdout/exit/side-effect files diffed; (b) **hook conformance** — stdin JSON fixtures → exit code + stdout + side-effect files diffed, with the fail-open contract asserted explicitly (internal crash → exit 0 + hooks.jsonl entry; deliberate denial → exit 2; a port that turns crashes into denials is red). (c) **lock conformance** — dedicated concurrency tests (see D9), since output-diffing cannot see race behavior. A group flips only when its suites are green; hook flips update BOTH wiring files (`.codex/hooks.json` and `.claude/settings.json`). The flip checklist also sweeps **invocation-string recognizers**, not just wiring: any code that pattern-matches the literal entry-point name (e.g. `bee-write-guard.mjs:499` `DISPATCHER_RE = /^bee\.mjs$/i` driving CLI-shape validation) must recognize the new name at flip, or the capability silently fails open while all conformance fixtures stay green. Never big-bang | D3 makes interleaving safe only if equivalence is proven per unit; hooks and locks are not command-output-shaped, so they need their own oracles |
| D8 | Distribution: prebuilt per-platform binaries (linux x64/arm64, macOS x64/arm64, windows x64) built in CI, published as **GitHub Releases assets** — never committed to git; the installer downloads the platform binary at install/onboard time. Windows x64 delivery is contingent on the Windows feasibility spike (Outstanding Questions) — if the spike fails, Windows ships later under its own PBI without blocking the other platforms | hosts must not need a Rust toolchain; the current channel (git clone + plugin marketplace) cannot carry binaries |
| D9 | The cross-process lock/lease protocol is part of the frozen storage contract, named explicitly: lock path scheme (sanitized name + sha256 prefix), lock body `{pid, session, ts, token}`, staleness windows (STALE_MS 30 s / HARD_STALE_MS 1 h) with pid-liveness probe, rename-based takeover verified by pid+token+ts identity, transient-FS retry policy, hooks-never-wait (`maxAttempts: 1`), and `.bee/logs/contention.jsonl`. Rust implements the same protocol semantics so mjs and Rust processes can hold/contend the same locks safely during the port | two simultaneous holders is a known escaped failure class; this contract is invisible to output-diff parity, hence its own D-ID and conformance suite |
| D10 | Exit criterion split: the bee **runtime** (CLI, hooks, statusline data path) requires no Node; **install/onboarding** (`install.sh`, `onboard_bee.mjs`, `plugin_distribution.mjs`) may keep using Node in this feature. Pure-binary installer stays a deferred follow-on | porting the installer now would widen scope without serving the 5 ms goal |
| D11 | Call-site migration: during the port window, `.bee/bin/bee.mjs` remains the entry point (groups it dispatches internally shrink as they flip — it execs `queen-bee` for flipped groups). At final flip, a managed invocation-string rewrite — **machinery that does not exist yet and is itself a deliverable of this feature** (onboarding today has no such substitution pass) — rewrites all managed invocation strings (skills, AGENTS.md, hook wiring, statusline) from `node .bee/bin/bee.mjs` to `.bee/bin/queen-bee`, and the mjs layer is removed from the host payload. Enumerated call-site classes: ~190 strings in AGENTS.md + skills; **11 code-embedded invocation strings emitted to the agent at runtime** (bee-session-close.mjs ×4, inject.mjs ×2, dispatch-prepare.mjs ×2, bee-chain-nudge.mjs ×2, compaction.mjs ×1, mirrored under packages/bee/); hook wiring; statusline. All migrated by the rewrite pass, never by hand | keeps every existing call site working mid-port; the transitional node exec shim costs node startup only until final flip |

### Agent's Discretion

- Rust crate layout, dependency choices (serde, clap vs hand-rolled args, etc.), and the graph engine for the test system — constrained by D5 (speed) and D8 (static, portable binaries; prefer musl/static linking, zero dynamic deps).
- Order of command groups in the incremental port (D7) — recommend hot paths first (hooks, status/inject, statusline) since they carry the 5 ms budget.
- Exact benchmark-harness mechanics (D5) and parity-fixture strategy, within the stated acceptance shapes.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| queen-bee | The single compiled Rust binary that replaces bee.mjs + hooks in host repos (end-state path `.bee/bin/queen-bee`) |
| freeze | Feature-freeze of `.mjs` mechanics: bugfixes allowed with the D1 mirror artifact; new capabilities land in Rust only |
| parity harness | Golden-output rig running the same command on the same fixture store through mjs and Rust, diffing stdout/exit/side-effect files (per D7a) |
| conformance suite | The non-output-shaped oracles: hook contract tests (D7b) and lock protocol tests (D7c/D9) |
| flip | The moment a registry group's dispatch switches from mjs to queen-bee in host-facing wiring; final flip = managed-text rewrite + mjs removal (D11) |
| runtime vs install | D10 split: runtime = CLI/hooks/statusline (no Node allowed at end-state); install = onboarding machinery (Node still allowed) |

## Specific Ideas And References

- User's stated end-state: agent wall time dominated by the Agent itself; bee overhead becomes noise (<5 ms).
- Scout-measured baseline on this WSL2 box: `node -e ""` ≈ 20 ms; importing bee.mjs ≈ 60–70 ms; hooks end-to-end 90–160 ms; `status` p50 360 ms / p95 486 ms startup-excluded (profile: git spawnSync ~97 ms, transcript tail ~37 ms, JSON reads ~65 ms, GC ~28 ms, module compile only ~15 ms). Cold start is the whole cost on the thin hook path, but only ~15% of `status` — the 5 ms target is won by zero-subprocess design + fast I/O (per D5), not by translation alone.
- `.claude/statusline-command.sh:60` invokes node per statusline render — same hot path, moves to queen-bee (D5 scope).

## Existing Code Context

Scout digest: `docs/history/rust-port/reports/mjs-inventory.md` (surface map) — with review corrections: `bee.mjs` is ~7,259 lines; the dedup runtime surface queen-bee must reproduce (`bee.mjs` + `.bee/bin/lib/` + `.bee/bin/hooks/`) is ~38,300 lines; repo-wide mjs ≈ 158 files / ~120k lines; registry holds 116 command defs across 19 group prefixes incl. `status` (and `perf`, `worktree`, `config`, `herding`, `recovery`, `doctor`, `dispatch` — absent from the first inventory pass).

### Integration Points

- `.bee/bin/lib/command-registry.mjs` — the CLI surface enumeration authority (116 defs, 19 groups); D7's flip-unit list derives from it.
- `.bee/bin/lib/lock.mjs` — the D9 lock protocol source of truth (incl. hooks-never-wait `maxAttempts: 1`, takeover identity check, contention log).
- `.bee/bin/hooks/` — 9 `bee-*.mjs` hook impls + `adapter.mjs` + `tokenize-command.mjs`, fail-open contract; wiring in BOTH `.codex/hooks.json` and `.claude/settings.json`.
- `scripts/run_verify.mjs` + `scripts/impact_registry.mjs` — current impacted-run doctrine (4 edge types incl. spawn-argv — finer than a crate graph; D6's done-bar exists because of this).
- `packages/bee` + `scripts/install.sh` + `onboard_bee.mjs` — install/onboarding flow (stays Node per D10); where D8 binary download and D4 source exclusion land.
- `.claude/statusline-command.sh` — third node entry point on the hot path; moves to queen-bee.

## Canonical References

- `AGENTS.md` — workflow law the mechanics implement; unchanged by the port.
- `docs/knowledge/index.md` — critical patterns; several govern porting risk (shim side-effect loss, resolver-sweep completeness — "the seam ships when the LAST consumer moves", vendoring drift canaries).
- `docs/knowledge/areas/workflow-state/holds-and-the-coordination-lock.md` — lock/hold semantics the D9 conformance suite asserts.
- `docs/knowledge/areas/performance-log/cli-self-timing.md` — why timings.jsonl is not the D5 acceptance instrument (R1: direct CLI runs only).

## Outstanding Questions

### Deferred To Planning

- [ ] Graph engine/shape for the test system (cargo metadata crate graph vs custom module graph) — answered by a planning spike measuring impacted-run latency against the D6 zero-false-negative bar.
- [ ] Spawn-inclusive cold-exec latency of a static Rust binary on WSL2 (and Windows) — the D5 feasibility spike; expected ~1–3 ms but must be measured before the target is treated as proven.
- [ ] Parity harness fixture strategy — synthesized fixture stores vs snapshots of this repo's `.bee/` — answered during planning by inventorying store variance.
- [ ] Windows portability of the binary + hooks wiring (D8 contingency; intersects the existing Windows portable-suites P2 backlog item).
- [ ] Port order of the 19 registry groups after the hot-path head (hooks/status/inject) — planning sequences by risk and dependency.
- [ ] Per-command 5 ms feasibility on the host-real fixture for the heavy commands (`status`, `worktree merge`) — the D5 spike; a proven-unreachable command gets an explicit budget via supersession, never a shrunk fixture.

## Deferred Ideas

Out-of-scope ideas captured during exploring. Not lost, not planned.

- Pure-binary installer (npm/Node-free onboarding) — natural follow-on once D8+D10 land; separate PBI.
- Porting `run_verify.mjs`'s registry for host-project JS test surfaces (beyond bee's own mechanics) — deferred; bee mechanics first.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Validating and reviewing use locked decisions for coverage and UAT.
