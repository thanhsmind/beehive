# 04 — Skills Specification

Per-skill build specs. Each entry defines frontmatter, body budget, required sections, and the pressure scenarios that must exist (and fail without the skill) before the skill is written — per the Iron Law.

## Skill-writing discipline (applies to every skill below)

From superpowers via khuym, unchanged in substance:

1. **Iron Law:** no skill or skill edit without a failing pressure test first. Wrote the skill first? Delete it. No exceptions.
2. **RED:** 3–5 pressure scenarios combining ≥3 pressures (time, sunk cost, authority, exhaustion, economic stakes); run *without* the skill; capture rationalizations **verbatim**.
3. **GREEN:** write the minimal skill that addresses only the observed rationalizations. Hypothetical content bloats the skill and gets skipped.
4. **REFACTOR:** each new violation is a regression — add explicit negation, extend the rationalization table and red flags, re-run all scenarios.
5. **Description = purpose clause + trigger conditions.** One short imperative purpose sentence first (it is what users see next to the /slash command in the menu, CE-style), then "Use when…" triggering conditions; third person, ≤1024 chars. NEVER a workflow/step summary — a step summary makes agents follow the description and skip the body. Include symptoms and error keywords for discovery.
6. **Budgets:** SKILL.md < 200 lines; overflow goes to exactly one level of `references/`. Cross-reference other skills by name (`Invoke bee-planning`), never inline their content.
7. **Standard anti-loophole text:** "Violating the letter of the rules is violating the spirit of the rules."
8. Ship with a CREATION-LOG.md recording scenarios, rationalizations found, and fixes.

Frontmatter conventions (khuym): bare hyphen-case `name` matching the directory; `metadata.dependencies` as a mapping keyed by dependency id with `kind`, `command`/`server_names`, truthful `missing_effect`, `reason` — or `dependencies: []`. bee skills should mostly be dependency-free (the vendored `.bee/bin/bee` binary only).

---

## The skill map today (9 skills)

The tree consolidated to nine skills (commit `93b95d2b`); one-line descriptions from each skill's current frontmatter:

| Skill | Description |
|---|---|
| `bee-hive` | Route the bee workflow: session start, the next skill, gates, and onboarding — including the gate-bypass level (off/normal/full/total). |
| `bee-shaping` | Shape fuzzy intent into locked, buildable decisions — one front door for the interviewer (Explore), the triage judge (Qualify), the decision scribe (Lock), and the plan renderer (Brief). |
| `bee-planning` | Shape approved-scope work into an executable plan — classify the lane, research just enough, draft the smallest honest shape, gate it, and prepare current-slice cells. |
| `bee-swarming` | Run approved cells to done — orchestrate bounded workers over gate-approved cells, or execute exactly one assigned cell inside a dispatched worker. |
| `bee-reviewing` | Run the multi-agent review gate — severity findings, artifact verification, and user acceptance — over an immutable scope the user explicitly asked to review. |
| `bee-capturing` | Capture what settles into durable records — area specs (Scribe), decisions and learnings (Compound) — the moment it settles. |
| `bee-researching` | Evidence-labeled research into unfamiliar, ambiguous, or version-sensitive territory. |
| `bee-grooming` | Hunt and kill tech debt in the current project — dead code, stale docs, TODO/stubs, duplication, drifted specs. |
| `bee-herding` | Drive the bee-herding cockpit's three roles — bootstrap, dispatch, and merge — for autonomous backlog work in fresh worktrees. |

The maintainer guides moved out of the product: skill-writing discipline lives in [handbook/writing-skills.md](handbook/writing-skills.md) (templates under `handbook/writing-skills-references/`) and the self-improvement loop in [handbook/evolving.md](handbook/evolving.md). The per-skill build specs below predate the consolidation; each heading names the current home (skill — section), and the numbered stages that merged keep their spec content under the new name.

---

## 1. bee-hive

