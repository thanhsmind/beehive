---
artifact_contract: bee-plan/v1
mode: high-risk
# approved_gate2: <unset>
---

# Plan: SLP Contract Status + Original Request

Mode: `high-risk` — 4 risk flags: data-model, public-contracts,
covered-contract-change, multi-domain.
Why this is the least workflow that protects the work: both halves change
doors every worker passes (the claim door and the dispatch door), and one
half widens a validation pattern the whole decision log is written against —
a shape that is wrong here is wrong for every future dispatch, so the shape
is gated before a line is written.

Approach is folded in as `## Approach` rather than a separate `approach.md`:
the rejected alternatives are one line each and the risk map is six rows,
so a standalone file would only repeat this one.

Revision note: this is plan rev 2. Rev 1 went through an independent
plan check that raised three P1s — the citation field does not hold store
decision ids, the mint trap refuses everything on day one, and `role: test`
is a dead signal. All three are answered below, in Discovery 5-7 and in the
Approach. Every anchor below was re-verified by that check against
`37f6ae3a`; the four wrong anchors rev 1 carried are corrected.

## Requirements (from CONTEXT.md)

- **D1** — contract settled/unsettled status is a DERIVED view over the
  decision log; bee keeps no hand-maintained contract registry.
- **D2** — the label is the tag convention `contract:<name>` over the ACTIVE
  decision set: settled = active decision with no waiting trigger; unsettled
  = decision whose trigger is waiting or due.
- **D3** — cells cite contract decisions in the EXISTING `cell.decisions`
  field, and a prepare/claim-time tripwire refuses the dispatch when a cited
  decision is retired or trigger-waiting.
- **D4** — a test-writing cell that cites NO contract decision is refused
  (the mint trap).
- **D5** — the user's verbatim original request rides every cell and dispatch
  as an immutable field; intermediate layers may only ADD guidance.
- **D6** — `bee intent`'s existing verbatim anchor serves D5: its `request`
  field, under its DO-NOT-PARAPHRASE framing, is read at dispatch prepare and
  rendered into every worker, gather, reviewer and advisor prompt template.

## Discovery

Two advisor-tier reads plus one independent plan check over the live tree at
`37f6ae3a` (2026-08-29). Seven findings shape the work.

1. **The locked tag spelling is illegal today.** `tag_pattern_test`
   (`packages/bee-rs/crates/bee/src/verbs/decisions/scanners.rs:476`) enforces
   `/^[a-z0-9][a-z0-9-]*$/`; a colon is refused by `normalize_tags`
   (`scanners.rs:498`). It has exactly three call sites repo-wide
   (`scanners.rs:498`, `verbs_write.rs:271`, `tests.rs:1528`), so one widening
   covers both `decisions log` and `decisions tag`. No path or filename is
   ever built from a tag; query filtering is exact string match
   (`read.rs:361/389/476`); rendering uses `tags[0]` as a markdown section
   name (`render.rs:196`), where a colon is inert. Two side effects to carry:
   `TAG_PATTERN_DISPLAY` (`scanners.rs:474`) must change with the predicate or
   every refusal message lies, and `classify_decision_tags`
   (`scanners.rs:606-612`) appends each unknown tag to
   `docs/decisions/taxonomy.json` `candidates`, so the checked-in taxonomy
   grows one candidate per contract name. The taxonomy never refuses an
   unknown tag — only an empty tag list. The `contract:` namespace is free:
   2442 decision events, 5 tagged bare `contract`, 0 tagged `contract:*`.
2. **"Retired" has no implementation.** The store has supersession
   (`decisions/read.rs:156`), redaction (`read.rs:162`) and archiving
   (`decisions/mod.rs:116`) — no `retired` state. D3's "retired" therefore
   resolves to "not in the active decision set", which
   `active_decisions(root, false)` (`read.rs:146`) already computes.
