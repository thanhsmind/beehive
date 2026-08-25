---
type: bee.area
title: Doctrine Layer — model roles, fall-through, and escalation
description: "How work says which model should run it: an open set of job-named roles resolved through one parser, an ordered fall-through that warns instead of failing, cost held apart as an explicit escalation flag, and an explicit-only retry chain that never fires on a semantic failure."
timestamp: 2026-08-25
bee:
  id: doctrine-layer-model-roles-and-escalation
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [areas/doctrine-layer/overview.md]
  decisions: [model-role-split D1/D2/D3/D4/D5/D6/D8/D9/D10/D11/D12, escalate-off-disarm D1/D2, role-surface-cleanup D1, role-edge-hardening D1]
  sources: ["model-role-split (docs/history/model-role-split/CONTEXT.md, 34 cells, merged 2026-08-25)", "docs/discovery/model-role-split/MAP.md", "docs/history/research/oh-my-pi-model-roles-distill.md", "docs/history/model-role-split/reports/review-r2.md"]
  authoritative_for: "doctrine-layer: how a unit of work selects the model that runs it"
---

# Doctrine Layer — Model Roles, Fall-through, and Escalation

The resource being protected is the operator's ability to put the right model on
the right job. Before this concept, the selector was a three-value cost enum —
`extraction`, `generation`, `ceiling` — and cost words cannot express a job.
Nobody can configure "the model that is good at generation", because generation
is not a thing a model is good at. Real model strengths are job-shaped: one
model plans well, another tests well, another designs, another writes code. This
concept owns the vocabulary that replaced the cost enum, the resolution rules
that keep an open set safe, and the separate lever cost moved onto.

## Behaviors & Operations

**B1 — One parser owns the `models.<runtime>` shape** (model-role-split D1,
store `cd72ec97`). A single `resolve_role` in the drivers module reads the
config; the model guard calls through to it rather than carrying a second
implementation, and `resolve_advisor` collapses the same way. Why it is first
and not last: every other rule here writes into the shape this parser reads, so
a second copy would have to be edited in lockstep four more times. The evidence
that it must be structural rather than disciplined: the two guard tier lists had
already drifted four entries against five with nothing intending it.

**B2 — Roles are an open, fall-through set** (model-role-split D2, store
`06e49368`). A consumer names an ordered *list* of role names. An unset or
unresolvable name yields to the next; the last entry is always a name bee
resolves, so no existing host changes model when the feature lands. Any name
present in `models.<runtime>` is legal — the guard asks "is this configured",
never "is this one of four words". `CLAUDE_TIERS` and `CODEX_TIERS` end as
hand-maintained closed lists. A name nothing configures is **warned on stderr**,
never silently accepted and never a refusal. Falling through on an *absent*
configuration is not a downgrade: decision `72f3d6dd` already licenses a
fallback "ONLY when that tier is unconfigured", and a configured role is still
obeyed exactly.

**B3 — The work declares its job role** (model-role-split D3, store `3c9d6262`).
A cell carries a role, and its dispatch asks for an ordered list beginning with
that role. A new job role needs no new bee code and no new dispatch kind — an
operator writes `test: <model>` in `.bee/config.json`, marks a cell `role: test`,
and that cell's worker runs on that model. bee publishes as *config defaults*
only the names bee's own dispatch sites ask for; every other job name is the
user's to invent. Why bee publishes any at all: an open set only reaches the user
if something asks for the job name, and bee dispatches from four sites, so
`test` and `design` would otherwise be names nobody ever says.

**B4 — `role` is the cell's sole model selector** (model-role-split D4, store
`97ce5225`). `tier` is retired as a selector, and with it the closed three-value
enum on `bee cells tier` and `bee state worker add --tier`. The measurement that
settled it, taken over all 506 stored cells: 269 recorded `generation` (the
default anyway), 215 recorded nothing, and `extraction` was chosen twice — 95
percent of cells carried no signal in the field at all.

**B5 — Cost is a separate lever from job** (model-role-split D5, store
`97ce5225`). `ceiling` becomes an explicit escalation flag rather than a tier
value: run on the session model and charge the ration. The existing guard stays
unchanged in force — the 40 percent share refusal and its persisted reason,
where exactly 40 percent passes and 43 percent refuses. All 22 cells that carried
information in `tier` meant budget rather than model choice, and 20 of them said
`ceiling`. Holding cost apart also preserves decision `0015` with no carve-out:
`ceiling` is not a role name at all, so the open set needs no exception for it —
and since role-edge-hardening D1, `0015` has teeth: a configured
`models.<rt>.ceiling` key is named by the config validator for every value
shape, with a teach line pointing at `bee cells escalate`, instead of being
silently accepted and then poisoning a dispatch with the marker-plus-model
pair the guard denies.

**B5a — The escalation answer has three spellings, and the explicit false
wins** (escalate-off-disarm D1, escalate-off-disarm D2). `escalate: true`
means escalated; the legacy `tier: "ceiling"` string still reads as escalated
so a store that never ran the migration is unchanged; and an explicit
`escalate: false` means **disarmed** and outranks the legacy read everywhere —
the ration counter, the preamble's counter, and the migration pass, which
treats a present flag key of either value as final. The disarm door writes the
explicit false only on cells that carry the legacy spelling; every other cell
keeps absent-means-absent. Before this, `bee cells escalate --off` reported
success and disarmed nothing on exactly the 20 cells the live backfill
converted, and a later pass re-armed even a hypothetically effective disarm —
the red test's own output showed a recorded `false` flipped back to `true`.

