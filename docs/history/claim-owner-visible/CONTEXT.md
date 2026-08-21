# claim-owner-visible — locked context

## The defect

`bee cells list` and `bee cells show` emit the raw `.bee/cells/<id>.json`
record. That record carries `status: "claimed"` but its `claimed_by`,
`claimed_at`, and `session` fields are `null` — always, by construction.
The owner of a claim lives only in a different file,
`.bee/claims/<id>.json` (`session`, `workspace_id`, `claimed_at`,
`ttl_seconds`, `fence_epoch`).

An agent that surveys cells therefore reads a LIVE sibling claim as an
idle cell. Observed 2026-08-21: this session read `srg-1`/`srg-2` as
`status=claimed` with `claimed_by=null`, concluded they were free, and
offered the user to run them. Both were held by live session `2a7f3cd2`
bound to lane `store-reach-gaps`, heartbeat 30 seconds old.

## Why the existing guards did not catch it

The hard guard is real but sits one step too late.
`claim_cell_file` (`packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:671`)
opens the claim file with `create_new(true)` and refuses `CLAIMED` naming
the owner session. It fires only when an agent actually CLAIMS. The read
that precedes the claim is blind, and nothing at all fires when the agent
merely REPORTS to the user what it read.

`AGENTS.md:132-133` covers the same ground in prose ("Pick up cross-session
work with `bee cells claim-next`, never by browsing for open cells"), but
prose does not fire on a read.

A `claimed_by` field exists in this codebase only at
`packages/bee-rs/crates/bee/src/verbs/discovery.rs:79`, for discovery
tickets — never for cells.

## D1 — the fix is a derived annotation, not a stored field

`bee cells list` and `bee cells show` join `.bee/claims/<id>.json` and the
holding session's record into the emitted output as a DERIVED `claim`
object. The cell record on disk is never changed: claim ownership already
has exactly one home (`.bee/claims/`), and a second copy inside the cell
file would be a second source of truth that drifts the moment a claim is
swept or adopted.

Precedent: `with_verify_owner`
(`packages/bee-rs/crates/bee/src/verbs/cells/mod.rs:339`) already injects a
derived `verify_owner` annotation into `cells show` output. This follows
that shape.

## D2 — the annotation appears ONLY when a claim file exists

`packages/bee-rs/crates/bee/src/verbs/cells/tests.rs:296` asserts the exact
key list of `cells show` output
(`["id","title","status","verify","verify_owner","trace"]`). Annotating
unconditionally would break it and every other exact-shape reader.

So: a cell with no `.bee/claims/<id>.json` emits byte-identical output to
today. Only a cell that actually has a claim file gains the `claim` key.
This also makes the annotation self-limiting — it can only ever appear
where there is real claim data to report.

## D3 — the annotation reports liveness, not just identity

Naming the holder is not enough. A claim held by a dead session is
sweepable and effectively free; a claim held by a live session is
untouchable. The two must not read the same. The annotation carries the
holder AND the verdict, using the crate's own existing liveness reading —
`heartbeat_stale` (`claims.rs:601`: a `closed`/`dead` status, an
unparseable `last_heartbeat`, or a heartbeat older than
`HEARTBEAT_STALE_SECONDS` all read as stale) and `claim_expired`
(`claims.rs:90`, the TTL reading). No new liveness rule is invented here.

## D4 — the text line names the holder too

`summarize_cell` (`mod.rs:356`) renders the one-line-per-cell text of
`cells list` and `cells ready`. A human or agent reading the text output —
not the JSON — is exactly the reader that was misled. The claimed line
gains the holder and the verdict; every other status renders exactly as
today.

## Non-goals

- No new CLI flag. This is an output-shape change only, so
  `packages/bee-rs/crates/bee/src/generated/registry_payload.json` is not
  touched (that file declares flags).
- No change to claim/sweep/adopt behavior. Reading only.
- The `claimed_by`/`session` fields on the cell record stay null. They are
  not the home of this data and are not being promoted to it.

## Files

- `packages/bee-rs/crates/bee/src/verbs/cells/mod.rs` — the annotation and
  the text line.
- `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs` — its tests.

No overlap with the two in-flight lanes: `store-reach-gaps` holds
`verbs/state_group/workflows.rs`, `generated/registry_payload.json`,
`tests/registry_contracts.rs` (srg-1) and `verbs/worktree/registry.rs`
(srg-2).