3. **Triggers key to a decision by SHORT8, and reading them WRITES.**
   `TriggerRecord.decision` holds the first 8 characters
   (`verbs/triggers/mod.rs:75-85`, field written at `mod.rs:402`). The only
   reader, `read_and_evaluate` (`mod.rs:223`), is private AND persists a
   waiting-to-due flip mid-read (`mod.rs:249-251`) — it cannot be called from
   a refusal path that promises zero mutation. Two more facts: all 14 live
   records are `tier: manual`, and a manual trigger never reaches `due`
   (`mod.rs:262-266`) — so under D2 a contract decision with a manual trigger
   stays unsettled until a human resolves it, by design; and 4 of the 14
   `decision` keys are not short8s at all (`herding-`, `P72`, `p-c6e61d`,
   `wayfindi`), so the join must not treat a junk key as a match.
   Short8 collisions among the 2424 real decision ids: 0.
4. **The claim door is the funnel; it reuses a pre-scan, it does not load.**
   `cells claim`, `cells claim-next` and `dispatch prepare --claim` all reach
   `claim_cell_cross_session_ex` (declared
   `verbs/cells/handlers_write.rs:1400`). The `RED_BASE` slot
   (`handlers_write.rs:1478-1486`) is genuinely pre-mutation — its own comment
   says so, it calls `release_claim` before returning, and the status write is
   far below at `:1572`. At that point the cell data available is the
   caller-supplied `cell_for_budget` (`handlers_write.rs:1407`), which is
   `Some` on all three doors (`handlers_write.rs:1221`,
   `handlers_select.rs:827`) — the check reuses that pre-scan rather than
   re-reading. One hole: `cells add` preserves any truthy `status`
   (`verbs/cells/validate.rs:468`) and nothing validates it, so a cell added
   with `"status":"claimed"` never passes the claim door at all.
5. **`cell.decisions` does not hold store decision ids.** Measured over the
   92 live cells: 48 cite something, 81 citations total, and only 11 (13%)
   resolve to a decision in `.bee/decisions.jsonl`. The entry-length
   histogram is `{2: 61, 3: 5, 8: 11, 24: 1, 25: 3}` — the field is dominated
   by LOCAL D-IDs (`D1`, `D2`) from a CONTEXT.md table. Nothing validates the
   field (`verbs/cells/validate.rs:224,473` treat it as a bare string array).
   A tripwire that refuses every unresolvable entry would refuse 87% of citing
   cells; one that ignores them would be the decoration D3 forbids.
6. **`role == "test"` is a dead signal.** All 92 cells carry `role: code`;
   zero carry `test`. The vocabulary is deliberately open and never
   membership-checked (`verbs/cells/validate.rs:43,48`), so the arm can never
   be relied on — it can only ever be an additional trigger, never the signal.
7. **The path heuristic catches a minority, and the "accepted hole" is the
   majority.** A test-path classifier already exists and is broader than the
   globs rev 1 proposed to invent: `path_looks_like_test`
   (`verbs/cells/finish_support.rs:693`), covering a bare `test`/`tests`
   segment and `.test.` filenames, with anti-false-positive notes. It fires on
   30 of 92 cells with 0 false positives. But 67 of 92 cells name tests in
   their title or action, and only 27 of those 67 (40%) are caught by any path
   signal; 7 cells declare no `files` at all. In this repo the writer owns
   tests TDD-style inside the source file it was already touching, so
   "`role: code` adding a `#[cfg(test)]` module" is not a narrow hole — it is
   the dominant shape.

Also verified: `prompt_body_for` (`verbs/drivers/prepare.rs:727`) fills two
vars on the non-cell arm (`prepare.rs:758-761`) and nine on the cell arm
(`prepare.rs:779-789`); pass 2 of `render`
(`verbs/drivers/prompt.rs:103`, second pass from `:165`) refuses any template
var missing from its arm's slice — a new `{{original_request}}` must be added
to BOTH arms or every dispatch of the untouched kind dies at the door.
`bee dispatch prepare` reads no intent today.
`prompt_body_for` takes no session parameter, and `--session-id` is only
meaningful with `--claim`, which is refused for every non-cell kind
(`prepare.rs:2053-2058`) — so the session arm of the intent key walk is
unreachable from the dispatch door.

