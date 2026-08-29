---
type: bee.area
title: Workflow State — the unified command entry point and its catalog
description: "One entry point owning the single implementation of all nine verb groups, publishing a machine-readable catalog of every command it accepts, validating a request before dispatching it, and signalling a changed discovery surface without disturbing any command's ordinary output."
timestamp: 2026-07-26
bee:
  id: workflow-state-dispatch
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: [30606de4-5fae-4c9d-9e3f-8f47a494f8a3 (one unified command entry point publishing a machine-readable catalog), bbc6bcea (shim-retire D1 — the legacy per-group forwarders are deleted; bee is the sole shipped CLI), 8ef2bae6 (cli-ergonomics D1 — exhaustive refusal, every problem + a runnable example in one message), "80b64c20 (packages-engine-move D1-D5: onboarding/distribution engine relocated to packages/bee/scripts, strict-flag validation universal, migration-tooling pattern)", "b0ef4f66 (packages-engine-move-3: capture add --text friction record corrected — never a silent no-op, message now names the actual unknown flag via a universal dispatcher-level check)"]
  sources: ["harness-integration-adopt cells hia-1 and hia-2 (traces and reports, 2026-07-12)", "dispatcher-unify cells du-1..du-6 (traces and reports, 2026-07-12, flushed capture stubs b6a2233c/9e68432b)", "docs/specs/workflow-state.md#B8", "docs/specs/workflow-state.md#R12", "docs/specs/workflow-state.md#R13", "docs/specs/workflow-state.md#E10", "docs/specs/workflow-state.md#E11", "docs/specs/workflow-state.md#P6", "docs/specs/workflow-state.md#P10", docs/history/packages-engine-move/]
  authoritative_for: "workflow-state: unified command discovery, validation, and dispatch"
---

# Workflow State — the unified command entry point and its catalog

An automated assistant can only call what it can discover. This concept owns the
one surface that makes the workflow discoverable: a single entry point that owns
the implementation, a catalog that describes exactly the commands that surface
really accepts, and a validation step that refuses a malformed request before any
record changes.

## Behaviors & Operations

**A cell dispatch describes itself by the cell's own title
(dispatch-description-intent, cell ddi-1, 2026-08-06).** The Agent-call
description a `--kind cell` prepare returns is built from the cell's
title, so an operator watching the run reads the work's intent, not a
generic label; a titleless cell falls back to the dispatch date.

**The unregistered-worker refusal names the door that registers
(reachable-remedy, cell rr-1, 2026-08-26).** Worker registration rides the
`dispatch prepare --claim` path, so the refusal a worker reads when it is not
registered names exactly that door — not a control-plane verb that itself
refuses when run from the worktree where the message is read. The principle it
pinned: a refusal's FIX line must name a remedy reachable from where the
refusal is read.

