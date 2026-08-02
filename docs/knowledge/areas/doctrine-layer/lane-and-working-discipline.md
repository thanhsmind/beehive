---
type: bee.area
title: Doctrine Layer — lane classification and working discipline
description: "The standing rules that size a piece of work and govern its by-products: work-packet-first tiny/small shapes, product-file-only caps, test-anchored risk flags, classification before context loading, evidence-based re-lane demotion, diff-scaled review-wave ceremony, the shape-time survivors of the retired predictive-validation stage, the one canonical scratch home and its narrowed disposable-proof exemptions, the never-author-as-evidence rule, and the verify ladder."
timestamp: 2026-07-28
bee:
  id: doctrine-layer-lane-and-working-discipline
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [areas/doctrine-layer/overview.md]
  decisions: ["lane-ceremony-v3 D1-D10 (docs/history/lane-ceremony-v3/CONTEXT.md, 2026-07-19)", "f21efe6e (tree-hygiene D1/D4 — one canonical scratch home, the write-guard that enforces it)", I51 (issues-46-53 — claim-triggered baseline), I53a (issues-46-53 — sweeper reaches what the guard produces), "8ef2bae6 (cli-ergonomics D3 — scoped red-first, the small-lane parallel criterion, schema-first; 2026-07-24)", "lane-lean D1/D2 (evidence demotion default-on, ladder reaches tiny; 2026-07-27)", lane-lean D3 (small-diff standard review wave runs inline; 2026-07-27), "validation-diet D1/D5/D6/D9/D10 (docs/history/validation-diet/CONTEXT.md, 2026-07-28)"]
  sources: ["lane-ceremony-v3 cells lcv3-1..lcv3-4 (traces in .bee/cells/, reports docs/history/lane-ceremony-v3/reports/, 2026-07-19 — plan freeze, lane work-packet shapes, product-file caps, test-anchored flags, intake-first classification; each RED-first against the doctrine assertion suite)", "tree-hygiene (cell th-6, 2026-07-21 — write-guard scratch-shape denial + the three competing prose homes collapsed into one doctrine rule)", "docs/specs/doctrine-layer.md#R13", "docs/specs/doctrine-layer.md#R14", "docs/specs/doctrine-layer.md#R15", "docs/specs/doctrine-layer.md#R16", "docs/specs/doctrine-layer.md#R17", "issues-46-53 cells i-3 (GH #51 — the baseline gate is claim-triggered and lives in the execution discipline; GH #53-adjacent — a guard that directs writes obliges the sweeper to reach there; traces in `.bee/cells/`, 2026-07-23)", "lane-lean cells ll-1..ll-2 (traces in .bee/cells/, 2026-07-27 — re-lane demotion default-on + tiny ladder, diff-scaled review wave; skill-text cells, verified by skill_lint + release-manifest check)", "validation-diet cells vd-9/vd-12 (traces in .bee/cells/, reports docs/history/validation-diet/reports/, 2026-07-28 — the predictive-validation stage's two survivors folded into the shape step, the feasibility matrix and its delta rule retired with no replacement, the never-author-as-evidence rule added, and the disposable-proof area narrowed to two uses)"]
  authoritative_for: "doctrine-layer: lane classification and working discipline"
---

# Doctrine Layer — Lane Classification and Working Discipline

These rules ride the standing sheet because they decide how a piece of work is
*sized* and where its by-products *land* — questions that arrive before any
stage is invoked. The unnumbered verify-ladder rule travels with R17: both
govern the working residue of a cell rather than its content.

## Business Rules

- **R13** — Small work starts from an executable work packet, never a shrunken
  feature plan (lane-ceremony-v3 D3/D4/D5). The tiny lane's complete work shape
  is the request plus one work unit — the unit is the micro-plan, carrying the
  touched paths, the directive, the acceptance contract, the verification
  command, and the classification record (flag count, product-file count, lane);
  no plan document exists. The small lane's default shape is a short scoping
  synthesis logged through the decision log plus one-to-three units; a plan
  document is opt-in, written only when a durable multi-slice strategy genuinely
  needs one. In both lanes the approval order is fixed: draft unit(s) are
  previewed in the approval message, the inline reality check runs, THEN the one
  merged shape+execution approval is asked (or auto-recorded under bypass), and
  only after approval are units persisted and claimed — execution approval is
  never granted before the execution package exists, and never
  persist-then-preview.
