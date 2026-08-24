---
artifact_contract: bee-plan/v1
mode: high-risk
# approved_gate2: <unset until approval; then a date stamp — the only permitted post-approval write>
---

# Plan: Model Role Split

Mode: `high-risk` — 5 risk flags: data-model, public-contracts,
covered-contract-change, proof-weakening, multi-domain. Hard gate:
retiring the cell `tier` enum is validation removal, and D9 is a
migration over 506 stored records.

Why this is the least workflow that protects the work: the feature
rewrites the one field every dispatch reads, in six modules, while
deleting the validation and the tests that currently pin it. A smaller
lane would let a wrong `role` resolution ship silently — and silence is
this feature's whole failure mode, because fall-through completes the
work on the wrong model with no red output.

## Requirements (from CONTEXT.md)

- **D1** — one shared `resolve_role` in drivers; the guard calls it
  instead of carrying a second parser. Lands **first**.
- **D2** — roles are an open, fall-through set; a consumer names an
  ordered list; unconfigured names warn; `CLAUDE_TIERS`/`CODEX_TIERS`
  end.
- **D3** — the work declares its job role; the cell's role heads the
  dispatch's ordered list.
- **D4** — `role` is the cell's sole model selector; `tier` retires as a
  selector, with the enums on `bee cells tier` and
  `bee state worker add --tier`.
- **D5** — `ceiling` becomes an explicit escalation flag; the 40 percent
  ration and its reason stay in force.
- **D6** — accounting becomes a role mix plus an escalation share.
- **D7** — `role` is required on a cell, as `lane` is; presence and
  shape checked, never membership.
- **D8** — a recommended vocabulary ships as authoring guidance only.
- **D9** — backfill the 506 stored cells.
- **D10** — an explicit-only runtime fallback chain, separate from D2.
- **D11** — the chain's error gate: transient and infrastructural only.
- **D12** — `--kind extract`; its fate is a planning call (below).

## Discovery

Inspected during the discovery map, with evidence in
`docs/discovery/model-role-split/tickets/`: the two `resolve_tier`
implementations (`models.rs:318-383`, `model_guard.rs:442-467`); the
catch-all `_ => "advisor"` arm in `slot_for_kind` (`prepare.rs:34-40`);
the four private `MODEL_TIERS` copies; and the cell store itself —
a scan of all 506 cells showing `tier` present on 291 and carrying real
information on 22. Evidence command for the last one, re-runnable:
a JSON walk of `.bee/cells/**` counting `tier` and `lane`.

Three planning-time findings, not in CONTEXT.md:

1. `slot_for_kind`'s catch-all silently resolves the **advisor** slot
   for any unhandled kind. Every slice that touches kinds must close
   this arm, or a typo routes work to the advisor model.
2. `effort` is parsed, displayed by the preamble
   (`model_guard.rs:338-341`), and dropped for every `Resolved::Model`
   (`prepare.rs:800`, `:1050`, `:1063`). Only codex-native emits it.
   The claude Agent tool takes no effort parameter, so this is a
   harness limit, not a bee gap — S6 records it rather than fixing it.
3. `.opencode/skills/`, `.claude/skills/`, `.claude-plugin/skills/` and
   `.codex-plugin/skills/` are byte mirrors of `skills/`. Any guidance
   text (D8) must go through the regen chain, never hand-edited in five
   places.

## Approach

**Recommended path.** Land the shared parser first (D1), widen it to
list resolution with fall-through (D2), then move the selector from
`tier` to `role` on the cell (D3, D4, D7), split out the escalation
flag and its accounting (D5, D6), backfill history (D9), and only then
add the runtime chain (D10, D11) — which is independent of everything
above and could be dropped without harming the rest.

The ordering is not a preference: D1 is locked as first, and every
later step writes into the shape its parser owns.

**Rejected alternatives.**

- Move the cell field first and collapse the parsers afterwards —
  rejected by D1, and it would require writing each change twice.
- Ship the runtime chain (D10/D11) early because it is independently
  valuable — rejected: it is the only part with no dependency on the
  rest, so it is the safest thing to cut if the budget runs out. Cut
  candidates go last.
