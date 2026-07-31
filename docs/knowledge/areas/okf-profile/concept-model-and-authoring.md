---
type: bee.area
title: "Bee OKF Profile — the concept model, its frontmatter, and how a concept is authored"
description: "The closed nine-type vocabulary, the frontmatter field rules and their identity/path direction, the per-subject authority rules, the authoring gate that resolves where a settled truth is written and refuses a fork in three layers, the legacy carry-over map, and the four canonical worked examples with the body contract and the rebuild bar."
timestamp: 2026-07-22
bee:
  id: okf-profile-concept-model-and-authoring
  lifecycle: active
  areas: [okf-profile]
  required_context: [areas/okf-profile/overview.md]
  decisions: [D4, D10, D11, D12, D17, D18, D19, D23, D31, D32, D33, D36, G12, G13, G14]
  sources: ["okf-foundation cell okf-2 (bundle skeleton + this spec, 2026-07-22)", "okf-foundation cell okf-6 (critical-patterns.md -> patterns/ migration, work/okf-foundation/ work item + plan concepts, Templates section; trace in `.bee/cells/`, 2026-07-22)", CONTEXT.md `docs/history/okf-foundation/CONTEXT.md`, "docs/specs/okf-profile.md#E1", "okf-switchover-f3 cells f3-2, f3-3 (bundle-first scribing target, the three-layer anti-fork gate, and both doc trees resolved off the product root; capped with verify evidence, trace in `.bee/cells/`, 2026-07-22)", CONTEXT.md `docs/history/okf-switchover-f3/CONTEXT.md`]
  authoritative_for: "okf-profile: the concept model, its frontmatter, and concept authoring"
---

# Bee OKF Profile — The Concept Model, Its Frontmatter, and How a Concept Is Authored

This concept owns what a concept **is** — the closed nine-type vocabulary, the fixed field set and
its identity/path direction, the authority rules that make one subject have exactly one readable
owner, and the map that carries a legacy file's frontmatter across — together with how one is
actually written: the four canonical worked examples, the body contract, and the rebuild bar that
grades meaning where the checker can only grade form.

## Data Dictionary

**The nine concept types (D18, closed, slug-cased):**

| Type | What it holds |
|---|---|
| `bee.area` | Current truth about a subsystem. Several may share one `areas/<slug>/` directory — authority is per-subject, not per-directory (D31). |
| `bee.feature` | A feature's current-truth summary. |
| `bee.work-item` | One unit of work, at `work/<id>/`. |
| `bee.plan` | A work item's plan (absorbs `implement-plan.md`'s review-state role — D36). |
| `bee.delivery` | A work item's delivery record. |
| `bee.decision` | A locked decision, migrated form. |
| `bee.pattern` | A durable pattern **or pitfall** — `bee.polarity: practice \| pitfall` distinguishes them; there is no separate tenth "pitfall" type, because pattern and pitfall carry identical metadata and identical consumption (D18). |
| `bee.runbook` | An operational procedure. |
| `bee.evidence` | A proof artifact (e.g. a coverage report, D35). |

**Frontmatter field rules (D19, corrected by D32):**

- Root level: `type` (OKF-required) + `title`, `description`, `tags`, `timestamp` (+ `resource`
  only for a genuinely external asset).
- A nested `bee:` object carries `id`, `lifecycle` (`draft`\|`active`\|`superseded`\|`archived`),
  `areas`, `required_context` (bundle-relative **paths**), `decisions`, `sources`, and — per type —
  `lane`, `polarity`, `critical`, `authoritative_for`, `review_status`, `supersedes`/`superseded_by`.