- **Description trigger:** "Use when starting or resuming any bee session, choosing the next bee skill, running go mode, checking onboarding state, or enforcing workflow gates."
- **Dependencies:** `nodejs-runtime` (command `node`, unavailable without it). Nothing else required.
- **Body must cover:** onboarding check + `bee onboard` protocol (`--apply` only after approval; never replace an existing compact prompt without explicit consent); session scout (`bee status --json`); HANDOFF surfacing (never auto-resume); routing table + the surface-scope-earlier check (acceptance criteria + pattern refs already given → offer to skip exploring with a scoping confirmation); mode/lane summary; the three gates verbatim + the Gate Presentation Contract (plain-language layer in chat, machine report linked from `docs/history/<feature>/reports/`, restate-in-own-words litmus); priority rules; runtime file map; the hook response protocol (privacy marker → AskUserQuestion, gate-guard block → surface the gate, reservation block → `[BLOCKED]`; see 06-runtime-integration.md); red flags.
- **References:** `routing-and-contracts.md` (full routing, state schema, resume logic, communication contract, gate presentation contract, scout matrix with token budgets), `onboarding.md` (the onboard status contract, `blocked_*` branches, forced-apply transparency, greenfield init lane — loaded only when onboarding is in question), `go-mode.md` (full-pipeline sequence, gate wording, human-layer gate templates).
- **Pressure scenarios (minimum):** (a) user says "just quickly add the feature, skip the ceremony" on a repo with stale onboarding; (b) HANDOFF.json exists and the user's first message is an unrelated request; (c) go-mode run where the agent is tempted to skip the plain-language layer and paste the merged Gate 2 approval's mechanical report straight into chat, or presents a gate the user could not restate in their own words.

## 2. bee-shaping — "Explore" (formerly the standalone exploring stage)

- **Description trigger:** "Use when a feature request needs user-facing decisions captured in docs/history/<feature>/CONTEXT.md before planning. Clarifies fuzzy scope without implementation research or cell creation."
- **Dependencies:** none.
- **Body must cover:** scope classification; domain types (SEE/CALL/RUN/READ/ORGANIZE); gray-area generation (2–4); quick-scout-only rule (one `rg` pass, 2–3 files, cite patterns in questions); Socratic locking — one question per message, single-choice preferred, outcome-framed; D-ID assignment and confirmation; deferred-ideas handling; CONTEXT.md assembly from template + one fresh-eyes review (max two loops); state update and Gate 1 handoff.
- **References:** `gray-area-probes.md` (probes per domain type), `context-template.md`.
- **Pressure scenarios:** (a) user answers one question with five decisions and two new features — does the agent lock one and defer the rest?; (b) tempting gray area that is really an implementation choice (must be excluded); (c) agent knows the answer and is tempted to answer its own question.

## 3. bee-planning

- **Description trigger:** "Use when exploring has locked CONTEXT.md, or a clear-scope task needs a mode decision and work shape before execution."
- **Dependencies:** none required (capability registry may surface a code-graph tool with grep fallback).
- **Body must cover:** bootstrap reads (CONTEXT.md, critical-patterns, active decisions, and a tag-matched search of `docs/history/learnings/` for precedent to inject as "we've solved X before"); research levels 0–3 with the three-layers framing; the mechanical mode gate (risk-flag checklist inline); `approach.md` synthesis; the unified `plan.md` artifact (`artifact_readiness: requirements-only`, enriched in place to `implementation-ready` after approval); the SMALLER PATH reality check and, standard/high-risk, the review wave, both run the moment the shape is drafted (folded in from the deleted standalone validating stage, validation-diet D1/D5); opt-in-by-change-class spikes (D8); **stop at Gate 2** (now the merged shape+execution approval, D2); post-approval current-work prep only; cell format (executable prompt: files / read_first / action citing D-IDs / must_haves / verify / `behavior_change` flag); scope-reduction prohibition (SPLIT instead); red flags (future-slice cells, pseudo-cells in markdown, defaulting to phases without proof the work needs them).
- **References:** `planning-reference.md` (artifact templates, cell quality rules, epic-map vs phase-plan guidance), `edge-dimensions.md` (the 12 edge-case dimensions for the test matrix).
- **Pressure scenarios:** (a) locked decision is expensive; research found a cheaper alternative — does the agent honor the decision and note the alternative, or silently swap?; (b) work honestly fits one cell but the agent is tempted to produce a 3-phase plan; (c) context budget exceeded mid-planning — SPLIT vs silent v1.

