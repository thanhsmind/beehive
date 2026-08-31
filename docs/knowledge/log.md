## 2026-07-22

- Bundle created under `docs/knowledge/` (D17): `index.md` hand-seeded (root frontmatter carries
  only `okf_version`, per D4/OKF §9), this `log.md` opened. The format core —
  `.bee/bin/lib/knowledge.mjs`, the `bee knowledge check` verb, the emitter-first frontmatter
  codec, and the two-level OKF-error/profile-warning checker (D4/D13) — shipped in cell okf-1
  (feature `okf-foundation`, slice S1). This entry, the skeleton, and the Bee OKF Profile area
  spec (`docs/specs/okf-profile.md`) are cell okf-2.
- Generation takeover (cell okf-4, slice S3): `index.md` is **generated from now on** by
  `bee knowledge index` (D21) — the hand-seeded body is replaced by the byte-identical generated
  index (okf_version-only frontmatter kept, provenance moved into the HTML comment header;
  per-level indexes will appear as concepts land). `bee knowledge index --check` guards freshness
  in the verify chain, and `bee knowledge list` (D15) rows the bundle's concepts.
- First area migrated end-to-end (cell okf-5, slice S4a, D29): `docs/specs/advisor-protocol.md`
  re-authored into four `bee.area` concepts under `areas/advisor-protocol/` (`overview.md`,
  `triggers.md`, `consult-loop.md`, `slots-and-tiers.md`), frontmatter carried per D33. The
  legacy path is now a pointer stub carrying the full anchor map (D37: 26 anchors — B1-B4,
  R1-R9, E1-E6, P1-P7 — each mapped to its owning concept). Coverage is machine-checked by
  `scripts/okf_migrate.mjs --check advisor-protocol` (D35), now a chain suite in
  `scripts/run_verify.mjs`; the session-close capture nudge scans `docs/knowledge/**/*.md`
  mtimes alongside `docs/specs/` (D34).
- Critical patterns migrated, work item + plan authored, templates published (cell okf-6, slice
  S4b): `docs/history/learnings/critical-patterns.md`'s 47 dated pattern entries re-authored into
  47 `bee.pattern` concepts under `patterns/` (`bee.critical: true` throughout, `bee.polarity`
  judged per entry — 42 pitfall, 5 practice), each canonical by construction (generated via
  `emitFrontmatter`, zero `not_canonical` findings). The legacy path is now a pointer stub
  carrying the full 47-anchor map (D37: `PAT1`-`PAT47`). Coverage is machine-checked by the new
  `scripts/okf_migrate.mjs --check-patterns` (D35, additive — critical-patterns.md is a flat
  dated list, not a nine-section BA spec, so it does not fit `--check`'s `ANCHOR_REGISTRY`/area
  shape), now a chain suite in `scripts/run_verify.mjs`. This feature's own work item and plan
  now live as concepts too: `work/okf-foundation/work-item.md` (`bee.work-item`, D26, condensed
  from `CONTEXT.md`) and `work/okf-foundation/plan.md` (`bee.plan`, condensed from the frozen
  `plan.md`, carrying `bee.review_status: Approved`, D36). `docs/specs/okf-profile.md` gained a
  "## Templates" section with three canonical worked examples (`bee.work-item`, `bee.plan`,
  `bee.delivery`) — round-trip proven by pasting the `bee.delivery` example into a temp bundle
  file and confirming `bee knowledge check` reported zero `not_canonical` before removing it.

- Second area migrated, and the first under the honest derived gate (cell f2-3, feature
  `okf-migration-f2`, slice S3): `docs/specs/doctrine-layer.md` re-authored into seven
  `bee.area` concepts under `areas/doctrine-layer/` — `overview.md`,
  `placement-and-anchoring.md`, `unenforced-obedience.md`, `delegation-threshold.md`,
  `helper-classes-and-transports.md`, `native-wait-discipline.md`,
  `lane-and-working-discipline.md` — split by **topic** (what each rule governs) rather than
  by the BA template's headings, per the promoted surface-structure pattern. Frontmatter
  carried per D33, emitted canonically. The legacy path is now a pointer stub carrying the
  full 39-anchor map (D37: B1-B8 incl. B3a/B7a, R1-R17, E1-E5, P1-P7). Ground truth is
  DERIVED, not hand-listed (F8): pin `{ed65720, docs/specs/doctrine-layer.md, blob 351bf72,
  ba-nine-section, 39 anchors, unparsed_blocks: 2}`, with the pre-migration source committed
  verbatim at `docs/history/okf-migration-f2/sources/doctrine-layer.md`. The two id-less block
  starts the source really carries — a bold-lead continuation inside B8 and the unnumbered
  verify-ladder bullet after R17 — are pinned as unparsed rather than invented into anchors
  (D10); each still travels with the anchor whose block it sits in, so the F11 fidelity floor
  measures it. Coverage is machine-checked by `scripts/okf_migrate.mjs --check doctrine-layer`
  (D35), now a chain suite. Two gate defects surfaced and were closed in the same cell: the
  stub-row parser and the `bee.sources` claim matcher had not been widened alongside f2-4's
  letter-suffixed ids (so `B3a`/`B7a` could never be claimed by anything), and F12's drift
  median was being drawn across incomparable shapes — a `flat-pattern-list` migration is 1
  anchor per concept by construction, so pooling it with nine-section areas reported drift in
  already-shipped work no cell had touched. The median is now taken per pin `kind`.

