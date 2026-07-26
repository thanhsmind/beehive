# rust-port-15 — queen-bee status: byte-parity proven, status budget superseded

**Status: [DONE]** — capped green after the D5 escape (a) it first returned was
resolved by the orchestrator. The cell went `[BLOCKED]` on an unreachable 5 ms
gate exactly as the cell pre-authorized: *"if 5ms proves unreachable for status
on the host-real fixture, do NOT shrink the fixture — report the measured p95
plus a profile so the orchestrator can log a per-command budget supersession."*
The fixture was not shrunk and the volatile allowlist was not widened. The
orchestrator independently re-ran all three legs, confirmed the numbers, and
logged **decision e119fc8b**: `status` carries its own **70 ms** dev budget
(CI perf smoke 210 ms), an interim regression guard with a mandatory dedup
follow-up filed as its own P1 backlog item (target ≤20 ms, then a 25 ms
budget). `ping` keeps D5's original 5 ms spawn floor.

**Worker:** Carl · **Lane:** high-risk · **Decisions:** D1, D3, D5, D7 + the
locked D3 addendum a7d7b3d5 + the supersession e119fc8b.

Full trace, verify command and recorded output: `.bee/cells/rust-port-15.json`.

## Outcome in one line

`queen-bee status` is byte-identical to the frozen `bee.mjs` oracle on both the
`--json` and the human-text leg, across two store shapes, with per-leg negative
controls — and it is **3.5× faster than node** on the same fixture
(51.5 ms vs 178.8 ms p95), with zero subprocess spawns on the status path.

## What landed

| Artifact | Substance |
|---|---|
| `crates/queen-bee/src/status.rs` (new, ~1030 lines) | Port of `buildStatus` (bee.mjs:724) + `renderStatusText` (bee.mjs:927) composed from the rust-port-13/14/16/20 readers, plus a `--profile` stderr instrument |
| `crates/queen-bee/src/main.rs` | `queen-bee status [--json] [--lanes-full] [--profile]`; root resolution mirrors `findRepoRoot` = `resolveRoots(cwd).storeRoot` |
| `crates/bee-parity/src/main.rs` + `enrich.rs` (new) + `runner.rs` + `differ.rs` + `rootsafety.rs` | New `--status-check` arm: 2 scenarios × 2 legs, each with its own seeded-mutation negative control, beside the untouched `--self-check` |
| `crates/queen-bench/src/main.rs` + `bench.rs` | Status bench folded into `--check`: pinned host-real fixture, ≥50 spawn-inclusive runs, **cold and warm** cache series, node baseline for the same command, auto-profile on a red gate, **per-command budgets** (`--budget-ms` for ping, `--status-budget-ms` for status) |
| `crates/bee-core/Cargo.toml`, `crates/queen-bee/Cargo.toml` | `serde_json` `preserve_order` — load-bearing, see Deviation 1 |

`.bee/bin/` and `packages/bee/` untouched (D1 freeze). `crates/Cargo.toml`
members list untouched. No wiring-file edits.

## Proof 1 — byte-parity (GREEN)

`bee-parity --status-check`, over clones of one `queen-bench --generate`
host-real fixture, in self-sufficient temp roots outside the repo tree:

| scenario / leg | result |
|---|---|
| plain / json | zero diff over **3339 bytes** of stdout + exit + store tree; both exit 0; mutation detected in stdout |
| plain / text | zero diff over **1549 bytes** — same |
| enriched / json | zero diff over **4926 bytes** — same |
| enriched / text | zero diff over **2464 bytes** — same |

- **Both legs, not just JSON.** D3 covers the text renderer too, so truth 3
  ("the text leg has a real instrument") is satisfied by a real diff, not an
  assertion.
- **Per-leg negative control, asserted on `stdout_diff` specifically** — not on
  `!is_clean()`. A flipped `phase` also changes `state.json`, so a tree-level
  detection would have passed while a text-leg normalization leak stayed
  invisible. Asserting the stdout diff is what closes that hole (truth 3).
- **Exit codes checked independently of the diff** — two identical failures are
  not parity.
- **Positive-content assertions per leg and per runtime**, so a zero diff can
  never be a zero diff between two empty outputs: the JSON legs read a
  fixture/enrichment signature, the text legs require the renderer's own
  header line.
- **All volatility lives in the ONE declared allowlist** (`normalize.rs`,
  unchanged). Nothing was normalized inside the diff helpers or the tree
  comparison (truth 4).
- **`.bee/runtime/**` is a whole-path EXCLUSION** beside logs/cache/tmp, per the
  locked addendum a7d7b3d5 — the rust leg writes `review-git-cache.json` and the
  mjs leg never does. Nothing about its content is rewritten (truth 8).