- **R14** — Lane caps count product files only (lane-ceremony-v3 D6):
  production source, tests, and runtime configuration the behavior change
  itself must touch. Workflow bookkeeping, history and specification documents,
  plans/briefs/reports, and generated projections or manifests never count
  toward a lane cap — the workflow's own artifacts can never promote a change
  out of its honest lane.
- **R15** — The two experience-based risk flags are test-anchored
  (lane-ceremony-v3 D7): "changes behavior an existing test asserts (a covered
  contract must change)" and "the change requires weakening, deleting, or
  replacing existing proof". A covered bugfix that keeps existing tests green
  and adds a new one scores zero on both. The remaining flags and the
  2-3→standard / 4+→high-risk thresholds are unchanged.
- **R16** — Classification precedes context loading (lane-ceremony-v3 D8): the
  planning stage classifies the lane first from the request plus at most two
  targeted reads, then loads context scaled to the lane — targeted reads only
  for tiny, bounded for small, full bootstrap for standard and high-risk. The
  critical-patterns digest stays mandatory in every lane (it already rides the
  session preamble at zero extra cost). The lane decision re-runs upward any
  time evidence demands escalation; de-escalation requires cited evidence.
- **R16a — Evidence demotion is default-on and the ladder reaches tiny (lane-lean
  D1/D2, 2026-07-27).** The single per-feature re-lane checkpoint, fired after the
  first evidence pass, demotes to the smallest lane the measured evidence honestly
  supports: when the counted product-file touch set is within the target lane's
  threshold (small: at most 3; tiny: at most 2 plus one direct task), zero
  hard-gate flags sit on that touch set, and no gray area remains open, demotion
  is the default — staying in the heavier lane requires naming which condition
  actually failed. High-risk work never demotes, a hard-gate flag is an absolute
  floor, the checkpoint fires at most once per feature, and promotion on
  discovered risk stays available always. A demoted lane carries its full target
  contract (a demoted tiny is a real tiny: one direct task, merged gate, one
  dispatched worker).
- **R16b — The review wave's ceremony scales with the measured diff (lane-lean
  D3, 2026-07-27).** The wave lives in `bee-planning` (validation-diet D1/D5
  folded the deleted standalone validating stage's reality gate and review wave in; there
  is no standalone `validating` stage or phase). A standard-lane feature whose
  counted touch set is at most 5 product files with zero hard-gate flags runs
  the review wave inline on the session model — the same two mandates
  (structural adversarial check,
  cold-pickup cell review), the same finding vocabularies, the same
  one-shot-then-one-blocker-pass cap — without dispatching a separate reviewer.
  A hard-gate flag, a sixth product file, or genuine doubt about self-review
  independence restores the dispatched merged reviewer; high-risk always runs
  the dispatched persona panel. Rationale on record: the dispatched wave on a
  4-5-file diff costs more than it catches, and ceremony must never displace
  the main task (standing user feedback 5794a92a).
- **R16d — There is no longer a separate stage that judges a plan before code
  exists; its two worthwhile checks now run inside the shape step itself
  (validation-diet D1/D5/D6, 2026-07-28).** The predictive stage's matrix of
  feasibility checks, and its rule for handling a plan that drifted from
  reality mid-flight, are retired outright with no replacement — the
  empirical check bee already runs after code exists (build the change, then
  prove it against reality, undo cleanly if it fails) makes a redundant
  predictive check wasted effort for work that is cheap to revert. Two things
  that stage used to do are worth keeping, and both now run the moment a
  shape is drafted, inside the planning work itself: first, one inline
  question, asked every lane — is there a cheaper shape than the one just
  chosen that still honors every locked decision. Second, the review wave
  named in R16b — dispatched alongside the shape work rather than after it
  finishes, so when it does dispatch, the wait is whichever of the two takes
  longer, never their combined length, with its findings held until the
  merged approval.