- Third area migrated, the cleanest of the nine (cell f2-5, feature `okf-migration-f2`, slice
  S4a): `docs/specs/decision-memory.md` re-authored into a single `bee.area` concept,
  `areas/decision-memory/overview.md`, owning all nine anchors. Ground truth is DERIVED (F8):
  pin `{8710d03, docs/specs/decision-memory.md, blob 2e8ec59, ba-nine-section, 9 anchors
  (0 B / 9 R / 0 E / 0 P), unparsed_blocks: 0}`, with the pre-migration source committed
  verbatim at `docs/history/okf-migration-f2/sources/decision-memory.md`. This source had been
  filed by an earlier blind sweep as "shapeless, 0 anchors" — it was never shapeless, its nine
  `- **R1 — …**` rules were simply invisible to the pre-f2-4 classifier. Splitting the 39-line
  source into topic concepts (as doctrine-layer's 386 lines warranted) drove F12's drift
  telemetry outside its band the moment a third `area`-kind pin activated the running median: 4
  concepts over 9 anchors read as both too few anchors per concept and too many concepts per
  source line next to advisor-protocol and doctrine-layer's much larger sources. Consolidating
  to one concept — genuinely the right shape at this size, not a fit to the metric — brought
  both ratios back inside the [0.5x, 2x] band with content unchanged and fidelity at min/median/
  max 1.000 (verbatim re-homing). Coverage is machine-checked by `scripts/okf_migrate.mjs --check
  decision-memory` (D35), now a chain suite. `scripts/tests/test_okf_pins.mjs`'s refusal assertions for
  decision-memory were retired (it is no longer unscheme'd) and replaced with a pass assertion,
  leaving worktree-parallelism as the sole remaining refused area.

- Fourth area migrated (cell f2-6, feature `okf-migration-f2`, slice S4b):
  `docs/specs/verify-pipeline.md` re-authored into two `bee.area` concepts, split by TOPIC rather
  than the old spec's headings — `areas/verify-pipeline/suite-topology-and-discovery.md` (how
  suites are shaped and found: per-module suites with no monolith, the shared fixture helper,
  convention-based discovery, the loud deletion guard, Windows CI proving the real suites through
  that same discovery mechanism) and `areas/verify-pipeline/concurrency-and-hermetic-runs.md`
  (how a run itself stays safe: locked atomic-swapped regeneration, multi-worker checkout
  etiquette, hermetic session-id scrubbing proven by deterministic race/isolation suites). Ground
  truth is DERIVED (F8): pin `{72fd828, docs/specs/verify-pipeline.md, blob eab70d7,
  ba-nine-section, 14 anchors (0 B / 5 R / 4 E / 5 P), unparsed_blocks: 7}`, with the
  pre-migration source committed verbatim at
  `docs/history/okf-migration-f2/sources/verify-pipeline.md`. The 7 unparsed blocks are the
  source's entire "Behaviors & Operations" section — 7 bold-lead bullets with no B-id at all —
  and none is invented into an anchor (D10); their content still travels into whichever concept's
  topic it matches. Two concepts over 14 anchors and 133 source lines land anchors_per_concept at
  7.00 and concepts_per_100_source_lines at 1.50, both inside the [0.5x, 2x] band across the four
  now-pinned "area"-shaped sources — a single consolidated concept (the decision-memory shape)
  was tried first and found to push concepts_per_100_source_lines to roughly half the running
  median, outside the band; splitting by the source's two genuinely distinct concerns (what
  suites there are and how they're found, vs. how a run stays concurrency-safe and hermetic)
  fixed both ratios without touching a single anchor's wording. Fidelity: min/median/max 1.000
  (verbatim re-homing). RED-FIRST: with the R4 claim deliberately removed from
  `concurrency-and-hermetic-runs.md`'s `bee.sources`, `--check verify-pipeline` failed with the
  real coverage-loss finding (`LOST in concepts: R4`), then was restored to green. Coverage is
  machine-checked by `scripts/okf_migrate.mjs --check verify-pipeline` (D35), now a chain suite.

- Fifth area migrated (cell f2-7, feature `okf-migration-f2`, slice S4c):
  `docs/specs/performance-log.md` re-authored into three `bee.area` concepts, split by TOPIC
  rather than the old spec's headings — `areas/performance-log/sections-lifecycle-and-measurement.md`
  (the operator-driven lifecycle of a named section: opening, closing, one-shot recording,
  reading/rendering, the section data dictionary, and the measurement rules that make its
  token/timing numbers trustworthy), `areas/performance-log/persistent-store-and-sync.md` (the one
  shared, append-only, per-machine store every section lands in, and the automatic sync mechanism
  that populates it from real session activity without the operator running anything), and
  `areas/performance-log/cross-project-matrix.md` (the read-only cross-project rollup view built
  from that same store, grouped by last folder name). Ground truth is DERIVED (F8): pin `{46a56a4,
  docs/specs/performance-log.md, blob efdc9f2, ba-nine-section, 23 anchors (0 B / 11 R / 5 E /
  7 P), unparsed_blocks: 10}`, with the pre-migration source committed verbatim at
  `docs/history/okf-migration-f2/sources/performance-log.md`. The 10 unparsed blocks are the
  source's entire "Behaviors & Operations" section — 7 bold-lead paragraphs plus 3 of Measurement
  rules' own un-ided sub-bullets — and none is invented into an anchor (D10); their content still
  travels into whichever concept's topic it matches. The source's own text already separates
  "Populating the store (sync)" and "Building the matrix (read-only view)" into two distinct
  behavior paragraphs, so three concepts follows the source's own structure rather than forcing
  one; three concepts over 23 anchors and 226 source lines land anchors_per_concept at 7.67 and
  concepts_per_100_source_lines at 1.33, both inside the [0.5x, 2x] band across the five now-pinned
  "area"-shaped sources — a 2-concept shape (folding store+sync and matrix together) was checked
  first and found to put concepts_per_100_source_lines at 0.89, a 0.49x outlier just outside the
  band, confirming the 3-way split as the honest shape rather than a forced one. Fidelity:
  min/median/max 1.000 (verbatim re-homing). RED-FIRST: with the R4 claim deliberately removed from
  `sections-lifecycle-and-measurement.md`'s `bee.sources`, `--check performance-log` failed with
  the real coverage-loss finding (`LOST in concepts: R4`), then was restored to green. Coverage is
  machine-checked by `scripts/okf_migrate.mjs --check performance-log` (D35), now a chain suite.

- Sixth area migrated (cell f2-8, feature `okf-migration-f2`, slice S4d):
  `docs/specs/feedback-digest.md` re-authored into four `bee.area` concepts, split by TOPIC rather
  than the old spec's headings — `areas/feedback-digest/data-model.md` (the digest's own shape: the
  six allowed fields, the closed `kind` vocabulary, how `pain` is computed once, and the `dropped`
  list), `areas/feedback-digest/generation-and-refresh.md` (how a repository produces its own
  digest as a side effect of closing a feature, on request, or as a count only, and the closing
  routine's automatic refresh), `areas/feedback-digest/cross-repo-trust-boundary.md` (how the
  maintainers' repository reads another repository's already-written digest as hostile input, never
  trusting a boundary artifact it did not produce), and
  `areas/feedback-digest/ranking-and-self-improvement.md` (grouping the collected view by what a
  title means, scoring deterministically, and the gated process that turns a ranking into a
  shipped, human-approved change to the workflow itself). Ground truth is DERIVED (F8): pin
  `{3d69a2d, docs/specs/feedback-digest.md, blob eeb447e, ba-nine-section, 29 anchors (0 B / 15 R /
  6 E / 8 P), unparsed_blocks: 26}`, with the pre-migration source committed verbatim at
  `docs/history/okf-migration-f2/sources/feedback-digest.md`. The 26 unparsed blocks are the
  source's entire "Behaviors & Operations" section — five markdown subheadings (B1-B5) carrying no
  id the classifier recognizes, holding 18 unnumbered bold-lead paragraphs plus 8 un-ided
  continuation bullets — and none is invented into an anchor (D10); their content still travels
  into whichever concept's topic it matches. This is the highest unparsed ratio of the six now-pinned
  "area"-shaped sources: roughly half this area's substance lives in unnumbered prose, so the
  coverage gate governs less of it than it does elsewhere — worth surfacing, and not a reason to
  invent structure that was never there. Four concepts over 29 anchors and 356 source lines land
  anchors_per_concept at 7.25 and concepts_per_100_source_lines at 1.12, both inside the [0.5x, 2x]
  band across the six pinned sources — a 3-concept shape (folding the data model into whichever
  behavior concept used its fields) was computed against the same running median before authoring
  and found to put concepts_per_100_source_lines at 0.84, a 0.47x outlier just outside the band,
  confirming the 4-way split (matching the source's own Purpose / Data Dictionary /
  Behaviors&Operations / Business Rules top-level sections) as the honest shape rather than a
  forced one. Fidelity: min/median/max 1.000 (verbatim re-homing).
  RED-FIRST: with the R4 claim deliberately removed from `cross-repo-trust-boundary.md`'s
  `bee.sources`, `--check feedback-digest` failed with the real coverage-loss finding (`LOST in
  concepts: R4`), then was restored to green. Coverage is machine-checked by
  `scripts/okf_migrate.mjs --check feedback-digest` (D35), now a chain suite.

