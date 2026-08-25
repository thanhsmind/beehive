---
artifact_contract: bee-plan/v1
mode: high-risk
plan_rev: 2
# approved_gate2: <unset until approval; then a date stamp — the only permitted post-approval write>
---

# Plan: Model Role Split

Mode: `high-risk` — 5 risk flags: data-model, public-contracts,
covered-contract-change, proof-weakening, multi-domain. Hard gate:
retiring the cell `tier` enum is validation removal, and D9 is a
migration over 506 stored records.

Why this is the least workflow that protects the work: the feature
rewrites the one field every dispatch reads, in eight modules, while
deleting the validation and retargeting **120** tests that currently pin
it. A smaller lane would let a wrong `role` resolution ship silently —
and silence is this feature's whole failure mode, because fall-through
completes the work on the wrong model with no red output.

**Revision note (rev 2).** A two-reviewer wave found three blockers in
rev 1 and they are fixed here, named rather than quietly patched:
rev 1 declared S7–S9 droppable "if budget runs short" although they
carry locked decisions (removed — see Scope integrity); rev 1 absorbed
D12 with no slice owning the agent-name mapping (now S3); and rev 1
claimed the five skill trees are byte mirrors, which is false. Two
factual claims were also wrong and are corrected in Discovery.

## Requirements (from CONTEXT.md)

- **D1** — one shared `resolve_role` in drivers; the guard calls it
  instead of carrying a second parser. Lands **first**.
- **D2** — roles are an open, fall-through set; a consumer names an
  ordered list; unconfigured names warn; the list's last entry always
  resolves; `CLAUDE_TIERS`/`CODEX_TIERS` end.
- **D3** — the work declares its job role; the cell's role heads the
  dispatch's ordered list; bee ships config defaults only for the names
  its own dispatch sites ask for.
- **D4** — `role` is the cell's sole model selector; `tier` retires.
- **D5** — `ceiling` becomes an explicit escalation flag; the 40 percent
  ration and its reason stay in force, and so does the run-on-the-
  session-model half.
- **D6** — accounting becomes a role mix plus an escalation share.
- **D7** — `role` is required on a cell, as `lane` is.
- **D8** — a recommended vocabulary ships as authoring guidance only.
- **D9** — backfill the 506 stored cells.
- **D10** — an explicit-only runtime fallback chain, separate from D2;
  a chain key may name a role **or a concrete model**.
- **D11** — the chain's error gate: transient and infrastructural only.
- **D12** — `--kind extract`; its fate is a planning call, answered
  below, and its consequences are carried by S3.

## Discovery

Inspected across the discovery map and this plan's review wave.

**The two silent-resolve sites.** `verbs/drivers/models.rs:328` —
`let s = if CONFIGURABLE_SLOTS.contains(&slot) { slot } else { "generation" };`
— coerces *any* unrecognized slot name to `generation` and returns that
model. Under an open role set a mistyped or unconfigured `role` resolves
the generation model while `prepare.rs:1067` stamps `tier_source:
"cell"`, so the dispatch record asserts the cell chose it. Wrong model,
no warning, complete work. The second site is `slot_for_kind`'s
catch-all `_ => "advisor"` (`prepare.rs:34-40`). **Both must be closed
in S2**; closing one and leaving the other reproduces the defect.

**The guard's config is a struct, not a map.** `hooks/model_guard.rs:256-269`
declares fixed `Models`/`Slots` structs, `:374-395` normalizes literal
per-field defaults, and `configured_model_set` hardcodes
`["extraction","generation","review"]` at `:495` and `:510`. An open set
is therefore a struct-to-map rewrite of the guard's whole config layer,
not a list edit. It is also a hard dependency: a model configured under
`models.claude.test` fails the membership check at `:797-811` and
**denies**, so the cell work cannot land before it.

**Closed lists on the role axis — twelve, not four.** Beyond the four
private `MODEL_TIERS` copies: `CONFIGURABLE_SLOTS` and
`MODEL_NORMALIZE_SLOTS` (`models.rs:37`, `:40`, duplicated at
`status_full/mod.rs:70`), `CLAUDE_TIERS`/`CODEX_TIERS`
(`model_guard.rs:192-193`), `AGENT_TIER_BY_NAME` and the two
`AGENT_TIER_DEFAULTS_*` (`onboard/templates.rs:222-238`),
`PINNED_AGENT_TYPE` (`model_guard.rs:605`) with its twin
`pinned_agent_type` (`verbs/drivers/guard.rs:32-39`), and
`dispatch_kind_for_tier` (`model_guard.rs:660`).