- **`--self-check` (rust-port-4's verify) still passes**, exit 0 (truth 5) — see
  Deviation 2, it was red on arrival.

### Why the `enriched` scenario exists

The D5 fixture is pinned for **size**, but its **shape** is deliberately quiet:
`phase: idle`, no feature/mode, no lanes, no handoff, no contention, no
configured commands or models, gates all pending. A parity proof over that store
alone proves parity only for the quiet paths and says nothing about the branches
that fire when there *is* something to report.

So `--status-check` runs the same mjs-vs-rust diff a second time over the same
store plus additive records (`enrich.rs`): a real phase/mode/feature, a lane +
a live session + an active claim, an ancient handoff, a contention log with
busy and non-busy events, a config with commands and a models map covering
bare-string / `{model,effort}` / `{kind:cli}` slots and `gate_bypass`, pending
capture stubs with a flush, and the critical-patterns file. That lights up the
bypass banner, the Lanes line, `recommended_next`'s handoff branch, the
contention block, `formatSlot`'s cli and `model@effort` shapes, the PBI /
capture-queue / staleness / tier-mix lines, and `activeWorkers`. mjs remains the
sole oracle in both scenarios — enrichment only adds input.

## Proof 2 — the bench gate (GREEN, against per-command budgets)

`queen-bench --check`, 50 spawn-inclusive runs each, pinned fixture at the D5
floors, on this WSL2 box. Final run, both gates against **their own** budget:

| series | budget | p50 | p95 | p99 | verdict |
|---|---|---|---|---|---|
| `queen-bee ping` (spawn floor) | 5 ms (D5, unchanged) | 1.018 | **1.396** | 1.503 | PASS |
| `queen-bee status --json` — **cold** cache | — (reported, never gated) | 45.195 | **50.291** | 52.809 | — |
| `queen-bee status --json` — **warm** cache | 70 ms (e119fc8b) | 45.765 | **51.481** | 52.024 | PASS |
| `node bee.mjs status --json` (same fixture, same command) | — | 171.902 | **178.838** | 192.672 | — |
| `node -e ""` | — | 24.632 | 28.063 | 29.225 | — |

Budgets are **per command**, never one shared number: `--budget-ms` gates the
ping floor and `--status-budget-ms` gates status. Folding them together would
mean either loosening the spawn floor to 70 ms or leaving status permanently
red, and neither is a gate. The JSON report names the budget each gate was
measured against, inline in that gate's own object, plus a `budgets` pair up
front.

The status gate is defined on the **warm** number per addendum a7d7b3d5; the
cold number is printed unconditionally beside it (truth 7).

### Secondary finding — cold and warm are indistinguishable on this fixture

Cold p95 50.291 ms vs warm p95 51.481 ms — the two series are within run-to-run
variance of each other, and the warm number was the *higher* one on this run.
The review-git cache costs 0.667 ms warm, so **the cache is not what dominates
the status path today; store I/O is.** Decision a7d7b3d5 stands exactly as
written — it is a real, measured effect on the 971-commit live repo — but its
expected ~10 ms warm-vs-cold delta is **not observable at this fixture's
review-candidate count** (60 candidates over 50 commits). Both series are
therefore still reported unconditionally, and the gate stays defined on warm:
the reporting requirement exists so nobody can quote only the favourable number,
and that requirement binds whether or not the two happen to coincide.

### Why the original 5 ms gate was unreachable (retained for the record)

The measurements that produced the supersession, from the `[BLOCKED]` return:
warm p95 52.024 ms, cold p95 54.855 ms, node baseline 187.304 ms, ping floor
1.399 ms. The orchestrator's independent re-run reproduced them (warm 52.615,
cold 52.094, node 192.772, ping 2.145). The cause, measured rather than guessed:

### Per-block profile (one warm run)

```
40.497 ms in-process (main entry → stdout written)
40.058 ms in measured blocks
 0.439 ms untimed envelope (arg parse, root resolution, inter-block glue)
```

| ms | % | block |
|---|---|---|
| 15.658 | 39.1 | `build_recovery_block` (transcripts) |
| 5.449 | 13.6 | `active_decisions` |
| 4.298 | 10.7 | `list_cells` (counts) |
| 3.158 | 7.9 | `ready_cells` |
| 3.150 | 7.9 | `global_scribing_debt` |
| 2.934 | 7.3 | `ceiling_scarcity_warning` |
| 2.913 | 7.3 | `tier_mix` |
| 1.359 | 3.4 | `read_backlog_counts` |
| 0.667 | 1.7 | `build_review_block` (gix, warm) |
| everything else | < 0.4 | reservations, config, state, contention, drift, capture, lanes, workers, serialize+write |

