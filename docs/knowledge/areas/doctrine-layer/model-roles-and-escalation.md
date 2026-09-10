---
type: bee.area
title: Doctrine Layer — model roles, fall-through, and escalation
description: "How work says which model should run it: an open set of job-named roles resolved through one parser, an ordered fall-through that warns instead of failing, cost held apart as an explicit escalation flag, and an explicit-only retry chain that never fires on a semantic failure."
timestamp: 2026-08-26
bee:
  id: doctrine-layer-model-roles-and-escalation
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [areas/doctrine-layer/overview.md]
  decisions: [model-role-split D1/D2/D3/D4/D5/D6/D8/D9/D10/D11/D12, escalate-off-disarm D1/D2, role-surface-cleanup D1, role-edge-hardening D1]
  sources: ["model-role-split (docs/history/model-role-split/CONTEXT.md, 34 cells, merged 2026-08-25)", "docs/discovery/model-role-split/MAP.md", "docs/history/research/oh-my-pi-model-roles-distill.md", "docs/history/model-role-split/reports/review-r2.md", "role-slot-description cell rsd-1 (capture stub c1952702, flushed 2026-08-26)", "models-show-verb cells ms-1..ms-3 (capture stub feeed5df, flushed 2026-08-26)", "agent-model-unpin cells amu-1/amu-2 (capture stub 003a23fc, flushed 2026-08-26)"]
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

