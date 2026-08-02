# 01 — Distillation: What bee Takes From Each Upstream

bee follows the khuym method: read each upstream system, keep what holds up in practice for one developer, reject what adds weight without adding safety. This document is the audit trail of those choices.

## Summary matrix

| Idea | Source | Lands in bee as |
|---|---|---|
| 7-stage chain with explicit artifact handoffs | khuym | The bee chain (unchanged skeleton) |
| Human gates, never skipped (originally 4, validation-diet D2 merged shape+execution into one) | khuym | Gates 1, 2, 3 in `bee-hive` |
| Socratic decision locking, one question at a time, D-IDs | khuym / superpowers / gsd | `bee-shaping` ("Explore"/"Lock") |
| Mode gate: smallest honest workflow | khuym | Mode gate in `bee-planning` |
| Reality gate (SMALLER PATH survivor) + spikes (feasibility matrix deleted, D6) | khuym / gsd | `bee-planning` (folded in from the deleted standalone validating stage, validation-diet D1/D5) |
| Reservation-based worker isolation, `[DONE]/[BLOCKED]/[HANDOFF]/[NOOP]` | khuym | `bee-swarming` (orchestrator + "Execute" worker) |
| ~65% context budget → HANDOFF.json pause/resume | khuym | `bee-hive` priority rule |
| Plans as executable prompts (`must_haves`: truths/artifacts/key_links/prohibitions) | gsd-core | Cell + plan format in `bee-planning` |
| Goal-backward adversarial plan-checker with BLOCKER/WARNING severity | gsd-core | Plan-checker subagent in `bee-planning`'s review wave (folded in from the deleted standalone validating stage, validation-diet D5) |
| 4-level deviation rules for executors | gsd-core | `bee-swarming` ("Execute") |
| Wave-based parallelism from a dependency graph | gsd-core | `bee-swarming` |
| Research depth levels 0–3 | gsd-core | `bee-planning` scout step |
| EXISTS / SUBSTANTIVE / WIRED artifact verification | gsd-core / khuym | `bee-reviewing` |
| Three-lane mechanical risk model + hard-gate flags | repository-harness | Merged into bee's mode gate |
| Context phase × lane reading matrix with token budgets | repository-harness | `bee-hive` scout contract |
| Friction capture → backlog with predicted-vs-actual outcomes | repository-harness | `bee-grooming` |
| Entropy audit / hive health score | repository-harness | `bee-grooming` |
| Decision records with lifecycle + optional verify command | repository-harness / gstack | `decisions.jsonl` + `docs/decisions/` |
| Event-sourced decision log (decide/supersede/redact, append-only) | gstack | `.bee/decisions.jsonl` |
| Learnings JSONL injected into future session preambles | gstack / khuym | `bee-capturing` ("Compound") + `bee-hive` bootstrap |
| Cross-model second opinion at contentious gates | gstack | Optional step at Gates 2 and 3 |
| Docs generated from code where code owns the truth | gstack | bee build script (later phase) |
| Context isolation: task + interfaces + constraints only, never history | claudekit / superpowers | Worker spawn contract |
| File-based agent communication (reports/ dir, no MCP required) | claudekit | Worker results + review reports |
| Diff-aware testing (map changed files → affected tests) | claudekit | `bee-swarming` ("Execute") verify step |
| 12-dimension edge-case decomposition | claudekit | Reference checklist in `bee-planning`/`bee-reviewing` |
| Privacy/scout blocking of secrets and generated dirs | claudekit | `bee-write-guard` hook (Claude Code) + guardrail text (Codex) |
| Hook automation skeleton: config-gated, fail-open, injection-deduped, chain-nudging, state-syncing hooks over shared `lib/` | claudekit | 9-script hook skeleton in [06-runtime-integration.md](06-runtime-integration.md) |
| Unified plan artifact enriched in place (`artifact_readiness: requirements-only → implementation-ready`) | compound-engineering | `docs/history/<feature>/plan.md` frontmatter contract |
| Model tiers: extraction / generation / ceiling for subagent dispatch | compound-engineering | Spawn guidance in `bee-swarming` / `bee-planning` (review wave) / `bee-reviewing` |
| Headless mode: `mode:headless` on every skill — defer ambiguous decisions to a report, never hang on a question | compound-engineering | Shared skill standard |
| Severity corroboration: independent reviewers agreeing promotes a finding one level | compound-engineering | `bee-reviewing` synthesis rule |
| Autofix classes (gated_auto / manual / advisory) + owner routing as *signal, not gates* | compound-engineering | `bee-reviewing` finding schema |
| Verification-evidence contract when `behavior_change: true` | compound-engineering | `bee-swarming` ("Execute") trace + `bee-reviewing` gate |
| Learnings searched structurally by planning and review (precedent injection), not just stored | compound-engineering / gstack | `bee-planning` bootstrap + reviewing roster |
| Residual findings: durable file fallback when tracker filing fails | compound-engineering | `bee-reviewing` finishing step |
| Surface scope earlier: skip exploration when acceptance criteria + pattern refs already given | compound-engineering | `bee-hive` routing check |
| Iron Law: no skill without a failing pressure test | superpowers / khuym | [docs/handbook/writing-skills.md](handbook/writing-skills.md) |
| Description = trigger conditions only, never workflow summary | superpowers | Skill-writing checklist |
| Evidence-before-claims verification gate | superpowers | `bee-swarming` ("Execute") + `bee-reviewing` |
| Model selection per task complexity ("turn count beats token price") | superpowers | `bee-swarming` spawn guidance |
| Session-start hook injecting the routing skill | superpowers | SessionStart hook on both runtimes; AGENTS.md block as the bootstrap fallback |