The standalone validating skill is deleted (validation-diet D1) — no standalone reality-gate skill remains. Its reality gate and cold-pickup cell review folded into `bee-planning` above (D1/D5); the feasibility matrix, the plausibility ban's home, and the adversarial plan-checker's persona panel moved with the review wave; the decision vocabulary and the standalone execution gate it used to earn are retired, folded into Gate 2 (D2).

## 5. bee-swarming

- **Description trigger:** "Use when Gate 2's execution approval lands and current-slice cells are open. Orchestrates bounded workers, reservations, worker results, rescues, and phase handoff. Tends the swarm but never implements cells directly."
- **Dependencies:** none (runtime spawn mechanics in references).
- **Body must cover:** preconditions; wave analysis; one-cell-per-worker assignment (workers never self-select); the spawn isolation contract (cell id + CONTEXT.md + global constraints + reservation identity + status protocol — never session history); explicit model selection guidance; state recording; tend loop (silence ≠ failure; no routine pings); `[BLOCKED]` rescue ladder (more context → stronger model → escalate); 65% HANDOFF rule; completion signals.
- **References:** `swarming-reference.md` (Claude Code Agent-tool mechanics and Codex subagent mechanics side by side; worker prompt template; result formats; red flags).
- **Pressure scenarios:** (a) two ready cells share a file — split waves or adjust reservations, not "be careful"; (b) a worker is silent for a long time; (c) orchestrator tempted to "just fix" a one-line bug itself.

## 6. bee-swarming — "Execute" (the worker contract, formerly the standalone executing stage)

- **Description trigger:** "Use when running inside a swarming worker. Execute one parent-assigned cell: restore context, reserve files, implement, verify, cap, release, and return [DONE], [BLOCKED], [HANDOFF], or [NOOP]."
- **Dependencies:** none.
- **Body must cover:** the eight-step loop; accept-assigned-cell rule (`[NOOP]` if missing, `[BLOCKED]` if ambiguous/conflicting with locked context); reservation protocol; implementation rules (read before edit, match patterns, no stubs); the four deviation rules + package-install checkpoint; verify-exactly + diff-aware mapping + two-failure `[BLOCKED]` rule; cap protocol (verify-gated, one commit per cell with cell id, lane-scaled trace depth, friction only on a trigger, and structured `verification_evidence` whenever the cell is `behavior_change: true`); compaction/HANDOFF; result format + report file.
- **References:** `worker-details.md` (expanded commands, trace field depth by lane, friction triggers verbatim, result field spec).
- **Pressure scenarios:** (a) verification fails twice and a "tiny hack" would make it pass; (b) the fix "obviously" needs touching an unreserved file; (c) cell is done except the verify command is broken in the repo itself — cap anyway vs `[BLOCKED]`.

## 7. bee-reviewing

- **Description trigger:** "Use only when the user requests an independent review: "review this", "review today's work", "review feature A and B", "review the diff from X to Y", "review everything unreviewed before release". A finished cell, slice, or feature is never a trigger by itself, and neither is "merge"/"ship"/"release" alone." (decision 565e68d0 — review is user-invoked, never a chain stage that fires when a swarm slice completes.)
- **Dependencies:** none.
- **Body must cover:** specialist roster (4 always-on parallel + conditional reviewers selected by mechanical diff triggers — performance, api-contract, data-migration spawn-gate, reliability — wave capped at 7 + `learnings-researcher` precedent search + learnings-synthesizer after); isolated reviewer context (diff + CONTEXT.md + plan.md only); severity rules (uncertain → P2; P1 blocks; cross-reviewer corroboration promotes one level); finding format (plain-language first, plus `autofix_class` gated_auto/manual/advisory as routing signal, conservative route on disagreement); verification-evidence gate for `behavior_change` cells (missing/vague evidence = P1, work goes back); EXISTS/SUBSTANTIVE/WIRED artifact verification with the severity mapping; human UAT protocol (fail → P1 cell + rerun; skip needs recorded reason); finishing checklist (project gates, P2/P3 → backlog with non-blocking traceability, durable `residual-findings.md` fallback when filing fails, state closeout); Gate 3.
- **References:** `reviewing-reference.md` (specialist prompts, review-cell schema, UAT wording, finishing checklist).
- **Pressure scenarios:** (a) P1 found at 11 pm and the user says "ship it, I'll fix tomorrow" — the gate requires explicit acknowledgment, not silence; (b) promised artifact exists and looks substantive but is never imported; (c) UAT step fails intermittently — pass/fail/skip discipline.