- Seventh area migrated, and the largest so far (cell f2-9, feature `okf-migration-f2`, slice
  S4e): `docs/specs/onboarding.md` re-authored into eight `bee.area` concepts, split by TOPIC
  rather than the old spec's headings — `areas/onboarding/overview.md` (what onboarding is, its
  check/apply run modes, its two actors, and the unspecced remainder),
  `areas/onboarding/status-display-vendoring.md` (the opt-in status-display pair: detecting the
  opt-in, vendoring it, healing drift, staying entirely out of non-opted projects, what the line
  renders and why its context colour tracks the handoff mark, and the second runtime's
  machine-level status block), `areas/onboarding/managed-ignore-section.md` (the delimited block
  onboarding owns inside the project's ignore list: what it silences, what always stays
  version-tracked, the three exhaustive create/append/rewrite cases, and the already-tracked-path
  warning onboarding never acts on itself), `areas/onboarding/distribution-source-exclusivity.md`
  (selecting and proving exactly one distribution source, the Codex hybrid carve-out, the fenced
  cleanup in both directions, and the whole-run snapshot revalidated before the first mutation),
  `areas/onboarding/installer-entrypoints-and-source-staging.md` (the outer ring: fetching the
  source without materialising a full working tree, staging the COMPLETE release identity, the
  parity the two platform entry points owe each other, and a runtime whose tool is present but
  broken), `areas/onboarding/release-identity-and-version-parity.md` (one release version across
  every projection, the downgrade refusal, drift recomputed from real file content, the five
  source origins named rather than guessed, and the blast radius a forceable refusal must
  enumerate), `areas/onboarding/repo-local-guardrails.md` (the remembered opt-in that keeps a
  project's local guardrails current forever, and the second runtime's lifecycle hook merge
  discipline), and `areas/onboarding/host-project-artifacts.md` (the instructions import, the
  single unified dispatcher that retires nine helper scripts, the create-only state-layer landing
  pages, and the annotated configuration sample). Ground truth is DERIVED (F8): pin
  `{a06f59d, docs/specs/onboarding.md, blob c78ca9b, ba-nine-section, 58 anchors (0 B / 28 R /
  15 E / 15 P), unparsed_blocks: 20}`, with the pre-migration source committed verbatim at
  `docs/history/okf-migration-f2/sources/onboarding.md`. **`R20b` is the letter-suffixed id** the
  f2-4 widening was written for and the f2-3 stub-row/claim-matcher widening made claimable — it
  is owned like any other rule, and the RED-FIRST proof below deliberately targeted it. The 20
  unparsed blocks are the whole "Behaviors & Operations" section — 16 unnumbered bold-lead
  paragraphs, the "What the status display renders" lead paragraph, and the ignore section's three
  un-ided continuation bullets — none invented into an anchor (D10); their content still travels,
  verbatim, into the concept whose topic it matches. Eight concepts over 58 anchors and 690 source
  lines land anchors_per_concept at 7.25 and concepts_per_100_source_lines at 1.16, both inside the
  [0.5x, 2x] band across the seven pinned "area"-shaped sources; the alternative shapes were
  computed against the same running median before authoring — a 7-concept split (folding the
  overview's cross-cutting run modes and actors into a topic concept) lands 8.29 / 1.01, also in
  band, while every shape at 5 concepts or fewer puts concepts_per_100_source_lines at 0.48x or
  lower, outside it. The 8-way split was chosen on content, not on the metric: this is the first
  migrated area whose Purpose, run modes and actors genuinely govern all seven topics rather than
  belonging to any one of them. Fidelity: min/median/max 1.000 (verbatim re-homing).
  RED-FIRST: with the `R20b` claim deliberately removed from
  `installer-entrypoints-and-source-staging.md`'s `bee.sources`, `--check onboarding` failed with
  the real coverage-loss finding (`LOST in concepts: R20b`), then was restored to green. Coverage
  is machine-checked by `scripts/okf_migrate.mjs --check onboarding` (D35), now a chain suite.
  `scripts/tests/test_okf_pins.mjs`' section 12 — the guard that proves the extractor is not blind by
  requiring a NON-ZERO unparsed-block count from this very spec — now reads those same bytes from
  the committed, hash-verified source copy rather than the path this cell turned into a stub: the
  assertion follows the bytes, never the convenience.

- Eighth area migrated, and the first whose SOURCE had to be repaired before it could be pinned
  at all (cell f2-10, feature `okf-migration-f2`, slice S4f): `docs/specs/hook-runtime.md`
  re-authored into twelve `bee.area` concepts, split by TOPIC rather than the old spec's
  headings — `areas/hook-runtime/overview.md` (the frame every checkpoint sits inside: purpose,
  actors, hostile-input immunity, and "the guard's silence is never permission"),
  `catalog-projections-and-activation.md` (one catalog of record rendered into two projections
  with every difference named, and whether a project's checkpoints are enabled, rooted and
  trusted enough to fire), `write-guard-request-shapes.md` (the request shapes the guard can
  read: batch envelopes, workflow-command shape checks, and both recognised command forms),
  `governed-paths-and-the-intake-gate.md` (which targets are governed, the only-ever-shrinking
  always-writable set, and the gate that reads the phase rather than a closed feature's leftover
  approvals), `advisories-and-turn-control.md` (the advisory contract, session-stop output, and
  the one deliberate turn-control exception), `delivery-targets-and-the-fallback-command.md`
  (the two rendering targets and the full launch contract a rendered fallback command owes,
  Windows form included), `hook-source-exclusivity.md` (proof-gated arbitration in both
  directions), `dispatch-guard.md` (pre-spawn judgement of tier, model and helper type),
  `native-spawn-and-transport-classification.md` (the deliberate override pass-through and the
  capability probe's scoped, invalidatable verdict), `child-agent-attribution-and-audit.md` (the
  three checkpoints that observe without authorising), `coordination-refresh-and-session-init.md`
  (the durable state a checkpoint maintains as a side job), and
  `health-checks-and-proof-surfaces.md` (how the guardrails are inspected by a human and proven
  by the chain).
- **The duplicate `R14`, and why the repair happened before the pin.** The source shipped the
  rule id `R14` TWICE — the gate-bypass block-verdict rule (added for GitHub #18) and the
  write-guard dual command-shape rule (added by shim-retire D3). They are two genuinely
  different rules, not one rule stated twice, and the f2-4 sweep had already flagged the
  collision as pre-existing. Because anchors are keyed by id in a Map, the FIRST `R14`'s text
  was silently overwritten by the second's: unmeasurable by the F11 fidelity floor permanently,
  while set-equality could not see the pair's second member at all — and array-length counts
  never noticed, because 81 members with 80 distinct ids still add up. Neither rule may be
  dropped or merged to remove a duplicate, so the SOURCE was repaired first: the second
  occurrence in document order — the write-guard rule — was renumbered **`R14a`**, one token on
  one line, nothing else changed. The gate-bypass rule KEPT `R14` because every live citation of
  `hook-runtime R14` means it (this spec's own `R4` and `R10`, its gate-bypass pointer,
  `skills/bee-hive/references/routing-and-contracts.md`, and decision `4c1c5921`), so no
  external reference had to be churned and none silently changed meaning. `R14a` is a
  DISAMBIGUATION suffix, not a refinement suffix the way `R8a`/`R8b` refine `R8`; the pointer
  stub says so in prose and carries both rows.
- Ground truth is DERIVED (F8) from pin `{ab8cf6e, docs/specs/hook-runtime.md, blob a8907ce,
  ba-nine-section, 81 anchors (22 B / 24 R / 17 E / 18 P), unparsed_blocks: 8}` — 81 anchors with
  **81 distinct ids**, where the same file carried only 80 distinct ids before. Because the
  repaired bytes exist in no commit's tree, `okf_migrate` gained one branch and one branch only:
  a pin may declare `repaired_from` (the provenance blob at `commit:path`) plus `repair_reason`,
  and then the git leg addresses the PROVENANCE — still asserted exactly, so drifting provenance
  is as loud as a drifting pin — while the committed copy at
  `docs/history/okf-migration-f2/sources/hook-runtime.md` is the pinned bytes' only content
  address. An undeclared disagreement, a `repaired_from` that does not match, or a repair with no
  stated reason are all still `PIN_SHA_MISMATCH`; `scripts/tests/test_okf_pins.mjs` section 15 asserts
  every one of those from both sides, asserts the branch is inert on an unrepaired pin, and
  asserts the property the repair exists to create — every derived id distinct, with `R14` and
  `R14a` carrying their own separate texts so the floor measures each individually.
- The 8 unparsed blocks are all in "Behaviors & Operations" and none is invented into an anchor
  (D10): `B2`'s wrapped continuation line that happens to open with a bold run, `B3`'s three
  un-ided outcome bullets, and `B16`'s four un-ided case bullets. Each travels verbatim with the
  anchor whose block it sits in. Twelve concepts over 81 anchors and 762 source lines land
  anchors_per_concept at 6.75 and concepts_per_100_source_lines at 1.57, both inside the
  [0.5x, 2x] band across the eight pinned "area"-shaped sources and closer to the running median
  than any shape before them; alternatives were computed against that same median before
  authoring (6 concepts → 13.50 / 0.79, 15 → 5.40 / 1.97, both still in band), so the metric
  confirmed the 12-way split rather than driving it — the split follows the twelve genuinely
  different checkpoints this area describes. Fidelity: min/median/max 1.000 (verbatim re-homing).
  RED-FIRST: with the `R14a` claim deliberately removed from `write-guard-request-shapes.md`'s
  `bee.sources`, `--check hook-runtime` failed with the real coverage-loss finding (`LOST in
  concepts: R14a`), then was restored to green. Coverage is machine-checked by
  `scripts/okf_migrate.mjs --check hook-runtime` (D35), now a chain suite (73 suites).

- Ninth area migrated, and the only one of the eleven that genuinely needed a THIRD anchor
  scheme (cell f2-11, feature `okf-migration-f2`, slice S5): `docs/specs/worktree-parallelism.md`
  re-authored into seven `bee.area` concepts, split by TOPIC —
  `areas/worktree-parallelism/overview.md` (the two different kinds of parallelism, and what
  stays out of scope), `the-trust-model.md` (a worktree gets its own store only when GRANTED
  from the main store's registry, keyed by the git-verified id, and why a self-claiming marker
  inside the worktree changes nothing), `entering-creating-and-registering.md` (`worktree new`
  as the paved road, `worktree register` for adoption, the bootstrap contract, and every typed
  zero-mutation refusal), `returning-and-the-merge-gate.md` (the staged `--no-commit` merge, the
  verify gate that raises the semantic-conflict alarm before anything is committed, and
  `--cleanup`), `routing-and-visibility.md` (the D9 prose routing rule, the lane-first
  refinement that defers the grant to Gate 3, and the notices an ungranted worktree prints),
  `cross-worktree-holds.md` (the shared path-keyed ledger, its single-lock atomic acquisition,
  heartbeat renewal, three read taps and cell-scoped release), and
  `store-tiers-and-where-it-lives.md` (the log/cache/runtime tiers that decide what a merge may
  carry back, plus the implementation map).
- **The headings ARE the anchors, and that is why a third scheme exists.** Every previous
  "shapeless" verdict in this migration turned out to be a blind READER: `decision-memory` was
  filed as needing a bespoke scheme and its nine rules were sitting there all along, written
  `- **R1 — …**`, which f2-4's widening then read without ceremony. This source is the real
  thing. It carries no `B*`/`R*`/`E*`/`P*` id anywhere in its 225 lines and none of the four
  anchor-bearing nine-section headings, so `ba-nine-section` derives 0 anchors AND 0 unparsed
  blocks from it — genuine shapelessness, which looks nothing like decision-memory's hidden
  nine. F9 forbids forcing it into the nine-section shape and D10 forbids inventing numbered
  ids a source never had, so both easy answers were closed. What the source DOES have is its
  own narrative structure, ten `## ` headings written by its author, each opening a
  self-contained subject — so `okf_migrate` gained `narrative-sections`, where the anchor id is
  `S-` plus the heading text slugified (lowercased, every non-alphanumeric run collapsed to one
  hyphen). Nothing is invented: the structure the author actually wrote IS the ground truth,
  exactly as `flat-pattern-list` already treats a `## [YYYYMMDD] …` heading as an anchor.
- Three boundaries hold the scheme in place, each a test before it was a line of code
  (`scripts/tests/test_okf_pins.mjs` section 27). A `## ` heading becomes an anchor; a `###`
  SUBHEADING does not — its prose travels with the section containing it, so the fidelity floor
  measures it there, and the subheading itself is still REPORTED as an unparsed block so a
  subheading-heavy shape this scheme cannot see stays visible. A source with ZERO `## ` headings
  is REFUSED, typed `PIN_EMPTY_EXTRACTION` at the scheme itself, never passed 0/0 — that hole is
  the whole reason the gate exists, since an empty set from a reader that cannot see turns lost
  content into content that never existed. And two headings that slugify to the SAME id are
  refused outright, typed `PIN_DUPLICATE_ANCHOR`: that is the duplicate-`R14` hazard f2-10 had
  to repair a source to escape, closed here by construction so no narrative source can ever
  enter that state.
- Ground truth is DERIVED (F8) from pin `{687ac59, docs/specs/worktree-parallelism.md, blob
  df2f441, narrative-sections, 10 anchors, unparsed_blocks: 2}`. The 2 unparsed blocks are the
  `**Area:**` and `**Status:**` lines of the document preamble, which sit before the first `## `
  heading and therefore belong to no section; neither is invented into an anchor (D10) and both
  travel verbatim into `overview.md`. THE STRICT NO-OP IS THE SAFETY PROPERTY, as it was for
  f2-4's classifier widening and f2-10's repaired-pin branch: all nine pre-existing pins still
  derive their exact expected_counts after the scheme was added, proven against a FROZEN table
  in section 28 rather than against `PIN_REGISTRY`'s own numbers — which is how a relaxed pin
  would otherwise launder itself into a green.
- **F12's comparability key moved from the pin's `kind` to its SCHEME, and that too is a strict
  no-op.** f2-3 had already restricted the drift median to same-shape pins after
  `flat-pattern-list` (one anchor IS one concept by construction) reported permanent "drift"
  against nine-section areas that no cell had touched. `narrative-sections` has the opposite
  skew — whole SECTIONS are the anchors, so 225 lines yield 10 of them where a nine-section
  source of the same length yields 23 — and pooling it would have repeated exactly that defect,
  reporting an outlier for a shape the band was never drawn around. The scheme is what "same
  shape" has always meant; `kind` only ever approximated it, because until now every
  `kind: "area"` pin happened to be ba-nine-section. Section 29 asserts the regrouping: the
  eight ba-nine-section areas group exactly as the eight `kind: "area"` pins did, and
  critical-patterns stays alone exactly as `kind: "patterns"` did. With one narrative-sections
  pin there is no median yet, so this area's telemetry (anchors_per_concept 1.43,
  concepts_per_100_source_lines 3.10) REPORTS and never fails — never a gate weakened to fit,
  just a band not drawn on a single sample.
- Fidelity: min/median/max 1.000 (verbatim re-homing). RED-FIRST, both kinds: the scheme's own
  tests were captured failing before a line of it existed (`M.inventoryNarrativeSections is not
  a function`, plus the three boundary assertions and the missing `SCHEMES` entry), and then,
  with the `S-boundary-out-of-scope` claim deliberately removed from `overview.md`'s
  `bee.sources`, `--check worktree-parallelism` failed with the real coverage-loss finding
  (`LOST in concepts: S-boundary-out-of-scope`), then was restored to green. Coverage is
  machine-checked by `scripts/okf_migrate.mjs --check worktree-parallelism` (D35), now a chain
  suite (74 suites). Every registered area now declares a scheme, so `--verify-pins` reports no
  `SKIP-REFUSED` line at all — the PIN_NO_SCHEME refusal stays asserted against an ad-hoc pin,
  because the property is "an undecided shape is refused BY NAME", not "some area is still
  undecided".

- Eleventh and last area, first group of a SPLIT migration (cell f2-12, feature
  `okf-migration-f2`): `docs/specs/workflow-state.md` — 1464 lines, 140 anchors, the largest
  source of the eleven — is migrated across several cells per F10. This cell does the source
  REPAIR, the pin, and the first cluster group of okf-foundation D30's locked map only; the
  pointer stub, the anchor map and the chain gate belong to the LAST cell of the group, because
  a gate wired now would legitimately be red on the 136 anchors still unclaimed.
- THE HAZARD, and why it came first: this source carried **three** duplicated rule ids. `R19`,
  `R20` and `R21` each appeared twice inside one `## Business Rules` section — the
  fresh-session-handoff triple at L891-902 (planned-next preconditions live in the verb /
  auto-resume authority exists only at the fresh-session boundary / the work puller never widens
  authority) and the chain-integrity triple at L916-930 (the learning-capture phase is never
  settable / recording a knowledge sync demands executed work / the terminal state demands
  learning capture plus zero spec debt). Six genuinely distinct rules, three collisions, none a
  rule stated twice. Anchors are keyed by id, so each first member's text was silently
  overwritten by its second — permanently unmeasurable by the F11 floor, invisible to
  set-equality as the pair's second member — while 140 array members carrying 137 distinct ids
  added up perfectly every time. The precedent is hook-runtime's `R14` (cell f2-10) and the
  resolution is the same: no rule may be dropped or merged, so the SECOND occurrence of each id
  in document order (the chain-integrity family) was renumbered `R19a`/`R20a`/`R21a` — three
  tokens on three lines, no other byte changed — BEFORE the pin was captured. Which side moved
  was decided by live citations, and both sides have **zero**: `grep skills/ scripts/ hooks/
  .bee/bin/ AGENTS.md docs/specs/` finds no citation of any `workflow-state` rule id at all, in
  either direction, so no reference is churned whichever side moves. The tie broke exactly as
  f2-10's did, on document order; the one surviving external mention
  (`docs/history/fresh-session-handoff/reports/validation-s5.md`, citing
  `workflow-state.md B15/B16/R19-R21`) means the FIRST family, which kept its ids and still
  resolves. `R19a`/`R20a`/`R21a` are DISAMBIGUATION suffixes, not refinement suffixes the way
  `R8a`/`R8b` refine `R8`; the pin's `repair_reason` says so and the stub's anchor map will.
- Pin `{df3072d, docs/specs/workflow-state.md, blob 506fef9, repaired_from ed1644c,
  ba-nine-section, 140 anchors (37 B / 58 R / 25 E / 20 P), unparsed_blocks: 7}` — 140 anchors
  with 140 DISTINCT ids where the same file carried 137 before, with the repaired source
  committed verbatim at `docs/history/okf-migration-f2/sources/workflow-state.md` and
  hash-verified against `blob_sha`. Second use of f2-10's repaired-pin branch and no change to
  it: the git leg still addresses the PROVENANCE exactly, the committed copy is still the
  pinned bytes' only content address, and an undeclared, misdeclared or unexplained repair is
  still `PIN_SHA_MISMATCH`. `test_okf_pins` section 30 asserts all of that plus both directions
  of the repair itself — every derived id distinct, all six rules carrying their own separate
  texts, and the defect measured on the provenance blob rather than described (140 anchors,
  137 distinct, collisions exactly `R19`/`R20`/`R21`).
- The 7 unparsed blocks are all in Behaviors & Operations and none is invented into an anchor
  (D10): B9a's wrapped continuation line that opens with a bold run (`**read-only** with an
  evidence bundle …`, L214), B16's `**actively owned by another live session**` continuation
  (L356), and the five un-ided bold-lead paragraphs of the `### Closing a feature — the tail of
  the chain` subsection (L547/L553/L558/L564/L577). A `###` heading does not close an anchor's
  block, so those five travel inside **B24**'s text — which the multi-session group must carry
  to clear the floor, even though D30 homes the Closing-a-feature prose in `gates`.
- First group authored: `areas/workflow-state/overview.md` (Purpose + Entry Points & Triggers +
  the full Data Dictionary + Actors & Access — the source's un-numbered frame sections, so it
  claims no anchor) and `areas/workflow-state/gates.md` (B1 guarded feature start, B2 the closed
  phase vocabulary, B9a the high-risk execution consult precondition, B19 phase-owned routing
  mutation, plus the Closing-a-feature tail). Fidelity for the four claimed anchors:
  min/median/max 1.000 (verbatim re-homing). 136 anchors remain unclaimed for the follow-up
  cells, and `--check workflow-state` reports them honestly (140 anchors, 4 owned, 0 duplicated,
  136 lost) — it is deliberately NOT a chain suite yet, which is why the chain stays green at 74
  suites.

- **The migration is complete** — last group of the split area (cell f2-13, feature
  `okf-migration-f2`): the remaining **136** anchors of `docs/specs/workflow-state.md` are
  authored, the pointer stub carries the full 140-anchor map, and
  `scripts/okf_migrate.mjs --check workflow-state` is wired into the verify chain green at
  **140 anchors, 140 owned, 0 duplicated, 0 lost**, fidelity min/median/max **1.000**. Every
  area that lived in `docs/specs/` is now a knowledge bundle concept set; `docs/specs/` holds
  only pointer stubs plus `reading-map.md` and `okf-profile.md`.
- **The decomposition, and where it departs from D30 — reported, not quietly taken.** D30 locked
  NINE clusters as F2's input (`overview`, `gates`, `cells`, `handoff`, `recovery`,
  `multi-session`, `review-sessions`, `dispatch`, `advisor-consult`) and derived them from the
  BEHAVIOR line anchors. Those nine hold for the 37 behaviors. They do not survive the other
  half of D30's own instruction — that the 58 rules, 25 edge cases and 20 pointer bullets are
  "distributed to the concept each governs, not collected into a dumping-ground concept":
  distributed honestly, `cells` lands 37 anchors and `multi-session` 35, against a per-concept
  norm of 6-9 across the eight areas already migrated. That shape is precisely what F12's
  `anchors_per_concept` ratio exists to detect, and it detected it — at nine concepts the
  telemetry reads 15.56 (2.15x the running median 7.25) and 0.61 concepts/100 source lines
  (0.41x the median 1.50), outside the [0.5x, 2x] band on **both** metrics. So the two oversized
  clusters are split by topic INSIDE themselves, keeping D30's map as the spine:
  `cells` → `cells-authoring-and-revision` / `cells-scheduling` / `cells-attempt-budgets` /
  `cells-completion-judge-and-archive`; `multi-session` → `claims-and-ownership` /
  `sessions-lanes-and-identity` / `holds-and-the-coordination-lock` / `worktree-isolation`.
  Fifteen concepts; telemetry 9.33 anchors/concept (1.29x) and 1.02 concepts/100L (0.68x), both
  inside the band, and no other pinned area is knocked out of its band by the median shift. The
  split is a decomposition change, never a fidelity dodge: no concept was edited and no
  threshold was touched to clear the F11 floor, which every one of the 140 anchors clears at
  1.000.
