---
artifact_contract: bee-plan/v1
mode: high-risk
# approved_gate2: <unset until approval>
---

# Plan: Sweep At Every Door

Mode: `high-risk` — 4 risk flags: public-contracts, covered-contract-change,
multi-domain, data-model. No hard-gate flag (no auth, data loss, external
provider, or validation removal).

Why this is the least workflow that protects the work: the change makes an
already-live destructive path (a sweep that takes a claim away from its owner)
fire more often *and* raises its cost per firing (D4 turns a recoverable
`open` into a human-gated `blocked`), so the shape has to prove the sweep
cannot fire on a living session, and cannot write into the wrong store, before
it makes the sweep cheaper to reach.

## Requirements (from CONTEXT.md)

- **D1** — the sweep gains one new call site, `bee orient`. `claim-next` keeps
  its sweep; `bee status` stays report-only; no all-verb preflight.
- **D2** — a heartbeat-stale session record is marked dead in place
  (`status: "dead"`, `dead_at`), never deleted, no hard-stale deletion tier.
- **D3** — `bee recovery scan` is built as a releasing door: releases on
  invocation, no `--release` flag, no confirmation. `recovery window` stays
  unbuilt.
- **D4** — a swept cell goes to `blocked` with a reason naming the dead
  session and its worktree, replacing the current `claimed -> open` reset.
- **D5** — the sweep never writes across a store boundary: it always removes
  the claim, parks the cell only when the cell is readable in the sweeper's own
  store, and otherwise reports the cell id and holding worktree and leaves the
  cell untouched.
- **D6** — a door that cannot resolve its own caller session does not sweep;
  it reports the count of expired claims detected and writes nothing.

## Discovery

Six questions were carried or raised. All are answered against the source.