## Per-framework audit

### khuym (the skeleton)

**Keep (wholesale):**
- The chain and its handoff contract — every skill reads upstream artifacts, writes downstream artifacts, and ends with `[Outcome]. Invoke [next-skill].`
- `docs/history/<feature>/CONTEXT.md` as locked source of truth; `docs/history/learnings/critical-patterns.md` as mandatory pre-planning context.
- `.khuym/`-style runtime dir → `.bee/` (state.json, HANDOFF.json, reservations.json, onboarding.json).
- Worker loop: initialize → accept assigned work → reserve → implement → verify → close → release → return status token.
- Red-flags lists in every skill (they are the anti-rationalization teeth).
- Onboarding script that installs the AGENTS.md block, repo guardrails, and runtime files, with `--apply` gated on approval.
- Communication contract: plain-language first, scenario before jargon, answer in the order summary → current behavior → why it matters → concrete scenario → next step.

**Change:**
- **Beads (`br`/`bv`) become optional.** khuym hard-depends on the beads CLI for its task graph; that is its heaviest external dependency. bee's native task unit is the **cell**: a JSON file in `.bee/cells/` with id, title, status, deps, acceptance criteria, verify command, and lane. A small vendored CLI (`bee cells`) gives list/ready/claim/cap operations. If `br` exists, an adapter can mirror cells into beads — but nothing in the chain requires it.
- **gkg becomes optional.** Treat it as one entry in a capability registry (see repository-harness) with a documented grep fallback, not a first-class dependency that turns skills "unavailable."

**Reject:**
- The dependency posture machinery sized for many external CLIs (cass, cm, gkg, br, bv). bee declares dependencies the same way but ships with approximately zero required ones.

### gsd-core (the verification brain)

**Keep:**
- **Plans are prompts, not documents.** Cells and plan artifacts carry `must_haves`: observable `truths`, expected `artifacts` (path + what makes it substantive), `key_links` (what must be wired to what), and `prohibitions`. Task bodies use directive prose with files / read-first / action / verify / done.
- **Goal-backward plan-checker** with the adversarial stance ("assume the plan is flawed until evidence proves otherwise") and mandatory severity on every finding (BLOCKER / WARNING). bee trims gsd's 12 dimensions to 5: requirement coverage, task completeness, dependency correctness, key links, scope sanity.
- **Deviation rules** for executors: auto-fix bugs (1), auto-add missing critical functionality (2), auto-fix blocking issues (3), STOP and ask for architectural changes (4); package installs always checkpoint (slopsquat).
- **Scope-reduction prohibition:** if the work exceeds the context budget, return SPLIT RECOMMENDED — never silently deliver a "v1"/placeholder of a locked decision.
- **Decision traceability:** cell actions cite decision IDs ("per D3"); deferred ideas must not appear in plans.
- Wave analysis for parallel execution; verifier that re-checks only prior gaps on re-runs.
- Research levels 0–3 (skip / quick verify / standard / deep dive) as the planning scout dial.

**Reject:**
- 20+ runtime adapters, the 170KB reference docs, config schema sprawl, hardcoded 4-researcher fan-out (bee spawns researchers declaratively, only the ones the phase needs), and STATE.md frontmatter complexity.

