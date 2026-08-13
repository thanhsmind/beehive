# Sweep At Every Door — Context

**Feature slug:** sweep-at-every-door
**Date:** 2026-08-13
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | ORGANIZE

## Feature Boundary

A session that dies mid-work stops holding its work hostage: its expired claims are
released by a sweep that now also runs on `bee orient` and on a newly built
`bee recovery scan`, its session record is marked dead in place, and each cell it
was holding is parked as `blocked` for a human rather than silently reopened. The
feature ends at claim/reservation/session bookkeeping — it does not touch the
storage format, the lock protocol, or what a live session may do.

## Feature Origin

The reported symptom: on a project running several agents across several worktrees,
task status appears to be "forgotten" — a cell stays claimed with nobody working it.

The scout found the reaping machinery already exists and is correct:
`DEFAULT_CLAIM_TTL_SECONDS = 3600` (`packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:34`),
a self-renewing heartbeat throttled to 60s (`hooks/state_sync.rs:32`), and
`sweep_expired_claims` which removes a claim only when the claim's TTL has expired
**and** the owning session's heartbeat is stale
(`verbs/cells/handlers_select.rs:56-131`, `HEARTBEAT_STALE_SECONDS = 900`).

What is missing is not a mechanism but its reach:

- `sweep_expired_claims` is called from exactly one place — `bee cells claim-next`
  (`verbs/cells/handlers_select.rs:519`). No other verb, hook, or timer calls it.
- `bee recovery scan` / `bee recovery window` are declared but not built
  (`docs/handbook/register.md:304,384`). The working detection code lives inside
  `bee status --full` (`verbs/status_full/recovery.rs:264-466`) and only reports.
- `bee reservations sweep` has real TTL logic (`verbs/reservations/leases.rs:315-323`)
  but no automatic trigger.
- Session records are never reaped. `is_pid_alive` exists (`src/lock.rs:158-194`,
  `hooks/prompt_context.rs:1278-1310`) but is applied to lock files only, never to
  a session record.

So the fix is wiring, not a new storage engine. A SQLite migration was considered
and rejected in the same conversation — cells, decisions, and backlog are git-tracked
(`git ls-files .bee` = 370 files) and must stay mergeable across worktrees; this
matches `docs/decisions/0024-harness-cross-pollination-analysis.md:128`.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The sweep gains exactly one new door: `bee orient`. `bee cells claim-next` keeps its existing sweep. `bee status --full` stays report-only — `detect_crash_candidates` keeps describing crashed sessions without releasing anything. No all-verb preflight sweep is added. | `orient` is the mandated ritual for routing, starting, or resuming work (AGENTS.md), so every session that could pick up freed work already runs it. Keeps a status read from mutating another session's state, and keeps read verbs read-shaped. Decision `e642a9ad`. |
| D2 | A heartbeat-stale session record is marked dead in place — `.bee/sessions/<id>.json` gains `status: "dead"` and `dead_at` — never deleted. No second hard-stale deletion tier. | The transcript pointer and lane on a dead session record are what recovery mines for unsettled work; deletion destroys that evidence. Deadness becomes a stored fact rather than a value re-derived from heartbeat age at every call site. Decision `5f8779d2`. |
| D3 | `bee recovery scan` is built as a releasing door: it releases every qualifying crashed-session claim on invocation. No `--release` flag, no confirmation step. `bee recovery window` stays unbuilt. | The qualifying criteria are already conservative (claim TTL expired AND owning session heartbeat-stale AND transcript lacks a clean-end trio), so a qualifying candidate is one no live session can still be working. A second gesture would reintroduce the "nobody happened to run it" failure this feature exists to remove. Decision `501fa7c5`. |
| D4 | A cell whose claim is swept goes to `status: "blocked"` with a reason naming the dead session and its worktree — not back to `open`. This replaces `sweep_reset_cell`'s current `claimed -> open` reset. | A crashed session may have left half-written code in its worktree; reopening the cell invites the next agent to redo the work blind. Accepted cost: swept cells no longer flow back into `claim-next` automatically — the sweep frees the claim slot, a human frees the work. Decision `f2405c31`. |
| D5 | The sweep never writes across a store boundary. It always removes the qualifying claim; it parks the cell `blocked` (D4) only when the cell record is readable in the sweeping process's own store. Otherwise it reports the cell id and the holding worktree in its output and decision row and leaves the cell untouched — claim freed, cell still `claimed` until a human runs `bee cells reopen`. | Claims are control-plane and live on main, but a granted worktree keeps its own `.bee/cells`, so a claim on main can point at a cell in another store. `sweep_reset_cell` writes at `cells_dir(control)` (`handlers_select.rs:174-182`), and `roots.rs:157-162` names writing across that split as writing the right bytes into the wrong store — the reason `rsv::prelude` refuses granted worktrees (`emit.rs:38-46`). Decision `6b083af6`. |
| D6 | A sweep door that cannot resolve its own caller session does not sweep. `bee orient` resolves the caller from `BEE_SESSION_ID`/`CLAUDE_CODE_SESSION_ID`, then from `resolve_session_adopt`; when both fail it reports the count of expired claims detected and writes nothing. | Self-exclusion is inert without a caller identity, and `resolve_session_adopt` returns `None` exactly when several live sessions exist — the multi-agent case this feature targets. `bee orient` takes no `--session-id` today (`orient.rs:441-449`) and gains none. Decision `03ff5279`. |

### Agent's Discretion

- The exact wording of the `blocked_reason` string (D4), provided it names the dead
  session id and the worktree path where its work may sit.
- Where the shared sweep entry point lives and how `orient` and `recovery scan` both
  reach it, provided neither duplicates the criteria.