- **Id/path direction (D32 — corrects D19's original wording, which had it backwards):** an id is
  **never** computed from a file's path. Instead the path segments `areas/<slug>/` and `work/<id>/`
  are **derived from the id**. `bee.id` is identity; the file **path is the link target** an OKF
  consumer follows (§5) — both exist because links are paths but ids must survive the splits and
  moves migration performs.
- Field supply at migration time: `timestamp` is set from the source file's last git commit;
  `tags` carries across from existing frontmatter where present, absent otherwise; `resource` is
  set only for a genuinely external asset (D32).
- `title`/`description` are profile-required and the migrator **never invents** a value where it
  cannot derive one — the key is left absent and `check` warns, naming the file (D10). A fabricated
  summary reads as curated and gets trusted; the never-invent rule that governs prose (D10 in
  CONTEXT.md's wider sense) governs metadata too.

**Authority rules (D31 — supersedes an unimplementable "one `bee.area` per directory" rule, D28):**

- `bee.id` is **globally unique** across the bundle.
- `bee.authoritative_for` is unique **per subject** — no two concepts claim the same subject.
- Neither rule is "one directory, one concept": several `bee.area` concepts legitimately share one
  `areas/<slug>/` directory, because authority belongs to a *subject*, not a directory. A generated
  `index.md` carries no frontmatter at all (D4), so it can never be — and never was eligible to be
  — the area concept.

**Legacy frontmatter carry-over map (D33 — corrects an undercount: `workflow-state.md:1-10` carries
seven keys, not five):**

| Legacy key | Bundle target |
|---|---|
| `area` | `bee.areas` |
| `decisions` | `bee.decisions` |
| `sources` + `parity_sources` + `parity_decisions` | `bee.sources` |
| `updated` | `timestamp` |
| `coverage` | **dropped** — grep confirms no code reads it |

**`implement-plan.md` retirement (D36 — declared here, executed in F2):** the committed
`implement-plan.md` artifact is retired once the Brief step (`bee-shaping`) is rewired. Its review-state ladder
(`Draft → Ready for Review → Approved → Needs Revision → Shipped`) moves into `bee.plan`'s
`bee.review_status` field; the brief renders from that field instead of from a committed file. The
brief was already a self-declared projection, but it was also the carrier of real gate-mirroring
state and the surface approval happens on — so the retirement is declared now, with a named home
for that state, and executed only when the rewiring itself lands.

**Standing exemption — `docs/decisions/index.md` (D11):** this file keeps its existing path, owner,
and shape permanently. `bee knowledge index` never writes it, and no future generator claims it.
The exemption exists because it already sits outside the bundle (`docs/decisions/` is a legacy
tree, D17), so no conformance rule reaches it in the first place — one generated path keeps one
generator.

## Behaviors & Operations

**Resolving where a settled truth is written — the authoring gate.** The scribe never decides by
eye whether this repo keeps its state layer as concepts or as area spec files, and never restates
the rule in its own words. It asks one **bundle-mode predicate**, and that predicate answers true
only when the knowledge tree exists **and at least one concept inside it actually parses**. A
directory alone is not a bundle: a knowledge tree holding nothing but a placeholder file answers
false, and the scribe writes the legacy area spec instead. Both doc trees — the bundle and the
compatibility surface — are **product** documentation, so both are resolved off the repo's
declared product root. A repo that separates its workshop from the product it documents is
therefore graded on its real product docs, never on an empty workshop tree; every consumer that
touches either tree resolves it the same way, so no two of them can disagree about where the
product's docs live.

**The scribing-target answer is a fixed shape, not a path.** Asked for an area and a subject, the
gate returns the same seven fields on **every** answer in **every** mode: whether the repo is in
bundle mode, the `action` to perform, the area, the subject, the `path` to write, the `owner` when
one exists, and whether the index must be regenerated afterwards. The scribe writes to `path`,
performs exactly `action`, and regenerates the index when told to. Declaring the intent matters:
an author who believes the subject is new says so, and if the subject is in fact already owned the
answer names the owner instead of handing back a path.

**A `path` of nothing is a refusal, and a refusal is never a licence to choose a path.** Three
answers refuse, each naming what is wrong rather than degrading to a default:

| Refusal | Means | The scribe must |
|---|---|---|
| `fork_denied` | the subject is already owned by the named concept | update that owner in place, or declare the split inside it |
| `subject_required` | a new-concept intent arrived with no usable subject — empty, blank, absent, or punctuation only | name the subject and ask again |
| `duplicate_authority` | two or more concepts already claim this subject; every claimant is listed | collapse the rival claims to one authority first, then re-ask |

`subject_required` exists because the alternative is worse than a refusal: routing a nameless
new-concept request to the area's `overview` concept silently appends unrelated truth to the one
document readers treat as the area's front door. A request with no subject is not a request for
the default subject. Separately, the gate **throws** — naming the file — when any concept in the
bundle carries a malformed authority claim (a list, a boolean, an empty or blank string). A claim
the bundle cannot read is an owner the anti-fork gate cannot see, so it is a hard stop, never a
silent skip.

**The anti-fork gate has three layers, because exact matching on free text can never be
sufficient.** An independent judge defeated a single-layer version four ways in one sitting — a
trailing period, a cross-script look-alike character, a non-string claim, and an empty subject
that skipped the gate entirely — so the gate is built in depth:

1. **The match is a skeleton, not a string.** Subjects are compared after normalization,
   lowercasing, accent stripping, a cross-script confusable fold, and punctuation/whitespace
   collapse. Neither a trailing period nor a look-alike character buys a rival concept.
2. **Malformed input fails closed** — the three refusals and the throw above. A silently skipped
   claim is a fork with extra steps.
3. **The bundle-wide backstop bites.** Duplicate and malformed authority claims are chain-*failing*
   findings of the conformance check, grouped by the same skeleton (`conformance-check.md`). Layer
   1 can never catch a genuine word-order paraphrase — nothing that compares strings can — so a
   whole-bundle check is what refuses to let two authorities coexist.

No layer is softened to get a write through: a refused write means the bundle is wrong, not the
gate.

## Business Rules

- The vocabulary is closed at **nine** types (D18); a tenth type is never introduced to encode a
  distinction that fits inside an existing type's fields (pitfall inside pattern's `bee.polarity`
  is the standing example).
- Identity is `bee.id`, globally unique; authority is `bee.authoritative_for`, unique per subject —
  never "one concept per directory" (D31).
- An id is never derived from a path; a path is derived from an id (D32).
- `docs/decisions/index.md` is permanently exempt from generation and from this profile's reach —
  it already sits outside the bundle (D11, D17).
- `title`/`description` (and any profile-required field) are never fabricated; an absence is a
  named warning, not a guess (D10).
- Bundle mode is decided by the predicate, never by looking at the tree: existence alone is not a
  bundle, and at least one concept must actually parse (G12).
- Both doc trees are product documentation and both resolve off the declared product root — no
  consumer joins either path itself (G13).
- A refusal from the authoring gate is never worked around by choosing a path; the three refusals
  and the malformed-claim throw are each fixed at the source (G14).
- No subject is ever defaulted. A new-concept request without a usable subject is refused, never
  routed to the area's `overview` concept (G14).

## Edge Cases Settled

- A concept is any non-reserved `.md` **inside** `docs/knowledge/` — nothing outside the bundle is
  a concept, is checked, or carries a frontmatter obligation (D23). This removed a 593-file
  retrofit obligation, 14 prose-header legacy files, a filename-with-spaces hazard, and a
  `timestamp` double-source conflict in one decision.

## Open Gaps

- **The bundle has no home for UI visual snapshots.** The no-bundle state layer keeps settled
  screen snapshots under `docs/specs/visuals/<area>/`, and the Scribe discipline (`bee-capturing`) still requires one for
  every UI area whose screen visibly changed. The bundle profile defines no equivalent location,
  and `docs/specs/` is read-only for new content once a repo has migrated (G2 —
  `scripts/okf_specs_fence.mjs` fails the chain), so in bundle mode there is nowhere legitimate to
  put the image. Recorded rather than filled: inventing a path here would decide a profile
  question — is a binary asset a concept, a sibling of one, or out of the bundle entirely? — that
  no decision has settled. **Until it is settled, a bundle-mode UI area with a changed screen
  records the missing snapshot as an Open Gap in its own concept, naming the screen**; it never
  writes the image into the retired tree and never invents a bundle path (f4-6).

## Templates

Four canonical worked examples — one per type most authors reach for first. Each frontmatter
block below is the **exact emitter-canonical form** (byte-identical to what `emitFrontmatter`
produces and `parseFrontmatter` accepts back, D12): copy it, edit the values, and the round-trip
guard (`not_canonical`) stays silent. Round-trip proof (okf-6): the `bee.delivery` example below
was pasted into a temp file inside `docs/knowledge/patterns/`, `bee knowledge check --json` ran
zero errors and zero warnings (no `not_canonical` finding) with it present, and the temp file was
then removed — the other three examples are live bundle concepts (`bee knowledge check` already
grades them on every run).

Frontmatter is **always** produced by `emitFrontmatter` and never typed by hand — hand-written
blocks have been caught `not_canonical` repeatedly, including twice by the orchestrator that wrote
this profile.

### `bee.area` — `docs/knowledge/areas/performance-log/cross-project-matrix.md` (live)

The type `bee-capturing` ("Scribe") writes on every sync, capture, flush and harvest run when the repo is in
bundle mode. It is listed first because it is the one most authors reach for.

```yaml
---
type: bee.area
title: Performance Log — Cross-Project Matrix
description: "The read-only, per-project rollup view built from the shared persistent log, needing no prior tracking and grouped so different checkouts of the same project collapse into one row."
timestamp: 2026-07-22
bee:
  id: performance-log-cross-project-matrix
  lifecycle: active
  areas: [performance-log]
  required_context: [areas/performance-log/persistent-store-and-sync.md]
  decisions: [D 62a7c7fd]
  sources: [docs/history/perf-log/CONTEXT.md, docs/history/perf-log/plan.md, "cells perf-log-1, perf-log-2, perf-log-3 (capped, verified)", "docs/specs/performance-log.md#R11", "docs/specs/performance-log.md#P7"]
  authoritative_for: "performance-log: cross-project matrix"
---

# Performance Log — Cross-Project Matrix

## Purpose
## Entry Points & Triggers
## Data Dictionary
## Behaviors & Operations
## Actors & Access
## Business Rules
## Edge Cases Settled
## Open Gaps
## Pointers (implementation)
```

**`bee.authoritative_for` is the anti-fork field, and it is what makes this type different from
the other three.** It names, in one line, the SUBJECT this concept is the single truth for
(`"<area>: <subject>"`). Before authoring any new `bee.area` concept, the subject is looked up
across the whole bundle: if a concept already claims it, that concept is UPDATED IN PLACE and no
second file is created. Two concepts claiming one subject both parse, both list in the index, and
no reader can tell which is true — the `-v2` failure the one-file-per-area rule used to make
structurally impossible. `bee knowledge check` warns on a duplicate `authoritative_for` (D31); the
authoring gate is what stops the duplicate being written in the first place.

**The nine-section skeleton above is the body contract, not a suggestion** — the same nine
sections, in that order, that a BA-grade area document has always carried, so that splitting an
area into concepts does not silently downgrade body quality to "whatever the author felt like".
A concept may legitimately carry only the sections its subject has content for (the live example
above answers four of the nine, and says nothing where it has nothing) — what it may never do is
invent a different set of headings, or leave a section it does have content for unwritten.

**The rebuild bar is the acceptance test for the body** (not the frontmatter, which is the only
part a machine can grade): a competent agent given ONLY this concept and the concepts it names in
`required_context` — with the `Pointers (implementation)` section deleted — rebuilds the same
observable behaviour on a different technology, and a human reads it and understands every field,
behaviour, rule and role without opening the code. Outside `Pointers (implementation)` a concept
names NO language, framework, library, class, table, component or file: fields, screens, roles,
actions, jobs and messages are named in business vocabulary. Format-green is not quality-green —
`knowledge check` grades canonicality, and only the rebuild bar grades meaning.

`bee.areas` (plural) is area MEMBERSHIP — which subsystem(s) the concept belongs to, matched by
`knowledge list --area` and by the index generator. `bee.authoritative_for` (singular) is subject
OWNERSHIP. A concept normally carries both.

### `bee.work-item` — `docs/knowledge/work/okf-foundation/work-item.md` (live)

```yaml
---
type: bee.work-item
title: okf-foundation — Bee OKF Profile foundation
description: "Replace bee's document model with an OKF v0.1 bundle: docs/knowledge/, a validator, an index generator, a budget-aware context consumer, and the full migration loop proven end-to-end on one area."
tags: [okf, knowledge-bundle, migration, high-risk]
timestamp: 2026-07-22
bee:
  id: okf-foundation
  lifecycle: active
  required_context: [areas/advisor-protocol/overview.md, patterns/20260715-shipping-a-lib-file-means-shipping-the-manifest.md, patterns/20260716-a-tolerant-regression-net-frozen-green-before-the.md, patterns/20260712-cross-cell-contracts-and-census-carriers-are-plan.md]
  decisions: [D17-D38 active set (docs/history/okf-foundation/CONTEXT.md), "D29 (F1 proof area: docs/specs/advisor-protocol.md)", D30 (workflow-state.md decomposition locked as F2 input), D34 (ships list + guard propagation — ledger/manifest/plugin trees/session-close hook), "D35 (coverage report law: every numbered source anchor lands in exactly one concept)", D37 (pointer-stub anchor map — citations rewired in the same cell as the stub)]
  sources: [docs/history/okf-foundation/CONTEXT.md, docs/history/okf-foundation/plan.md]
  lane: high-risk
---

# okf-foundation — Bee OKF Profile Foundation

## Outcome

Replace bee's document model with an OKF v0.1 bundle governed by a **Bee OKF Profile** [...]

## Scope
## Acceptance
## Decisions
## Chosen Approach
```

A work item's body sections (agent's discretion, not profile-enforced) are: Outcome, Scope,
Acceptance, Decisions, Chosen Approach — each condensed FROM the feature's `CONTEXT.md`/`plan.md`,
never invented (D10). `bee.decisions` entries mix bare D-ids (`D30`) and quoted parenthetical
citations (`"D29 (F1 proof area: ...)"`) — the emitter quotes only the entries containing a colon,
comma, or other reserved character; both forms round-trip.