- Keep `tier` alongside `role` during a deprecation window — rejected
  by D4, and the 506-cell measurement shows nothing to deprecate
  gradually: 484 of 506 cells carry no signal.

**D12's fate — the planning call CONTEXT.md deferred.** `--kind extract`
(`8dad7c2e`) is **absorbed, not shipped as its own step.** Under D2 an
`extract`-shaped consumer simply asks `["read", "code"]` and
fall-through does the rest, so a fifth kind adds a `DISPATCH_KINDS`
entry, a `slot_for_kind` arm and a `dispatch_kind_for_tier` arm to buy
something the resolver now does for free. The decision is not
contradicted — its intent, "a read-only worker must be dispatchable" —
is delivered by S3's consumer lists. Recorded here so the deviation
from `8dad7c2e`'s literal text is named rather than silent.

**Risk map.**

| Component | Risk | Proof needed |
|---|---|---|
| Shared parser (D1) | LOW | Existing drivers + model_guard suites pass **unchanged** — that is the whole proof of a behavior-neutral collapse. |
| Fall-through + open set (D2) | **HIGH** | Ordered-list resolution tests; the warn fires on an unconfigured name; no path resolves a name the config does not carry without warning. |
| `slot_for_kind` catch-all | **HIGH** | A test that an unknown kind refuses instead of resolving the advisor slot. |
| Cell `role` required (D7) | MEDIUM | `bee cells add` refuses with a typed FIX line; every bee-internal cell author supplies one. |
| `tier` retirement (D4) | **HIGH** | Every reader of `tier` is found and moved; a grep-based inventory is part of the cell's must_haves. |
| Escalation ration (D5) | **HIGH** | The 40 percent refusal fires on the flag exactly as it fired on the tier value, including the `--reason` path. |
| Backfill (D9) | MEDIUM | Idempotent; a dry run reports counts matching 484 / 2 / 20 before any write. |
| Runtime chain (D10, D11) | MEDIUM | A semantic failure never advances the chain — the single most important negative test in this feature. |
| Guidance text (D8) | LOW | Regen chain runs; the five mirrors stay byte-identical. |

## Shape

**Feature outcome.** An operator writes `test: <a model good at tests>`
in `.bee/config.json`, marks a cell `role: test`, and that cell's worker
runs on that model — with no bee code change for the name `test`, and
with a warning if nothing configures it.

**Repo-reality basis.** bee already routes a per-cell dispatch judgment
through `prepare.rs:731-745` (`tier_source: "cell"`), already enforces a
required cell field (`lane`, `validate.rs:133-140`), and already renders
four job-named agents pinned to cost slots (`guard.rs:32-39`). This
feature reuses all three mechanisms; it invents no new plumbing.

| Epic | Capability / Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| E1 | One config parser | Two parsers of one shape already drifted 4-vs-5 with nothing intending it; every later slice writes into this shape | S1 | Existing suites pass unchanged |
| E2 | Open, fall-through resolution | The dead-slot defect class ends here; this is what makes a role name cheap | S2 | Order, warn, and no-silent-resolve tests |
| E3 | The cell declares its role | Delivers the actual user-visible capability | S3, S4 | End-to-end: config a name, mark a cell, watch it resolve |
| E4 | Escalation and its ration | `ceiling` carried a budget meaning that must not be lost while `tier` retires | S5 | Ration refusal fires identically on the flag |
| E5 | History and accounting | 506 stored cells and the tier-mix reporting must stay honest | S6 | Idempotent backfill, dry-run counts, role-mix output |
| E6 | Runtime chain | The one failure a chain genuinely absorbs is a quota wall; everything else must stay loud | S7, S8 | A semantic failure never advances the chain |
| E7 | Surfaces and guidance | Authors need the vocabulary where they write cells; five skill mirrors must not drift | S9 | Regen chain green, mirrors byte-identical |

**Slice queue.**