**The fixture blind spot.** Roughly 100 raw cell fixtures across 17 Rust
files write JSON straight to disk and bypass `validate_new_cell`
entirely. They will not break when `role` becomes required — they will
silently resolve the default. A green suite would prove nothing. S4
carries an explicit fixture sweep for this reason.

**Two rev-1 claims, corrected.**

1. `slot_for_kind`'s catch-all cannot be reached by a *typo*: `kind` is
   enum-gated against `DISPATCH_KINDS` at `prepare.rs:1358` and again at
   `:647`. The hazard is a newly added kind with no arm — which is what
   CONTEXT.md said and rev 1 overstated.
2. `effort` is dropped on **both** runtimes, not only claude. The claude
   Agent tool takes no effort parameter, so that half is a harness
   limit; but `prepare.rs:1011-1029` is a codex `spawn_agent` arm that a
   `Resolved::Model` falls into and it emits neither `model` nor
   `reasoning_effort`, on the one runtime that demonstrably accepts it
   (`:899`). S10's recorded non-delivery must name the codex drop
   separately, or the harness reason will read as covering it.
3. The five skill trees are **not** byte mirrors. `diff -rq skills
   .claude/skills` reports three files differing —
   `bee-hive/references/gates-and-delegation.md`,
   `bee-swarming/references/swarming-reference.md`,
   `bee-swarming/references/worker-details.md` — plus render metadata,
   and the trees differ from each other too. The cause is intended
   runtime-conditional rendering (`<!-- bee:only claude -->` blocks),
   not drift. The proof for D8 is therefore "the regen chain runs clean
   and each tree matches its own render", never byte identity.

## Approach