- **`B23`, the one anchor D30 homes twice.** D30's `cells` list names `B23` explicitly and its
  `multi-session` line names the contiguous range `B20-B24`, which contains it; only one concept
  may claim it. It is homed with the claims cluster (`claims-and-ownership.md`) on the source's
  own evidence: pointer `P18` reads "Multi-session hardening (B11/B21-B24, R36-R40)", so the
  implementation map itself groups `B23` with `B21`-`B24` and the rules that travel with it
  (`R39` in particular). `B7`, `B10`, `B17`, `B18`, `B25`-`B32`, `B34`-`B36` — the rest of D30's
  `cells` list — are unaffected.
- **The `B24` hazard f2-12 measured, carried rather than "fixed".** Because a `###` heading does
  not close an anchor's block, the whole `### Closing a feature — the tail of the chain`
  subsection (L534-581) sits inside **`B24`**'s extracted text. `B24`'s owning concept,
  `sessions-lanes-and-identity.md`, therefore carries that prose verbatim — and `gates.md`
  carries the same prose, because the closing tail is topically its and f2-12 authored it there.
  The prose is deliberately in BOTH concepts; only the anchor CLAIM is unique, so coverage still
  reports 0 duplicated. Nothing was moved out of `gates.md` and nothing was trimmed out of
  `B24`: `B24` measures 281 anchor tokens with 0 missing, ratio 1.000.