## Approach

### Half B (D5, D6) — the walking skeleton, S1

Expose the intent anchor reader from `verbs/intent_group.rs` as a
crate-visible, **feature-keyed** read and call it from `prompt_body_for`,
rendering the verbatim `request` under the existing `PRECOMPACT_HEADER`
framing (`intent_group.rs:250`) into a `{{#if original_request}}` block in all
four templates. The var joins both var slices.

**The deferred question "which anchor does a featureless dispatch read?" is
answered: none.** The resolution order at the dispatch door is the cell's own
`feature`, then the active feature from state, then nothing — the
`DEFAULT_INTENT_KEY` fallback (`intent_group.rs:113-131`) is deliberately NOT
taken. Evidence: `read_anchor_at` (`intent_group.rs:216`) applies no TTL and
no staleness check, and the live `.bee/intent/default.json` today carries a
request from 2026-08-25 about an unrelated, already-shipped bug. Rendering
that under a header reading "VERBATIM · DO NOT SUMMARIZE · DO NOT PARAPHRASE"
is worse than rendering nothing — it would be the exact meaning-drift D5
exists to prevent. The session arm is unreachable from this door anyway.
A gather or advisor dispatch with no active feature renders no block, byte
for byte as today.

### Half A (D1-D4) — S2 through S5

**S2 — the tag pattern (D2, verbatim).** Widen the slug predicate to admit at
most one interior colon, so `contract:<name>` is writable exactly as locked,
and change `TAG_PATTERN_DISPLAY` with it. Backward-compatible: every existing
slug still validates.

**S3 — the derived status read (D1, D2).** A pure read, no store, no
registry. Add a read-only trigger reader beside `read_and_evaluate` that
returns records without the waiting-to-due write, then join
`active_decisions` against it on short8. The result for one decision id is
`settled` (active, no waiting/due trigger), `unsettled` (active, trigger
waiting or due), or `unknown` (not in the active set — superseded, redacted,
archived, or never logged). Junk trigger keys match nothing.

**S4 — the citation tripwire (D3).** In the claim body beside `RED_BASE`, and
again in `prepare`'s `kind == "cell"` arm so a cell claimed BEFORE its cited
decision changed state is still refused at dispatch — D3's letter names the
dispatch, and the claim door cannot see that window.

Answering Discovery 5, the tripwire's subject is defined, not assumed: an
entry in `cell.decisions` is a **store citation** only when it resolves to a
decision in the store (full id, or an unambiguous short8 prefix). A local
D-ID like `D1` is a pointer into a CONTEXT.md table, not a store citation, and
is passed over silently. The tripwire refuses when a store citation resolves
to `unknown` or `unsettled`; it never refuses on an unresolvable local id.
S4 also closes the `cells add` status hole (Discovery 4) by refusing a new
cell whose `status` is anything but `open` — without it the guard is
bypassable in one line of JSON.

**S5 — the mint trap (D4), warn-then-deny.** The signal, stated with its
limits rather than around them:

- **Armed arm (can refuse):** the cell declares a test-shaped path in `files`
  — reusing `path_looks_like_test` (`finish_support.rs:693`), not a new glob
  set — or carries `role: test`. 30 of 92 live cells, 0 false positives.
- **Advisory arm (warns, never refuses):** any other cell whose title or
  action names test writing. 67 of 92 cells. Too soft to refuse on; loud
  enough to be worth saying.
- **The hole, named:** a `role: code` cell adding a `#[cfg(test)]` module
  inside a source file it was already touching is the DOMINANT test-writing
  shape in this repo (Discovery 7), and the armed arm does not catch it. The
  advisory arm is what covers it. Closing it properly needs a real cell field
  declaring test intent — which D1's "nothing new to forget to update" spirit
  argues against adding on a guess, so it is deferred with this measurement
  attached rather than invented here.