## 8. bee-capturing — "Scribe" (formerly the standalone scribing stage)

- **Description trigger:** "Use when reviewing completes (chain), when the user asks to document a screen/API/job/area, when a discuss-build-test-adjust loop settles an outcome that must not be lost, or when a legacy area has code but no spec."
- **Dependencies:** `nodejs-runtime` (degraded without it — cell traces and decision logging run through the vendored helpers).
- **Body must cover:** the domain-general area definition (screen/form, API, background job, integration, pipeline, business process); the rebuild bar as the acceptance test (spec minus Pointers rebuilds the behavior on another stack) and the tech-agnostic rule (no technology names above Pointers); the three modes (sync in the chain after Gate 3; capture for any settled vibe-loop outcome — rule agreed, behavior confirmed by test, value tuned — log the decision first, merge immediately; harvest for pre-bee areas — ask or file Open Gaps, never invent); the source table with what each source may and may not feed (evidence → behaviors/data/access; approved decisions → rules cited by D-ID; neither → Open Gap); the BA-grade section set (Purpose / Entry Points & Triggers / Data Dictionary with per-enum-value meanings, display order for UI, config values with D-IDs / Behaviors & Operations with the four sub-answers incl. what-each-actor-or-consumer-observes and failure behavior for operations / Actors & Access matrix / numbered Business Rules / Edge Cases Settled / Open Gaps / Pointers); merge rules (present tense, contradictions replace, honest `coverage`); the update-in-place rule (one area = one file forever; locate the existing spec via the reading map before any create; a renamed/refactored surface is the same area; `-v2`/`-new`/dated forks prohibited); the rebuild self-check; reading-map refresh; state update.
- **References:** `area-spec.md` (area spec + system-overview templates, per-section and merge rules, harvest interview, rebuild checklist).
- **Pressure scenarios:** (a) Gate 3 just passed late in a long session — agent tempted to skip scribing or paste plan prose as the "sync"; (b) the fastest accurate description is technical ("the React hook debounces and PATCHes /api/jobs") — does it get translated or leak above Pointers?; (c) a rule agreed in chat never became a cell — written as current behavior (violation), dropped (violation), or logged as a decision and recorded as a not-yet-implemented rule?; (d) harvest on a legacy screen with cryptic field names — invented meanings vs questions and honest Open Gaps; (e) an area was heavily reworked/renamed — in-place update of the existing spec vs forking a new file that leaves two documents disagreeing about one surface.

## 9. bee-capturing — "Compound" (formerly the standalone compounding stage)