**B6 — Accounting follows the split** (model-role-split D6, store `97ce5225`).
The tier-mix count at close becomes a role mix plus an escalation share, and the
preamble's ceiling-erosion advice keys off the flag rather than the retired
value.

**B7 — The recommended vocabulary is guidance, never an enum**
(model-role-split D8, store `4eaf1b71`). `code`, `read`, `test`, `docs`,
`review`, `design` are
carried on the planning surface and in `bee cells add --help`, and nowhere is
membership checked. Enforcing the list would move drift from author habit into a
hand-maintained list, which is the exact defect B1 exists to remove.

**B8 — Stored history is migrated in one whole-store pass**
(model-role-split D9, store `4eaf1b71`). `bee cells backfill-roles` gives every
pre-`role` cell the
role it would have carried: `tier: generation` and no-tier take `role: code`,
`extraction` takes `role: read`, and `ceiling` takes `role: code` plus the B5
escalation flag. Role and flag move in the SAME pass on purpose — the ration
divides by a whole-store scan, so a store where some escalations answer the flag
and some answer the tier would misreport it. The stored `tier` string itself is
left in place: D4 retires `tier` as a selector and does not order history
rewritten. The verb is idempotent, so an interrupted run is finished by running
it again. Applied to the live store on 2026-08-25: 564 scanned, 540 migrated.

**B9 — The runtime fallback chain is explicit-only** (model-role-split D10,
store `50808d48`), and is held apart from B2's resolution fall-through — they
are different mechanisms answering different questions. There is no built-in
default chain for any role, so absent configuration a failure stays loud exactly
as it does today. A chain key may name a role or a concrete model, and a
model-keyed chain follows that model wherever it is assigned. Every chain step is
recorded on the dispatch. Because bee never executes dispatches itself, the chain
is a **published contract on the payload**, not a retry loop bee runs.

**B10 — The chain's error gate never absorbs a semantic failure**
(model-role-split D11, store `50808d48`). A step fires only on: quota or rate
limit, provider auth or policy rejection, empty response, malformed tool call
where replay is safe, stream stall or connection reset, or 5xx. A step **never**
fires on a tool error, a wrong or unwanted result, a failed proof, or a red test.
Falling to a weaker model on a semantic failure would hide the defect; under this
gate no *result* failure is ever absorbed, so bee's loud posture is preserved
rather than quietly reversed.

**B11 — There is no `extract` dispatch kind; the role carries it instead**
(model-role-split D12, store `8dad7c2e`, as resolved by planning). D12 was
recorded when a fifth `--kind extract` looked necessary, before D2 and D4 were
settled. Under B2's fall-through it stopped being load-bearing, so it was
**absorbed rather than shipped**: `--kind` still takes exactly `cell`, `gather`,
`reviewer`, `advisor`, and a read-shaped consumer asks
`--kind gather --role extraction`, which resolves to `bee-extract` on the
extraction model. It was recorded as an open question and answered, never
silently dropped — the answer is in `docs/history/model-role-split/plan.md`
("D12's fate").

## The one deliberate silent case

Every rule above refuses or warns rather than resolving silently, with a single
exception that is deliberate and bounded: `code` or `read` asked on a runtime
whose `models.<runtime>` configures neither of them. That is the pre-roles
migration window, and it closes the moment either key is **written** — an
explicit null counts, because a written key means the operator knows the
vocabulary, and a present-but-null name resolves Budget at its own slot rather
than falling through (role-edge-hardening D1). The boundary is pinned by a
test proven to fail under the exact mutation the r2 review said the suite
would miss.

Three more edges hardened by the same decision and by role-surface-cleanup D1:
the ordered list never repeats a name, so the fall-through warn fires once per
genuinely distinct next name; the advisor identity folds case at both doors,
so a mis-cased `"Advisor"` key cannot make the marker door and `--kind
advisor` answer differently; and a fallback-chain key that names neither a
wildcard, a configured role, nor any resolvable model — a chain no dispatch
can ever travel under — is warned by name instead of dying silently.

## Boundaries

- The cell-field view of `role` — that it is required on `bee cells add` exactly
  as `lane` is (model-role-split D7) — lives in
  `areas/workflow-state/cells-authoring-and-revision.md`, which is authoritative
  for cell fields. This concept is authoritative for what the value *means* and
  how it resolves to a model.
- Which worker class a dispatch spawns, and what capability surface it gets, is
  `areas/doctrine-layer/helper-classes-and-transports.md`.
- When a mechanical step is delegated at all is
  `areas/doctrine-layer/delegation-threshold.md`. Roles answer "which model",
  never "should this be delegated".

## Open Gaps

- The escalation ration's denominator moved from `ceiling / tiered` to
  `escalated / the feature's cells`. The change is necessary — a tier-shaped
  denominator reads 0 once `role` is required — and is argued at all three code
  sites, but D5's wording still says the ration is "unchanged in force".
- On opencode the rendered agent file is the enforcement rather than a dispatch
  model param, so a configured `models.opencode.code` is currently ignored.
- `role` is the one required field with no revision door: it is absent from the
  cell `UPDATE_FIELDS` list and is not named as frozen either.