- The shape of the `status: "dead"` write in D2 (which existing store helper, which
  lock name), provided it holds the `sessions` lock as the existing writers do.
- Whether `bee reservations sweep`'s existing TTL sweep is folded into the same pass
  as the claim sweep or stays a separate call — both are correct, planning picks on
  the evidence.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| door | A command whose normal invocation triggers the sweep. `claim-next` is the existing door. D1 adds `orient` to the *existing* sweep; D3 separately builds `recovery scan`, a new command that is a door by construction. "Exactly one new door" in D1 means one new call site on the existing sweep, not one door in the feature. |
| sweep | The pass that releases expired claims and, per D4, parks their cells. Distinct from *reap*. |
| reap | Marking a heartbeat-stale session record dead (D2). Never a deletion. |
| dead session | A session whose `last_heartbeat` is older than `HEARTBEAT_STALE_SECONDS` (900s). Not a pid-liveness claim — no session pid is ever checked. |
| qualifying candidate | A claim that satisfies all three release criteria: TTL expired, owning session heartbeat-stale, transcript lacking a clean-end trio. |

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_select.rs:56-131` —
  `sweep_expired_claims`, the whole existing sweep including the `.adopting` gate
  re-verification and the `sessions` store lock. The new doors call this, they do
  not reimplement it.
- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_select.rs:174-222` —
  `sweep_reset_cell`, the function D4 changes.
- `packages/bee-rs/crates/bee/src/verbs/status_full/recovery.rs:264-466` —
  `build_recovery_block` / `detect_crash_candidates` / `has_clean_end_trio`. The
  detection half of `recovery scan` already exists here and is reused, not rewritten.
- `packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:90-125` — `claim_expired`,
  `claim_active`, `claim_expiry`; `:598-604` — `heartbeat_stale`.
- `packages/bee-rs/crates/bee/src/verbs/reservations/release.rs:214-308` —
  `run_sweep` / `sweep_exec`, the reservation-side sweep.

### Established Patterns

- Lock-then-read-modify-write-atomic under a named lock (`src/lock.rs`,
  `src/fsutil.rs:138-149`). Every store write in this feature follows it; there is
  no CAS-on-rev anywhere in bee and this feature does not introduce one.
- Opportunistic sweep on acquisition — `integration_queue.rs:376` calls
  `lease_store::sweep_expired_leases` on every `try_become_processor`. D1's
  `orient` door is the same pattern applied to claims.
- Fence epochs on ownership transfer (`claims.rs:339-393`, `adopt_claim`).

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:499` — where `orient`'s
  run path already sits; the D1 door attaches here.
- `packages/bee-rs/crates/bee/src/router.rs` — a new `recovery` verb group needs its
  registration door (D3).
- `packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs:113-151, 230-275` —
  session record read/write, where D2's `status: "dead"` write lands.
- `docs/handbook/register.md:304,384` — the two declared-but-not-built rows;
  `recovery scan` moves out of that list, `recovery window` stays in it.

## Canonical References

- `docs/knowledge/areas/workflow-state/index.md` — the area this feature changes;
  concepts `sessions-lanes-and-identity` (line 22) and `recovery` (line 25).
- `docs/decisions/0024-harness-cross-pollination-analysis.md:128,133` — the standing
  rejection of a wholesale SQLite substrate, and the open question about a derived
  index that this feature deliberately does not answer.

## Outstanding Questions

### Answered In Planning

All four are resolved with evidence in `plan.md` ("Discovery"). Summary:

- [x] **Worker sessions never run `bee orient`** — `.bee/bin/prompts/worker-cell.md`
  goes from the pre-claimed cell straight to `bee cells finish`, and the SessionStart
  preamble is built by `build_session_preamble`
  (`hooks/session_preamble/budget.rs:448`), not by orient's code path. D1 therefore
  covers orchestrator sessions only.
- [x] **No, the sweep has no self-exclusion** —
  `sweep_expired_claims(control: &Path, now: f64)` (`handlers_select.rs:62`) takes no
  caller id, and `claim-next` does not pass the one it holds at `:517`. Confirmed
  reachable: a single tool call running past 900s emits no `PostToolUse`, so a live
  session ages past `HEARTBEAT_STALE_SECONDS`. D6 and the E1 epic answer this.
- [x] **`blocked` is nonterminal** — `policy.rs:296-304` and `:473-491` count
  `open|claimed|blocked` alike, so a swept cell refuses `bee state start-feature`;
  `bee close` does not refuse but holds its auto-archive
  (`handlers_meta.rs:867-899`). The escape is `bee cells reopen --id <id> --reason <r>`
  (`handlers_close.rs:874-924`), no special flag.
- [x] **Reservations sweep keeps its own door this round** — `sweep_exec`
  (`verbs/reservations/release.rs:250`) is already callable in-crate, so folding it in
  stays available to a later feature at no cost today.

## Deferred Ideas

Out-of-scope ideas captured during shaping. Not lost, not planned.

- Migrating cells/claims to SQLite — rejected this round: `.bee/cells/**`,
  `.bee/decisions.jsonl`, and `.bee/backlog.jsonl` are git-tracked and must stay
  git-mergeable across worktrees.
- A derived, gitignored SQLite index (`.bee/cache/index.db`) rebuilt from the JSON
  by `bee state rebuild-projections`, for fast cross-worktree queries — the open
  question already recorded at `docs/decisions/0024...:133`. Not needed for this
  feature.
- Pid-liveness checks on session records (not just lock files) — a stronger death
  signal than heartbeat age, but it needs a stored pid that claims and sessions do
  not currently carry.
- `bee recovery window` — stays unbuilt per D3.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