**Wave-batch dispatch (workflow-lessons wfl-4, 2026-08-11; scoped and bounded,
dispatch review P1/P2, 2026-08-12; unwind claim-taken precision, dispatch
review delta hpf-3, 2026-08-12).** `bee dispatch wave --runtime <rt>`
prepares every ready unclaimed cell of the current schedule wave through the
identical claim+reserve+payload path as per-cell `dispatch prepare --kind cell
--claim`, under auto worker names (`w-<cell>`). Output `{wave, skipped,
economics}`; one refusal (foreign claim, reservation conflict) lands its cell
in `skipped` with a typed reason and never aborts the batch. Per-cell prepare
stays the fallback and the single-cell path. This door mutates the shared
control plane, so — unlike its read-only sibling `cells schedule` — it always
scopes to exactly ONE feature: an explicit `--feature`, else the calling
session's bound lane, else the default record's own `feature`; nothing
resolving is a typed refusal, never a silent every-feature grab. `--limit <n>`
(a positive integer) caps how many cells of the wave are actually claimed —
the rest stand untouched, not reported — bounding a speculative claim batch to
what the caller can actually spawn workers for. A claim+reserve that fails
partway through (an unproven shape, not the door's own typed refusals)
reports whether the claim door itself ever took the claim: an untaken claim
(the door's own exotic-shape delegate fires before any claim mutation, even
over a cell another agent already holds) is never force-unclaimed — it lands
in `skipped` as `claim_refused` with no unwind note, because there is nothing
to unwind and forcing one would write straight through a claim conflict. A
claim the door DID take is unwound best-effort — claim, reservations, AND the
worker row `dp-r1` registered for it — before its cell lands in `skipped`; an
unwind that itself fails earns its own `unwind_failed` reason rather than
folding into `reservation_conflict`.

**Worker Result form (workflow-lessons wfl-1, 2026-08-11).** The rendered
worker prompt requires a fenced JSON Result form `{outcome, commit, files,
tests, deviations}` beside the final status token, and `cells finish --report`
validates exactly those five keys (unknown key refused, missing keys named)
into append-only `trace.report`; the flag absent keeps old finish behavior
unchanged. Tending reads the form, never parses worker prose.

**The user's verbatim request rides every dispatch
(slp-contract-original-request, cell scor-1, 2026-08-29).** `dispatch prepare`
reads the intent anchor and renders its `request` — unchanged, under the
anchor's own DO-NOT-PARAPHRASE header — into a conditional
`{{original_request}}` block carried by all four prompt templates
(`worker-cell`, `gather`, `reviewer`, `advisor`). The var joins BOTH var
slices of `prompt_body_for`: a template var missing from its arm's slice
kills every dispatch of that kind at the door, while the same omission inside
an `{{#if}}` is silent, so each kind owns a positive carrier test. Resolution
is **feature-keyed only** — the cell's own `feature`, then the active feature
from state, then nothing. The `default` key is never read from this door by
any route, derivation included: the anchor carries no TTL and no staleness
check, so a default anchor left by unrelated work days ago would be printed
as the user's own words under a verbatim banner, which is exactly the
meaning drift the rule exists to prevent. Nothing resolvable renders no
block at all, byte-identically to the pre-anchor payload. The anchor itself
is owned by
[`../hook-runtime/the-intent-anchor-and-compaction-survival.md`](../hook-runtime/the-intent-anchor-and-compaction-survival.md).

**A cell may not run against an unsettled or retired contract decision
(slp-contract-original-request, cell scor-4, 2026-08-29).** The cell's
existing `decisions` field is the citation slot, and one shared check reads
it at TWO doors: the claim body (which `cells claim`, `cells claim-next` and
`dispatch prepare --claim` all funnel through) and `dispatch prepare
--kind cell`. Both, not one: a cell claimed BEFORE its cited decision was
superseded, or before a trigger keyed to it reopened, slips a claim-only
tripwire entirely. The check refuses on `CONTRACT_RETIRED` (the cited
decision left the active set) and on `CONTRACT_UNSETTLED` (it is active but
carries an open `waiting`/`due` trigger), and it mutates nothing on either
path — the claim door releases its claim file and leaves the cell `open`,
and the trigger store is byte-identical because the read goes through the
non-evaluating trigger reader.

The half that decides whether the guard is usable: an entry that does not
resolve to a store decision is **passed over** — silently, refusing nothing.
`cell.decisions` is dominated by local `D1`-style ids pointing into a
CONTEXT.md table; only 13% of live citations resolve to the store, so
refusing on an unresolvable entry would stall 87% of citing cells. The
matching authoring-side repair is that `bee cells add` now refuses any
`status` but `open`: a payload could otherwise mint an already-`claimed`
cell that never passed the claim door at all.

**A test-writing cell that cites no contract decision is refused — the mint
trap (slp-contract-original-request, cell scor-5, 2026-08-29).** The tripwire
above asks "did the contract you cited move?"; this asks "did you cite a
contract at all?". Without it a contract nobody logged reads exactly like
"there is no contract", and the worker mints one by writing tests against it.
The two rules run at the claim door over ONE shared store read, with separate
typed codes (`CONTRACT_UNCITED` is the trap's), so a single red proof says
which rule fired.

The cell record has no field marking a test-writing cell, so the signal has
two unequal arms. The **armed** arm can refuse: the cell declares a
test-shaped path in `files` — judged by `path_looks_like_test`, the
classifier the repo already owns, never a new glob set — or carries
`role: "test"`. Measured over the 92 live cells the path arm fires on 30 with
0 false positives; `role: test` fires on 0, and the role vocabulary is open
and never membership-checked, so it is an additional trigger and never the
signal. The **advisory** arm only warns, in every state: any other cell whose
`title` or `action` names test writing, which is 67 of 92 cells — too soft to
refuse on, loud enough to say.

"Cites no contract decision" is precise, not "cites nothing": the trap fires
when none of `cell.decisions` resolves to a store decision carrying a
`contract:<name>` tag. A local `D1`-style id resolves to nothing and settles
nothing; a real store decision with no `contract:` tag does not satisfy it
either. This is the tag-AWARE rule, and it is the deliberate opposite of the
tripwire's tag-blindness.

**The named hole, recorded rather than hidden.** A `role: code` cell adding a
`#[cfg(test)]` module inside a source file it was already touching is the
DOMINANT test-writing shape in this repo, and the armed arm cannot see it:
only 27 of the 67 cells that name tests in their title or action carry any
test-shaped path, and 7 declare no `files` at all. The advisory arm is what
covers it. Closing it properly needs a real cell field declaring test intent,
which the "nothing new to forget to update" rule argues against inventing on
a guess. It is deferred with that measurement attached, under trigger `the-mint-trap-s-advisory-arm-has-fired-o__d853e4c6`.
A test asserts that such a cell CLAIMS.

**The ramp.** The refusal ships fully built, but while the ACTIVE decision set
holds zero decisions tagged `contract:<name>` no cell could satisfy the rule,
and a rule nobody can satisfy is a dead workflow rather than a guard — so the
armed arm warns instead, and the warning says what will end the ramp. The
first `contract:<name>` decision flips it to refusing. That condition is
DERIVED from the same active-decision read the tripwire uses: no config key,
no flag, no stored counter, because a second thing to forget to update is
what the derived-status rule exists to refuse.

**B8 — Unified command discovery and dispatch.** Every workflow operation — all
nine verb groups — is available both through its specialized entry point and
through one unified entry point, and the unified side owns the single
implementation: each specialized entry point is a thin forwarder whose output
is byte-identical to the unified path, and a new verb is added exactly once
(one catalog entry plus one handler), never re-implemented in a forwarder.
The unified entry
point publishes the complete command catalog in human-readable and
machine-readable forms. It validates required parameters and their value shapes
before dispatch, then invokes the same underlying operation as the specialized
entry point; it does not run one command-line program from another. For the same
valid request, observers receive the same result and exit outcome through either
surface. This includes revising an open or blocked work cell's allowed plan
fields. An unknown command is refused with the nearest known command when one is
available. A malformed request is refused with the command, field, and reason,
without executing the operation — and the refusal is exhaustive (cli-ergonomics
D1, 8ef2bae6): every missing and invalid parameter is named in the one refusal,
alongside a runnable example taken from the catalog entry, so a caller never
discovers problems one retry at a time. The structured error keeps the first
problem in its legacy fields (existing consumers unchanged) and carries the
full list additively. Legacy verbs that deliberately own their own checks
(DB3) gained the same all-at-once behavior inside the handler layer, on their
original error channel. After a catalog change, observers receive a
separate diagnostic signal while the requested command's normal output keeps its
stable shape. Validation also rejects, on stderr with exit 1, any parsed flag
absent from the invoked verb's own registry schema — the two global flags
(`--json`, `--help`) are always accepted, and the refusal names the exact
flag, the verb, and every flag the verb's registry actually declares. This
central check fires after `validate()` and strictly before every handler
dispatch, so it also covers the two pre-existing bespoke per-handler checks
(`cells update`, `state worker prune`), left in place unchanged. A handler
that reads a flag indirectly through a shared helper (e.g. `session-id`/
`force-ownership` via the ownership-flags helper several `cells` verbs share)
gets that flag declared in its own registry entry rather than the validator
ever being loosened to tolerate an undeclared one (packages-engine-move D4;
decision 80b64c20).

## Business Rules

- R12 — The unified entry point serves all nine command groups from one
  implementation; the specialized entry points are thin forwarders with
  byte-identical output, and a new verb is added once — one catalog entry plus
  one handler, never a second implementation in a forwarder (decision
  30606de4-5fae-4c9d-9e3f-8f47a494f8a3; dispatcher-unify decision 2026-07-12).
- R13 — The published command catalog and executable dispatch surface describe
  the same command set. Every published example is exercised against the real
  operation, so a documented but unusable command is a verification failure
  (decision 30606de4-5fae-4c9d-9e3f-8f47a494f8a3).
- R14 — An unknown flag is refused, never silently accepted or ignored:
  `bee <verb> --<unknown>` exits 1 with stderr naming the flag, the verb, and
  every flag the verb's registry actually declares (e.g. `capture add --text x`
  → `capture add: unknown flag --text (known: area, did, files, help, json,
  lane, outcome, source)`). Correction of an earlier friction reading: `capture
  add --text` was never a silent no-op — it already exited 1 via
  `requireFlag('outcome')` before this rule — the actual defect was message
  quality (the refusal never named `--text` as the real unknown flag) plus an
  orchestrator habit of reading only the last stderr line; the fix makes the
  check universal across every verb and names the offending flag every time.
  A gap between a handler's real flag usage and its declared registry schema
  is always closed by declaring the flag, never by loosening the validator
  (packages-engine-move D4; decisions 80b64c20, b0ef4f66).
- R15 — The dispatch door renders the intent anchor's request verbatim into
  every prompt kind, resolving the anchor by FEATURE only — the cell's own
  feature, then the active feature — and never by the `default` or session
  key. With nothing resolvable, every runtime × kind payload is byte-identical
  to its pre-anchor bytes (slp-contract-original-request D5, D6; decisions
  3899fa60, 9c0104e0).
- R16 — A cell's `decisions` entries are checked at BOTH the claim door
  (`bee cells claim`, `cells claim-next`, `dispatch prepare --claim` — all
  three funnel through one claim body) and the dispatch door
  (`dispatch prepare --kind cell`), through ONE shared check. Both doors are
  needed: a cell claimed BEFORE its cited decision changed state slips a
  claim-only tripwire, and the claim door cannot see that window. The check
  refuses on two derived statuses — `CONTRACT_RETIRED` (the cited decision
  is not in the ACTIVE set: superseded, redacted or archived; the store has
  no `retired` state, so that is what "retired" means) and
  `CONTRACT_UNSETTLED` (the decision is active but a trigger keyed to it is
  `waiting` or `due`). `settled` passes. Every refusal is typed and mutates
  nothing: the claim door releases its claim file and leaves the cell
  `open`; the dispatch door records no dispatch; the trigger store is
  byte-identical, because the read goes through the non-evaluating trigger
  reader rather than the one `triggers list` uses
  (slp-contract-original-request D2, D3; decisions 9c0104e0, ca9960f5).
- R17 — An entry in `cell.decisions` that does NOT resolve to a store
  decision is passed over: refusing nothing, warning nothing. The field does
  not hold store decision ids. Measured over the 92 live cells: 48 cite
  something, 81 citations total, only 11 (13%) resolve, and the
  entry-length histogram is `{2: 61, 3: 5, 8: 11, 24: 1, 25: 3}` — the field
  is dominated by LOCAL D-IDs (`D1`, `D2`) pointing into a CONTEXT.md table.
  Refusing on an unresolvable entry would refuse 87% of citing cells for
  using the field the way the repo has always used it. An entry resolves on
  an exact id, or on a prefix of at least 8 characters matching exactly one
  candidate; resolution runs against the active+ARCHIVE union, never the
  active set alone, which is what makes the retired case reachable at all
  (slp-contract-original-request D3; decision 9c0104e0).
- R19 — A cell whose `files` declare a test-shaped path (per
  `path_looks_like_test`), or whose `role` is `test`, is refused at the CLAIM
  door with `CONTRACT_UNCITED` when none of its `cell.decisions` entries
  resolves to a store decision tagged `contract:<name>`. The refusal mutates
  nothing: the claim file is released and the cell stays `open`. Two arms and
  one ramp qualify it — a cell whose `title`/`action` merely names test
  writing WARNS and is never refused, in any state; and while the ACTIVE
  decision set holds zero `contract:<name>` decisions the armed arm also only
  warns, naming what will make it refuse. The armed arm is blind to a
  `role: code` cell adding a `#[cfg(test)]` module inside a source file it
  already touches — the accepted, measured hole. Claim door only, unlike R16:
  the trap's signal is static cell-record data, with no claim-then-change
  window (slp-contract-original-request D4; decisions 9c0104e0, d853e4c6).
- R18 — A new cell's `status` must be `open` or absent. `bee cells add`
  refuses any other value, naming the field and the verb that owns the
  transition. Without it, `"status":"claimed"` in the payload minted a cell
  that never passed the claim door, so every pre-claim deny — the citation
  tripwire, the red base, the no-route escalation, the uncapped-deps check —
  was bypassable in one line of JSON. Scoped to authoring: the
  claim/verify/cap/block/drop transitions keep their own guards
  (slp-contract-original-request D3; decision 9c0104e0).

## Edge Cases Settled

- A catalog fingerprint change never appears inside the requested command's
  ordinary result. Consumers that parse normal output therefore remain stable
  while diagnostics can still report that discovery metadata changed.
- A missing required parameter, a value with the wrong shape, or an unknown
  command is rejected before any workflow record changes.
- A citation tripwire that cannot read the decision log warns on stderr and
  lets the work through, rather than refusing on evidence it never read —
  the same "cannot know" arm the red-base deny already takes.
- The tripwire is tag-blind. `contract:<name>` names which decisions are
  contracts for a human reader; the derived status the doors consume joins
  the active decision set against open triggers and never reads a tag. So an
  untagged active decision passes exactly like a tagged one, and an untagged
  decision with an open trigger refuses exactly like a tagged one — the
  fail-safe direction for a refusal path.
- The mint trap's ramp condition is a read, not a switch. Nothing is stored,
  configured or flipped when it arms: logging the first decision tagged
  `contract:<name>` is the whole state change, and removing that decision
  puts the trap back to warning.
- The bare tag `contract` does not arm the ramp and does not satisfy a
  citation — only the `contract:<name>` namespace does. Five live decisions
  carry the bare tag and none of them names a contract; counting them would
  arm the rule on history that never opted in.
- Both contract rules skip their store reads entirely when neither could
  speak — no citations and nothing naming tests — so the ordinary cell pays
  what it paid before either rule existed.

## Pointers (implementation)

- Unified dispatcher and catalog: `the bee binary`,
  `packages/bee/lib/command-registry.mjs`, and
  `packages/bee/lib/validate-args.mjs`, mirrored under `.bee/bin/`.
  Evidence: `.bee/cells/hia-1.json`, `.bee/cells/hia-2.json`, and
  `docs/history/harness-integration-adopt/reports/`.
- Unified dispatcher (all nine groups): `the bee binary` owns
  registry + handlers; dispatcher-unify (`.bee/cells/du-{1..6}.json`,
  `docs/history/dispatcher-unify/`) first made every legacy per-group script a
  2-line forwarder with byte-identical output, then shim-retire (D1, decision
  bbc6bcea; `.bee/cells/shim-retire-{1..6}.json`) deleted those forwarders
  outright — `bee` is now the sole shipped CLI, no forwarders remain.
- Contract-citation tripwire (R16, R17): one shared check,
  `contract_citation_refusal` in
  `packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs`, called from
  the claim body beside the `RED_BASE` deny and from `prepare`'s
  `kind == "cell"` arm in
  `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`. The derived
  status and the citation resolver it consumes live in
  `packages/bee-rs/crates/bee/src/verbs/decisions/read.rs`
  (`contract_status_over`, `resolve_store_citation`,
  `open_trigger_decision_keys`). The authoring-side status refusal (R18) is
  in `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs`
  (`validate_new_cell_problems`). Evidence: `.bee/cells/scor-4.json`.
- Mint trap (R19): `contract_claim_refusal` (the claim door's one entry for
  both contract rules), `mint_trap_over`, `declared_test_path`,
  `mint_trap_ramp_warning` and `mint_trap_advisory_line` in
  `packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs`, sharing
  `ContractReads` with the R16 tripwire. The path classifier it reuses is
  `path_looks_like_test` in
  `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs`. Tests:
  `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs` (refuse, allow, the
  ramp's two states, and the named hole asserting it claims). Evidence:
  `.bee/cells/scor-5.json`.
- Unknown-flag rejection (R14): `main()` in `the bee binary` (mirrored
  `.bee/bin/bee`), firing after `validate()` and before every handler
  dispatch; registry gaps declared in `packages/bee/lib/command-registry.mjs`
  (`cells.claim`/`claim-next` `--isolate`, `state.gate` `--owner`,
  `state.start-feature` `--isolate`, `config.get`/`set`/`unset` `--local`,
  `cells.verify`/`cap`/`block`/`unclaim`/`reopen` `--session-id`/
  `--force-ownership` via the shared `ownershipFlags()` helper). Red-first
  regression: `packages/bee/scripts/tests/test_bee_cli.mjs` (295 passed/1
  failed before the fix, 296 passed after). Evidence:
  `.bee/cells/packages-engine-move-3.json`,
  `docs/history/packages-engine-move/reports/packages-engine-move-3.md`.