## Pointers (implementation)

- The bundle-mode predicate, the scribing-target resolver and the frontmatter emitter:
  `bundleMode`, `scribingTarget`, `emitFrontmatter` in `.bee/bin/lib/knowledge.mjs` (mirrored at
  `packages/bee/lib/knowledge.mjs`).
- The product-root resolution both doc trees share: `resolveProductRoot` in
  `.bee/bin/lib/state.mjs` — the same resolver the session preamble (`inject.mjs`), the backlog
  and the session-close hook already use.
- The scribe's own routing prose: `skills/bee-capturing/SKILL.md` ("Scribe" section).
- Coverage for the gate, its three refusals and the repo-divorce topology:
  `packages/bee/tests/test_bundle_mode.mjs`.

### `bee.plan` — `docs/knowledge/work/okf-foundation/plan.md` (live)

```yaml
---
type: bee.plan
title: okf-foundation — Plan
description: "The seven-slice plan for the Bee OKF Profile foundation: format core, chain wiring, generators, data, consumer, startup bridge, promote v1."
tags: [okf, plan, high-risk]
timestamp: 2026-07-22
bee:
  id: okf-foundation-plan
  lifecycle: active
  required_context: [work/okf-foundation/work-item.md]
  decisions: [D2 (chain never red between cells), D22 (consumer in scope), D29, D30, D34, D35, D38 (loop closure over coverage)]
  sources: [docs/history/okf-foundation/plan.md, docs/history/okf-foundation/CONTEXT.md]
  lane: high-risk
  review_status: Approved
---

# okf-foundation — Plan

## Mode gate
## Approach
## Slices
## Risk map
## Rejected Alternatives
```

