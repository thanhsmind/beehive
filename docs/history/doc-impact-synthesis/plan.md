# doc-impact-synthesis — plan

**Lane:** standard · **Route flags:** data-model (decisions event gains
`feature`), covered-contract-change (close doors, decisions log) ·
**Files:** ~10 product

## Goal

Impact-driven doc maintenance on the phase-1 substrate: touched decisions
find their citing docs and force the fix (D1), locked CONTEXT decisions
synthesize into area specs (D2), deferral prose in docs joins the trigger
registry (D3), then one bounded backfill proves the doors (D4).

## Shape — slice 1: 3 sequential cells (kds-1 → kds-2 → kds-3)

Sequential by named reason: kds-2 needs kds-1's `feature` field; kds-2 and
kds-3 both edit `drivers/close.rs`.

### kds-1 — log-time impact walk + feature linkage (D1a)

Files: `verbs/decisions/verbs_read.rs`, `verbs/decisions/tests.rs`
(+ read-only reuse of `render.rs::sweep_decision_citations`).

- After the event append (seam: `verbs_read.rs:637`, `touches` full ids
  and the new event id in scope), run
  `sweep_decision_citations(root, id, short8)` for EACH resolved
  `touches:` id; every hit enqueues a capture stub via the existing
  `add_capture_stub` shape (`supersede.rs:56-86`) with
  `source: "touches-sweep"`, outcome naming file:line, the touched id,
  and the new deciding id. Supersede's own sweep stays untouched.
- Stamp `feature` on every new `decide` event: derived from the calling
  context's active feature (same session-bound-lane-else-default
  resolution `state route` uses); absent resolution writes no field.
  Readers tolerate absence (legacy lines unchanged).
- Tests: touches log enqueues one stub per citing file; no touches → no
  sweep; `feature` field present when a feature is bound, absent
  otherwise; supersede sweep unchanged.

### kds-2 — impact door at close (D1b)

Files: `verbs/drivers/close.rs`, `verbs/drivers/tests.rs`.

- `build_impact_door`: collect the closing feature's decision ids —
  structured `feature` field first (kds-1), plus the
  `best_scribing_stamp_ms`-windowed fallback for pre-kds-1 events
  (canonical ledger copy, `state_group/ledger.rs:94-128`) — sweep
  citations for each (`sweep_decision_citations`), and for each hit file
  check the capture queue: a hit not covered by a flushed stub naming
  that file blocks close, detail naming file:line and the remedy
  (fix the doc, flush the stub). Blocking, with the same recorded-
  deferral escape shape as the freshness door.
- Door slots after knowledge-freshness in the exit chain and all three
  doors vecs.
- Tests: unfixed citation in a doc → close refuses naming the file;
  flushed stub covering it → clear; deferral decision → demoted with
  reason; no decisions with the feature → clear.

### kds-3 — routing + doc-deferral doors (D2, D3)

Files: `verbs/drivers/close.rs`, `verbs/drivers/tests.rs`, plus a small
CONTEXT-table parser module (`verbs/drivers/context_table.rs` or inline).

- **Routing door (D2)**: parse the closing feature's
  `docs/history/<feature>/CONTEXT.md` "Locked Decisions" CANONICAL pipe
  table (the bee-shaping template grammar: `## Locked Decisions` heading,
  `| ID | Decision |` header, `| D<n> |` rows). Recon: historical files
  diverge (bullet lists, split sub-tables) — an unparseable table is a
  LOUD door detail ("table not canonical — route manually and record"),
  blocking until a recorded routing decision covers the feature; never a
  silent pass, never a guess. For each parsed D-ID, routed means: the
  bundle carries a citation token `<feature-slug> D<n>` or the D-ID row's
  logged short8 in any area file (body or frontmatter `decisions` list),
  OR a logged decision tagged `feature-local` names `<feature-slug> D<n>`.
  Unrouted D-IDs block, detail listing them with both remedies.
- **Doc-deferral door (D3)**: over the closing feature's touched doc
  files (`feature_touched_files`, `close.rs:873-892`, filtered to
  `docs/`), scan only the ADDED lines of each file's diff vs the
  feature's base (git diff — added lines only, recon question resolved:
  historical quotes never trip it). Added lines matching
  `matches_deferral_prose` (kdt-3's matcher, exported for reuse) without
  a trigger citation (backtick trigger id or `[[trigger:<id>]]`) that
  resolves via `trigger_registered()` block close with the
  create-the-trigger teach line.
- Both doors: same deferral-decision escape shape; slot after impact
  door.
- Tests per door: canonical table parses and routes; bullet-list CONTEXT
  → loud unparseable detail; unrouted D-ID blocks; feature-local tag
  clears; added deferral line without trigger blocks, with registered
  trigger clears; removed/context diff lines never trip.

## Slice 2 — bounded backfill (D4), headline only, cells cut after slice 1

- kds-4: run the three new doors' checks over the LIVE areas list from
  the 2026-08-16 audit; fix what they surface (route unrouted decisions
  of the current-generation features, register doc-borne conditions,
  reconcile remaining semantic staleness); file the 110-CONTEXT
  historical routing sweep as a standalone backlog campaign row. Proof:
  a fresh close-shaped dry run over a probe feature reports all three
  doors clear.

## Verify

`commands.test`: `PATH="$HOME/.cargo/bin:$PATH" cargo test --release
--manifest-path packages/bee-rs/Cargo.toml` at every cap. kds-3 keeps the
regen chain clean if any instruction surface changes.

## Cost if wrong

- kds-1 additive (stub enqueue + optional field); rollback = skip sweep.
- kds-2/kds-3 doors can over-block: every door carries the recorded-
  deferral escape and a one-bool demotion rollback, same as phase 1.
- The CONTEXT-table strictness is deliberate: an unparseable table
  blocking close is the mechanism teaching the canonical grammar
  forward — named cost, not an accident.