**Recommended path.** Collapse the parser (D1), open the resolver and
close both silent-resolve sites (D2), rebase the agent-name mapping off
tiers onto roles (D12's carrier), give the cell its required role
(D3, D7), backfill history (D9), rehome the escalation ration (D5),
then retire `tier` and move accounting (D4, D6), and finally add the
runtime chain (D10, D11) and the remaining surfaces.

**Ordering is load-bearing in three places, and rev 1 got one wrong.**

- D1 is locked as first; every later slice writes into the shape its
  parser owns.
- S2 must precede the cell work: without the guard's map rewrite, a
  configured role name is *denied* by `configured_model_set`.
- **The backfill must precede the ration rehoming.** Rev 1 ordered
  escalation before backfill. Three counters read `cell["tier"]`
  store-wide — `handlers_close.rs:1088` (the 40 percent refusal),
  `status_full/cells.rs:584`, `session_preamble/store.rs:311`. Between
  a rehoming and a backfill, `ceiling` would be 0 and `tiered` 0, so
  `share = 0.0` (`handlers_close.rs:1103`) and the refusal could never
  fire, while `SCARCITY_MIN_TIERED = 3` (`status_full/mod.rs:120`) would
  silence the preamble warning. D5 says the ration stays "unchanged in
  force"; rev 1's order broke exactly that. S5 now lands before S6.

**Rejected alternatives.**

- Move the cell field first, collapse the parsers afterwards — rejected
  by D1, and each change would be written twice.
- Keep `tier` alongside `role` in a deprecation window — rejected by D4;
  the 506-cell measurement shows nothing to deprecate gradually, since
  484 of 506 carry no signal.
- Put the skill-payload guidance in the final slice — rejected on
  review: between the cell field landing and the guidance landing, five
  skill trees would advertise a payload `bee cells add` now refuses, and
  this repo builds itself with bee. The guidance edit moves into S4
  beside the field.

**D12's fate — the planning call CONTEXT.md deferred.** `--kind extract`
(`8dad7c2e`) does **not** ship as a fifth dispatch kind. Under D2 a
read-shaped consumer asks `["read", "code"]` and fall-through does the
rest, so a new kind would buy what the resolver now does for free.

But absorption is only honest if the decision's recorded *consequences*
get an owner, and rev 1 gave them none. `8dad7c2e` states its intent as
"bee-extract was rendered, onboarded and documented while prepare could
never return it", and names three consequences: the guard's
`dispatch_kind_for_tier` arm, the widened `a2f85972` member set, and the
swarming reference's `subagent_type: bee-extract` instruction becoming
reachable. An ordered role list resolves a *model*; it never touches
`pinned_agent_type` (`prepare.rs:810-811`), so after the cell work
`dispatch prepare` still could not return `bee-extract`.

**S3 is that carrier.** It rebases the whole agent-name mapping off
tiers onto roles, retires the six live "no --kind for the extraction
tier yet" strings (`model_guard.rs:768, 772, 828, 855, 892, 917`) and
the test that pins them (`:1631`), widens the `a2f85972` member set, and
resolves the swarming-reference contradiction. The deviation from
`8dad7c2e`'s literal spelling is named here; its substance ships.

**Risk map.**

| Component | Risk | Proof needed |
|---|---|---|
| Shared parser (D1) | MEDIUM | Same assertions, **retargeted** — see the S1 note below; "unchanged" is not achievable. |
| `models.rs:328` coercion | **HIGH** | An unconfigured or misspelled role never resolves a model silently: it warns and falls through, or refuses. The single most important positive test. |
| `slot_for_kind` catch-all | **HIGH** | An unknown kind refuses instead of resolving the advisor slot. |
| Guard config struct→map (D2) | **HIGH** | A role configured outside the old three slots passes `configured_model_set` instead of denying. |
| Agent-name rebase (S3) | MEDIUM | `dispatch prepare` can return every rendered agent; no refusal text names a tier that no longer exists. |
| Cell `role` required (D7) | MEDIUM | Typed FIX refusal; **and** the fixture sweep — no fixture silently resolves a default. |
| Backfill (D9) | MEDIUM | Idempotent; dry run reports 484 / 2 / 20 before any write; re-running changes nothing. |
| Escalation ration (D5) | **HIGH** | The 40 percent refusal fires on the flag exactly as on the tier value, including `--reason`, with no window where the share reads 0. |
| `tier` retirement (D4) | **HIGH** | The inventory below is closed, item by item — a grep alone returns ~25 false positives from unrelated `tier` words. |
| Runtime chain (D10, D11) | MEDIUM | A semantic failure never advances the chain. |
| Guidance text (D8) | LOW | Regen chain clean; each tree matches its own render. |

## Shape

**Feature outcome.** An operator writes `test: <a model good at tests>`
in `.bee/config.json`, marks a cell `role: test`, and that cell's worker
runs on that model — with no bee code change for the name `test`, and
with a **warning** if nothing configures it.

**Repo-reality basis.** bee already routes a per-cell dispatch judgment
through `prepare.rs:731-745`, already enforces a required cell field
(`lane`, `validate.rs:133-140`), and already renders four job-named
agents pinned to cost slots (`guard.rs:32-39`). This feature reuses all
three mechanisms; it invents no new plumbing.

| Epic | Capability / Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| E1 | One config parser | Two parsers of one shape already drifted 4-vs-5 with nothing intending it | S1 | Same assertions, retargeted |
| E2 | Open, fall-through resolution | Ends the dead-slot class **and** both silent-resolve sites | S2 | No path resolves an unconfigured name without warning |
| E3 | Agent names ride roles | D12's carrier; `bee-extract` is rendered but unreturnable | S3 | Every rendered agent is reachable through the one door |
| E4 | The cell declares its role | The actual user-visible capability | S4 | End-to-end: configure a name, mark a cell, watch it resolve |
| E5 | History stays honest | 506 stored cells; the ration must never see a zero window | S5, S6 | Idempotent backfill; ration fires identically |
| E6 | `tier` retires | The selector and its accounting move together | S7 | Closed inventory, item by item |
| E7 | Runtime chain | The one failure a chain absorbs is a quota wall | S8, S9 | A semantic failure never advances the chain |
| E8 | Surfaces | Authors need the vocabulary; the preamble publishes slots | S10 | Regen clean; recorded non-deliveries named |

**Slice queue.**

| Slice | Contents | Depends on |
|---|---|---|
| **S1** | One `resolve_role` in drivers; the guard calls it; `resolve_advisor` collapses. Resolve the cli-purpose parameter at each of the guard's five call sites, and keep `Resolved::Native` out of `configured_model_set`. | — |
| **S2** | Ordered-list resolution + fall-through, with the last entry guaranteed to resolve. Guard config struct→map; `configured_model_set` stops hardcoding three slots. **Close `models.rs:328` and `slot_for_kind`'s catch-all.** Warn on unconfigured. Retire `CLAUDE_TIERS`/`CODEX_TIERS`, `CONFIGURABLE_SLOTS`, `MODEL_NORMALIZE_SLOTS`. | S1 |
| **S3** | Agent-name mapping rebased onto roles: `PINNED_AGENT_TYPE`, `pinned_agent_type`, `dispatch_kind_for_tier`, `AGENT_TIER_BY_NAME`, `AGENT_TIER_DEFAULTS_*`, `{{TIER_MODEL}}` rendering. Retire the six extraction-refusal strings and the test pinning them; widen the `a2f85972` member set; fix the swarming-reference contradiction. **Carries D12.** | S2 |
| **S4** | Cell gains required `role` (D7) with its typed refusal; ship config defaults for `code`, `read`, `review`, `advisor` (D3); dispatch asks the ordered list headed by the cell's role; **the skill-payload and doc guidance move here, not to S10**; the ~100-fixture sweep. | S3 |
| **S5** | Backfill the 506 cells (D9) via a dry-runnable, idempotent migration verb. Lands **before** any ration rehoming. | S4 |
| **S6** | Escalation flag (D5): the ration, `--reason`, and the run-on-the-session-model half (`models.rs:322` `Resolved::Inherit`) all rehomed onto the flag. | S5 |
| **S7** | `tier` retires as a selector (D4) and accounting moves (D6). Closes the full inventory below. | S6 |
| **S8** | Runtime chain config and resolution (D10), explicit-only, **including model-keyed and `provider/*` chains**, recorded on the dispatch. | S2 |
| **S9** | The error-class gate (D11) with the semantic-failure negative tests. | S8 |
| **S10** | Preamble published-slots block; config reference; recorded non-deliveries (`effort`, on **both** runtimes). | S7, S9 |

**The `tier` retirement inventory (S7), closed item by item.** A grep
returns ~25 false positives — `verbs/triggers/mod.rs`,
`herding/split_lock.rs`, `herding/wave.rs` and `guard.rs`'s
`logical_tier` all use the word for something else. The real list:
`handlers_close.rs:1114`, `:1141`, `:1146-1166`; `cells/util.rs:78`;
`handlers_write.rs:437`; `validate.rs:164-172`;
`state_group/workers.rs:86-101, 125-127, 147-150, 187-193`;
`router.rs:68`; `cells/mod.rs:11`; plus six sites rev 1 missed —
`devtools/statusline.rs:368-372` (the `tiered` flag),
`hooks/cli_shape.rs:675, 725-734, 951-958` (schema and three tests that
go red the moment the verb goes), `status_full/build.rs:288` (the
`tier_mix` key in `bee status --json`, a **public-contract rename**),
`verbs/drivers/guard.rs:43, :101` (`logical_tier` inside an object its
own comment marks frozen), `handlers_close.rs:370` (cap-refusal FIX text
naming `--tier`, which goes stale rather than red), and
`handlers_close.rs:1137` (`trace.tier_reason`, persisted on 20 stored
cells — D5 preserves the reason, so this slice names who renames the
key).

**Answers to CONTEXT.md's deferred-to-planning questions.**

1. *Does `--kind extract` ship?* No — absorbed, with S3 carrying its
   consequences. Reasoned above.
2. *Where the escalation flag lives.* A boolean on the cell plus the
   existing reason string renamed to match, **not** a reserved role
   name. A reserved name would be the one exception to D2's open set,
   and validation would have to special-case it in every author path.
3. *Whether `effort` is delivered.* No. Recorded as a non-delivery in
   S10, naming the claude harness limit and the codex `spawn_agent`
   drop separately.
4. *How the backfill is applied.* A one-time migration verb with
   `--dry-run`, idempotent, reporting its 484 / 2 / 20 counts before
   writing. Not lazy-on-read: three counters scan the whole store, so a
   partially migrated store would misreport the ration.
5. *The ordered list per consumer.* Cell execution `[<cell role>,
   "code"]`; a read dispatch `["read", "code"]`; a review dispatch
   `["review", "code"]`; the advisor `["advisor"]` alone — no
   fall-through, preserving decision `4faf1de9`. `code` is the backstop
   and resolves to the runtime's built-in default when unconfigured,
   which is what makes D2's last-entry guarantee true.

**Current slice to prepare: S1 + S2.** Together they are demonstrable
without touching a cell: configure `models.claude.myrole`, ask for it,
watch it resolve; ask for an unconfigured name, watch it **warn** and
fall through rather than silently landing on `generation`. S1 alone is a
refactor with nothing to show, which is why the walking skeleton is the
pair.

**A note on S1's proof, corrected from rev 1.** "Existing suites pass
unchanged" is not achievable and claiming it would be dishonest: four
tests call the guard's private three-argument `resolve_tier` directly
(`model_guard.rs:1252, 1256, 1260, 1266`) against the guard's own
`Resolved` enum, whose variants differ in shape from the drivers enum.
Deleting the private function forces those four to be rewritten. The
honest proof is **same assertions, retargeted** — every behavior
currently pinned stays pinned, with the call sites moved. Two real
behaviors also need a decision rather than a lift: the guard's parser is
purpose-blind and refuses every cli slot unconditionally
(`model_guard.rs:441`, `:456`) while the surviving one is
purpose-parameterized (`models.rs:341-348`), so each of the guard's five
call sites must be given a purpose deliberately — `:978`, driven by a
`[bee-tier: …]` marker, is genuinely ambiguous; and `Resolved::Native`
carries a readable model string that must **not** widen
`configured_model_set` (`:497`).

## Test matrix

High-risk: probes per applicable dimension. Each cell's writer judges
existing coverage first and authors only the gap.

**Scale, stated honestly for the `proof-weakening` flag.** 120
tier-touching tests exist repo-wide: `model_guard.rs` 31 of 38,
`verbs/drivers/tests.rs` **51 of 172** (the largest block, and the one
S1/S2 hit first), `verbs/cells/tests.rs` 16, `status_full/tests.rs` 9,
`session_preamble/tests.rs` 6, `cli_shape.rs` 3, `drivers/guard.rs` 2,
`state_group/tests.rs` 1, `write_guard/tests.rs` 1. Every cell that
touches a test in this set records the **retarget / delete** split on
its cap: a retargeted test keeps its assertion and moves its call site;
a deleted test must name the decision that made its subject cease to
exist. A cell that deletes without naming one is a proof-weakening cap
and is refused.

| # | Dimension | Applicable | Probes |
|---|---|---|---|
| 1 | User types | No | bee has no actor model. |
| 2 | Input extremes | **Yes** | Empty role string; whitespace-only; a 500-char name; a name colliding with a reserved word; a one-entry list; an empty list. |
| 3 | Timing | No | Resolution is synchronous. |
| 4 | Scale | **Yes** | The backfill over 506 cells; 50 configured role names; a list walked to its last entry. |
| 5 | State transitions | **Yes** | A pre-change cell read after; a cell mid-flight during backfill; a worker registered with the old `--tier`; the ration observed across the S5→S6 boundary, asserting no zero-share window. |
| 6 | Environment | **Yes** | claude / codex / opencode; herding-shaped, cli-shaped and explicitly-null slots. |
| 7 | Error cascades | **Yes** | Every chain entry exhausted; a chain naming a vanished model; a chain looping to its own head. |
| 8 | Authorization | No | No trust boundary crossed. |
| 9 | Data integrity | **Yes** | Backfill idempotence; an interrupted partial backfill; the 484 / 2 / 20 counts asserted before write; `trace.tier_reason` survives its rename on all 20 cells. |
| 10 | Integration | **Yes** | `dispatch prepare` payload unchanged for callers passing no role; the guard's marker path; onboarding's rendered agent files; **the `tier_mix` key rename in `bee status --json`, which is a public-contract change and needs its own probe**. |
| 11 | Compliance | No | No regulatory surface. |
| 12 | Business logic | **Yes** | The ration fires on the flag exactly as on the tier value; a semantic failure never advances a chain; a configured role is always obeyed (`72f3d6dd`); an unconfigured name **never** resolves silently. |

The two tests without which this feature is not implemented: **an
unconfigured or misspelled role never silently resolves a model**, and
**a red test result never advances a fallback chain**.

## Scope integrity

Every locked decision D1–D12 ships. No slice is a budget candidate:
rev 1 ranked S7–S9 as droppable, which would have dropped `50808d48`
and `4eaf1b71` without a supersede, and CONTEXT.md ranks nothing. If
the work does not fit, the answer is SPLIT RECOMMENDED — slice
boundaries the owner chooses between, never a quiet reduction.

Should a split be needed, the honest boundary is after S7: S1–S7 deliver
the role capability end to end; S8–S10 deliver the runtime chain and the
remaining surfaces. Both halves would still be committed work, with the
second half's decisions intact and unshipped, and that is the owner's
call to make, not the plan's.

## Out of scope

- Capability as a declarative requirement (vision, long context).
- Per-provider model catalogues, ranking, or auto-selection.
- `ceiling` becoming configurable — decision `0015` stands.
- Delivering `effort` — recorded as a known non-delivery in S10, for
  both runtimes, with their different causes named.
- Anything about what a worker does once dispatched.
