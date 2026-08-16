# CONTEXT — rust-write-guard

**Feature slug:** `rust-write-guard` · **Date:** 2026-07-22 · **Source:** user instruction ("port riêng write-guard hook sang Rust"), which is part (2) of backlog **P59** (decision `2e224c90` — superseded 2026-08-16 by `f7ea4ce2`: the full rewrite that decision rejected later happened via rust-port).

## Boundary

In scope: the per-tool-call cost of the write-guard hook, and the mechanism that would let a second (Rust) implementation of it exist without the two drifting apart. Out of scope: porting the bee CLI itself (P59 keeps that rejected — the CLI's remaining cost is OS-bound I/O and git, not language); changing what the guard decides (every rule stays exactly as it is today — this is a speed change, never a policy change).

## Measured facts (this repo, 2026-07-22, medians of 10)

| | cost |
|---|---|
| bare `node -e 1` | **21 ms** |
| hook on a Read-shaped call | **42 ms** |
| hook on a Bash+git-shaped call | 48 ms |
| `node -e "import('./lib/state.mjs')"` alone | **42 ms** |

The decomposition is decisive and was not what P59 assumed: **the guard's decision logic is essentially free.** Loading `state.mjs` by itself costs the same 42 ms as running the entire hook, so the hook's non-node cost is ~21 ms of *module loading* (its import graph is 3,503 lines: state.mjs 1,964 + guards.mjs 948 + reservations.mjs 256 + worktree-holds.mjs 335), and ~0 ms of actual deciding. The remaining 21 ms is node's own start-up floor, which no JavaScript change can remove.

Consequences for the ask:
- A JS-only fix (load less) can reach roughly **25 ms** — a ~40% cut, hours of work, zero new risk.
- Rust can reach roughly **3 ms** — a ~93% cut, but only by adding a build step and a second implementation of security-critical rules to a project that today ships by copying files with no build at all.

## Locked decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | **Conformance vectors come first, and they are the deliverable that makes a port safe.** One table-driven corpus of `(hook input, repo state) → expected verdict + reason class`, executed against the JS implementation today and against any future implementation. A port is only allowed to ship when it passes the same corpus, byte-for-byte on the verdict. | Two implementations of a guard is two chances to be wrong, and drift is the failure mode this repo has hit all week (lib mirrors, hook mirror, three prose homes, taxonomy schema). The repo's own answer to that has always been a parity test; this is that idiom applied before the divergence exists rather than after. |
| D2 | **Trim the hook's import graph first.** The hook loads a 3,503-line graph to make a decision that costs ~0 ms. Give the guard path only what it needs (a slim resolution module, or dynamic imports inside the branches that actually need reservations/holds), keeping `state.mjs` the single source of truth for the values it owns. Target ≤25 ms per call, proven by the same 10-run median method. | It is most of the achievable win, it is needed regardless (the Rust build will always need a JS fallback path for unsupported platforms), and it is measurable this afternoon instead of next week. |
| D3 | **Rust is a fast path, never the only path.** The binary answers the calls it is certain about; anything it does not fully recognize — an unknown tool shape, an unparsed command, a state it cannot resolve — defers to the JS implementation rather than guessing. Absent or incompatible binary ⇒ the JS hook runs, unchanged. | The guard's existing discipline is fail-open on internal error; a port must not convert "I am not sure" into either a wrong allow or a silent absence of the guard. A deferring fast path keeps exactly one authoritative implementation of the hard cases. |
| D4 | **No binary ships without reproducible per-platform builds and verification.** Linux x64/arm64, macOS x64/arm64, Windows x64, built in CI, checksummed, and verified at install; the installer picks by platform and silently falls back to JS when there is no match. bee's copy-only distribution model is otherwise unchanged. | This is the real cost of the port, and it is the part that outlives the code. Naming it here stops it from being discovered late. |
| D5 | **The port changes speed, never policy.** No rule is added, removed, relaxed or reinterpreted in the same feature. Any rule change happens in JS first, gets vectors, and only then reaches the binary. | A performance change that quietly alters a safety decision is the worst possible outcome of this work. |

## Pinned terms

- **Conformance vector** — one `(input, state) → expected verdict` case, authoritative for every implementation.
- **Fast path** — the Rust binary's answer for cases it fully recognizes.
- **Defer** — the fast path declining to answer, handing the call to the JS hook.

## Scout paths

- `.bee/bin/hooks/bee-write-guard.mjs` (the shim) · `.bee/bin/lib/guards.mjs:checkWrite` (all decision logic)
- `.bee/bin/lib/state.mjs` (1,964 lines — the dominant import cost), `reservations.mjs`, `worktree-holds.mjs`
- `hooks/test_write_guard.mjs` (147 checks — the seed corpus for D1's vectors)
- P59 / decision `2e224c90` (the original trigger-conditioned plan, whose cost model this CONTEXT corrects; superseded 2026-08-16 by `f7ea4ce2` after rust-port shipped the full rewrite)

## Open questions (for planning)

- Whether the vectors live as JSON data (language-neutral, preferred) or stay expressed in the JS suite and are exported.
- Whether the fast path covers only the ALLOW-common cases at first (Read/Glob/Grep on ordinary paths) — leaning yes: highest frequency, lowest risk, and a wrong defer costs only speed.

## Deferred ideas

- A resident/daemon guard (socket or long-lived process) — rejected for now, same no-daemon rule the rest of bee follows.
- Porting other hooks; measure them first, the same way.