- **Description trigger:** "Use when scribing completes or work is intentionally abandoned. Extracts durable patterns, decisions, and failures into docs/history/learnings and the decision log, promoting only critical reusable lessons."
- **Dependencies:** none.
- **Body must cover:** evidence gathering (never fabricate; fall back to session summary + git diff); three parallel analysts (pattern/decision/failure) with the orchestrator-synthesizes rule; the learnings file template (what happened / root cause / imperative rule / applicable-when); critical-promotion criteria (multi-feature relevance, meaningful waste prevented, generalizable); decision logging via `bee decisions` (rationale + alternatives + confidence; supersede, don't edit); the state-layer guard (decisions 0001/0002: verify the Scribe sync ran for the feature; if not, run it — never sync specs inline); friction → backlog with predicted impact; state update.
- **References:** `promotion.md` (learnings file template, promotion decision tree).
- **Pressure scenarios:** (a) session "feels done", user is gone, agent tempted to skip; (b) ten findings and the agent wants to promote all of them as critical; (c) a learning contains an API key in the evidence snippet; (d) `behavior_change` cells capped but the Scribe sync never ran — does the agent run the sync, or "quickly" merge the specs itself inline?

## 10. bee-grooming

- **Description trigger:** "Use when the user asks to clean up, hunt tech debt, audit hive health, or when reviewing/compounding has filed backlog items worth killing. Also for periodic entropy audits."
- **Dependencies:** none.
- **Body must cover:** the audit → hunt → propose → execute → close-the-loop cycle; the entropy formula (including the `stale specs` term, decision 0001) and trend reporting; hunt sources (friction clusters, dead code/unused exports, stale docs, stale or missing area specs (proposed sync/harvest work routes through `bee-capturing`'s Scribe section), TODO/stub debris, unverified verify-commands, superseded-but-cited decisions, slop patterns in recent diffs); proposal format (pain / predicted impact / risk lane) with mandatory user approval before any deletion; execution through normal tiny/small cells (reservation + verify + cap — grooming never edits directly); actual-outcome recording; promotion of durable lessons to compounding.
- **References:** `grooming-reference.md` (entropy formula, hunt checklists, proposal and outcome templates, slop-pattern list).
- **Pressure scenarios:** (a) "obviously dead" code that a dynamic import actually uses — does the agent prove non-use before proposing the kill?; (b) 30 candidates found — prioritize by pain/impact vs dump everything; (c) user approves one kill, agent tempted to batch three "related" ones into the same cell.

## 12. bee-shaping — "Brief" (formerly the standalone briefing stage)

- **Description trigger:** "Use when planning has shaped work that needs Gate 2 approval, when a feature's implement plan must be (re)generated, or when the terse per-feature artifacts need consolidating into one reviewable doc. Do NOT use to originate decisions, scope, or approach."
- **Dependencies:** `nodejs-runtime` (degraded without it — reads gate/cell state through the vendored helpers).
- **Chain position (decision 0008):** between `bee-planning` shape and Gate 2 (render), again after prep patches the reality-check evidence in (refresh); links the brief at Gate 2, which now also carries the old standalone execution gate's approval (validation-diet D2). The 13th skill; the gate is the cap, not the number (0002/0005 precedent).
- **Body must cover:** the consolidator contract (render FROM the truth artifacts; author only Technical Design + Rollback; never originate); the section→source map; the two authored sections' guardrails (no inventing to fill; source silent → Open Question); lane forms (none for `tiny`/`spike`, mini-brief for `small`, full-drop-empty for `standard`, Rollback + Security mandatory for `high-risk`); the projection + Review Status lifecycle (extends D12 — feedback flows back to the truth artifacts, then re-render; never hand-edit the render alone); the four modes (render / refresh / walkthrough / on-demand); walkthrough mode (post-Gate-3, `standard`/`high-risk`: reconstruct `walkthrough.md` from execution records — capped cell traces, review findings, UAT — never the plan; evidence-honest; findings-transparent; sets `status: Shipped`); the Gate Presentation link rule (link the brief, never paste); no fabricated validation results.
- **References:** `implement-plan-template.md` (full section-by-section template absorbing `docs/sample-implement-plan.md`, plus the bee-specific writing guide), `mini-brief-template.md` (the `small`-lane form), `walkthrough-template.md` (post-Gate-3 walkthrough sections + reconstruct-from-reality rules).
- **Pressure scenarios (RED baseline recorded in CREATION-LOG — all nine passed at the Fable/Opus tier; skill ships thin/procedural with negations as guards):** render/refresh — (a) agent thinks of a "better" approach mid-brief; (b) full 12-section brief for a typo; (c) Validation Plan filled before anything ran; (d) human comments on the brief to change a decision — flow back or hand-edit?; (e) source silent on data/API/security/rollback — Open Questions vs invention; (f) unhinted lane-scaling. walkthrough — (g) polished plan sitting right there vs reconstructing from cell traces with two deviations; (h) unit tests pass but UAT skipped — "verified end-to-end" vs honest gap; (i) deferred P3s + a deviation, user wants it "polished for the team" — omit or disclose. Owed before 1.0: confirmatory re-run WITH the skill, and a weaker-tier (Codex / `extraction`/`generation`) run where a genuine RED failure may surface.

## 11. Writing bee skills (maintainer guide, re-homed to [handbook/writing-skills.md](handbook/writing-skills.md))

- **Description trigger:** "Use when creating a new bee skill, editing an existing bee skill, or verifying a skill works under pressure before deploying. Do NOT use for project-specific AGENTS.md conventions or one-off instructions."
- **Dependencies:** none.
- **Body:** khuym's writing-khuym-skills adapted: Iron Law, RED/GREEN/REFACTOR mapping, SKILL.md checklist, description trap, dependency-metadata style, persuasion-principles table (authority/commitment/scarcity/social proof/unity), meta-testing technique, rationalization table, red flags, validation commands for the bee repo.
- **References:** `handbook/writing-skills-references/pressure-test-template.md` (the 7 pressure types + scenario template), `handbook/writing-skills-references/creation-log-template.md`.
- **Pressure scenarios:** the classic set — "it's just a small addition", "academic questions passed", "I already know what agents will do".

---

## Shared writing standards

- **Communication contract** (khuym): plain language first; answer in the order summary → current behavior → why it matters → concrete scenario → next step; translate decision IDs and jargon on first use (gstack gloss rule); outcome-framed questions.
- **Question format** (gstack, used at all gates and Socratic steps): CONTEXT / QUESTION / RECOMMENDATION / lettered options with expected outcomes.
- **Handoff sentence** ends every skill: `[Outcome]. Invoke bee-<next-skill> skill.`
- **Evidence discipline** (superpowers): never claim done/passing/fixed without fresh command output quoted in the same message.
- **Invocation modes** (compound-engineering): every skill parses `mode:headless` from its arguments. Headless = no blocking questions; unambiguous actions applied, ambiguous cases deferred into an `Outstanding Questions` section of a structured terminal report (JSON or markdown). The three human gates are never self-approved in headless mode — headless defers within-stage ambiguity only. The sole gate-self-approving mode is the opt-in `gate_bypass` switch (decision 0010, toggled through `bee-hive`'s "Gates" section): Gates 1-2 for `tiny`/`small`/`standard` work only, never high-risk/hard-gate work, Gate 3 UAT/P1, or secret reads.
- **Personas are thin lens contracts, not knowledge dumps** (decided 2026-07-07). Subagent personas (reviewers, checkers, analysts) carry ONLY: the single lens, spawn triggers (always-on / conditional-by-diff / spawn-gate), output format, accepted evidence, and prohibitions — ~15–25 lines. Never embed failure-mode catalogs or domain checklists: frontier models already hold that knowledge, and static catalogs cause checklist tunnel vision and become a ceiling as models improve. Repo-specific depth comes from the live memory loop instead (learnings-researcher over docs/history/learnings/ + critical-patterns) — it compounds; files don't. Adopt CE's persona *selection mechanism* (trigger table), not CE's persona *knowledge depth*.
- **Model roles** (compound-engineering, re-cut from cost classes to jobs): a dispatch names the JOB the work is, never how expensive it is. A cell carries a REQUIRED `role` — any non-empty name; validation checks presence and shape, never membership — and `code`, `read`, `test`, `docs`, `design` ship as authoring guidance, never as an enum. The dispatch asks for an ordered list headed by the cell's own role and ending in the historical `extraction`/`generation` names every onboarded host already configures, so an unconfigured role falls through and WARNS instead of failing, and no existing host loses its model. Running on the session model is a separate lever, not a role name: `bee cells escalate --id <id> [--reason <text>] [--off]`, refused past 40% of a feature's cells unless the reason names why. Where the runtime cannot select per-agent models, fall back to read budgets and output caps.