### superpowers (the discipline)

**Keep:**
- **TDD-for-skills** (khuym already carries this; bee keeps khuym's version): RED (baseline pressure scenarios without the skill, capture rationalizations verbatim) → GREEN (minimal skill addressing observed failures only) → REFACTOR (close loopholes, rationalization table, red flags) → validate & CREATION-LOG.
- **Skill Discovery Optimization:** description = triggering conditions only. A workflow summary in the description makes agents follow the description and skip the body. Every time.
- "Violating the letter of the rules is violating the spirit of the rules" as standard anti-loophole text.
- Evidence-first completion: identify the proving command, run it fresh, read the full output, only then claim. Red-flag words: "should", "probably", "seems to".
- Subagent dispatch hygiene: one task, its interfaces, global constraints — no pasted session history (a real dispatch hit 42k chars of which 99% was history).
- Model selection guidance: cheap model for mechanical tasks, capable model for architecture and final review; specify the model explicitly or you silently inherit the most expensive one.
- Session-start hook that injects the routing skill (bee ports this for Claude Code; on Codex the AGENTS.md block plays this role).

**Reject:**
- Overlapping execution skills (`executing-plans` vs `subagent-driven-development` vs `dispatching-parallel-agents`) — bee has exactly one orchestrator (`swarming`) and one worker (`executing`). Platform-specific prose embedded in skill bodies goes to `references/`.

### claudekit (the context economy)

**Keep:**
- **Context isolation protocol:** subagents get ~100 tokens of task context (naming, paths, acceptance, constraints), never the conversation. This is the spawn contract for all bee subagents.
- **File-based communication:** workers and reviewers write markdown reports to `docs/history/<feature>/reports/` with deterministic names; the orchestrator reads files, not memories. Works offline, diffable, no MCP required.
- **Diff-aware testing:** map changed files → co-located tests → mirror dirs → import graph; full suite only when config changed or >70% affected.
- **12-dimension edge-case scouting** (user types, input extremes, timing, scale, state transitions, environment, error cascades, authorization, data integrity, integration, compliance, business logic) as a reference checklist used by planning (test matrix) and reviewing (coverage probe).
- **Privacy/scout blocking:** never read `.env`/secrets without explicit approval; never burn context on `node_modules`/`dist`. Hook-enforced on both runtimes, with the guardrail text as the fallback wherever hook execution is unconfirmed.
- **The hook automation skeleton** (adopted after a second, closer read — see [06-runtime-integration.md](06-runtime-integration.md)): claudekit's hooks are not decoration, they are the machine that keeps the workflow honest. bee ports the five load-bearing patterns: config-gated hooks (`isHookEnabled` against one config file), fail-open crash wrappers with logging, **injection dedup** (context reminders reserve a scope and skip when recently injected), **chain-nudging via SubagentStop matchers** (claudekit's `cook-after-plan-reminder`: when a Plan agent stops, the harness itself tells the main agent the next stage — bee generalizes this to its whole chain), and **state persistence as a side effect** (`session-state.cjs` on PostToolUse/SubagentStop/Stop keeps state files fresh without model discipline). Plus the `lib/` extraction pattern: hook logic lives in shared modules so another runtime's integration reuses the same brain.

**Reject:**
- 14 agents + 40 skills + a 16-hook sprawl with overlapping concerns (bee: 11 decision-gated skills, 9 thin hooks wrapping shared `lib/` modules), env-var context injection (bee puts subagent context inline in the spawn prompt), multi-kit distribution, configurable naming DSL.

### repository-harness (the measurable hive)

**Keep (reimplemented file-based, no Rust binary):**
- **Mechanical risk flags:** auth, authorization, data model, audit/security, external systems, public contracts, cross-platform, existing behavior, weak proof, multi-domain. 0–1 flags → tiny/small; 2–3 → standard; 4+ or any hard-gate flag → high-risk. The mode gate stops being a judgment call.
- **Policy ≠ operations:** markdown docs describe *how* to work; JSON/JSONL records store *what happened* (cells, traces, decisions, backlog) so they can be queried and audited.
- **Trace tiers by lane:** tiny work records a one-line summary + outcome; high-risk work records actions, files, decisions, errors, friction. Mechanical field checks, agent can self-score.
- **Friction capture triggers** (verbatim-worthy): had to infer a missing rule; validation unclear or too expensive; stale/contradictory doc; repeated manual step that should be a template; out-of-scope but important; unattributable failure. Only record friction when one trigger fires.
- **Backlog outcome loop:** every grooming/backlog item records *predicted* impact at creation and *actual* outcome at close.
- **Entropy score:** orphaned cells ×10 + unverified cells ×5 + stale decisions ×5 + backlog-without-outcome ×2 + stale work ×3 + broken tools ×8, capped at 100. Grooming reports it every run.
- **Capability-based tool registry:** steps ask for a capability ("code-graph", "browser-test"); absent capability is a clean documented fallback, never a failure. This is how gkg/beads/cass become optional.