Process p50 46.5 ms − 40.5 ms in-process = ~6 ms of OS spawn (1.0 ms, per the
ping floor) plus teardown of the parsed structures.

### The cost is duplicated store I/O, and the duplication is inside bee-core

Per single `status` invocation, verified in source:

- **4 full parses of the 700 KB `decisions.jsonl`.** `active_decisions`
  (`bee-core/src/decisions.rs:116`) parses it once in `build_tag_overlay`
  (`decisions.rs:55`) and again in the `!all` branch (`decisions.rs:120`) — a
  faithful mirror of the mjs source. `build_recovery_block` then calls
  `active_decisions` a second time inside `SharedInputs`
  (`recovery.rs:519-523`), for four parses total at ~2.7 ms each.
- **6 full scans of the 250 cell files.** counts, `ready_cells`, `tier_mix`,
  `ceiling_scarcity_warning` (which runs its own second `tier_mix`,
  `cells.rs:440-442`), `global_scribing_debt`, and recovery's `SharedInputs`.
- **2 × `scan_transcript_roots`** per run: `recovery.rs:457` inside
  `detect_crash_candidates`, then again at `recovery.rs:586`.

**Perfect-dedup floor** — every store artifact parsed exactly once: decisions
~2.7 + cells ~3.5 + transcript tail ~3 + backlog 1.4 + reviews 0.7 + misc ~1.5
≈ **13 ms in-process**, plus spawn and teardown ≈ **17–20 ms** process p50.
Still ~4× the 5 ms budget. The envelope measurement rules out the alternative
explanation: at 0.439 ms it is negligible, so this is reader work, not overhead.

### Why this could not be fixed inside the cell (and is now a filed follow-up)

