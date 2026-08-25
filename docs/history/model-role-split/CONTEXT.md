# Model Role Split — Context

**Feature slug:** model-role-split
**Date:** 2026-08-25
**Shaping session:** complete
**Scope:** Deep
**Domain types:** ORGANIZE | RUN | READ

## Feature Boundary

bee stops selecting a worker model by cost tier and starts selecting it
by the **job the work declares**: a cell carries a required, open-ended
`role`, one shared parser resolves it against `models.<runtime>` with
fall-through, `ceiling` becomes an explicit escalation flag rather than
a tier value, and an explicit-only runtime chain may carry a dispatch
past a transient provider failure. It ends at model selection and its
bookkeeping; it does not touch what a worker does once dispatched, and
it introduces no provider catalogue or model ranking.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

Every row below was settled in the `docs/discovery/model-role-split/`
map and carries its store D-ID. The store record is the single source;
this table is the citation surface for cells.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | **One parser lands first.** A single `resolve_role` in the drivers module owns the `models.<runtime>` shape; the model-guard calls it rather than carrying its own second implementation (`model_guard.rs:442-467`), and `resolve_advisor` collapses the same way. This is the **first** step of the sequence. — store `cd72ec97` | Every remaining step writes into the shape it parses; a second parser would be edited in lockstep four more times. The two guard tier lists had already drifted 4 against 5 with nothing intending it. |
| D2 | **Roles are an open, fall-through set.** A consumer names an ordered *list* of role names; an unset or unresolvable name yields to the next; the last entry is a name bee always resolves. Any name present in `models.<runtime>` is legal — the guard asks "is this configured", not "is this one of four words". `CLAUDE_TIERS` and `CODEX_TIERS` end as hand-maintained closed lists. A name nothing configures is **warned**, never silently accepted. — store `06e49368` | Falling through on an *absent* configuration is not a downgrade: decision `72f3d6dd` already licenses a fallback "ONLY when that tier is unconfigured". A configured role is still obeyed exactly. |
| D3 | **The work declares its job role.** A cell carries a role; its dispatch asks for an ordered list beginning with that role. A new job role needs no new bee code and no new dispatch kind. bee publishes as *config defaults* only the names bee's own dispatch sites ask for; every other job name (`test`, `design`, `docs`, `migrate`) is the user's to invent. — store `3c9d6262` | An open role set only reaches the user if something *asks* for the job name. bee dispatches from four sites, so `test` and `design` would otherwise be names nobody says. |
| D4 | **`role` is the cell's sole model selector.** `tier` is retired as a selector, with it the closed three-value enum on `bee cells tier` and `bee state worker add --tier`. — store `97ce5225` | Measured on all 506 stored cells: 269 recorded `generation` (the default anyway), 215 recorded nothing, `extraction` was chosen twice. 95 percent of cells carried no signal in the field. |
| D5 | **`ceiling` becomes an explicit escalation flag**, not a tier value: run on the session model and charge the ration. Today's guard stays unchanged in force — the 40 percent share refusal (`handlers_close.rs:1063`, refusal `:1126-1133`) and its persisted reason. — store `97ce5225` | All 22 cells that carried information in `tier` meant budget, not model choice; 20 of them `ceiling`. It also preserves decision `0015` with no carve-out: `ceiling` is not a role name at all, so the open set needs no exception. |
| D6 | **Accounting follows the split.** The tier-mix count at close (`handlers_close.rs:1054-1102`) becomes a role mix plus an escalation share; the preamble's ceiling-erosion advice (`hooks/session_preamble/store.rs:309-320`) keys off the flag. — store `97ce5225` | — |
| D7 | **`role` is required on a cell**, exactly as `lane` is: `bee cells add` refuses without it. The value is any non-empty name — validation checks presence and shape, never membership. — store `4eaf1b71` | The store's own natural experiment: the required field (`lane`) is present on 506 of 506 cells, the optional one (`tier`) on 291. An optional role reproduces the `tier` outcome, where a configured per-job model fires on about half the cells that wanted it and the miss is silent. |
| D8 | **A recommended vocabulary ships as authoring guidance, never as an enum**: `code`, `read`, `test`, `docs`, `review`, `design`, carried on the planning surface and in `bee cells add --help`. — store `4eaf1b71` | Enforcing a list would move drift from author habit into a hand-maintained list, which is the defect D1 exists to remove. |
| D9 | **Backfill of the 506 stored cells:** `tier: generation` and no-tier cells take `role: code`; the 2 `extraction` cells take `role: read`; the 20 `ceiling` cells take `role: code` plus the D5 escalation flag. All are capped history — bookkeeping, not behavior. — store `4eaf1b71` | — |
| D10 | **An explicit-only runtime fallback chain**, held apart from D2's resolution fall-through. No built-in default chain for any role, so absent configuration a failure stays loud exactly as today. A chain key may name a role or a concrete model; a model-keyed chain follows that model wherever assigned. Every chain step is recorded on the dispatch. — store `50808d48` | Explicit-only keeps decisions `3ceba8f5` D2, `267192c1` and `4faf1de9` intact rather than reopening them; the advisor keeps its no-fallback behavior unless the owner configures a chain deliberately. |
| D11 | **The chain's error gate.** A step fires only on: quota or rate limit, provider auth or policy rejection, empty response, malformed tool call where replay is safe, stream stall or connection reset, 5xx. A step **never** fires on a semantic failure — a tool error, a wrong or unwanted result, a failed proof, a red test. — store `50808d48` | Falling to a weaker model on a semantic failure would hide the defect. Under this gate no *result* failure is ever absorbed, so bee's loud posture is preserved rather than reversed. |
| D12 | **`bee dispatch prepare` gains `--kind extract`**, resolving the extraction slot and returning `bee-extract`; `gather` keeps its existing default. — store `8dad7c2e` | Landed before D2/D4 were settled. Under D2's fall-through it is no longer load-bearing for correctness; planning must decide whether it still ships as its own step or is absorbed by the role work. Recorded as an Open Question below rather than silently dropped. |