**Reject:**
- The Rust CLI + platform binaries + checksum installer, SQL schema migrations, the 20-command CLI surface, benchmark-repo coupling, rigid phase sequencing, 17-doc bulk.

### gstack (the memory and the second opinion)

**Keep:**
- **Event-sourced decisions:** `.bee/decisions.jsonl`, append-only, three event kinds (`decide`, `supersede`, `redact`); "active" is computed, never edited. Each event: decision, rationale, alternatives considered, scope, date, source, confidence. Write-time secret/PII rejection; datamark on read so resurfaced text can't act as instructions.
- **Learnings injection:** top-N relevant learnings and recent active decisions are surfaced at session bootstrap (`bee-hive`), not just stored.
- **Cross-model dispatch as a gate feature:** at Gate 2 and Gate 3, optionally ask the *other* runtime's model for a second opinion. Agreement is a strong signal to report; disagreement is surfaced to the user verbatim. Never auto-resolve.
- **Docs from code** (deferred to a later bee phase): generate command references in skills from the actual helper scripts, so docs can't drift.
- **Three layers of knowledge** framing for research: tried-and-true, new-and-popular, first-principles — prize layer 3, log it as a learning when found.
- **Outcome-framed questions:** "what breaks for users if X?" instead of "should we X?".

**Reject:**
- 6000-line skills, 50+ helper binaries, the browser daemon, review-army monolith, per-IDE conditional paths, LLM-judge eval costs during early development.

### compound-engineering (the compounding loop, industrialized)

Every's CE plugin (29 skills, `/lfg` full-pipeline orchestrator, TypeScript converter layer targeting 8 runtimes) is the most production-hardened implementation of the philosophy bee's `compounding` stage comes from: 80% of effort into planning and review, 20% into execution, and every cycle feeding the next.

