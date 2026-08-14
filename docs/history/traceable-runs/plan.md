---
artifact_contract: bee-plan/v1
mode: high-risk
# approved_gate2: <unset until approval>
---

# Plan: Traceable Runs

Mode: `high-risk` — 4 risk flags: public-contracts, covered-contract-change, multi-domain, data-model
Why this is the least workflow that protects the work: the change touches two shapes that 78 production call sites already read, and it rewrites the doctrine that decides when a human stops — so the shape has to prove it adds fields rather than replacing them, before a single reader is touched.

## Requirements (from CONTEXT.md)

- **D1** — Every file-touching request writes a shaped brief and enters an explicit `awaiting-approval` state on its feature/workflow record BEFORE any source edit, at every lane including `tiny` and `docs`.
- **D2** — `gate_bypass` no longer decides whether the record exists, only whether the run halts; an auto-approval writes the same record with `actor: "auto"` plus the bypass level and reason.
- **D3** — A gate becomes a record carrying `state` (`pending` | `approved` | `rejected`), `actor` (`user` | `auto`), and a timestamp. `pending` is persisted, not derived.
- **D4** — Cell and feature/workflow each get a real, persisted status vocabulary including an explicit waiting state. Stored, not computed at read time.
- **D5** — Deferred capture, scribing, review, and promote-proposal work become records in ONE claimable queue, each carrying feature, cells, areas, files, and reason, plus a claim/lease.
- **D6** — The mandatory flow is scoped to requests that write a file, code AND docs. A pure question creates no record.
- **D7** — The dashboard is out of scope; this feature delivers the persisted data and the CLI/JSON surface a dashboard reads.

## Discovery

Four parallel scans of the live tree, 2026-08-14.

- **The gap D1 names is real and has a precise cause.** `tiny`/`small` route straight to `bee-planning` (`skills/bee-hive/references/routing-and-contracts.md:33`), so `bee-shaping`'s Lock — the single writer of `CONTEXT.md` — never runs; the ceremony table (`routing-and-contracts.md:120-126`) lists Gate 1 as a human stop only for `standard`/`high-risk`.
- **The `docs` lane escapes the guard by accident, not by design.** The pre-execution-gate boundary already exists and already refuses source writes: `hooks/write_guard/checks.rs:493-509`, firing when `is_gated_phase(phase)` (`exploring` or `planning`, `paths.rs:469-471`) and `approved_gates.execution != true`. It is lifted by `GATE_ALLOWED_PREFIXES = [".bee/", "docs/", "plans/", "AGENTS.md"]` (`guards.rs:27`). The guard never reads `mode` at all — a docs-lane session writes freely purely because its paths start with `docs/`.
- **Migration cost is the dominant risk, and it is countable.** `approved_gates` + the workflow `gates` map: **37 distinct call sites** (18 writers / 15 branch-readers / 4 display-readers) across **133 non-test matches** plus ~12 test files; `GATE_NAMES` is duplicated three times (`state.rs:31`, `hooks/state_sync.rs:33`, `verbs/workflow_store/mod.rs:95`) and `default_gates()` six times, all on the projected-boolean side. Cell `status`: **41 distinct call sites** (9 writers / 22 branch-readers / 6 display-readers / 4 counters). Both shapes are additionally published as external contracts — `docs/02-architecture.md:138`, `docs/07-contracts.md:53,60,129`, `docs/handbook/register.md:48,365`, `skills/bee-hive/references/gates-and-delegation.md:53,99`, `skills/bee-swarming/references/worker-details.md:52`.
- **Every primitive D5 needs already exists.** `lock::acquire_store_lock` (`lock.rs`, `MAX_ATTEMPTS` ≈ 5 s, stale-takeover only for a provably-dead pid) and `resolve_session_id` (`leases.rs:566-596`) are reusable with zero new code. The O_EXCL claim protocol with a `.adopting` gate file and a `fence_epoch` CAS (`claims.rs:665-751`, `339-393`), the dual-condition stale sweep (TTL expired **AND** owner heartbeat stale, `handlers_select.rs:57-68`), and the append-then-fold event-sourcing pattern with a named lock (`backlog.rs:437,479-540`) are reusable as *patterns*; only the record schema, the JSONL file, and the fold function are new.

Evidence commands: `rg -n "approved_gates" packages/bee-rs/crates/bee/src`, `rg -n '"status"' packages/bee-rs/crates/bee/src/verbs/cells`, `rg -n "GATE_ALLOWED_PREFIXES" packages/bee-rs/crates/bee/src/hooks/write_guard`.

## Approach

