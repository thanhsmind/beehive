---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset until approval>
---

# Plan: Awaiting Human

Mode: `standard` — 2 risk flags: public-contracts, multi-domain
Why this is the least workflow that protects the work: one field, one setter, three clearing paths, and one read surface — but the field changes the meaning of `run_state`, which shipped days ago and is already read by status, orient, and the preamble, so the census of those readers is the part that needs care.

## Requirements (from CONTEXT.md)

- **D1** — `awaiting-approval` covers every moment the agent waits on the human, not only a pending gate. ONE state, plus a field naming what is waited on.
- **D2** — The mark ends three ways, all live: the `UserPromptSubmit` hook clears it on the human's next message; the agent may clear it explicitly; a mark whose session heartbeat is stale expires.
- **D3** — A question asked with no active feature still records the wait, on the default state record. Session-scoped first, feature-scoped only when a feature is live.
- **D4** — Stale expiry reuses the claims dual-condition rule: expiry alone never clears, the owning session's heartbeat must also be stale.
- **D5** — No dashboard. The persisted state and the CLI/JSON that exposes it.

## Discovery

- **The field to extend already exists.** `run_state` landed in `traceable-runs` cell trun-7 on the workflow record, with a closed vocabulary (`shaping`/`awaiting-approval`/`running`/`blocked`/`done`) and a typed refusal on an unknown value. `awaiting-approval` is currently derived from a `pending` gate entry alone — that derivation is what D1 widens.
- **The projection trap is documented and cost the last feature a named warning.** `apply_workflow_d1_fields` in `workflow_store/projections.rs` copies a FIXED field list into `.bee/state.json`. A new field omitted there never reaches the projection, and a byte-identical rebuild test still passes — vacuously. Any new field must be added there with a test asserting the projection carries it.
- **The clearing hook exists and already runs on every human message.** `hooks/prompt_context.rs` binds `UserPromptSubmit` and composes the phase/gate/next-action reminder in `build_prompt_reminder`. It is both D2's clearing point and the natural place to surface a live wait.
- **The staleness rule D4 reuses is proven and dual-condition.** `cells/claims.rs` reclaims only when the lease is expired AND the owner's heartbeat is stale (`DEFAULT_CLAIM_TTL_SECONDS`, `HEARTBEAT_STALE_SECONDS`), with the caller's own session excluded first.
- **Census is the known failure mode here.** The scribing-debt reconciliation in `traceable-runs` needed three judge rounds because each pass found reader copies the previous had missed — the count went 3 → 5 → 6. Any change to what `run_state` means must enumerate its readers the same way, up front.

Evidence commands: `rg -n "run_state" packages/bee-rs/crates/bee/src`, `rg -n "apply_workflow_d1_fields" packages/bee-rs/crates/bee/src`, `rg -n "HEARTBEAT_STALE_SECONDS" packages/bee-rs/crates/bee/src`.

## Approach

**Recommended path.** Add a `waiting_on` structure beside `run_state` rather than encoding the subject into the state string: the state stays a closed five-value vocabulary that existing readers already branch on, and the detail rides a sibling field that only new readers consult. `run_state` then reads `awaiting-approval` whenever a live `waiting_on` mark exists OR the existing pending-gate condition holds — one state, two sources, exactly D1. The mark carries what is being waited on, when it was asked, and which session owns it; the owner is what makes D4's heartbeat check possible and what lets a mark live on the default record when no feature does (D3).

**Rejected alternatives**
- A second `awaiting-answer` state — the user rejected it when choosing D1: two values for one condition forces every reader to check both.
- Encoding the subject inside the state string — turns a closed vocabulary into free text and breaks the typed refusal trun-7 built.
- Clearing only in the agent's hands — rejected as D2: a forgotten clear is a permanent lie, which is the failure this whole line of work removes.

**Risk map**

| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| Widening what `run_state` means | MEDIUM | Readers shipped days ago branch on it; a miscount is the exact failure that cost `traceable-runs` three judge rounds | An enumerated census of every `run_state` reader in the cell, each either updated or explicitly unaffected |
| Hook clearing (D2) | MEDIUM | The hook runs on every human message in three runtimes; a clear that throws would break the turn | A test that the clear is best-effort and never fails the hook; confirm `UserPromptSubmit` coverage per runtime in the hook manifests |
| Stale expiry (D4) | MEDIUM | Expiry that ignores the heartbeat silently erases a live wait during a long human pause | A test where the lease is expired but the heartbeat is fresh, asserting the mark survives |
| Session-scoped mark (D3) | LOW | The default record already exists and is written through the same lock discipline | A test that a mark set with no active feature is readable and clearable |
| Read surface (D5 boundary) | LOW | Additive JSON | Existing keys unchanged, asserted |

## Shape

**Phase plan.** One slice, three cells; the first is the walking skeleton and the other two are independent of each other.

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| ah-1 | The mark exists on the record, a verb sets it, `run_state` reads `awaiting-approval` from it, and the projection carries it | Nothing else can be built or observed until the mark is real | Set a mark with no feature active; `bee status --json` reports `awaiting-approval` and the subject | ah-2, ah-3 |
| ah-2 | The three clearing paths: the hook, the explicit verb, stale expiry | A mark that cannot end is worse than no mark | Send a message; the mark is gone | — |
| ah-3 | Read surface: status, orient, and the session preamble name the live wait | The state is worthless if no surface shows it | `bee orient` names what the run is waiting on | — |

**Current slice to prepare: all three.** ah-2 and ah-3 both depend on ah-1 and are independent of each other.

## Test matrix

Standard lane, so the triad at its smallest demonstrating size.

- **Happy path** — a mark is set, `run_state` reads `awaiting-approval`, `bee status --json` reports the subject, the human's next message clears it, `run_state` returns to what it was.
- **Edge cases** — a mark set with no active feature (D3); a lease expired but heartbeat fresh, so the mark SURVIVES (D4); two sources at once (a pending gate and a live mark) still yielding exactly one `awaiting-approval`; clearing a mark that does not exist is a no-op, not an error.
- **Error paths** — an unknown wait kind or an empty subject refuses with a typed error and writes nothing; a clear that fails inside the hook never fails the hook itself.

## Out of scope

- The dashboard (D5).
- Inferring an unmarked wait — the agent asking in prose and forgetting to mark it. Recorded as a deferred idea in CONTEXT.md; it needs intent inference at the session-stop hook, a different problem from recording a declared wait.
- Any change to the five `run_state` values or to the gate record fields shipped by `traceable-runs`.