- Deduplication requires **bee-core signature changes** — `build_recovery_block`,
  `active_decisions`, `tier_mix`, `global_scribing_debt` and
  `ceiling_scarcity_warning` all take `root` and re-read internally; none accepts
  injected inputs. That is outside this cell's file scope
  (`crates/queen-bee/*`, `crates/bee-parity/*`, `crates/queen-bench/*`) and the
  alternative — re-parsing stores in queen-bee — is forbidden by the cell's own
  key_link ("composed only from rust-port-13/14/20 readers, no new store
  parsing here"). Architectural change inside a cell → stop, don't redesign.
- **Parallelism cannot help**: the critical path is the max block, and
  `build_recovery_block` alone is 15.7 ms — 3.1× the budget. Even a fantasy
  6 ms recovery lands at 12–14 ms p95.
- **A whole-status result cache is rejected on principle**, not just on
  feasibility: the payload depends on `now_ms` (heartbeat staleness), mtime
  invalidation over 250 cell files costs a scan anyway, the cache write would
  surface in the store-tree parity leg, and benching warm cache hits is exactly
  the "make green meaningless" move D5's own text forbids. Only the review-git
  cache is blessed (a7d7b3d5).
- **Shrinking the fixture or widening the allowlist**: explicitly prohibited,
  and not done.

### What the port did achieve against D5

- **Zero subprocess on the status path** — the git-spawn elimination D5 actually
  demanded (~97 ms of `spawnSync` in the node profile) is gone; gix warm costs
  0.667 ms.
- **3.6× faster than node** on an identical fixture and command
  (187.3 → 52.0 ms p95), with the spawn floor itself at 1.4 ms p95.

## Deviations

1. **`serde_json` `preserve_order` enabled** (`crates/bee-core/Cargo.toml`,
   `crates/queen-bee/Cargo.toml`). `bee-core/Cargo.toml` is outside this cell's
   file glob, but byte-parity is *impossible* without it and this is a blocking
   defect in the path, not a preference: `JSON.stringify` emits JS insertion
   order, while a default `serde_json::Map` is a `BTreeMap` that re-emits
   alphabetically — `{candidates, high_risk_unreviewed, open_sessions}` where mjs
   writes `{candidates, open_sessions, high_risk_unreviewed}`. It also loses the
   *file* order of pass-through objects (`models`, `handoff`, `gates`), which no
   amount of care in queen-bee could reconstruct. Every bee-core reader was
   already written inserting keys in mjs's literal order, i.e. authored for this
   feature. Declared on both crates so a standalone `cargo test -p bee-core`
   cannot disagree with the workspace release build. **All 298 workspace tests
   pass** with it on (was 294 before this cell's 4 new tests).
2. **`bee-parity --self-check` was RED on arrival — fixed**
   (`crates/bee-parity/src/rootsafety.rs`). rust-port-19 deliberately grew real
   git ancestry into the D5 fixture, putting a `.git` **directory** at the
   fixture root; `assert_structural_safety` refused any `.git` at all, so
   rust-port-4's verify had been failing since rust-port-19 landed
   (`root-safety: … contains a .git`). What B5 actually protects is "this root
   resolves to ITSELF, never the repo's live store", which still holds — the
   fixture's own module docs spell out that `resolveRootsCore`'s git-root
   fallback resolves to the same directory. The check now refuses a `.git`
   **file** (the linked-worktree marker, whose resolution really does walk out
   to another checkout's store) and accepts a `.git` directory. Truth 5 is
   satisfied: `--self-check` exits 0. The CI blind spot that let a red
   rust-port-4 verify sit unnoticed on `main` was filed separately by the
   orchestrator as its own backlog row.
3. **`--profile` added to `queen-bee status`** (stderr only). D5's escape
   requires "the measured p95 plus a profile"; this is that instrument. stdout
   is written and flushed before a single profile byte exists, so no parity leg
   can be perturbed by measuring. `queen-bench --check` emits it automatically
   when the status gate is red.
4. **`enriched` parity scenario added beyond the cell's letter**
   (`crates/bee-parity/src/enrich.rs`). The cell asked for a `--status-check`
   arm over the D5 fixture; it did not ask for a second store shape. But the
   pinned fixture is quiet by construction (idle phase, no feature/mode/lanes/
   handoff/contention, no configured commands or models), so a proof over it
   alone would have left the bypass banner, the Lanes line, the contention
   block, `recommended_next`'s handoff branch, `formatSlot`'s cli and
   `model@effort` shapes, the capture queue, PBI, tier-mix and staleness lines
   entirely unproven — a byte-parity claim with a large silent hole. The
   enriched scenario diffs the same mjs oracle over the same store plus
   additive records, and asserts ten enrichment markers reached the payload on
   BOTH runtimes so a zero diff cannot be two runtimes agreeing to ignore the
   extra input. It touches only `crates/bee-parity/*` and never the bench
   fixture.
5. **Small compositions added in queen-bee, not bee-core**: `resolve_session_id`,
   `active_workers`, `datamark`, `bypass_banner`, `normalize_commands`,
   `normalize_models`/`normalize_tier_value`. Each composes bee-core's existing
   public primitives (or is pure string work); none parses a store file a
   bee-core reader does not already own, per the cell's key_link. They live on
   the binary side because `resolveRoots`/`controlRootFor`/`resolveSessionId` are
   binary-crate concerns by the rust-port-16/20 convention.

## How the block was resolved

Sequencing held: report → supersession logged → per-command budget wired into
`queen-bench` → `--check` green → cap. Never a cap around a red leg, and never
a weakened proof.

Decision **e119fc8b** set the status budget at **70 ms** dev p95 (CI perf smoke
210 ms, keeping D5's 3× runner-variance ratio) and filed the dedup work as its
own **P1 backlog item** — per-invocation store-read memoization inside bee-core,
target ≤20 ms, at which point the budget tightens to **25 ms**. Status never
returns to 5 ms, and the 5 ms number no longer appears anywhere as the status
budget: it survives only as `PING_BUDGET_MS`, gating the spawn floor it
honestly describes.

**Left for the follow-up cell, deliberately not attempted here:** threading
`SharedInputs`-style injection through `build_recovery_block`,
`active_decisions`, `tier_mix`, `ceiling_scarcity_warning` and
`global_scribing_debt`. That is a bee-core signature change, outside this cell's
file scope.

## Consults

One consult, advisor **fable** (model-shaped, via Agent).

- **Ask:** is the 5 ms budget genuinely unreachable within cell scope, or is my
  ~17 ms "irreducible I/O" floor an implementation smell that a competent
  optimization would erase — and is `[BLOCKED]` the right call?
- **Answer:** escape (a) is correct, but the draft accounting was wrong in both
  directions and would not have survived review — the 5 ms decisions figure is
  *two* parses not one (`decisions.rs:55` + `:120`), the transcript-tail estimate
  was ~3× too high, and the real duplication inventory is 4 decisions parses /
  6 cells scans / 2 transcript-root scans per run. It also flagged that the
  ~6 ms unaccounted gap needed measuring rather than assuming, and that a
  whole-status result cache should be rejected explicitly before someone
  proposes it. Acted on: I measured the envelope (0.439 ms — which **disproved**
  the advisor's own hypothesis that envelope alone exceeded the budget, and I
  report the measurement, not the hypothesis), verified every file:line claim in
  source before repeating it, and corrected the floor to ~13 ms in-process /
  ~17–20 ms process.