**Recommended path — add fields, never replace them.** The persisted gate entry gains `state`, `actor`, `at`, `reason`, and `bypass_level` (per D2's "level and reason"); the entry's own `approved: bool` stays and is kept equal to `state == "approved"` (per D3).

Three existing writers author a gate entry WITHOUT going through `set_gate`, and each must be extended in the same slice or the two fields desync on their first use — the review found all three, with a repo test already pinning the mechanism (`verbs/workflow_store/tests.rs:72-75`, "A patch carrying only `approved` PRESERVES the base's rev stamp"):

- `gates_patch_from_record` (`verbs/workflow_store/handoff.rs:514-534`) — builds each entry from an empty map and inserts only `approved` (+ the rev stamp); `merge_gates` (`record.rs:89-117`) then overlays it onto the base, so a stale `state` survives while `approved` flips. This is the entry author, not `set_gate.rs`.
- `legacy_gates_to_workflow_gates` (`verbs/state_group/feature.rs:319-332`) — hardcodes `{approved, approved_for_plan_rev: null}`, dropping the new fields; it is the backfill used at `feature.rs:438` and `feature.rs:462`.
- `default_gate_entry` (`verbs/workflow_store/record.rs:69-74`) and the `default_gates()` resets at `verbs/state_group/policy.rs:390,506`.

**One divergence is intended and must not be "fixed".** The *projected* `state.json.approved_gates.<gate>` boolean — the one the write guard actually reads at `checks.rs:494-497` — is `is_approved && rev_effective` (`verbs/workflow_store/projections.rs:25-51`), so `bee state plan-rev bump` deliberately leaves the record `approved: true` while the projection reads `false`. The equivalence invariant below is scoped to the RECORD entry only; plan-rev staleness is an allowed, tested divergence at the projection. The workflow record gains a `run_state` field (per D4); the existing cell `status` string keeps its five values and gains only the transitions that are genuinely new. This is not a compromise on D3/D4 — the states are still **persisted, not derived**, which is the property the user chose; what stays derived is the legacy boolean the 37 readers already consume. It is also exactly the discipline this repo already enforces for `state.json` and lane files: the record wins and the projection is mechanically rebuilt from it (R65, `docs/knowledge/areas/workflow-state/workflow-records-and-projections.md`).

**Rejected alternatives**
- Replace the boolean and the status string at all 78 sites — buys no capability the additive shape lacks, and turns one feature into a repo-wide sweep with 12 test files rewritten.
- Compute run state at read time from existing fields — cheapest, but the user explicitly rejected it: `pending` must survive a restart.
- Give each deferred kind its own claim verb — keeps four vocabularies alive and makes a parallel drain agent learn four commands, which is the capability D5 exists to deliver.

**Risk map**

| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| Gate record shape (D3) | HIGH | Three writers author a gate entry outside `set_gate` and would desync it; a drifting projection silently mis-gates every write guard | Full declared suite green, plus a test proving the RECORD entry's `approved` always equals `state == "approved"` after every one of the four write paths — with the plan-rev projection divergence asserted as intended, not repaired |
| Write-guard docs hole (D6) | HIGH | `GATE_ALLOWED_PREFIXES` has THREE consumers (`checks.rs:498` gated phase, `checks.rs:483` idle intake, `hook_local.rs:548` worktree-first); editing it in place locks bee out of writing its own `CONTEXT.md` and `plan.md`, which `hooks/write_guard/tests.rs:1769-1780` already pins as allowed | Red-first test: a source write at a gated phase still refuses; `docs/history/<feature>/` and `.bee/` still pass; the existing docs-allowed test still passes |
| Brief-always doctrine (D1) | HIGH | Rewrites the routing and ceremony tables every session reads; a wrong edit misroutes all future work | `bee dev regen` green — verified real, `bee dev regen --help` prints the three-step chain render-skill-trees → onboard --apply → release-manifest --write; it is a source-checkout verb, not a `.bee/config.json` key — plus the CI drift check |
| `run_state` on the record (D4) | MEDIUM | New field on a record with a byte-identical rebuild invariant | Delete the projection, rebuild, assert byte-identical |
| Deferred queue store (D5) | MEDIUM | A new shared store means a new lost-update surface; the repo has a pattern for exactly this failure | Multi-process race test with a negative control, following `test_state_projection_race.mjs`'s shape |
| Read surface (D7 boundary) | LOW | Additive JSON output | Snapshot of the new command's JSON |

## Shape

**Feature outcome.** Any request that writes a file leaves a record a dashboard can read at every moment of its life: shaped, awaiting approval, approved by whom and when, running, blocked, done — with whatever it deferred sitting in one queue another agent can claim.

**Repo-reality basis.** The gate boundary, the claim protocol, the stale sweep, the event-sourced fold, and the store lock all already exist and are cited above; this feature adds three fields, one store, one guard condition, and one doctrine rewrite.

| Epic | Capability / Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| E1 | Persisted approval state (D2, D3) | Nothing today can say "awaiting approval"; a gate is a bool and a refusal is a bare timestamp | S1 | Suite green + projection-equivalence test |
| E2 | Brief on every lane (D1, D6) | `tiny`/`small` skip shaping entirely, and `docs/` walks through the guard | S2 | Red-first guard test + `bee dev regen` green |
| E3 | Run-state vocabulary (D4) | Cell and feature cannot name waiting, and a dashboard should read one field | S3 | Projection rebuild byte-identical |
| E4 | One claimable deferred queue (D5) | Two of the four deferred kinds are derived scans with nothing to claim and no payload | S4 | Multi-process claim race with a negative control |
| E5 | Dashboard read surface (D7 boundary) | The data is worthless if no command exposes it | S1 (thin) + S3 (full) | JSON output test |

**Slice queue**

- **S1 — walking skeleton (current slice).** `bee state start-feature` itself writes every gate as `state: "pending"` (`policy.rs:390,506` — the natural automatic attachment point), an actor approves one through `bee gate` with the new `--actor`/`--bypass-level`/`--reason` flags, and one read command prints the entry. A real run enters `awaiting-approval` without anyone typing it; the approval and the read-back are operator-driven. End to end, no stubs. Depends on nothing.
- **S2 — brief on every lane.** Doctrine rewrite, plus SPLITTING `GATE_ALLOWED_PREFIXES` into a gated-phase list and an intake list rather than editing the shared constant — the gated-phase list keeps `docs/history/<feature>/` and `.bee/`, so bee can still write its own brief and plan. Depends on S1.
- **S3 — full run-state vocabulary + the complete read surface.** Must extend `apply_workflow_d1_fields` (`verbs/workflow_store/projections.rs:103-113`, which copies exactly `phase`, `feature`, `mode`, `approved_gates`, `summary`, `next_action`) or `run_state` never reaches `.bee/state.json` and the byte-identical rebuild proof passes vacuously. Depends on S1.
- **S4 — the unified claimable deferred queue.** Depends on nothing in S1–S3; may run in parallel with S2/S3 once S1 lands.

**Current slice to prepare: S1 only.** Later slices stay as the headlines above, not cells.

## Test matrix

High-risk, so the applicable dimensions of `edge-dimensions.md`, each mapped to a cell truth. Every cell's writer judges existing coverage first (`verbs/state_group/tests.rs`, `verbs/workflow_store/tests.rs`, `hooks/write_guard/tests.rs` already carry gate fixtures) and authors only the gap.

- **5 State transitions** — every gate transition: unset → pending → approved; unset → pending → rejected; pending → pending (re-ask, idempotent); approved → rejected (the existing revocation path, which must keep stamping `gate_revoked_at`); rejected → approved. The impossible combinations refuse rather than silently overwrite.
- **9 Data integrity** — after each of the four entry-authoring paths (`set_gate`, `gates_patch_from_record`, `legacy_gates_to_workflow_gates`, `default_gate_entry`), the RECORD entry's `approved` equals `state == "approved"`; a record written by the old shape (no `state` field) still reads correctly and derives `state` from the boolean, so an existing repo never mis-gates on upgrade. Separately asserted as INTENDED: after `plan-rev bump`, the record stays `approved: true` while the projected boolean reads `false`.
- **3 Timing** — a session that dies between "pending" and the approval leaves a record that is still readable and still `pending`, never a half-written entry.
- **6 Environment** — a repo with zero workflow records (the standing C1 fallback, R65) behaves exactly as before.
- **2 Input extremes** — an empty or unknown `actor`, and an unknown `state` value, refuse loudly with a typed error rather than defaulting silently, matching the existing `WORKFLOW_MISSING`/`WORKFLOW_CORRUPT` discipline.
- **7 Error cascades** — a gate write that fails mid-way leaves neither the record nor the projection changed.

## Out of scope

- The dashboard itself (D7). This feature ships its data source and the JSON to read it.
- Retro-filling run records for already-closed features — the evidence no longer exists.
- Replacing the legacy `approved_gates` boolean or the five cell-status strings at their 78 call sites. They stay, projected from the new fields.
- Migrating `.bee/capture-queue.jsonl` and `.bee/review-candidates.jsonl` into the new queue. S4 decides whether the queue absorbs or wraps them; this plan does not pre-commit that.
