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

`queen-bee status` is byte-identical to the frozen `bee.mjs` oracle across
**six legs** — `--json`, the human text render, and `--json --lanes-full`, each
over two store shapes — with per-leg seeded-mutation negative controls. It is
**3.5× faster than node** on the same fixture (51.0 ms vs 179.2 ms p95), with
zero subprocess spawns on the status path.

## What landed

| Artifact | Substance |
|---|---|
| `crates/queen-bee/src/status.rs` (new, ~1030 lines) | Port of `buildStatus` (bee.mjs:724) + `renderStatusText` (bee.mjs:927) composed from the rust-port-13/14/16/20 readers, plus a `--profile` stderr instrument |
| `crates/queen-bee/src/main.rs` | `queen-bee status [--json] [--lanes-full] [--profile]`; root resolution mirrors `findRepoRoot` = `resolveRoots(cwd).storeRoot` |
| `crates/bee-parity/src/main.rs` + `enrich.rs` (new) + `runner.rs` + `differ.rs` + `rootsafety.rs` | New `--status-check` arm: 2 scenarios × 3 legs, each with its own seeded-mutation negative control, beside the untouched `--self-check`; env-controlled spawns so branch coverage cannot depend on the ambient shell |
| `crates/queen-bench/src/main.rs` + `bench.rs` | Status bench folded into `--check`: pinned host-real fixture, ≥50 spawn-inclusive runs, **cold and warm** cache series, node baseline for the same command, auto-profile on a red gate, **per-command budgets** (`--budget-ms` for ping, `--status-budget-ms` for status) |
| `crates/bee-core/Cargo.toml`, `crates/queen-bee/Cargo.toml`, `crates/queen-bench/Cargo.toml` | `serde_json` `preserve_order` — load-bearing, and semantics-changing for `Map::remove`; see Deviation 1 |
| `crates/bee-core/src/state.rs`, `state_projection.rs` | Lane-row key order fixed to `defaultLaneRecord`'s; `shift_remove` at the HANDOFF projection — both found in the rework round |
| `crates/bee-core/tests/projection_parity.rs` | Byte-level HANDOFF.json parity + an explicit key-order sentinel (parsed-`Value` equality is order-blind under `preserve_order`) |

`.bee/bin/` and `packages/bee/` untouched (D1 freeze). `crates/Cargo.toml`
members list untouched. No wiring-file edits. `crates/bee-core/*` was edited in
the rework round — outside the cell's original glob, and free after rust-port-17
released its reservations; reserved under Carl before any edit.

## Proof 1 — byte-parity (GREEN)

`bee-parity --status-check`, over clones of one `queen-bench --generate`
host-real fixture, in self-sufficient temp roots outside the repo tree:

| scenario / leg | result |
|---|---|
| plain / json | zero diff over **3339 bytes** of stdout + exit + store tree; both exit 0; mutation detected in stdout |
| plain / text | zero diff over **1549 bytes** — same |
| plain / json+lanes-full | zero diff over **3284 bytes** — same |
| enriched / json | zero diff over **5266 bytes** — same |
| enriched / text | zero diff over **2542 bytes** — same |
| enriched / json+lanes-full | zero diff over **5223 bytes** — same |

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
  Deviation 4, it was red on arrival.

### Rework round — the `preserve_order` blast radius, and what it was hiding

An independent goal-check judge returned NEEDS_REVISION on one check,
`declared-deviations-justified`: the `preserve_order` deviation understated its
blast radius, and its record actively denied it ("No bee-core SOURCE was
edited; the change is one additive cargo feature"). That claim was **false in
effect** and is retracted.

In serde_json 1.0.150 (`src/map.rs:156-165`), `Map::remove` is
`#[cfg(feature = "preserve_order")] return self.swap_remove(key)`, and
`swap_remove` "perturbs the position of what used to be the last element".
Enabling the feature therefore silently changed the semantics of every
pre-existing `Map::remove` in the workspace. Two call sites were affected:

| site | what it feeds | fix |
|---|---|---|
| `bee-core/src/state.rs` `build_lane_rows` | status's `lanes` block | row now built explicitly in `defaultLaneRecord` order; the removal is gone entirely |
| `bee-core/src/state_projection.rs` `rebuild_handoff_projection` | `.bee/HANDOFF.json`, a D3 store file | `shift_remove` (mjs drops the same keys with order-preserving `delete`) |

**Sweep.** A workspace-wide hunt for `.remove(` / `remove_entry(` found only two
other hits, both order-irrelevant and now annotated in code: `backlog.rs:170` is
`Vec::remove` (already order-preserving, not a `Map`), and `decisions.rs:174` is
a `std` `HashMap::remove` whose output sequence comes from a separate `order`
vector, never from map iteration. rust-port-17's new write paths (`claims.rs`,
`reservations.rs`, `holds.rs`) and its new queen-bee hooks were included and
carry no `Map::remove`. `preserve_order` is now declared on all three
serde_json consumers so no crate can compile against different `Map` semantics
than its siblings.

