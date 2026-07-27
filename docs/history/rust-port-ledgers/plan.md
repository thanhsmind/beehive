---
artifact_contract: bee-plan/v1
mode: high-risk
feature: rust-port-ledgers
parent_feature: rust-port
slice: 4 (of the rust-port epic map, plan.md:29)
---

# rust-port slice 4 — ledger command groups

Continuation of the `rust-port` feature (closed at `compounding-complete` after slices 0–3).
Locked decisions remain `docs/history/rust-port/CONTEXT.md` — this plan cites them, never
reinterprets them.

## Mode gate (mechanical)

Risk flags counted from the flag list:

| Flag | Hit | Why |
|---|---|---|
| data model | YES | the six ledger stores are the frozen on-disk storage contract (D3) |
| audit/security | YES | `datamark` / `SECRET_CONTENT_PATTERNS` / `INJECTION_PATTERNS` neutralization is ported here |
| public contracts | YES | every ported verb's stdout/exit is a public CLI contract (D7a) |
| cross-platform | YES | D8 targets five platforms; path + locale semantics differ |
| external systems | YES | `backlog.add` and `reviews record` spawn `git` |
| multi-domain | YES | six independent stores + a shared lock protocol (D9) |
| changes behavior an existing test asserts | NO | ported code stays DARK; mjs behavior is untouched |
| weakening existing proof | NO | proof is added, never removed |

6 flags, hard-gate flag present (audit/security) → **high-risk**. Smaller modes are
insufficient: `standard` would skip the persona panel over a slice that reimplements the
secret-redaction and cross-process-lock paths in a second language.

Product-file count: `crates/**` only (new Rust modules + parity harness arms). `.bee/**`,
`docs/**` and the vendored `.bee/bin/` mirror are never counted (D6 of the lane rules).

## Scope

Port the six **ledger** command groups from the canonical `packages/bee/` source into the
`queen-bee` / `bee-core` crates, byte-parity proven, **ported code stays DARK — no wiring
flip** (the standing constraint carried since `validation-slice1.md:26`; the flip is its own
slice per D7 and D11).

| Group | Canonical source | Verbs | Store |
|---|---|---|---|
| `intent` | `packages/bee/lib/intent.mjs` (302) + `bee.mjs:4059-4132` | set, show, advance, clear | `.bee/intent/<key>.json` |
| `capture` | `packages/bee/lib/capture.mjs` (118) + `bee.mjs:4009-4056` | add, list, flush, count | `.bee/capture-queue.jsonl` |
| `decisions` | `packages/bee/lib/decisions.mjs` (982) + `bee.mjs:1940-2230` | log, supersede, redact, active, search, archive, tag, render | `.bee/decisions.jsonl`, `.bee/decisions-archive.jsonl`, `docs/decisions/index.md` |
| `backlog` | `packages/bee/lib/backlog.mjs` (784) + `bee.mjs:3670-3944` | counts, rank, badges, add, propose, pbi.add, pbi.status, pbi.amend, pbi.list, render, findings | `.bee/backlog.jsonl`, `docs/backlog.md` |
| `reviews` | `packages/bee/lib/reviews.mjs` (474) + `bee.mjs:4300-4448` | create, list, show, record, candidate.add, candidates, status | `.bee/reviews/<id>.json`, `.bee/review-candidates.jsonl` |
| `feedback` | `packages/bee/lib/feedback.mjs` (926) + `bee.mjs:4448-4500` | digest, count, collect, rank | reads `.bee/backlog.jsonl` + `.bee/decisions.jsonl`; writes `.bee/feedback-digest.json` |

**Out of scope, explicitly:** any flip of these groups in host wiring; the `--help --json`
schema payloads beyond byte-parity of what mjs already emits; the remaining hooks, the
statusline path, distribution, and the final flip (separate slices).

## Discovery — L1 (repo precedent, cited)

No candidate comparison was needed: slices 0–3 already fixed every mechanic this slice
reuses. Cited precedent rather than research:

- Module layout + dispatch: `crates/queen-bee/src/main.rs:20-36` (flat `match` arm per
  command), `crates/queen-bee/src/lib.rs:14-17` (`pub mod`), `crates/queen-bee/src/status.rs`
  as the reference-size port (1304 lines for one heavy command).
- Parity recipe: `crates/bee-parity/src/main.rs:343-438` (`check_one_leg`) with
  `differ::diff_legs` + the **seeded-mutation negative control** at `main.rs:427-432`.
  Fixtures come from `queen-bench --generate`, never hand-authored.
- Ordering: `serde_json` with `preserve_order` is load-bearing in every crate
  (`crates/bee-core/Cargo.toml:25-37`); `Map::remove` must be `shift_remove`
  (`bee-core/src/state_projection.rs:326`).
- Atomic writes and jsonl appends: `bee-core/src/fsutil.rs:229-254`, `:260-270`, oracle-checked
  against the real mjs in `bee-core/tests/fsutil_oracle.rs`.
- D9 lock protocol already ported in slice 0 (`bee-core/src/lock.rs`, 1126 lines) — this slice
  consumes it, it does not reimplement it.
- Existing read-side bee-core modules to extend rather than duplicate: `decisions.rs` (243),
  `backlog.rs` (248), `capture.rs` (73), `reviews.rs` (976).

## Approach

**Chosen path — a shared group-dispatch seam first, then one cell per group, smallest group
first.**

`main.rs` today is a flat `match` over single-word commands. Six groups × 38 verbs cannot be
bolted onto that without every group cell reinventing arg parsing, `--json` emission and its
own parity arm. So cell 1 builds the seam (group/verb dispatch + a *generic* bee-parity
command-check that runs an arbitrary `bee <group> <verb> <args…>` on both runtimes and diffs
stdout, exit code and the resulting `.bee/` + `docs/` tree), and every later cell adds one
group behind it plus its scenario list.

Order is smallest-and-most-isolated first so the seam is proven before the risky groups land:
`intent` (no lock, no subprocess, per-key files) → `capture` (needs the shared `datamark`
module) → `decisions` write side (D9 lock) → `decisions` read/render side → `backlog` →
`reviews` → `feedback` (pure aggregator, depends on decisions+backlog being byte-right).

**Rejected alternatives.**

- *Port all six in one cell.* ~3600 JS LOC → a `status.rs`-sized module per group; one cell
  would exceed any honest context budget and make a red parity run unattributable.
- *Port only the pure `lib/*.mjs` logic and leave handlers in mjs.* The gather established the
  handler blocks live inline in `bee.mjs` and carry the flag validation, the git side effects
  and the usage-text fallbacks — a port that stops at `lib/` cannot satisfy D7a, which diffs
  the **command's** stdout/exit.
- *Unify the two lock call patterns* (`withDecisionsLockSync` vs `backlog.mjs`'s bespoke
  `addPbi` retry constants). Rejected: D3/D9 freeze observable protocol semantics, and the two
  differ in retry counts. Replicate both; note the duplication as a follow-up PBI.
- *Normalize the inconsistent sorts in `feedback.mjs`.* Rejected for the same reason — port
  each call site's exact comparator (`feedback.mjs:185,606,654` bare `.sort()` vs
  `:648,649,819,976` `localeCompare`). Cleanup after parity, never during.

**Risk map.**