The rollout is a ramp, not a scope cut, following the precedent already in the
same function: `NO_ROUTE_RECORD` warns once per session, then refuses
(`handlers_write.rs:1465-1471`). The refusal ships fully built; while the
store holds zero `contract:*` decisions the armed arm can be satisfied by
nobody, so it warns. It refuses from the moment the first `contract:*`
decision exists. That condition is derived from the same read S3 builds — no
new switch to forget.

### Rejected alternatives

- Spell the tag `contract-<name>` to dodge the pattern change — rejected: D2
  locks the spelling `contract:<name>`, and widening is backward-compatible.
- Tripwire only at `dispatch prepare` — rejected: inline `tiny` cells never
  pass it.
- Tripwire only at claim — rejected: misses the claim-then-change window, and
  D3's letter names the dispatch.
- Refuse on every unresolvable `cell.decisions` entry — rejected: 87% of
  citing cells would be refused for using the field the way the repo has
  always used it.
- Fall back to the `default` intent anchor for a featureless dispatch —
  rejected: no staleness check exists, and the live default is four days
  stale and about other work.
- Invent new test-path globs — rejected: `path_looks_like_test` already
  exists and is broader.
- Add an `original_request` field to the cell record — rejected by the locked
  decision's own rationale: closed schema, N copies are N truncation chances.
- Build a contract-name to trigger reverse index — rejected: it is the second
  registry D1 refuses.

### Risk map

| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| Tag slug pattern widening | MEDIUM | Every decision ever logged validates through it; too loose and a typo becomes a namespace | One interior colon accepted; no colon still accepted; two colons, leading colon, trailing colon all refused; `TAG_PATTERN_DISPLAY` matches the predicate; every tag already in `.bee/decisions.jsonl` still validates |
| `{{original_request}}` in 4 templates | MEDIUM | A var missing from an arm's slice kills every dispatch of that kind at the door | Byte-identical-when-absent across every runtime x kind, mirroring `drivers/tests.rs:6641`; a template var absent from its slice fails loudly, with a test that proves it |
| Feature-keyed-only anchor read | MEDIUM | Silently rendering nothing is safe; silently rendering the wrong request is not | A featureless dispatch renders no block even when `.bee/intent/default.json` exists and is non-empty |
| Read-only trigger reader | MEDIUM | The existing reader writes; a refusal path that writes is not a refusal path | Trigger files are byte-identical after a refusal; a junk `decision` key matches no decision |
| Citation tripwire (D4 excluded) | HIGH | A false refusal stalls every worker | Refuses on unknown and on waiting/due; ALLOWS on active-no-trigger, on resolved trigger, on a local D-ID, on an empty list; zero mutation on every refusal |
| Mint trap | HIGH | Refusing on a heuristic | Each arm tested; warns while zero `contract:*` decisions exist; refuses once one exists; the named hole has its own test asserting it passes |

## Shape — epic map

**Feature outcome.** A worker can tell whether the contract it is about to
write tests against is settled, and is refused when it is not; and the user's
own words ride every dispatch untouched.

**Repo-reality basis.** Every piece reuses a door that exists: the decision
log with supersession, forced triggers and a tag query; the intent anchor with
its DO-NOT-PARAPHRASE header; the shared claim body with its typed
pre-mutation refusal grammar; the single dispatch-prepare render; and
`path_looks_like_test`.

| Epic | Capability / Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| E1 | The user's words reach the worker | D5/D6; the anchor survives compaction today but never reaches a dispatch | S1 | Absent-anchor byte-identity per runtime x kind; a present anchor appears verbatim in all four rendered prompts; a featureless dispatch renders nothing |
| E2 | Contract status is writable, then readable | D1/D2; the label cannot be logged at all until the pattern admits it, and cannot be refused on until it can be read | S2, S3 | Pattern round-trip; settled/unsettled/unknown derived correctly over a read-only trigger join |
| E3 | The tripwire has teeth | D3/D4; a citation nothing checks is decoration | S4, S5 | Typed zero-mutation refusals at both doors, allow-cases pinned; the mint trap's ramp and its named hole both tested |