**B1 — One parser owns the `team.<runtime>` shape** (model-role-split D1,
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
present in `team.<runtime>` is legal — the guard asks "is this configured",
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
`team.<rt>.ceiling` key is named by the config validator for every value
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
converted, and a follow-on migration pass re-armed even a hypothetically
effective disarm — the red test's own output showed a recorded `false`
flipped back to `true`.

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

**B12 — A role slot may describe itself, and only the door line reads the
description** (role-slot-description, cell rsd-1, PBI p-a1399c00; display order
amended by leader-sees-team D1 and D4 on 2026-09-09 — the leader chooses a team
member by job, so every read surface leads with role and description and shows
the transport last). A
`team.<runtime>` role slot may carry an optional description string.
Exactly one surface renders it: the dispatch-door roles line, reading the
RAW config and clipping to 60 characters, as `name=model ("desc")`. The
normalizer drops the field, which is what keeps resolution, the model
guard, and dispatch prepare blind to it — a display string can never
steer a dispatch. The validator already tolerated unknown keys; that
tolerance is now pinned by test. Amended by leader-sees-team D4
(2026-09-09): the dispatch-door roster prints every role one per line
with no descriptions (≤ 60 characters per line, no six-role cap);
descriptions appear only in `bee team show`.

**B13 — `bee team show` is the one read door for the role table**
(models-show-verb, cells ms-1/ms-2/ms-3, PBI p-0a5e6c44; the rule that the
leader picks by job and the transport is config ships to every host under
leader-sees-team D3 — one sentence in the rendered AGENTS.md block, the
delegation contract and the worker prompt; renamed from
`bee models show` by team-config-rename D1 on 2026-09-09, when the config block
itself went from `models.<runtime>` to `team.<runtime>` — the block is the
leader's roster of who does which job, not a list of model ids, and the old
name made agents read it as one. `models` is still read as an alias, folded
into `team` once at config load (team-config-rename D2): `team` wins when both
are present, a `models`-only config gets one `note legacy_models_key` line from
`bee doctor` and one line in the preamble's Dispatch door, and `bee models
show` still runs with one stderr notice.) It prints the
raw `team.<runtime>` slots verbatim — description intact — plus the
built-in defaults, each row source-marked. `bee status --json` keeps
descriptions display-only; the default config seeds described roles; and
the missing-role refusal and the `bee cells add` help both send an
author to this verb before assigning a role. Resolution, guard, and
dispatch are untouched by the verb — it reads, never resolves. Amended
by leader-sees-team D4 (2026-09-09): `team show` resolves for display
(role, source, description, model, transport) and still writes nothing;
the roster is one role per line.

**B14 — A rendered agent file carries no model pin; the dispatch payload
is the one authority** (agent-model-unpin, cells amu-1/amu-2). On the
first runtime, rendering a known agent file is unconditional and writes
no `model:` line — the model reaches the worker only through the
dispatch payload's model param. The drift check flips accordingly: a
PRESENT model line in an agent file is the drift, not a missing one.
Opencode is unchanged on purpose — there the rendered file IS the
enforcement (see Open Gaps).

**B15 — Blind-lane and hat-wave seats carry their own roles, and an empty
seat yields to the advisor** (lane-model-diversity D1/D2/D3/D4/D5). The
eight seat names — `lane-1..lane-3` and `hat-facts-gaps`,
`hat-risks`, `hat-value`, `hat-alternatives`, `hat-user-impact` — live in
`SEAT_ROLES` beside `tier_role_list` in the drivers module (case-folded
membership), and each configures a model in the same open `team.<runtime>`
table as every other role (D1). On `--kind advisor` only, a declared seat
whose slot resolves nothing — absent, null, or shapeless — rebinds to the
advisor slot instead of refusing, keeping `resolve_advisor`'s one-name
no-floor walk as the tail and stamping the marker `advisor` while the
dispatch record keeps the asked-for seat as `economics.requested_role`
(D2/D4); every other kind, and every non-seat name, keeps the
`role_not_configured` refusal. A configured `hat-*` slot without a
`description` is a `bee doctor` advisory, never a resolution input — the
display-only law of B12 stands (lane-model-diversity D3, venue decision).
The two procedure homes in `gates-and-delegation.md` name the seats and
point back at `SEAT_ROLES` as the constant of record (D5).

**B15a — Planning's plan check rides the advisor-kind hat seats, not the
`review` tier** (proactive-leader-intake D4, decision `b34fdea9`). The old
merged plan reviewer was a `review`-role dispatch; it is retired as a seat
holder. The plan check is now the plan-step hat wave — three `hat-*` seats by
default, five on high-risk — dispatched `--kind advisor`, which is the one
kind whose empty-seat rebind (B15) applies, so a host that configured no
`hat-*` slot still gets the wave through its advisor slot instead of a
`role_not_configured` refusal. Only the Structure mandate moved with it: the
cold-pickup cell review stays with the leader, and the `review` role keeps its
other consumers (bee-reviewing's specialists, exploring's fresh-eyes) plus its
`null` fallback to generation. No constant changed — `SEAT_ROLES` and the
resolution path are untouched; what moved is which skill asks for which kind.

**B16 — `team.pi` is a herding-only role table with a settled value set**
(pi-support D5/D6). The pi runtime resolves roles from the same one config
home, but every slot must resolve `kind: herding` — the dispatch door's
`pi_requires_herding` refusal covers plain-string, native, cli, budget and
the escalation path by construction, because Pi has no subagent surface.
The settled values (pi-support D6): heavy roles (code, test, docs, review)
ride herding agents running `claude --model opus`, advisor rides
`claude --model fable`, and the cheap roles (read, extraction, generation,
supervisor) ride `agy-flash` — herding constrains the transport, never the
model vendor. Delivery status (pi-result-mailbox): a Pi worker's result
returns, and the path to plan on is the SYNCHRONOUS one every runtime
shares — `bee herding run`'s own output, carrying `report_path` when the
worker wrote a report. The Pi-only async half is opt-in per dispatch
(`--inbox-session <token>`) and carries named limits: at-least-once
delivery with `job_id` as the dedupe key, a drain that needs a live Pi
session, and header-only injection (the report body is never injected).
See `docs/config-reference.md`, "Pi — team.pi is herding-only".

**B17 — A gather asks for the read job** (gather-reads-the-read-slot D1/D3,
store `295f3d03`). A `--kind gather` dispatched with no `--role` resolves the
`read` slot, walking `[read, generation]` and taking the first name the host
configures. `extraction` is NOT in that tail, and its absence is the decision:
extraction was the tier era's cheapest slot and never the gather slot, so a
legacy host that configures `extraction` + `generation` and no `read` keeps its
gathers on `generation` byte for byte — a walk through extraction would move
them onto the cheapest model silently, the exact downgrade B2's fall-through
rule exists to prevent. The read-shaped CELL list is a different consumer with
a different history (B8 backfilled `tier: extraction` cells to `role: read`)
and stays `[read, extraction, generation]`; the two lists differ on purpose and
each says why. The agent comes from the ASKED name and the kind, never from the
winner: a role-less gather pins `bee-gather` whichever spelling of the read job
won, while `--kind gather --role extraction` — or `--role read`, the same job
under its other spelling — keeps B11's `bee-extract`. A broad multi-file hunt
is therefore the ROLE-LESS gather, never `--role read`, and the seeded `read`
description says so. The
name that won the walk is the name the dispatch travels under — `[bee-tier:
<winner>]` and `economics.logical_tier` — on this path and on the cell-role
path; every other path keeps its asked name, so a review-less host's reviewer
is untouched.

## The one deliberate silent case

Every rule above refuses or warns rather than resolving silently, with a single
exception that is deliberate and bounded: `code` or `read` asked on a runtime
whose `team.<runtime>` configures neither of them. That is the pre-roles
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


## Pi has fan-out paths bee does not use — and that stays a choice, not an oversight

The premise behind `pi_requires_herding` is that Pi ships no **built-in**
Task/Agent tool, which is true. The stronger reading — that herding is Pi's only
possible fan-out — is not. A study of pi 0.84.4 found three unused paths:
`pi.registerTool()` works at runtime and Pi ships a working subagent example
extension that spawns pi child processes from markdown agent definitions; the Pi
SDK's `createAgentSession` gives in-process child sessions; and `pi --mode rpc`
gives headless children.

The user was shown all three on 2026-09-02 and **declined to supersede** the
locked decision: Pi dispatch stays herding-only (store `9f5c6d17`), and `pi`
means the pi binary 0.84.x alone, not omp (store `5d87f14e`). So the rule stands
on a choice about one transport, not on an absence of alternatives — which is
what a later reader needs to know before proposing the "obvious" fix again.

**`pi` and `omp` are two different binaries.** Both were installed side by side
on the development machine: pi (`@earendil-works/pi-coding-agent`) and omp
(`github:can1357/oh-my-pi`, `~/.omp` paths). bee supports pi only — there is no
`.omp` directory and no omp entry among the runtimes. omp loads legacy pi
extensions, rewriting the old import paths and shimming the trust call, so an
extension written to pi's **narrower** surface runs on both while the reverse is
false. A belt meant to reach two targets therefore stays inside the portable
core: the extension factory, `on(session_start|tool_call|tool_result)`,
`registerTool`, `registerCommand`, and `sendMessage`/`appendEntry`.

## Open Gaps

- The escalation ration's denominator moved from `ceiling / tiered` to
  `escalated / the feature's cells`. The change is necessary — a tier-shaped
  denominator reads 0 once `role` is required — and is argued at all three code
  sites, but D5's wording still says the ration is "unchanged in force".
- On opencode the rendered agent file is the enforcement rather than a dispatch
  model param, so a configured `team.opencode.code` is currently ignored.
- `role` is the one required field with no revision door: it is absent from the
  cell `UPDATE_FIELDS` list and is not named as frozen either.