**Keep:**
- **Unified plan artifact.** Brainstorm and plan write to *one* document with an `artifact_contract` and `artifact_readiness: requirements-only | implementation-ready` frontmatter, enriched in place — no handoff ambiguity about which doc is canonical. bee adopts this: `docs/history/<feature>/plan.md` starts as the shape (requirements-only) at Gate 2 and is enriched to implementation-ready during post-approval prep, instead of separate `shape.md` + story-pack files.
- **Model tiers as semantic cost classes.** Subagent dispatch declares `extraction` (cheapest capable: retrieval, quoting, evidence gathering), `generation` (mid: evidence-driven work, mechanical verification), or `ceiling` (orchestrator's own model: judgment calls). On runtimes without per-agent model selection, tiers degrade to the inherited model and cost control falls back to read budgets and output caps. This formalizes superpowers' "turn count beats token price" into something declarable per spawn.
- **Headless-first mode design.** Every CE skill accepts `mode:headless`: run without user prompts, apply only unambiguous actions, classify ambiguous cases as deferred/stale, and end with a structured report containing an Outstanding Questions section. This is what makes skills orchestratable — a pipeline step must never hang on a question. bee adopts it as a shared skill standard (gates remain human-only; headless mode defers *within-stage* ambiguity, it never self-approves a gate).
- **Review synthesis mechanics:** independent parallel personas score severity independently, and **corroboration across reviewers promotes a finding one level**; every finding carries an `autofix_class` (`gated_auto` / `manual` / `advisory`) and an owner — explicitly *signal for routing, not an apply gate*; disagreement resolves to the more conservative route.
- **Verification-evidence contract.** When a change carries `behavior_change: true`, review requires structured `verification_evidence` (tests inspected, tests added/changed, red-failure or characterization evidence, verification run, deliberate exceptions). Missing or vague evidence sends the work back — it never passes forward silently.
- **Structural precedent injection.** Planning and review don't merely have access to learnings — their grounding phases *require* searching them ("we've solved X before; here's what we learned"), and review runs a dedicated learnings-researcher persona before synthesis. bee already injects critical-patterns at bootstrap (khuym); CE's addition is the tag-matched search inside planning and reviewing.
- **Residual findings durability.** Deferred P2/P3 findings are filed to a tracker; if filing fails they land in a durable fallback file and the PR body lists them. Nothing evaporates because an API call failed.
- **Surface scope earlier.** Before exploring, check whether the user already supplied acceptance criteria and pattern references; if so, offer to skip straight to planning with a scoping confirmation. Respects users who know what they want.

**Adopt later (roadmap phase 4, proven but not spine):**
- **Repo-profile cache:** a question-agnostic project profile (stack, deps, conventions) derived once per repo+HEAD and cached, shared by all grounding skills — a real token saver, but tmp-cache plumbing bee doesn't need on day one.
- **Plan-review persona panel** (coherence + feasibility always-on; product/design/security/scope lenses conditional): bee's single adversarial plan-checker with 5 dimensions covers the standard lane; the panel is the high-risk-lane upgrade.
- **Feedback sweep** (`ce-sweep`): deterministic ingestion of Slack/GitHub/email feedback through a state machine into an execution-ready plan — a natural grooming extension, deferred and single-source (GitHub Issues) first.

**Reject:**
- 29 skills and the all-in-one surface (bee keeps its skill count small, with decision-gated additions — decision 0002); motivational prose that changes no behavior; phase-checkpoint narration ("now entering Phase 2…"); 80-line token-parsing preambles (extract to a reference); byte-duplicated persona files across skills without a shared-include mechanism; HTML output mode; session-history mining in compounding (learnings files are the durable source); the full codex-delegation state machine (consent flows, sandbox modes, circuit breakers — bee's swarming contract stays simple).

## The four ideas bee adds that no single upstream has

1. **One task model across the whole chain.** Upstreams split "plan task", "bead", "story", "trace" across different stores. bee's **cell** carries plan fields (`must_haves`, deps, lane), execution fields (reservation, status, verify command), and trace fields (outcome, friction) in one JSON record with lane-scaled strictness.
2. **Grooming as a first-class stage.** Upstreams treat debt as review fallout (khuym P3s), audit output (harness), or a dashboard (gstack `/health`). bee gives it a dedicated skill with its own loop: audit entropy → hunt candidates (dead code, stale docs, unverified cells, friction clusters) → propose kills with predicted impact → execute as tiny/small cells → record actual outcomes.
3. **Dual-runtime as a contract, not a port — one brain, two belts.** The workflow artifacts (`.bee/`, `docs/history/`, cells, gates) are runtime-neutral, and the rules are enforced from one shared `bin/lib/` codebase in two ways: the CLI helpers enforce mechanically on *both* runtimes (cap-requires-verify, gate-locked claiming, reservation conflicts), and the 9-script hook skeleton — rendered from one shared catalog and wired for both runtimes — adds harness-level guards, deduped reminders, and chain-nudges on top. Codex is not a degraded copy; it runs the same helper-enforced rules with AGENTS.md as its bootstrap vector. Full design and parity matrix: [06-runtime-integration.md](06-runtime-integration.md).
4. **A state layer next to the logs (decisions 0001, 0002).** Every upstream memory mechanism is history-shaped — append-only decisions, dated learnings, per-feature docs. bee adds the complementary state-shaped artifacts: `docs/specs/<area>.md` (a *BA-grade, technology-agnostic* functional spec of a long-lived area — field meanings, behaviors, role visibility, business rules — overwritten to match reality, rebuildable-on-another-stack by design) and `docs/specs/reading-map.md` (what lives where). A dedicated BA discipline, `bee-capturing`'s "Scribe" section, writes them: syncing `behavior_change` cell deltas after review, capturing rules agreed in discussion the moment they're agreed, and harvesting first specs for pre-bee areas. The hive scout reads the touched area's spec before its code; grooming's entropy score measures spec rot (`stale specs` term); compounding guards the handoff. Log answers "how did we get here"; spec answers "where are we" — a fresh session reads spec → decisions → history, in that order. Full design: [02-architecture.md](02-architecture.md) §state layer, [decisions/0001-state-layer.md](decisions/0001-state-layer.md), [decisions/0002-scribing-skill.md](decisions/0002-scribing-skill.md).