| Slice | Contents | Depends on |
|---|---|---|
| **S1** | One `resolve_role` in drivers; guard calls it; `resolve_advisor` collapses. Behavior-neutral. | — |
| **S2** | Ordered-list resolution + fall-through; open-set membership check; warn on unconfigured; retire `CLAUDE_TIERS`/`CODEX_TIERS`; close `slot_for_kind`'s catch-all arm. | S1 |
| **S3** | Cell gains required `role` (D7) with its typed refusal; `bee cells add` help carries the D8 vocabulary; dispatch asks the ordered list headed by the cell's role. | S2 |
| **S4** | `tier` retires as a selector: `bee cells tier` and `state worker add --tier` enums go; every reader inventoried and moved. | S3 |
| **S5** | Escalation flag replaces `ceiling` as a tier value; the 40 percent ration and `--reason` rehomed unchanged. | S4 |
| **S6** | Backfill the 506 cells (D9); tier-mix becomes role-mix + escalation share; preamble advice keys off the flag; record `effort` as a known non-delivery. | S5 |
| **S7** | Runtime chain config shape and resolution (D10), explicit-only, recorded on the dispatch. | S2 |
| **S8** | The error-class gate (D11), with the semantic-failure negative tests. | S7 |
| **S9** | Guidance text through the regen chain: AGENTS block, planning surface, config reference. | S3 |

S7–S9 are the cut candidates if the budget runs short; S1–S6 are the
locked capability and cannot be reduced without a supersede.

**Current slice to prepare: S1 + S2** — one parser, then list
resolution. Together they are demonstrable end-to-end without touching
a cell: configure `models.claude.myrole`, ask for it, watch it resolve;
ask for an unconfigured name, watch it warn and fall through. S1 alone
would be a refactor with nothing to show, which is why the walking
skeleton is the pair.

## Test matrix

High-risk: probes per applicable dimension. Each cell's writer judges
existing coverage first (`model_guard.rs` already carries ~55 tier
tests) and authors only the gap.

| # | Dimension | Applicable | Probes |
|---|---|---|---|
| 1 | User types | No | bee has no actor model; every caller is the same operator. |
| 2 | Input extremes | **Yes** | Empty role string; whitespace-only; a name 500 chars long; a name equal to a reserved word (`ceiling`); a role list of length 1; an empty list. |
| 3 | Timing | No | Resolution is synchronous and has no ordering hazard. |
| 4 | Scale | **Yes** | The D9 backfill over 506 cells; a config carrying 50 role names; a fall-through list walked to its last entry. |
| 5 | State transitions | **Yes** | A cell authored before the change (no `role`) read after it; a cell mid-flight when the backfill runs; a worker registered with the old `--tier` enum. |
| 6 | Environment | **Yes** | claude vs codex vs opencode runtimes; a herding-shaped slot; a cli-shaped slot; a slot that is explicitly null. |
| 7 | Error cascades | **Yes** | The chain's own failure — every entry exhausted; a chain naming a model that no longer exists; a chain that loops back to its own head. |
| 8 | Authorization | No | No trust boundary is crossed. |
| 9 | Data integrity | **Yes** | Backfill idempotence — running it twice changes nothing the second time; a partial backfill interrupted midway; the 484 / 2 / 20 counts asserted before write. |
| 10 | Integration | **Yes** | `dispatch prepare` payload shape unchanged for callers that pass no role; the guard's marker path; onboarding's rendered agent files. |
| 11 | Compliance | No | No regulatory surface. |
| 12 | Business logic | **Yes** | The 40 percent ration fires on the flag exactly as on the tier value; a semantic failure never advances a chain; a configured role is always obeyed (decision `72f3d6dd`). |

The single most important negative test in this feature: **a red test
result, a failed proof, or a tool error must never advance a fallback
chain.** If that test is missing, D11 is not implemented.

## Out of scope

- Capability as a declarative requirement (vision, long context).
- Per-provider model catalogues, ranking, or auto-selection.
- `ceiling` becoming configurable — decision `0015` stands.
- Delivering `effort` on the claude runtime — a harness limit, recorded
  in S6 as a known non-delivery rather than fixed here.
- Anything about what a worker does once dispatched.