`bee.review_status` carries the `Draft → Ready for Review → Approved → Needs Revision → Shipped`
ladder (D36) — the field `implement-plan.md`'s review state moves into once the Brief step is
rewired (declared here, executed in F2). A `bee.plan`'s `required_context` typically points back
at its own work item, since the plan only makes sense alongside the outcome/scope it implements.

### `bee.delivery` — worked example (illustrative; not a committed concept — okf-foundation is not
closed yet), grounded in cell okf-5's real cap trace (`.bee/cells/okf-5.json`)

```yaml
---
type: bee.delivery
title: okf-foundation cell okf-5 — advisor-protocol migrated end-to-end
description: "Delivery record for cell okf-5: docs/specs/advisor-protocol.md re-authored into four bee.area concepts, coverage-gated (D35), full chain green."
tags: [okf, delivery, migration]
timestamp: 2026-07-22
bee:
  id: okf-foundation-okf-5-delivery
  lifecycle: active
  areas: [advisor-protocol]
  required_context: [work/okf-foundation/work-item.md]
  decisions: [D29, D35, D37]
  sources: [.bee/cells/okf-5.json, docs/specs/advisor-protocol.md]
  lane: high-risk
---

# okf-foundation cell okf-5 — Delivery

## Outcome

`docs/specs/advisor-protocol.md` migrated end-to-end into 4 `bee.area` concepts [...]

## Evidence
## Verify
```

A `bee.delivery`'s `sources` may cite a cell trace file directly (`.bee/cells/<id>.json`) — a
`.bee/*.json` path is outside the bundle and is never itself a concept (D2/D23), but `sources` is
free-text provenance, not a `required_context` link target, so citing it is not a dangling-target
finding. `areas` names the subsystem(s) the delivered work touched, matching one or more
`bee.area` concepts' `bee.areas` membership.