### Agent's Discretion

The owner answered D1's ordering question, D2, D3, D4 and D5 directly
(map tickets 001, 002, 006). On 2026-08-25 the owner delegated the
remaining tickets to the agent — "cứ làm tới khi ra plan không cần hỏi
thêm" — so **D7, D8, D9 (ticket 007), D10, D11 (ticket 003) and D1
(ticket 004) are the agent's calls under that delegation**, each
recorded with its evidence and each overturnable by the owner without
a supersession ceremony.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| role | The **job** a piece of work is: `code`, `read`, `test`, `review`. An open-ended name a cell declares and a config maps to a model. The sole model selector. |
| tier | The retired **cost** word (`extraction`/`generation`/`ceiling`). Kept in this document only to name what is being retired. |
| escalation flag | The cell marking that says "run on the session model and charge the ration" — what `ceiling` actually meant. |
| fall-through | Resolution-layer behavior: an unset or unresolvable role yields to the next name in the consumer's list. No failure involved. |
| chain | Runtime-layer behavior: a configured ordered list of models a dispatch may move along **after** a transient failure. Distinct from fall-through. |
| published default | A role name bee's own dispatch sites ask for, and therefore ship configured. Distinct from D8's recommended vocabulary, which is guidance only. |

## Specific Ideas And References

- `~/Projects/refs/oh-my-pi` @ `2b66ee69` — the tool whose model-roles
  screen started the map. Distilled at
  `docs/history/research/oh-my-pi-model-roles-distill.md`. What it
  contributed: roles are mixed-axis there too (six job, two cost, two
  capability), and what makes that workable is fall-through plus an
  open set, not axis purity. Its `retry.fallbackChains` error gate is
  the direct source of D11.
- The owner's framing, 2026-08-24: `extraction` and `generation` are
  cost words, so nobody can judge which model is "good at generation",
  while real model strengths are job-shaped — some plan well, some test
  well, some design well, some code well. D3 and D4 exist to serve that.

## Existing Code Context

From the discovery reads. Downstream agents read these before planning.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:318-437` —
  `resolve_tier` and `resolve_advisor`; the surviving implementation D1
  collapses onto.
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:731-745` —
  already prefers a cell's own recorded value over the slot default,
  writing `tier_source: "cell"`. The mechanism D3 and D4 reuse.
- `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:133-140` —
  the required-`lane` refusal, the exact pattern D7 copies.
- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1063`,
  `:1126-1133` — the 40 percent ration and its reason; D5 keeps these
  in force and rehomes their key.

### Established Patterns

- Typed refusal with a FIX line naming the remedy — every guard and
  verb in this area follows it; D7's new refusal must too.
- One door for dispatch (`bee dispatch prepare`, decision `c80e0220`) —
  unchanged by this feature; the role travels through it.
- Explicit-only degradation (`3ceba8f5` D2) — the posture D10 extends
  rather than reverses.

### Integration Points

- `packages/bee-rs/crates/bee/src/hooks/model_guard.rs` — the second
  parser (`:442-467`), `CLAUDE_TIERS`/`CODEX_TIERS` (`:192-193`),
  `PINNED_AGENT_TYPE` (`:605-629`), `dispatch_kind_for_tier`
  (`:660-666`), the marker parser (`:195-224`). D1 and D2 both land here.
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:34-40` —
  `slot_for_kind`, whose catch-all `_ => "advisor"` arm silently
  resolves the advisor slot for any unhandled kind. A live hazard for
  any kind added by this feature.
- `packages/bee-rs/crates/bee/src/onboard/agents.rs:129-147` and
  `onboard/templates.rs:222-235` — render `{{TIER_MODEL}}` into each
  agent file from the slot map; agents naming a role changes what
  onboarding writes.
- Four private `MODEL_TIERS` copies —
  `verbs/cells/validate.rs:29`, `verbs/state_group/mod.rs:166`,
  `verbs/status_full/mod.rs:60`, `hooks/session_preamble/mod.rs:106`.
- `verbs/cells/handlers_close.rs:1114-1152` (`bee cells tier`) and
  `verbs/state_group/workers.rs:89` (`--tier` enum) — D4 retires both.
- `hooks/session_preamble/` — publishes the resolved slots per decision
  `46827304` D2; an open role set changes that block.