- **R16c — Test runs are the scarce resource (test-runs-lean D1/D2,
  2026-07-27).** *(Superseded in part 2026-07-31 — decision 412e9b3a,
  docs/specs/test-simple.md: per-cell verify commands and the verify-once
  economics are retired; `bee cells finish` now runs the one declared
  `commands.test` at every cap and writes the result record the
  orchestrator reads instead of re-running. The mutation-proof scoping
  survives as craft. Kept as the historical record.)* Verify-once: in a
  serial tiny/small dispatch the worker's
  recorded verify output is the cap evidence — the orchestrator repeats the
  command only when the report smells, the wave ran parallel workers, or the
  cell is high-risk/hard-gate; proof stays recorded output, it just is not paid
  for twice. Proving a new suite load-bearing by mutation is owed only when it
  guards high-risk/hard-gate behavior, at most one cycle, optional elsewhere.
  A cell's verify field carrying the impacted or full chain is a planning
  defect the worker refuses — the impacted run belongs to the slice close,
  never inside a cell. Origin: one session paid five broad suite runs and a
  test-heavy diff for a ~40-line guard (standing user feedback, strengthens
  5794a92a).
- **R17** — There is one canonical scratch home (`f21efe6e`): every ephemeral
  file bee writes for its own working purposes — judge payloads, evidence/
  deviation files, batch inputs, digests, verify logs, probe/debug scripts,
  review manifests — goes to `.bee/tmp/<feature-or-session>/`; disposable
  feasibility code goes to `.bee/spikes/<feature>/`. Deliverables (reports,
  specs, decisions, backlog, the cell/decision stores, plugin renders) keep
  the paths their own workflow stage already requires — never rerouted
  through scratch. A write-guard denies a scratch-shaped write that targets a
  tracked directory instead, naming `.bee/tmp/` in the refusal; scratch is
  swept at feature close and session finish via `bee tmp sweep`. Three
  procedure references used to each state a partial, competing version of
  this home — they now cite this rule instead.

- **R17a — A guard that directs writes somewhere obliges the sweeper to reach
  there.** The write-guard's refusal names the scratch **root** and tells the
  author to write there; the sweeper's own inventory saw only directories, so
  every plain file written exactly where the guard sent it was unreachable by
  every flag — including the one documented as clearing the lot. Bee was
  contradicting itself: one half of the rule directed the write, the other half
  could not see it, and the gap was invisible because both halves independently
  looked correct. The sweeper's inventory therefore covers what the guard
  actually produces, and the per-feature sweep reaches a feature's artifacts
  whether they sit in that feature's own directory or loose in the root under
  bee's own `<feature>-<n>` cell-id naming.

  Two safeguards ride the widened reach, because a sweeper that deletes more is
  a sweeper that can delete wrongly. Containment is unchanged — every candidate
  is proved inside the scratch home when the removal is planned and proved again
  immediately before it happens. And a name-prefix match is an **inference**,
  not the exact-name override: it requires a separator boundary so a short
  feature name can never swallow a longer unrelated one, and it refuses to
  remove a sibling that is itself live, reporting that refusal rather than
  swallowing it. The general rule: when one mechanism decides where by-products
  go, the mechanism that reclaims them is specified against that same shape, not
  against the shape someone assumed.