**A second bug fell out of proving the first.** Adding the coverage exposed a
D3 break that had nothing to do with `swap_remove`: mjs `laneRecordFrom`
(`state.mjs:1638`) builds `{...defaultLaneRecord(feature), ...parsed}`, and a JS
spread over an existing key overwrites **in place**, so the emitted order is the
*default's* — `schema_version, feature, mode, phase, …`. The Rust port
serialized the shared `State` struct, which declares `phase` before
`feature`/`mode` (that being `defaultState()`'s order for `state.json`), and so
emitted `schema_version, phase, feature, mode, …`. `build_lane_rows` now builds
the row in `defaultLaneRecord`'s order explicitly.

**Why it had stayed invisible** — two compounding gaps, both now closed:
`--status-check` never ran `--lanes-full`, *and* the harness inherited the
ambient `CLAUDE_CODE_SESSION_ID`, which resolves to a session with no record in
the fixture, so `buildLaneSummary`'s `active` was always `null` on a developer
machine and no full lane record was ever serialized. `runner.rs` now clears that
variable and sets `BEE_SESSION_ID` explicitly per scenario, and
`assert_enriched_signature` requires a full lane record plus its
`bound_sessions` on both runtimes — the harness fails loudly if that path ever
stops being exercised, rather than silently proving less.

### Red-first: both regressions verified to fail

Reverting `state_projection.rs` to `remove` (`cargo test -p bee-core --test
projection_parity`):

```
thread 'rebuild_handoff_projection_single_open_record_parity' panicked at
bee-core/tests/projection_parity.rs:220:9:
mjs  key order: ["kind", "written_at", "cell", "writer_session"]
rust key order: ["kind", "writer_session", "written_at", "cell"]
test result: FAILED. 16 passed; 1 failed
```

Reverting `build_lane_rows` to struct-order serialization
(`bee-parity --status-check`):

```
bee-parity --status-check: FAIL — [enriched/json] mjs vs queen-bee reported a diff
  mjs :  "active": { "schema_version": "1.0", "feature": …, "mode": …, "phase": … }
  rust:  "active": { "schema_version": "1.0", "phase": …, "feature": …, "mode": … }
```

Note what the pre-existing assertions could *not* do: `projection_parity.rs`
already compared the two HANDOFF.json files with `assert_eq!` on parsed
`Value`s, and that can **never** catch this — under `preserve_order`,
`serde_json::Map` is an `IndexMap`, and `IndexMap`'s `PartialEq` ignores order.
The new `assert_projected_bytes_identical` compares raw bytes and prints both
key orders on failure.

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

1. **`serde_json` `preserve_order` enabled — and it is NOT purely additive**
   (`crates/bee-core/Cargo.toml`, `crates/queen-bee/Cargo.toml`,
   `crates/queen-bench/Cargo.toml`). Byte-parity is *impossible* without it:
   `JSON.stringify` emits JS insertion order, while a default
   `serde_json::Map` is a `BTreeMap` that re-emits alphabetically — and that
   also destroys the *file* order of pass-through objects (`models`, `handoff`,
   `gates`, `commands`) which nothing on the queen-bee side could reconstruct.
   **Blast radius, previously understated and now corrected:** the feature
   changes `Map::remove` into `swap_remove` workspace-wide (serde_json 1.0.150,
   `src/map.rs:156-165`), silently altering two pre-existing call sites —
   `state.rs` `build_lane_rows` (status's `lanes` block) and
   `state_projection.rs` `rebuild_handoff_projection` (`.bee/HANDOFF.json`, a
   D3 store file). Both are fixed and both are now *proven*: byte-level
   HANDOFF.json parity with a key-order sentinel in `projection_parity.rs`, and
   six mjs-diffed legs including `--lanes-full` in `--status-check`. Reverting
   either fix turns something red (quoted above). The rest of the workspace was
   swept; the two remaining `.remove(` calls are order-irrelevant and annotated
   in code. All 314 workspace tests pass.
2. **Lane-row key order fixed** (`crates/bee-core/src/state.rs`). A real D3
   break, independent of the `swap_remove` hazard and surfaced only by the
   coverage this rework added: mjs emits `defaultLaneRecord`'s key order
   (`schema_version, feature, mode, phase, …`), the port emitted the shared
   `State` struct's (`schema_version, phase, feature, mode, …`). `build_lane_rows`
   now composes the row explicitly, which also removes the `workers` deletion
   entirely — never emitting a key the mjs default does not declare beats
   emitting and removing it.
3. **Harness determinism** (`crates/bee-parity/src/runner.rs`). Every spawned
   leg now clears `CLAUDE_CODE_SESSION_ID` and sets `BEE_SESSION_ID` per
   scenario. Inheriting the environment made branch coverage depend on who ran
   the harness — under a Claude Code session the active lane never resolved, and
   that is precisely how the lane-row bug above survived a green run. Parity was
   never at risk (both runtimes read the same env); the *proof* was.
4. **`bee-parity --self-check` was RED on arrival — fixed**
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
5. **`--profile` added to `queen-bee status`** (stderr only). D5's escape
   requires "the measured p95 plus a profile"; this is that instrument. stdout
   is written and flushed before a single profile byte exists, so no parity leg
   can be perturbed by measuring. `queen-bench --check` emits it automatically
   when the status gate is red.
6. **`enriched` parity scenario (and the `--lanes-full` leg) added beyond the cell's letter**
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
7. **Small compositions added in queen-bee, not bee-core**: `resolve_session_id`,
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