**Slice queue.**

| Slice | Content | Deps |
|---|---|---|
| **S1 (current)** | E1 — intent anchor rides every dispatch, feature-keyed only | none |
| S2 | E2 — tag pattern admits one colon | none |
| S3 | E2 — read-only trigger reader + derived contract status | S2 |
| S4 | E3 — citation tripwire at both doors + `cells add` status hole | S3 |
| S5 | E3 — mint trap, warn-then-deny | S4 |

S2 is split from S3 because the pattern widening is the precondition for the
user to log any `contract:` decision at all, and until one exists S3 has no
real data to test against. S4 is split from S5 because the two refusals share
no signal and no rollout profile — one is a deterministic join, the other a
heuristic with a ramp — so one red proof would not say which rule was wrong.

**Current slice to prepare: S1.** It is the walking skeleton — end to end,
real behavior, no stubs: a real anchor read from the real store, rendered into
the real templates, visible in a real `bee dispatch prepare` payload.

## Test matrix

High-risk, so the 12 edge dimensions, applicable ones only. Each cell's writer
judges existing coverage first and authors only the gap.

| Dim | Probe |
|---|---|
| 2 Input extremes | Empty anchor; whitespace-only `request`; a request containing `{{` and `{{#if}}` (template injection); a very long request; non-ASCII and Vietnamese text preserved byte for byte |
| 4 Scale | A `cell.decisions` list with many ids — one pass over the active set, not N store reads |
| 5 State transitions | Decision active, then superseded between claim and dispatch (the window the prepare-arm check exists for); trigger waiting to due to resolved; the first `contract:*` decision appearing flips the mint trap from warn to refuse |
| 6 Environment | No `.bee/intent/` directory at all; no `.bee/triggers/` directory; no taxonomy file |
| 7 Error cascades | Corrupt intent JSON; unreadable trigger file — the dispatch degrades to "no anchor" / "unknown", never fails the dispatch |
| 9 Data integrity | Two decision ids sharing a short8; a trigger `decision` key that is not a short8 at all; trigger files byte-identical after any refusal |
| 10 Integration | A template carrying `{{original_request}}` while its arm's var slice does not — fails loudly at the door, with a test |
| 12 Business logic | One valid citation among invalid ones; a local D-ID only (allowed); an empty list on a non-test cell (allowed); an empty list on a test-path cell (warn, then refuse); leading/trailing/double colon tags |

Triad baseline for every cell: happy path, the refusal path, and the
byte-identical-absent path.

## Out of scope

- Any new store, registry, or interface enumeration (D1).
- A contract-name to trigger reverse index (CONTEXT.md "Deferred Ideas").
- Changing who may settle a contract — decisions stay user-gated.
- A new cell field declaring test intent — deferred with the 40%/67-of-92
  measurement attached (Discovery 7), not invented on a guess.
- Repairing `opencode_plugin_contracts` (see Base state); filed as PBI
  `p-45481f13`.

## Base state

`cargo test --release --no-fail-fast` on `wt/slp-contract-original-request`
(base `37f6ae3a`): every target green except one —
`-p bee --test opencode_plugin_contracts`, test
`every_registered_write_or_read_capable_opencode_tool_is_mapped_or_named_as_a_gap`,
which panics with "opencode tool-registry derivation broke: the installed
opencode binary's tool-id Set literal ... was not found — has the installed
opencode-ai version changed shape?".

This is a property of the LOCAL opencode install, not of the repo, and it
touches no file this feature touches. Named deviation from the green-base
rule: this feature's cells claim with `--fix-first` naming this target, and
the repair is filed as PBI `p-45481f13` rather than folded into cluster 4.
Every cell's own proof line is still scoped-green over what it changed.
