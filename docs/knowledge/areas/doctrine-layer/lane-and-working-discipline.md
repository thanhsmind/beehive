---
type: bee.area
title: Doctrine Layer — lane classification and working discipline
description: "The standing rules that size a piece of work and govern its by-products: work-packet-first tiny/small shapes, product-file-only caps, test-anchored risk flags, classification before context loading, evidence-based re-lane demotion, diff-scaled plan-check ceremony, the shape-time survivors of the retired predictive-validation stage, the one canonical scratch home and its narrowed disposable-proof exemptions, the never-author-as-evidence rule, and the verify ladder."
timestamp: 2026-07-28
bee:
  id: doctrine-layer-lane-and-working-discipline
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [areas/doctrine-layer/overview.md]
  decisions: ["lane-ceremony-v3 D1-D10 (docs/history/lane-ceremony-v3/CONTEXT.md, 2026-07-19)", "f21efe6e (tree-hygiene D1/D4 — one canonical scratch home, the write-guard that enforces it)", I51 (issues-46-53 — claim-triggered baseline), I53a (issues-46-53 — sweeper reaches what the guard produces), "8ef2bae6 (cli-ergonomics D3 — scoped red-first, the small-lane parallel criterion, schema-first; 2026-07-24)", "lane-lean D1/D2 (evidence demotion default-on, ladder reaches tiny; 2026-07-27)", lane-lean D3 (small-diff standard review wave runs inline; 2026-07-27), "validation-diet D1/D5/D6/D9/D10 (docs/history/validation-diet/CONTEXT.md, 2026-07-28)", "proactive-leader-intake D3/D4 (423e1664, b34fdea9 — the plan check is the plan-step hat wave on advisor-kind seats; 2026-08-30)", "pstack-adoption D1/D2/D3/D4/D5 (docs/history/pstack-adoption/CONTEXT.md, 2026-09-01 — the route class gets a procedure: four cited playbooks, the perf class, the checkable-CoS refusal, and the dismissed-findings duty)"]
  sources: ["lane-ceremony-v3 cells lcv3-1..lcv3-4 (traces in .bee/cells/, reports docs/history/lane-ceremony-v3/reports/, 2026-07-19 — plan freeze, lane work-packet shapes, product-file caps, test-anchored flags, intake-first classification; each RED-first against the doctrine assertion suite)", "tree-hygiene (cell th-6, 2026-07-21 — write-guard scratch-shape denial + the three competing prose homes collapsed into one doctrine rule)", "docs/specs/doctrine-layer.md#R13", "docs/specs/doctrine-layer.md#R14", "docs/specs/doctrine-layer.md#R15", "docs/specs/doctrine-layer.md#R16", "docs/specs/doctrine-layer.md#R17", "issues-46-53 cells i-3 (GH #51 — the baseline gate is claim-triggered and lives in the execution discipline; GH #53-adjacent — a guard that directs writes obliges the sweeper to reach there; traces in `.bee/cells/`, 2026-07-23)", "lane-lean cells ll-1..ll-2 (traces in .bee/cells/, 2026-07-27 — re-lane demotion default-on + tiny ladder, diff-scaled review wave; skill-text cells, verified by skill_lint + release-manifest check)", "validation-diet cells vd-9/vd-12 (traces in .bee/cells/, reports docs/history/validation-diet/reports/, 2026-07-28 — the predictive-validation stage's two survivors folded into the shape step, the feasibility matrix and its delta rule retired with no replacement, the never-author-as-evidence rule added, and the disposable-proof area narrowed to two uses)"]
  authoritative_for: "doctrine-layer: lane classification and working discipline"
---

# Doctrine Layer — Lane Classification and Working Discipline

These rules ride the standing sheet because they decide how a piece of work is
*sized* and where its by-products *land* — questions that arrive before any
stage is invoked. The unnumbered verify-ladder rule travels with R17: both
govern the working residue of a cell rather than its content.

## Business Rules

- **R18** — A route class carries a PLAYBOOK, and a plan CITES it rather than
  copying it (pstack-adoption D1 as superseded by `132551fb`, D2, D3; cells
  psa-1..psa-6, 2026-09-01). `bee route --set --class` was a validated enum that
  no procedure read: nothing branched on the value, so `bugfix` and `refactor`
  scaled ceremony identically. Four classes now carry a named step list, homed
  once in `bee-planning/references/planning-reference.md` ("Class playbooks") —
  `perf` (baseline, change, re-measure; "it feels faster" is not a result),
  `bugfix` (reproduce, watch it fail, find the mechanism, fix, re-run the same
  reproduction), `refactor` (record existing behavior first and keep that record
  green at every step; a behavior change is a separate cell), and `research`
  (read-only, trace the runtime path, name the sources that came up empty).
  `perf` is the eighth enum value; it is safe beside the `mode`-carries-a-class
  leak (`close.rs`, `uat.rs`) only because it is absent from
  `ROUTE_LANE_VALUES`, and `tests/route_class_parity.rs` now pins the vocabulary
  across all four documents that name it, reading both lists out of
  `workflows.rs` as text rather than re-declaring them. The plan cites the
  playbook by name and anchor and never transcribes it: a copied step list goes
  stale and can be satisfied by transcription (`docs/history/research/pstack-xia.md`
  lists verbatim playbook todo-lists under *What must not be ported*). A step
  that does not apply stays VISIBLE with its recorded reason — the existing
  named-deviation rule — and is never a refusal. The playbook binds the ROUTE
  class only; the cell-level `change_class` enum is a different taxonomy.