| Component | Risk | Proof required |
|---|---|---|
| Markdown renderers (`docs/decisions/index.md`, `docs/backlog.md`) | HIGH | byte-diff of the rendered file *and* the `render --check` drift verdict, both runtimes |
| `localeCompare(…, 'en', {numeric:true})` in `reviews.mjs:135` | HIGH | parity scenario with ids `review-2`/`review-10`/`review-1` present |
| `datamark` regex neutralization (`decisions.mjs:1046-1054`) | HIGH | a shared unit oracle spawning the real mjs over an adversarial corpus |
| D9 lock use by `decisions` + the separate `backlog` PBI lock | HIGH | lock-conformance (D7c): concurrent mjs↔rust writers, no lost update |
| `git` side effects (`bee.mjs:3773-3786`, `reviews.mjs:401`) | MEDIUM | assert exact argv + cwd; a widened pathspec is a red |
| JS key-insertion order in appended jsonl | MEDIUM | covered by `preserve_order` + byte-diff of the appended line |
| fail-open vs fail-loud split in `reviews.mjs:10-16` | MEDIUM | scenario feeding a corrupt session file to both a mutation verb and `list` |
| `resolveInScope` realpath containment (`feedback.mjs:11-25`) | MEDIUM | symlink-escape scenario asserting identical refusal |

## Slice cells (current slice only)

| Cell | Unit | Depends on |
|---|---|---|
| rpl-1 | group/verb dispatch seam in `queen-bee` + generic `bee-parity --cmd-check` harness (arbitrary command, stdout+exit+tree diff, seeded-mutation negative control) | — |
| rpl-2 | `intent` group (4 verbs) — first group through the seam | rpl-1 |
| rpl-3 | `bee-core::datamark` (secret + injection patterns, control-char strip) with an mjs oracle test, then the `capture` group (4 verbs) | rpl-1 |
| rpl-4 | `decisions` write side: log, supersede, redact — on the ported D9 lock, incl. lock conformance vs a live mjs writer | rpl-3 |
| rpl-5 | `decisions` read side: active, search, archive, tag, render (+ `render --check` byte drift on `docs/decisions/index.md`) | rpl-4 |
| rpl-6 | `backlog` group: friction/finding appends, `pbi.*` behind its own retry-constant lock, `render`/`render --check` on `docs/backlog.md`, scoped `git add`/`git commit` argv parity | rpl-3 |
| rpl-7 | `reviews` group: session files, candidates jsonl, strict-vs-fail-open split, numeric locale sort, scoped git spawn | rpl-1 |
| rpl-8 | `feedback` group: digest/count/collect/rank aggregator, `resolveInScope` containment, per-call-site comparator fidelity | rpl-5, rpl-6 |

Each cell's `verify` is the scoped Rust build plus its own parity arm — never the full
configured verify (ci-owned-verify D1/D6):

```
cargo build --release --manifest-path crates/Cargo.toml && \
  cargo run --release --manifest-path crates/Cargo.toml -p bee-parity -- --<arm>
```

## Test matrix (edge dimensions sketched at high-risk depth)

- **Empty / absent store** — every verb against a store file that does not exist yet.
- **Corrupt row** — one unparseable jsonl line; fail-open readers skip, strict readers throw.
- **Ordering** — ids that separate byte sort from numeric locale sort; key insertion order in
  appended rows.
- **Concurrency** — two writers (one mjs, one rust) contending the same lock; `.bee/logs/contention.jsonl` shape.
- **Unicode / control chars** — `datamark` corpus incl. ` -`, fake role tags, code fences.
- **Path** — symlinked `.bee/`, a path outside scope, Windows-shaped separators (the
  `windows-path-identity` learnings apply: never assert a platform-shaped string as ambient truth).
- **Side effects** — the git argv is exactly the narrow pathspec; a widened scope is a red.
- **Idempotence** — `render` twice produces a byte-identical file and a clean `--check`.

## Open questions carried into validating

- Does the generic `--cmd-check` harness need per-verb *input* fixtures (stdin payloads), or do
  argv scenarios cover the six groups? (`decisions log` takes long text; check how mjs reads it.)
- `feedback digest` writes `.bee/feedback-digest.json` — confirm whether the handler or the lib
  owns that write, since the gather could only infer it from `bee.mjs:4453`.
- Whether a `LEDGER_BUDGET_MS` is owed in `queen-bench` for these groups, or whether D5's hot-path
  list (hooks, status, inject, statusline) leaves them unbudgeted.