## Canonical References

- `docs/discovery/model-role-split/MAP.md` — the closed map and its
  closing note.
- `docs/discovery/model-role-split/tickets/001…007` — the seven
  questions and their answers, with the evidence each was decided on.
- `docs/history/research/oh-my-pi-model-roles-distill.md` — the source
  read, its dependency matrix and cross-cutting sweep.
- Store decisions: `8dad7c2e`, `06e49368`, `3c9d6262`, `97ce5225`,
  `4eaf1b71`, `50808d48`, `cd72ec97`. Touched and still active:
  `a2f85972`, `72f3d6dd`, `3ceba8f5`, `267192c1`, `4faf1de9`,
  `46827304`, `c80e0220`, `0015`.

## Outstanding Questions

### Resolve Before Planning

- [ ] None. The map closed with no fog and no open tickets.

<!-- bee:not-a-deferral: all five questions were answered by planning and shipped on 2026-08-25; each carries its outcome inline below, so this section is a closed record of what planning decided, not a promise to act later. -->

### Deferred To Planning


All five resolved. Outcomes recorded at close:

- [x] **Does `--kind extract` (D12) still ship as its own step?** D2's
      fall-through removes the correctness need for it. Planning decides
      whether it lands, is absorbed into the role work, or is dropped —
      a sequencing call, not a product one.
      **Outcome: absorbed, not shipped.** There is no fifth dispatch
      kind — `--kind` still takes exactly `cell`, `gather`, `reviewer`,
      `advisor`. Under D2 a read-shaped consumer asks
      `--kind gather --role extraction` and the fall-through returns
      `bee-extract` on the extraction model, so a separate kind would
      buy only what the resolver already does. Reasoned in
      `plan.md` ("D12's fate").
- [x] **Where the escalation flag lives on the cell record** — a boolean
      plus reason, or a reserved role name that validation rejects in
      the role field. D5 fixes the behavior, not the field shape.
      **Outcome: a boolean plus reason.** The cell carries `escalate:
      true` and `trace.escalation_reason`; the reserved-role-name shape
      was rejected because `ceiling` must not be a role name at all
      (D5). `bee cells escalate` is the door, `--off` removes the flag.
- [x] **Whether `effort` is delivered as part of this feature.** It is
      configured (`models.rs:167-181`), displayed by the preamble
      (`model_guard.rs:338-341`), and dropped at the door for every
      `Resolved::Model` (`prepare.rs:800`, `:1050`, `:1063`) — only the
      codex `native` branch emits it (`:898-899`). On the claude runtime
      the Agent tool takes no effort parameter, so this may be a harness
      limit rather than a bee gap. Planning must determine which, and
      either deliver it or record it as a known non-delivery.
      **Outcome: a harness limit, recorded as a known non-delivery.**
      `effort` still reaches the payload only on the codex `native`
      branch (`prepare.rs:1170-1171`, `reasoning_effort`). The claude
      Agent tool takes no effort parameter, so there is nothing for bee
      to emit there. Not a bee gap; no cell was spent on it.
- [x] **How the 506-cell backfill (D9) is applied** — a migration verb,
      a one-time script, or lazily on read. All cells are capped.
      **Outcome: a migration verb.** `bee cells backfill-roles`, with
      `--dry-run` first. Explicitly *not* lazy-on-read: three counters
      divide by a whole-store scan, so a half-migrated store would
      misreport every one of them. Applied to the live store on
      2026-08-25 — 564 scanned, 540 migrated, idempotent on re-run.
- [x] **The dispatch's ordered list per consumer** — the exact list each
      of bee's four dispatch sites asks for. D3 fixes the shape
      (`[<cell role>, <execution default>, <backstop>]`), planning fixes
      the literal names.
      **Outcome:** `cell_role_list` (`prepare.rs:98`) returns
      `[role, "extraction", "generation"]` for `read` and
      `[role, "code", "generation"]` for every other role. A known
      defect is filed as a P2: for `role == "code"` the head repeats,
      so the fall-through warn fires twice.

<!-- /bee:not-a-deferral -->

<!-- bee:not-a-deferral: these three are recorded NON-GOALS, not postponed work — each names why it is out of scope (no consumer, charted out of scope, or settled by an existing decision) and none promises a later delivery. -->

## Deferred Ideas


- **Capability as a declarative requirement** ("this work needs vision",
  "this needs long context") — the source read expresses capability as
  a named role plus a post-hoc filter, never as a requirement the
  resolver matches. Out of scope; bee has no consumer for it.
- **Per-provider model catalogues or auto-selection** — named Out of
  Scope by the map at charting time. bee configures a model per role; it
  does not rank or discover models.
- **`ceiling` becoming configurable** — settled by decision `0015`, not
  reopened.

<!-- /bee:not-a-deferral -->

## Handoff Note

<!-- bee:not-a-deferral: this note describes which sections each downstream stage READS — it documents the record's own machinery and promises no later work. -->

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.

<!-- /bee:not-a-deferral -->

Sequencing is itself locked: **D1 lands first.** Every other decision
writes into the shape D1's shared parser owns.