- **R19** — Two duties keep an agent's own judgment auditable (pstack-adoption
  D4, D5; cells psa-5, psa-6, 2026-09-01). The herding dispatch role refuses a
  candidate whose Condition of Satisfaction names no command, path, or state it
  can evaluate, and its skip line says so — a reading rule, never a keyword
  list; it fires at dispatch time only, and the authoring-time gap is filed,
  not closed. And a review report carries the findings it DISMISSED, each with
  the reason it went. That one lands in two places or it is decorative: the
  reviewer's instrument records the drop, and the report renders it. An
  invisible filter asks the human to trust the lead's judgment; a shown one
  lets them reverse it.

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
- **R16b — The plan check's ceremony scales with the measured diff (lane-lean
  D3, 2026-07-27), and the check itself is the plan-step hat wave
  (proactive-leader-intake D3/D4, `423e1664` / `b34fdea9`, 2026-08-30).** The
  check lives in `bee-planning` (validation-diet D1/D5 folded the deleted
  standalone validating stage's reality gate and review wave in; there is no
  standalone `validating` stage or phase). A standard-lane feature whose
  counted touch set is at most 5 product files with zero hard-gate flags runs
  the check inline on the session model — the same two mandates (structural
  adversarial check, cold-pickup cell review), the same finding vocabularies,
  the same one-shot-then-one-blocker-pass cap — without dispatching anything.
  A hard-gate flag, a sixth product file, or genuine doubt about self-review
  independence dispatches the hat wave instead: `hat-*` seats dispatched
  `--kind advisor`, three by default and all five on high-risk, whose
  synthesis carries the structure mandate back to the leader. The cold-pickup
  mandate never dispatches — it stays with the leader at cell drafting on both
  paths. The procedure itself — seats, instruments, budget, quorum, the
  advisor-ref absorption and its timing law — has exactly one home,
  `skills/bee-hive/references/gates-and-delegation.md` ("Hat wave"); this rule
  points there and never restates it. Rationale on record: dispatched ceremony
  on a 4-5-file diff costs more than it catches, and ceremony must never
  displace the main task (standing user feedback 5794a92a).
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
  2026-07-27).** The live rule: no door runs tests. The agent owns test
  scope, picks the narrowest proof its change type needs, and `bee cells
  finish` RECORDS that proof line and refuses a red one
  (rule: agents-proof-at-cap). Proving a new suite load-bearing by mutation
  is owed only when it guards high-risk/hard-gate behavior, at most one
  cycle, optional elsewhere. A cell's verify field carrying the impacted or
  full chain is a planning defect the worker refuses — the impacted run
  belongs to the slice close, never inside a cell. Origin: one session paid
  five broad suite runs and a test-heavy diff for a ~40-line guard (standing
  user feedback, strengthens 5794a92a).
  Superseded notes, in order: per-cell verify commands and the verify-once
  economics retired (412e9b3a, 2026-07-31, docs/specs/test-simple.md); the
  cap-time suite run retired in turn (decisions 58ec9664 and 1f534837,
  2026-08-18).
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
  issue when red. The live rule: `bee cells finish`, `bee close`, and
  `bee worktree merge` each CHECK the proof line already recorded on the cap
  and run no test themselves; CI is the one place the full declared command
  still runs, on every push (rule: agents-proof-at-cap).
  Superseded notes, in order: `commands.test` at every finish/close and a
  re-run at merge (412e9b3a, 2026-07-31, docs/specs/test-simple.md); that
  cap-time run retired (decisions 58ec9664 and 1f534837, 2026-08-18).

  **The claim is still the trigger, not arrival.** It is stated claim-first,
  in the execution discipline rather than in any startup checklist, because a
  conditional rule rendered inside an unconditional list reads as
  unconditional: an agent working a numbered "every session" list
  top-to-bottom used to run a minute-long chain to answer a question that
  touched no cell. A session that answers, reads or explores without ever
  claiming owes no CI check either. Nothing about the gate's strength
  changed — a red result is still surfaced and still becomes its own
  fix-first cell (rule: agents-never-build-on-red). What changed is the
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
  `.bee/expertise/tests.md`, not as machinery. Since 2026-08-18 the end-of-cell
  run is neither full nor executed by the door: the worker runs the narrowest
  proof its change type needs and the door only records it.)* (2) *The small-lane parallel criterion* —
  serial stays the default; cells may run in parallel only when every cell's
  file set INCLUDING regen targets (release manifest, onboarding ledger,
  plugin mirrors) is provably disjoint; any shared generated artifact forces
  serial; in doubt, serial. (3) *Schema-first* — load a command group's
  schema (`bee <group> --help --json`) before its first use in a session:
  one roundtrip beats a flag-error ladder. All three came out of one session
  audit where ~40% of orchestrator calls were retries while total CLI wall
  time was under one second — the scarce resource is roundtrips, never CLI
  runtime.