- **The stub resolves an old `R19`/`R20`/`R21` citation in both directions** (f2-12's repair):
  the fresh-session-handoff family KEPT those ids and its rows point at `handoff.md`; the
  chain-integrity family now reads `R19a`/`R20a`/`R21a` and its rows point at `gates.md`. Both
  triples carry an explicit "kept this id" / "shipped as a second `R19`" note in the map's `Was`
  column, and a prose section above the map spells out which meaning resolves where — the same
  treatment `hook-runtime`'s `R14`/`R14a` received. All 140 rows are in one table; the
  resolution guidance is prose beside it, never a second row for an id (a duplicated row is a
  gate failure by design).
- Citations rewired in the same cell as the stub (D37): `docs/specs/reading-map.md`'s area entry
  now points at the bundle and names all fifteen concepts, and its two code-line citations
  (`schedule.mjs` → `cells-scheduling.md`, `recovery.mjs` → `recovery.md`) resolve into the
  bundle instead of the retired spec. `docs/history/**` is archive and stays untouched.
  Chain: 75 suites (74 + this area's gate), `SUITE_FLOOR_COUNT` 59 → 60.
- **The profile moved into the bundle it defines** (cell `f3-5`, slice S4, G6) — the twelfth and
  last area migration, and the only one whose source is the spec the bundle is validated
  against. `docs/specs/okf-profile.md` re-authored into five `bee.area` concepts under
  `areas/okf-profile/`, split by TOPIC: `overview.md` (purpose, entry points, actors, open
  gaps), `concept-model-and-authoring.md` (the nine closed types, the frontmatter field rules
  and their id/path direction, per-subject authority, the legacy carry-over map, and the four
  worked templates with the body contract and the rebuild bar), `conformance-check.md` (the
  two-level check and its exact OKF-error / profile-error / profile-warning codes, the
  emitter-first codec, the never-writes boundary), `context-and-promote.md` (the budget-aware
  manifest, the measured relevance ranking, the propose-never-write `promote` loop closer, the
  session preamble), and `migration-and-coverage-gates.md` (the coverage report, the
  content-addressed pin, the unparsed report, the fidelity floor, the drift telemetry, the
  pointer-stub anchor map). The legacy path is now a D37 pointer stub carrying the full 24-row
  anchor map. Coverage is machine-checked by `scripts/okf_migrate.mjs --check okf-profile`,
  pinned to blob `9267d3e` at `53d8111`: **24 anchors, 24 owned, 0 duplicated, 0 lost**;
  fidelity min/median/max all 1.000 against the 0.60 floor (F11), with no concept edited and no
  threshold touched. The gate was proven RED first — one anchor unclaimed reports
  `FAIL … 23 owned, 0 duplicated, 1 lost / LOST in concepts: P7 is claimed by no concept's
  bee.sources` — then restored.
- **`rules: 0` is MEASURED, not a gap.** This source's `## Business Rules` section carries nine
  top-level bullets and not one opens with an `R<n> —` id; they were written as plain prose.
  D10 forbids inventing the nine ids they never had, so `rules` is 0 and those nine blocks are
  counted in `unparsed_blocks` instead. They are still migrated verbatim into the concept whose
  subject each states — what they are not is anchor-gated, exactly as `feedback-digest`'s
  unnumbered behavior prose is not. F9's "report rather than force" was honored: nothing was
  reshaped to make the scheme fit.
- **The code-fence hazard this source carries, pinned rather than papered over.** The anchor
  extractor does not track markdown code fences, so the `bee.area` template's own fenced
  `## Pointers (implementation)` line opens a spurious accounting section over the Templates
  prose, and three bold-lead paragraphs in that stretch land in `unparsed_blocks`. It moves no
  anchor — that stretch holds no top-level `- ` bullet, so `P1`-`P7` remain the real Pointers
  section's seven, verified by dumping each derived anchor's text before the pin was written.
  `expected_counts.unparsed_blocks: 17` is asserted precisely so a future fence-aware extractor
  becomes a loud failure instead of a silent reshaping.
- **The fence's named exception SHRANK by one.** `okf-profile.md` was a named exception in
  `scripts/okf_specs_fence.mjs` protecting the interval between f3-4 and this migration. That
  interval is over, and the exception was REMOVED rather than relabelled: leaving it would have
  been a name-based pass that keeps saying yes if the stub's `migrated_to` is ever dropped —
  precisely the silent-rot the structural stub branch exists to prevent. The file now passes as
  an ordinary structural stub, asserted both ways in `--selftest` (the real stub passes as
  `stub`; an `okf-profile.md` without `migrated_to` fails as new content).
- Citations rewired in the same cell (D37): `bee-scribing`'s template pointer, `bee-hive`'s
  work-item offer, `inject.mjs`'s session-preamble line and `test_bundle_mode.mjs`'s template
  assertion all resolve into `areas/okf-profile/concept-model-and-authoring.md`;
  `reading-map.md`'s entry points at the bundle. `docs/history/**` is archive and stays
  untouched. Chain: 79 suites, `SUITE_FLOOR_COUNT` 60 → 62.
- **Scribing sync for `okf-switchover-f3`** — the feature's own settled behaviour written into the
  bundle it built. Two gaps closed against an audit of what the four `behavior_change` cells had
  actually left captured: the **authoring gate** (the one bundle-mode predicate and its
  "a directory alone is not a bundle" rule, the seven-field scribing-target answer, its three
  fail-closed refusals `fork_denied`/`subject_required`/`duplicate_authority` plus the throw on a
  malformed authority claim, the three-layer anti-fork gate, and both doc trees resolving off the
  declared product root) merged **in place** into the existing owner
  `areas/okf-profile/concept-model-and-authoring.md` — a new `## Behaviors & Operations` section,
  four new business rules and a `## Pointers (implementation)` section, with the frontmatter
  re-emitted rather than hand-edited; and the **read-only fence** authored as a new concept,
  `areas/okf-profile/specs-read-only-fence.md`, claiming the previously unowned subject
  "okf-profile: the docs/specs read-only fence" — structural stub recognition (never by filename,
  because a rotted allowlist stops fencing silently), the closed and reasoned named-exception set,
  placeholders pinned to their emptiness, inertness in a host with no bundle, and the rule that a
  named exception whose interval ends is REMOVED rather than relabelled. f3-1's relevance ranking
  was already fully stated in `context-and-promote.md` (B6b) and needed nothing. The bundle is 122
  concepts across 140 files, 0 errors and 0 warnings; chain 79 suites green.
- **The edges caught up with the core** (`okf-integration-close-f4`). A mechanical audit after the
  switchover found the core correct and guarded, and the *edges* still teaching the retired model —
  the silent-rot class the programme exists to prevent, with no suite red anywhere. Four cells
  closed it. Two of the findings were confirmed by direct observation of a live session preamble
  rather than inferred, and the before-state is preserved at
  `docs/history/okf-integration-close-f4/reports/red-preamble-before.md`: the `Critical patterns`
  digest was printing four lines of YAML delimiters and six of the pointer stub's forwarding
  address — not one lesson — and the `Project map` was counting the read-only compatibility surface
  as "Specced areas: 11" while instructing every session to "read the spec before the code".
  Sharpest documentation gap: the two `bee-hive` reference files the skill explicitly defers to for
  "the full routing table" and "the full pipeline" contained `docs/knowledge` **zero** times
  (measured, against 4 and 1 for `docs/specs`), so an agent following the *full* instructions was
  taught the order G4 replaced. Both preamble sections and all three reference passages now resolve
  through the one bundle predicate, captured in `areas/okf-profile/context-and-promote.md` as **B8**;
  the no-bundle branch is proven byte-identical two independent ways (a permanent bundle-less
  fixture pinning each line as an exact literal, and a whole-preamble byte comparison), so a host
  that never migrated cannot tell the migration happened.
- **The digest lists the NEWEST criticals, and that is load-bearing.** The bundle index's critical
  rows sit in date order, so a first-N cut would have surfaced the ten oldest lessons forever —
  swapping the stub's arbitrary cut for another one. The digest therefore states the total, lists
  the most recent, and names the full index; it deliberately does not rank, because ranking needs a
  work item to rank against and a preamble has none. Degradation is silence, never a fallback: with
  no generated index the digest emits nothing rather than re-printing the retired file.
- **A false red the harness itself was manufacturing** (`f4-4`, captured in
  `areas/verify-pipeline/concurrency-and-hermetic-runs.md`). The verify runner scrubbed two identity
  variables for hermeticity and missed a third — the agent-name prefix bee's *own* swarm rule
  mandates on write-heavy commands. Obeying that rule and then running the configured verify leaked
  the name into every spawned suite, where it became the acting agent identity inside the write
  guard and inverted "the acting session's own hold must never block its own write". Reproduced
  deterministically: 31 passed / 0 failed without the variable, 28 passed / 3 failed with it. It
  fooled two execution workers and the orchestrator in one sitting before the third worker found
  it. The acceptance test is now stated as a property — the chain passes with the prefix set and
  without it, identically.
- **Grooming can finally see this bundle.** Its debt hunt carried no notion of the layer that now
  holds the truth, so bundle rot was invisible to the one skill whose job is finding rot. It now
  counts only what the bundle already proves mechanically — the conformance check's own findings and
  areas with code but no concept — never a hand-maintained list, and without inventing the
  `knowledge stale` verb that remains backlog P68.
- **The instruction layer got a gate, and the sequence that forced it is the point** (`f4-5`,
  `f4-6`). Three successive hand audits of one question each found gaps the previous audit had
  missed — seven, then two, then six — with the chain green before, during and after all three. The
  third audit's own conclusion was that a fourth would find an eleventh.
  `areas/okf-profile/the-instruction-layer-fence.md` now owns the rule that ends the sequence: in
  bundle mode, a line on a governed instruction surface that names the retired state layer must
  carry its branch **on that line**. Line-local is not a stylistic choice — the document-level rule
  was built first, measured, and discarded because it passed **four of six** misroutes humans had
  already found by hand: one skill named the bundle four times and still called the retired pattern
  file mandatory pre-work reading, and one hook was bundle-aware in its logic while still printing
  the wrong destination. An agent reads a bullet or an injected message in isolation, and a branch
  three paragraphs above does not travel with it.
- **A guard that routed an agent into another guard.** Run red against pristine HEAD, the new fence
  named 24 misroutes across 10 files, nine of which no audit had found. The sharpest was the
  session-close capture nudge: its logic had already been migrated — it measures staleness across
  both trees — so in a migrated repo it fires precisely **because the bundle is stale**, and then
  told the agent to write into the compatibility surface, which the read-only fence fails the chain
  for accepting. The logic was migrated, the message was not, and nothing connected the two.
  Recorded as R5 there: a guard's message is part of that guard.
- **What that fence deliberately does not do.** It proves a branch is *present* on a line, never
  that it is *correct* — a line naming both trees but describing them backwards passes, and grading
  meaning stays the rebuild bar's job. A cross-reference naming no path ("startup step 5 is
  unchanged", pointing at a step that had since moved and changed) is structurally invisible to it;
  that limit is stated rather than papered over. And UI visual snapshots still have no home in the
  bundle — recorded as an Open Gap in `concept-model-and-authoring.md`, not a home invented to close
  a red.

## 2026-08-26

- **A product description of the whole bee product now exists at `docs/product-description/`.**
  34 documents on one eight-section skeleton written from the agent's point of view, a glossary, a
  goal statement, four verification checklists (715 rows), and `bug-triage.md` with 24 deduplicated
  defects — six of them high, including a secret-guard Bash walk-around, an `=true` boolean
  inversion, commands advertised but not built, and a review preflight reporting `passed: true`.
  It lives inside the beehive repo because the write guard blocks external paths. It is an
  outside-in behavior description, not part of this bundle's authority: where it and an area spec
  disagree, the spec and the code win, and the disagreement is a defect to file.

## 2026-08-31

- **Compounded `dispatch-one-door`.** Three prose surfaces (`gates-and-delegation.md`,
  `swarming-reference.md`, and the model-guard's own FIX strings) each told an agent a different
  preferred dispatch shape for the same operation, and all three disagreed with `bee dispatch
  prepare`, the resolver that already computed the right answer. The fix touched no dispatch
  logic — it pointed every reader and every refusal at the resolver verb instead of letting each
  one describe the resolver's current output in prose. Knowledge sync (hook-runtime
  `dispatch-guard.md` B16a/B18) had already landed at cap time; this pass added the decision log
  entry for D2 (direct `subagent_type: bee-*` naming stays legal where the slot is model-shaped)
  and promoted the generalizable half as
  `patterns/20260831-prose-that-recommends-a-shape-drifts-from-the-resolver-that-computes-it.md`.
- **Flushed the capture queue's touches-sweep and promote-proposal backlog** (245 pending stubs:
  197 touches-sweep citation reconciliations across 40 decision pairs, 28 promote-proposal
  reviews, 20 direct settlement merges) — dispatched to parallel workers partitioned by disjoint
  target file so no two workers touched the same doc.
