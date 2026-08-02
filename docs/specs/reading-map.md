# Reading Map

Where things live. Read the touched area's spec before its code.

Cold start — no area in mind yet:
[`docs/codebase-overview.md`](../codebase-overview.md) is the one-read map of
the tree as it stands after the R6 Node cutover: entry points, what lives in
each module, the conventions that hold, and the recorded test command. Read it
before this file when you do not yet know which area you are in.

## Area specs

- [`docs/knowledge/areas/decision-memory/`](../knowledge/areas/decision-memory/index.md) — what
  the system remembers about its own decisions, migrated into the knowledge bundle
  (okf-foundation D20/D29; okf-migration-f2 F9): the three field failures, mandatory write-time
  classification against a canonical taxonomy, append-only retro-tagging with read-time overlay,
  the derived index as the guaranteed recall surface (reading order spec → decision index →
  history), the supersede citation sweep that makes a reversal reach every citing artifact, the
  archive split with union reads, and the per-CoS-clause done-flip rule (GH #32/#33/#34) — one
  concept, `overview.md`, since the source is one coherent topic at nine rules. The legacy path
  [`decision-memory.md`](./decision-memory.md) remains as a pointer stub carrying the full
  9-anchor map (D37).
- [`docs/knowledge/areas/onboarding/`](../knowledge/areas/onboarding/index.md) — what onboarding
  installs and keeps current in a host project, migrated into the knowledge bundle
  (okf-foundation D20/D29; okf-migration-f2 F9): purpose, the check/apply run modes and the
  actors (`overview.md`); the opt-in status-display pair — detection, vendoring, drift healing,
  staying out of non-opted projects, what the line renders, and the second runtime's
  machine-level status block (`status-display-vendoring.md`); the delimited ignore-list block
  onboarding owns, what it silences and what always stays version-tracked
  (`managed-ignore-section.md`); plugin-first/repo-copy exclusivity, the Codex hybrid carve-out,
  proof-gated cleanup in both directions and the whole-run snapshot
  (`distribution-source-exclusivity.md`); the installer entry points — sparse source staging,
  the complete release identity, and a present-but-broken runtime tool
  (`installer-entrypoints-and-source-staging.md`); one release version across every projection,
  the downgrade refusal, honest drift reporting, source-origin classification and the force
  blast radius (`release-identity-and-version-parity.md`); the remembered guardrail opt-in and
  the second runtime's lifecycle hook wiring (`repo-local-guardrails.md`); and the artifacts a
  host keeps — instructions import, unified dispatcher, state-layer landing pages, annotated
  config sample (`host-project-artifacts.md`). Unresolved live-install surfaces remain Open
  Gaps. The legacy path [`onboarding.md`](./onboarding.md) remains as a pointer stub carrying
  the full 58-anchor map (D37, `R20b` included).
- [`docs/knowledge/areas/hook-runtime/`](../knowledge/areas/hook-runtime/index.md) — the
  lifecycle guardrails around the assistant, migrated into the knowledge bundle
  (okf-foundation D20/D29; okf-migration-f2 F9): the frame every checkpoint sits inside —
  purpose, actors, hostile-input immunity, and "the guard's silence is never permission"
  (`overview.md`); one catalog of record rendered into per-runtime projections with every
  difference named, plus whether a project's checkpoints are enabled, rooted and trusted enough
  to run (`catalog-projections-and-activation.md`); the request shapes the write guard can read
  — batch envelopes, workflow-command shape checks, both recognised command forms
  (`write-guard-request-shapes.md`); which write targets are governed, the only-ever-shrinking
  always-writable set, and the intake gate that reads the phase rather than a closed feature's
  leftover approvals (`governed-paths-and-the-intake-gate.md`); the advisory contract,
  session-stop output, and the one deliberate turn-control exception
  (`advisories-and-turn-control.md`); the two rendering targets and the launch contract a
  fallback command owes, Windows form included (`delivery-targets-and-the-fallback-command.md`);
  proof-gated arbitration keeping exactly one hook source active (`hook-source-exclusivity.md`);
  pre-spawn judgement of tier, model and helper type (`dispatch-guard.md`); the deliberate
  override pass-through and the capability probe's scoped verdict
  (`native-spawn-and-transport-classification.md`); the three checkpoints that observe without
  authorising (`child-agent-attribution-and-audit.md`); the durable state a checkpoint maintains
  as a side job (`coordination-refresh-and-session-init.md`); and how the guardrails are
  inspected by a human and proven by the chain (`health-checks-and-proof-surfaces.md`). Named
  coverage gaps remain Open Gaps. The legacy path [`hook-runtime.md`](./hook-runtime.md) remains
  as a pointer stub carrying the full 81-anchor map (D37) and the record of the duplicate `R14`
  that was disambiguated to `R14` + `R14a` before the migration pin was captured.
- [`docs/knowledge/areas/doctrine-layer/`](../knowledge/areas/doctrine-layer/index.md) — the
  standing instructions an assistant always carries, migrated into the knowledge bundle
  (okf-foundation D20/D29): purpose, vocabulary, actors and open gaps (`overview.md`); what
  belongs on the always-loaded sheet versus a stage's procedure reference — a rule needed when
  no stage is running is silently absent from every such turn if buried in a reference — how
  doctrine reaches every project by copy, and the anchor tests that keep a rule from drifting
  back out of the layer (`placement-and-anchoring.md`); the rules with nothing enforcing them —
  an unblocked action is not an approved action, doctrine is not gated, the machinery is run by
  the assistant, its vocabulary stays out of the conversation (`unenforced-obedience.md`); the
  delegation threshold, gather delegates and deciding never does (`delegation-threshold.md`);
  the two helper classes, the read-only capability surface and the gather-only external-command
  tier (`helper-classes-and-transports.md`); the native Codex empty-wait discipline
  (`native-wait-discipline.md`); the lane-ceremony contract — work-packet-first tiny/small
  shapes, product-file-only caps, test-anchored risk flags, intake-first classification — plus
  the one canonical scratch home and the verify ladder (`lane-and-working-discipline.md`). The
  legacy path [`doctrine-layer.md`](./doctrine-layer.md) remains as a pointer stub carrying the
  full 39-anchor map (D37).
- [`docs/knowledge/areas/advisor-protocol/`](../knowledge/areas/advisor-protocol/index.md) —
  second opinions for workers and the orchestrator, migrated into the knowledge bundle
  (okf-foundation D20/D29): purpose, vocabulary, and actors (`overview.md`); who may consult
  the configured adviser and the mandatory pre-approval consult for high-risk work
  (`triggers.md`); the stuck-worker consult loop, the read-only rule, and what advice may
  never do — approve, override, write (`consult-loop.md`); configuration authority,
  transports, and event-based staleness (`slots-and-tiers.md`). The legacy path
  [`advisor-protocol.md`](./advisor-protocol.md) remains as a pointer stub carrying the
  full anchor map (D37).
- [`docs/knowledge/areas/workflow-state/`](../knowledge/areas/workflow-state/index.md) —
  the durable workflow record, migrated into the knowledge bundle (okf-foundation
  D20/D29/D30): purpose, entry points, the full Data Dictionary and actor access
  (`overview.md`); the guarded feature-start that can never inherit approvals or bury
  unfinished work, the closed phase vocabulary, the high-risk execution consult
  precondition, phase-owned routing mutation, the gate-bypass ladder, and the closing tail
  (`gates.md`); user-invoked review sessions with frozen scope (`.bee/reviews/`), the
  append-only candidates ledger, and derived review statuses (verified/unreviewed/in
  review/reviewed/review stale) (`review-sessions.md`); the unified nine-group command
  entry point and its catalog (`dispatch.md`); the worker adviser consult — a stuck worker
  asks a configured stronger model, on failure only, budgeted (`advisor-consult.md`); how
  a unit of work is authored and its plan revised, with the plan document frozen at its
  gate (`cells-authoring-and-revision.md`); the computed dispatch schedule and the
  dependency cycle refused at the write (`cells-scheduling.md`); the append-only attempt
  history, lifetime claim/failure budgets enforced at the claim door, and the audited reset
  door (`cells-attempt-budgets.md`); the behavior-class completion floor on proof-of-red
  evidence, the append-only structured judge verdict with its honest model-independence
  stamp and the reopen it forces on a capped unit, and the journaled archive transaction
  (`cells-completion-judge-and-archive.md`); the two-kind handoff and the cross-lane work
  puller (`handoff.md`); crash-candidate detection and transcript mining (`recovery.md`);
  atomic single-winner claims with TTL + heartbeat on every claim path, typed refusal
  codes, gate-protected adoption/reclaim, and ownership-checked claimed-unit mutation with
  an audited force door (`claims-and-ownership.md`); the self-derived session identity,
  per-feature lanes every reader resolves through, and throttled self-renewing heartbeats
  and leases (`sessions-lanes-and-identity.md`); cross-session file holds and the
  serialized coordination-store lock behind concurrent reservation/state writes
  (`holds-and-the-coordination-lock.md`); and opt-in isolated worktree dispatch with one
  validated main coordination store, canonical contained reservation checks, and
  transactional merge/revert/preservation rules (`worktree-isolation.md`). The legacy path
  [`workflow-state.md`](./workflow-state.md) remains as a pointer stub carrying the full
  140-anchor map (D37), including the disambiguation rows for the duplicate
  `R19`/`R20`/`R21` the source shipped.
- [`docs/knowledge/areas/okf-profile/`](../knowledge/areas/okf-profile/index.md) — the Bee OKF
  Profile, migrated into the bundle it describes (G6). Purpose, the entry-point verbs, actors and
  open gaps (`overview.md`); the nine closed concept types plus the practice/pitfall polarity bit,
  frontmatter field rules with the id-derives-path direction, authority uniqueness (`bee.id`,
  `bee.authoritative_for`), the legacy-frontmatter carry-over map, the declared (not yet executed)
  retirement of `implement-plan.md`, the standing exemption for `docs/decisions/index.md`, and the
  four canonical worked examples with the body contract and the rebuild bar, plus the authoring
  gate that resolves where a settled truth is written — the one bundle-mode predicate, the
  seven-field scribing-target answer, its three fail-closed refusals, and the three-layer anti-fork
  gate (`concept-model-and-authoring.md`); the exact OKF-error/profile-error/profile-warning finding
  codes `bee knowledge check` emits (`conformance-check.md`); the budget-aware `context` manifest,
  its measured relevance ranking, `promote`'s propose-never-write contract and the session preamble
  (`context-and-promote.md`); the per-migration coverage report, the content-addressed pin, the
  fidelity floor, the drift telemetry and pointer-stub anchor maps
  (`migration-and-coverage-gates.md`); and the read-only fence over this very tree — structural
  stub recognition, the closed named-exception set, pinned unwritten placeholders, and inertness in
  a host with no bundle (`specs-read-only-fence.md`). Bee's own contract, not a ratified OKF
  standard. The legacy
  path [`okf-profile.md`](./okf-profile.md) remains as a pointer stub carrying the full 24-anchor
  map (D37).

## Not yet specced

- The workflow's skills themselves (`skills/bee-*`) have no area
  specs by convention. Their contracts live in `docs/07-contracts.md` and in each skill's own
  `SKILL.md` (creation logs under `docs/decisions/skills/`); `scripts/render_openai_metadata.mjs`
  projects each canonical frontmatter identity into `agents/openai.yaml` for Codex, and its
  `--check` mode guards drift. The self-improvement loop itself is a maintainer guide,
  `docs/handbook/evolving.md`. The self-improvement *process* behavior is specced in
  `docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md` (B5).
- `docs/specs/system-overview.md` does not exist. Offered, not yet written.

## Elsewhere

- `scripts/lib/run-module-worker.mjs` — shared isolated test-entrypoint runner
  for onboarding, hook, command, metadata, and concurrency verification.
- Communication doctrine (plain language, Gate Presentation Contract, Silent Bookkeeping —
  bee mechanics never narrated into chat, work language only; decision 1689af1b) lives in
  `skills/bee-hive/references/routing-and-contracts.md` § Communication Contract, mirrored as
  hive law 11 (`skills/bee-hive/SKILL.md`) and host critical rule 10 (`packages/bee/AGENTS.block.md`).
  The *placement* rule behind every such mirror — always-applies doctrine belongs in
  `AGENTS.block.md`, a stage's own procedure detail may stay in `references/` — is specced in
  `docs/knowledge/areas/doctrine-layer/placement-and-anchoring.md` (R1/B2); read it before
  authoring or relocating any rule.
- `docs/history/learnings/critical-patterns.md` — mandatory pre-work rules for agents.
- `docs/history/<feature>/` — how a feature was decided, planned, validated, reviewed, and shipped.
- `docs/decisions/` — numbered design decisions. `.bee/decisions.jsonl` — the live decision log.
- `docs/backlog.md` — the product backlog. `.bee/backlog.jsonl` — friction and findings.
- `.bee/state.json` and `.bee/backlog.jsonl` are **CLI-owned**: every mutation goes through `bee.mjs state` (generic routing changes require the selected record's pre-change phase as owner; gates use their dedicated verb; worker/scribing-run remain dedicated; `worker prune` cleans `.bee/workers` transients — prefix keep-set, fail-closed destructive verbs) or `bee.mjs backlog add`; direct edits are denied by the write-guard, and a standing suite keeps `packages/bee/` byte-identical to `.bee/bin/`.
- `docs/history/research/` — standalone `bee-researching` briefs (topic-slug files; each leads with its Bottom Line).
- `packages/bee/hooks/` — the catalog and handlers for the installed plugin projection and generated repository fallback. It declares the pre-spawn model-tier guard on both runtimes (Claude via its dispatch tools, Codex via its native spawn call on the observed envelope — unobserved shapes pass through open) and the paired Codex-only child-start/child-stop audit; onboarding activates exactly one source. Vendored handlers live under `.bee/bin/hooks/`.
- `packages/bee/scripts/plugin_distribution.mjs` and `test_plugin_distribution.mjs` — shared strict distribution planner/prover and transaction suite. `scripts/install.sh`, `scripts/install.ps1`, and release-inventory tests are the two platform entrypoints and package proof.
- [`docs/knowledge/areas/verify-pipeline/`](../knowledge/areas/verify-pipeline/index.md) — test
  topology & discovery, migrated into the knowledge bundle (okf-foundation D20/D29;
  okf-migration-f2 F9): per-module suites (the monolith is deleted; 430-check conservation),
  convention-based suite discovery in `scripts/run_verify.mjs` (zero-registration; serial by
  filename convention), the floor+existence manifest guard, and the shared fixture helper
  (`scripts/lib/test-fixture.mjs`) split by TOPIC into `suite-topology-and-discovery.md`; the
  lock+tmp-swap plugin-tree render, multi-worker checkout etiquette, and hermetic session-id
  scrubbing proven by deterministic race/isolation suites into
  `concurrency-and-hermetic-runs.md`. The legacy path
  [`verify-pipeline.md`](./verify-pipeline.md) remains as a pointer stub carrying the full
  14-anchor map (D37). Read before adding/moving any test file.
- [`docs/knowledge/areas/performance-log/`](../knowledge/areas/performance-log/index.md) — the
  global cross-project performance log, migrated into the knowledge bundle (okf-foundation
  D20/D29; okf-migration-f2 F9): sections summarizing a piece of work's per-model token cost
  (new/cached/total), parallelism, and active running time, plus a cross-project HTML matrix
  (`~/.config/beehive/performance.html`) that auto-refreshes at session close. Driven by the
  `bee perf start|stop|section|log|render|report` command group + `maybePerfRefresh` in
  `packages/bee/hooks/bee-session-close.mjs`. Core, cross-project scan, and HTML renderer live in
  `packages/bee/lib/perf.mjs`. Split by TOPIC into `sections-lifecycle-and-measurement.md`,
  `persistent-store-and-sync.md`, and `cross-project-matrix.md`. The legacy path
  [`performance-log.md`](./performance-log.md) remains as a pointer stub carrying the full
  23-anchor map (D37).
- [`docs/knowledge/areas/feedback-digest/`](../knowledge/areas/feedback-digest/index.md) — how a
  repository turns its own workflow records into a safe portable snapshot, how the maintainers'
  repository reads other repositories' snapshots without trusting them, and how the collected view
  is ranked and fed to the gated self-improvement process, migrated into the knowledge bundle
  (okf-foundation D20/D29; okf-migration-f2 F9). Split by TOPIC into `data-model.md` (the six
  allowed fields, the closed `kind` vocabulary, `pain`, and `dropped`), `generation-and-refresh.md`
  (producing a repository's own digest as a side effect of closing a feature),
  `cross-repo-trust-boundary.md` (the maintainers' repository reading another repository's digest
  as hostile input), and `ranking-and-self-improvement.md` (grouping the collected view and the
  gated self-improvement process). The legacy path
  [`feedback-digest.md`](./feedback-digest.md) remains as a pointer stub carrying the full
  29-anchor map (D37).
- [`docs/knowledge/areas/worktree-parallelism/`](../knowledge/areas/worktree-parallelism/index.md)
  — how one session fans independent work into git worktrees, each running its own bee
  lifecycle and reconciled to the main checkout on `git merge`, migrated into the knowledge
  bundle (okf-foundation D20/D29; okf-migration-f2 F9). Split by TOPIC into `overview.md` (the
  two different kinds of parallelism — P40's swarm-worker worktrees that only remove git-index
  contention, versus this area's independent-feature worktrees — and what stays out of scope);
  `the-trust-model.md` (a worktree gets its own store only when GRANTED from the main store's
  registry, keyed by the git-verified id, and why a self-claiming marker inside the worktree
  changes nothing); `entering-creating-and-registering.md` (`worktree new` as the paved road,
  `worktree register` for adoption, the bootstrap contract, and every typed zero-mutation
  refusal); `returning-and-the-merge-gate.md` (the staged `--no-commit` merge, the verify gate
  that catches a semantic conflict before anything is committed, and `--cleanup`);
  `routing-and-visibility.md` (D9's prose routing rule, the lane-first refinement that defers
  the grant to Gate 2's execution component, and the notices an ungranted worktree prints); `cross-worktree-holds.md`
  (the shared path-keyed ledger in the main store, its single-lock atomic acquisition,
  heartbeat renewal, three read taps and cell-scoped release); and
  `store-tiers-and-where-it-lives.md` (the log/cache/runtime tiers that decide what a merge may
  carry back, plus the module/resolver/CLI/test map). This is the ONE area whose source carried
  no numbered anchors at all, so its anchors are the source's own `## ` headings, slugified —
  the `narrative-sections` scheme, added for it (F9/D10). The legacy path
  [`worktree-parallelism.md`](./worktree-parallelism.md) remains as a pointer stub carrying the
  full 10-anchor map (D37).
- `packages/bee/bee.mjs` + `packages/bee/lib/command-registry.mjs` — the sole shipped CLI (`bee.mjs <group> <verb>` over all 10 command groups, the 10th being `perf`; originated as an additive dispatcher in harness-integration-adopt, decision 30606de4, `docs/decisions/0024`, then made the sole canonical *and* sole shipped surface by shim-retire, D1, decision bbc6bcea — the 9 legacy per-group shims are deleted); `command-registry.mjs` is the single source of truth for the command surface. Contract in `docs/07-contracts.md`; spec-before-code still applies — read the touched area's spec before this code.
- `packages/bee/lib/schedule.mjs` — the computed work schedule (`computeSchedule`/`detectCycles`: dep layering + declared-path overlap packing into waves; consumed by `bee cells schedule`, cycle refusal in `cells.mjs` add/update, and the swarming/validating prose). Spec: `docs/knowledge/areas/workflow-state/cells-scheduling.md` (B17/B18, R26/R27).
- `packages/bee/lib/recovery.mjs` — crash-recovery transcript mining (`detectCrashCandidates`/`readTranscriptTail`/`hasCleanEndTrio`/`lastDurableSettlement`/`computeMiningWindow`/`buildMiningPrompt`: stale-heartbeat + non-clean-transcript-tail + work-in-flight detection; bounded mining window; the down-tier miner prompt). Consumed by `bee recovery scan|window` and the `status` recovery block; imports `perf.mjs` (transcript resolution) + `claims.mjs` (heartbeat/session), never imported by `command-registry.mjs`. Spec: `docs/knowledge/areas/workflow-state/recovery.md` (B33, R51).