- **R17b — An artifact is never authored for the sole purpose of being
  deleted once it has served as evidence, and the scratch spike location is
  narrowed to exactly two legitimate uses (validation-diet D9/D10,
  2026-07-28).** Evidence is what the ordinary course of building already
  produces — a failing check's own output, the trail an error leaves, the
  record that a verification command ran clean, or the difference between
  two states already committed — never a throwaway artifact staged purely to
  produce a string and then discarded. A proof that some prior behavior held
  is written at the real location where it will actually ship and stays
  there; the reason is auditability, since a deleted artifact leaves only a
  claim about something nobody can inspect anymore. Exactly one class of
  artifact is exempt: a feasibility proof taken on deliberately before a
  change is judged safe to build directly, owed only when the change moves
  data, touches security, reaches an outside system with a side effect, or
  leans on a technique the repository has never used before — never as a
  routine step everything passes through. The scratch spike location itself
  now holds only two legitimate uses — that opt-in feasibility proof, and the
  throwaway visual sketch used while resolving a gray area with a human eye —
  and is never an evidence store.
- **The verify ladder (cli-performance D4, `e54878b1`) — retired by
  ci-owned-verify D1/D6:** a cell's verify is its TARGETED suite (seconds),
  run red-first and green by the worker; the full configured chain is now
  CI-owned and never runs locally. Its former four milestones are all
  superseded: the session-first-claim moment is now a cheap CI-status check
  (latest full-verify run on the base branch plus any open `verify-red`
  issue, never a local run), wave close and worktree merge both run the
  impact-registry-scoped `commands.test` instead of the full chain, and the
  full chain itself runs on push in CI, auto-filing a deduped `verify-red`
  issue when red. *(Amended 2026-07-31 — decision 412e9b3a,
  docs/specs/test-simple.md: `commands.test` now runs at every
  `bee cells finish` and again at `bee close`; `bee worktree merge`
  re-runs `commands.test` against the staged merge as the last net.)*

  **The claim is still the trigger, not arrival.** It is stated claim-first,
  in the execution discipline rather than in any startup checklist, because a
  conditional rule rendered inside an unconditional list reads as
  unconditional: an agent working a numbered "every session" list
  top-to-bottom used to run a minute-long chain to answer a question that
  touched no cell. A session that answers, reads or explores without ever
  claiming owes no CI check either. Nothing about the gate's strength
  changed — a red result is still surfaced and still becomes its own
  fix-first cell, and building on red is still forbidden. What changed is the
  proof itself: a local run became a CI-status read, and the four milestones
  collapsed into one CI-owned full pass plus registry-scoped local checks.
  full-run-retirement (cell frr-1) completed the retirement: the session-finish
  obligation and the release gate — the last two local full-suite mandates —
  now run the impacted registry scope as well, never the full chain locally
  (repair recorded by scribing-integrity D5, 2026-07-24).
  Judges and reviewers never run the full chain as part of a verdict. Proven
  the day the ladder first landed: the wave-close run caught a real escape
  (raw NUL bytes in a lib file) that every targeted suite had missed — now
  the impacted run's job. Companion performance
  idiom for derived read paths (D1/D2, cells cp-1/cp-2): shared inputs are
  read once per call and threaded down — never re-read per item — and
  repeated child-process answers are memoized in a pass-local map that dies
  with the pass; no cross-call caches, no TTLs, no daemons.

- **Three roundtrip disciplines (cli-ergonomics D3, `8ef2bae6`):** (1) *Scoped
  red-first* — the red run executes only the tests the cell adds or changes;
  the full targeted verify chain runs exactly once, at the end, before cap
  (a full-suite red loop is the named waste: one audited worker ran 271 tests
  per loop for 4 new assertions). *(Since 2026-07-31 / 412e9b3a the
  end-of-cell run is the declared `commands.test` executed by
  `bee cells finish`; red-before-green itself survives as craft in
  `.bee/expertise/tests.md`, not as machinery.)* (2) *The small-lane parallel criterion* —
  serial stays the default; cells may run in parallel only when every cell's
  file set INCLUDING regen targets (release manifest, onboarding ledger,
  plugin mirrors) is provably disjoint; any shared generated artifact forces
  serial; in doubt, serial. (3) *Schema-first* — load a command group's
  schema (`bee <group> --help --json`) before its first use in a session:
  one roundtrip beats a flag-error ladder. All three came out of one session
  audit where ~40% of orchestrator calls were retries while total CLI wall
  time was under one second — the scarce resource is roundtrips, never CLI
  runtime.