1. **The orient door covers orchestrators only.** A dispatched worker never
   runs `bee orient` — `.bee/bin/prompts/worker-cell.md` has no occurrence of
   it and goes from the pre-claimed cell straight to `bee cells finish`
   (`worker-cell.md:11-30`). The SessionStart preamble does not run orient's
   code either: it is `build_session_preamble`
   (`hooks/session_preamble/budget.rs:448`, documented "Pure: reads state,
   never writes"), while `orient_next_command` / `orient_decision_line` /
   `orient_worktree_context` (`orient.rs:26,38,55`) are referenced only from
   `orient.rs` and `verbs/status_full/tests.rs`. (`skills/bee-swarming/SKILL.md:19`
   says orient "shows where the work stands **either way**" — it does not
   restrict orient to orchestrators; the restriction is the worker prompt's
   silence, which is the evidence that counts.)

2. **The sweep can take a living session's claim, and nothing stops it.**
   `pub(crate) fn sweep_expired_claims(control: &Path, now: f64) -> MR<()>`
   (`handlers_select.rs:62`) accepts no caller session id and contains no
   self-exclusion. `claim-next` resolves its own `session` at `:517` and does
   not pass it to the sweep at `:519`. Staleness is heartbeat age only
   (`heartbeat_stale`, `claims.rs:598-604`, 900s), and heartbeat touches fire
   only from event hooks (`hooks/state_sync.rs:32`, 60s throttle;
   `prompt_context.rs`). **A single tool call running longer than 900s emits no
   `PostToolUse`**, so a live session mid-command ages past the line. With the
   3600s claim TTL the reachable failure is: session A holds an hour-old claim,
   spends 15+ minutes inside one long command, session B's sweep takes it.

3. **`blocked` is nonterminal, with teeth beyond the cell.**
   `verbs/state_group/policy.rs:296-304` and `:473-491` both count
   `open|claimed|blocked` as nonterminal, so a swept cell **refuses
   `bee state start-feature`**, FIX line pointing at `bee cells drop`.
   `bee close` does not refuse but holds its auto-archive
   (`verbs/cells/handlers_meta.rs:867-899` — terminal is `capped|dropped`
   only). `claim-next` never sees it: `ready_cells` filters `Some("open")`
   (`handlers_select.rs:410`). Unblock is `bee cells reopen --id <id> --reason <r>`
   (`handlers_close.rs:874-924`). A fourth consumer exists: `verbs/feedback.rs:1045-1053`
   turns any cell with a truthy `trace.blocked_reason` into a friction candidate
   at `pain: 1.0`, so every swept cell starts feeding `bee feedback`.

4. **The store split is the sharp edge, and the orient door is the first
   caller to meet it.** `sweep_reset_cell` writes at `cells_dir(control)`
   (`handlers_select.rs:174-182`). Today that is always safe because the only
   caller enters via `rsv::prelude`, which serves ordinary checkouts only and
   **refuses** a granted worktree (`verbs/reservations/emit.rs:38-46`;
   invariant stated at `handlers_select.rs:32-35`). `orient` uses the wide door
   (`resolve_store_root_worktree`, `orient.rs:462-470`) and serves granted
   worktrees natively (`router.rs:115`), where store root and control root
   diverge (`status_full/tests.rs:570-576`). `roots.rs:157-162` names writing
   across that split as writing "the right bytes into the wrong store". D5
   resolves this by never writing across it.

5. **The caller identity `orient` would pass does not exist.** `orient`'s arg
   routing is a closed six-shape match with no `--session-id`
   (`orient.rs:441-449`), and `run()` resolves no session
   (`orient.rs:454-459` serves `Verb::Status` and `Verb::Orient` from one
   body). The in-crate sources are `resolve_session_flag_env`
   (`claims.rs:627-634`, env `BEE_SESSION_ID`/`CLAUDE_CODE_SESSION_ID`) and
   `resolve_session_adopt` (`claims.rs:638-652`), which returns `Some` only
   when exactly one fresh session record exists — `None` precisely in the
   multi-agent case. Prior art for env-based self-exclusion already exists:
   `detect_crash_candidates` excludes the current session by env id
   (`verbs/status_full/recovery.rs:270-283`). D6 resolves this.

6. **The verb's own shipped contract text contradicts the change.**
   `src/generated/registry_payload.json` describes `orient` as a "**Read-only**
   session-start context packet"; that payload is compiled in via `include_str!`
   (`registry.rs:3-13`) and renders `--help`. No regen chain covers it —
   `bee dev regen` is `render-skill-trees -> onboard --apply -> release-manifest --write`
   (`router.rs:90-92`) — and `src/generated/` is write-guarded
   (`hooks/write_guard/guards.rs:216`). The same file describes `recovery.scan`
   as "Cheap and side-effect-free", which D3 falsifies in slice 2.

Call-site audit (verified, both reviewers agree): `sweep_expired_claims` has
**one** production caller (`handlers_select.rs:519`) and 4 test call sites
(`verbs/cells/tests.rs:2041, 2078, 2094, 2248`); `sweep_reset_cell` is defined
at `:174` and called once at `:112`. Nothing outside the crate. Three tests
assert the `claimed -> open` outcome (`tests.rs:2053, 2098, 2260`) plus one
decision-row text assertion (`:2074`). Stale doc comments after D4:
`verbs/cells/mod.rs:43-44`, `handlers_select.rs:34,60,170`.

Evidence commands: `rg -n "sweep_expired_claims|sweep_reset_cell" packages/bee-rs/crates/bee/src`,
`rg -n "blocked" packages/bee-rs/crates/bee/src/verbs/state_group/policy.rs`,
`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`.

## Approach

**Recommended path.** Make the sweep caller-aware, store-safe, and correct in
its verdict *before* widening its reach. One cell (E1) changes
`sweep_expired_claims` to take the calling session and skip that session's own
claims (per D6's contract), and changes `sweep_reset_cell` to write
`trace.blocked_reason` and `status: "blocked"` when the cell is reachable in
the sweeper's store, or to report and skip when it is not (per D4 and D5) —
one function pair, one coherent diff, one pass over the three tests that assert
`claimed -> open`. Only then does `bee orient` call it (E2, per D1), resolving
its caller from env and declining to sweep when it cannot (per D6), and
correcting its own registry contract text in the same cell. D2 and D3 follow in
slice 2: neither is on the hazard's critical path.

**Rejected alternatives.**
- *Ship the orient door first, fix safety after* — rejected: D4 makes every
  wrong sweep cost a human intervention.
- *Split D4's verdict from the sweep-signature change* — rejected: both edit
  `sweep_reset_cell`/`sweep_expired_claims` and the same three tests; two cells
  buy a serialized wave and two red-test passes for no isolation gain.
- *Resolve each claim's owning store through the grant registry and write there*
  — rejected by D5: it opens a cross-worktree write path that
  `roots.rs:157-162` and the write guard currently forbid.
- *Sweep anonymously when the caller is unresolvable* — rejected by D6.
- *Fix the >900s heartbeat gap inside long commands* — rejected for this
  feature: repo-wide hook/heartbeat semantics, a fifth domain. The residual
  (session B sweeps live session A) is an accepted risk, recorded below.
- *An all-verb preflight sweep* — rejected by D1.

**Risk map.**

| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| `sweep_reset_cell` store reach | HIGH | The orient door is the first caller where control root ≠ store root; the wrong choice writes a cell into a store that does not own it (`roots.rs:157-162`) | Test: sweep run where the cell is absent from the sweeper's store removes the claim, writes no cell, and names cell + worktree in its output |
| `sweep_expired_claims` signature change | MEDIUM | Every caller must pass a session; a missed call site sweeps with no self-exclusion | Compiler (no default args in Rust) + a test that the caller's own claim survives |
| D4 blocked verdict | MEDIUM | Turns a self-healing state into a human-gated one; blocks `start-feature` (`policy.rs:296-304`) and feeds `bee feedback` (`feedback.rs:1045`) | Test: swept cell is `blocked` with `trace.blocked_reason` naming session + worktree; `cells reopen` clears it |
| Residual: session B sweeps live session A | MEDIUM, accepted | Heartbeat is the only liveness signal and a >900s single command outruns it | Named at the gate; no code proof — mitigation out of scope by decision |
| `bee orient` gains a write | LOW | orient already appends `.bee/logs/timings.jsonl` (`verbs/mod.rs:99-107`, called `orient.rs:499`); it is not read-only in fact | Test: orient sweeps an expired claim, leaves a live one alone, and `bee status` releases nothing |
| `orient` registry contract text | LOW | Says "Read-only"; no regen chain, guarded directory | The corrected text present and `tests/registry_dispatch.rs` green |
| Session `status: "dead"` write (slice 2) | LOW | Additive fields under the existing `sessions` lock | Test: stale session marked, live session not, no record deleted |

## Shape

**Feature outcome.** A session that dies mid-work stops holding its claims
hostage: an orchestrator running `bee orient`, or anyone running
`bee recovery scan`, releases that session's expired claims, parks each held
cell it can reach as `blocked` naming where the half-finished work sits, names
the ones it cannot reach, and marks the dead session's record — while the
sweeping session's own claims are never touched.

**Repo-reality basis.** The reaping machinery is correct and shipped
(`claims.rs:34,90-125,598-604`; `handlers_select.rs:56-131`); what is missing
is reach (one call site, `:519`), a safe caller contract, a store-boundary
rule, and a verdict that admits half-finished work exists.

| Epic | Capability / Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| **E1** | Sweep safety, store boundary, verdict | The sweep can take a live session's claim (`handlers_select.rs:62`) and can write a cell into the wrong store (`:174-182`); D4 makes each wrong take cost a human | 1 | Caller's own claim survives; reachable cell becomes `blocked` with `trace.blocked_reason` naming session + worktree; unreachable cell is named, not written |
| **E2** | The second call site | D1 — `orient` is the ritual every routing/resuming orchestrator runs (AGENTS.md); today only `claim-next` sweeps | 1 | `orient` sweeps an expired claim, leaves a live one alone, declines when the caller is unresolvable (D6); `bee status` releases nothing; registry text corrected |
| **E3** | Dead session is a stored fact | D2 — deadness is re-derived from heartbeat age at every call site, never recorded | 2 | Stale session gains `status: "dead"` + `dead_at`; live session unchanged; no record deleted |
| **E4** | The releasing door | D3 — `recovery scan` is advertised and refuses; detection already exists (`status_full/recovery.rs:264-466`) | 2 | `recovery scan` releases qualifying claims; `recovery window` still refuses; `registry_dispatch.rs` green; its "side-effect-free" description corrected |

**Slice queue.**

- **Slice 1 (current)** — E1 then E2. Walking skeleton: end-to-end, real
  behavior, no stubs — a crashed session's expired claim is released by
  `bee orient`, its cell parked `blocked` naming the worktree when reachable
  and named-but-untouched when not, while the caller's own claim survives.
  E2 depends on E1 (it calls the new signature).
- **Slice 2** — E3 and E4, independent of each other, both depending on slice 1.
  Headlines only, no cells yet: *mark a heartbeat-stale session dead in place*;
  *serve `recovery scan` from a new `recovery` verb group, flipping only its own
  registry marker and correcting its description*. Slice 2 also owns the
  `docs/handbook/register.md:304,384` declared-but-not-built rows.

**Current slice to prepare:** slice 1, two cells (E1, E2), E2 depending on E1.

**Cell-facing specifics the wave found missing** (these ride the cells, stated
here so the plan is the single source):

- The reason is written at `trace.blocked_reason`, the key `bee cells block`
  writes (`handlers_close.rs:782`) and `bee cells reopen` clears (`:914`) —
  a top-level key would be invisible to both and to `feedback.rs:1045`.
- The worktree in the reason comes from the claim's optional `workspace_id`
  (`claims.rs:692-711`) resolved through `verbs/workspace_store.rs:16,174` to
  its `root`. When the claim carries no `workspace_id`, or the workspace record
  is missing, the reason says so explicitly rather than omitting the clause.
- Self-exclusion is by claim `session` field equality against the resolved
  caller id, applied before the `.adopting` gate acquisition — the existing
  heartbeat-then-gate order is pinned by `tests.rs:2266-2271`.
- The sweep's decision-row rationale currently ends "…returned to open rather
  than left claimed-but-unclaimable forever" (`handlers_select.rs:120-127`);
  that sentence becomes false under D4 and is rewritten with the verdict.
- Stale doc comments to correct: `verbs/cells/mod.rs:43-44`,
  `handlers_select.rs:34,60,170`.
- Error-type bridge: `cells` uses `MR<T> = Result<T, Fail>` (`cells/util.rs:36`),
  `status_full` uses `R<T> = Result<T, Ex>` (`status_full/mod.rs:157`) — E2 maps,
  it does not unify them.

## Test matrix

High-risk — the 12 dimensions. Each cell's writer judges existing coverage
first (`verbs/cells/tests.rs` holds `sweep_resets_only_the_claim_it_actually_removed`
at :2004, `sweep_of_a_sessionless_claim_names_none_in_its_decision_row` at
:2084, `sweep_skips_a_gated_claim_and_leaks_no_gate_file` at :2219) and authors
only the gap.

| # | Dimension | Applies | Probe |
|---|---|---|---|
| 1 | User types | Yes | Caller with a session id; caller with none (D6 — declines, writes nothing); claim whose own `session` is null (sessionless path pinned at tests.rs:2084 — reason text must state "sessionless") |
| 2 | Input extremes | Yes | Claim with `ttl_seconds` absent, zero, negative, non-finite — `claim_expired` (`claims.rs:90-102`) keeps treating these as never-expiring |
| 3 | Timing | Yes | Claim exactly at TTL boundary; heartbeat exactly at 900s; claim expired + heartbeat fresh (must NOT sweep); heartbeat stale + TTL alive (must NOT sweep) |
| 4 | Scale | Partial | Sweep over many claim files — one unreadable claim must not abort the pass |
| 5 | State transitions | Yes | `claimed -> blocked` is the only transition the sweep may make; a cell already `blocked`, `open`, `capped`, or `dropped` is untouched (`handlers_select.rs:187` gate) |
| 6 | Environment | **Yes, primary** | Sweep where the cell is absent from the sweeper's store (D5): claim removed, no cell written anywhere, cell id + worktree named in output and decision row. Sweep from a granted worktree vs from main, both asserted |
| 7 | Error cascades | Yes | Corrupt claim JSON, corrupt session record, missing workspace record, missing `.adopting` gate file — each skips loudly and leaks no gate file (tests.rs:2219) |
| 8 | Authorization | Yes | The caller's own claim is never swept even when TTL and heartbeat both say stale — the E1 contract |
| 9 | Data integrity | Yes | No session record deleted (D2); the decision row carries the new `claimed -> blocked` wording and the rewritten rationale, not the old text (tests.rs:2074); `bee cells reopen` clears `trace.blocked_reason` written by the sweep |
| 10 | Integration | Yes | After a sweep: `bee state start-feature` refuses naming the blocked cell (`policy.rs:296-304`); `cells ready`/`claim-next` do not offer it (`:410`); `bee feedback` counts it once (`feedback.rs:1045`); `bee close` reports its archive held (`handlers_meta.rs:867-899`) |
| 11 | Compliance | No | No regulatory surface |
| 12 | Business logic | Yes | `bee orient` sweeps; `bee orient --json` sweeps; `bee status` and `bee status --lanes-full` release nothing (D1) — pinned by test, since one `run()` body serves both verbs (`orient.rs:454-459`) |

Declared suite at every cap:
`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`.

## Out of scope

- Any storage-engine change. Cells, decisions, and backlog stay git-tracked
  JSON — CONTEXT.md "Deferred Ideas" and
  `docs/decisions/0024-harness-cross-pollination-analysis.md:128`.
- Cross-store cell writes via the grant registry — forbidden by D5.
- Closing the >900s heartbeat gap. Accepted residual risk.
- Pid-liveness on session records — needs a stored pid neither claims nor
  sessions carry.
- `bee recovery window` — stays unbuilt per D3, marker stays.
- Folding `bee reservations sweep` into the same pass — `sweep_exec`
  (`verbs/reservations/release.rs:250`) stays callable in-crate for a later
  feature.
- Any change to `src/lock.rs` or the claim fence-epoch mechanism
  (`claims.rs:339-393`).
