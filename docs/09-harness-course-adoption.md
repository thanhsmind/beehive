# 09 — learn-harness-engineering Deep Dive: What Else bee Should Adopt

A full read of the learn-harness-engineering course (12 lectures + templates + reference protocols + OpenAI-advanced SOPs, synthesizing OpenAI "Harness engineering" and both Anthropic long-running-agent articles) against bee v0.1.4 as built. Same verdict format as [08-harness-adoption.md](08-harness-adoption.md): **already covered / adopt now / adopt later / skip**.

The course's frame: a harness = Instructions + Tools + Environment + State + Feedback, and every agent failure attributes to exactly one of five layers (task specification, context provision, execution environment, verification feedback, state management). bee is strong on four of the five subsystems. The systematic hole is the **environment/runtime seam**: bee knows everything about its own machinery and nothing about how the *host project* starts, runs, and verifies.

## Already covered in v0.1 (no action)

| Course mechanism | Where bee has it — often stronger |
|---|---|
| Feature list as harness primitive: triple (behavior, verification, state) + evidence, pass-state gating, harness-controlled transitions (L08) | The cell: `must_haves` + `verify` + `status` + `trace.verification_evidence`; `bee.mjs cells cap` mechanically refuses without a passing verify + recorded output — the course's "harness executes and decides" is literal here |
| Externalized termination + worker/checker separation (L09, L11) | Cap-requires-verify-with-proof; `bee-reviewing` as independent checker (EXISTS/SUBSTANTIVE/WIRED, UAT, Gate 3); "an assertion is not evidence" |
| Repo as system of record, knowledge next to code (L03, encode-knowledge SOP) | `docs/specs/` state layer + reading-map + system-overview; policy-vs-ops split; the **rebuild bar** is a stronger per-area form of the course's Fresh Session Test |
| Short router entry file, progressive disclosure, ≤200 lines (L04, prompt-calibration) | AGENTS block ~90 lines; SKILL.md <200 lines with one `references/` file each (khuym/superpowers rule) |
| Cross-session continuity: progress state, decision log, checkpoints, handoff threshold (L05) | `state.json` + `HANDOFF.json` at ~65% context (course says 60%), `decisions.jsonl` event-sourced, one commit per cell; resume never auto-continues |
| Scope discipline: externalized scope surface, next-task visibility, "what must not change" (L07) | Cells with deps/`ready`, orchestrator-assigned (workers never self-select), `must_haves.prohibitions`; `state.json.next_action` |
| Sprint contract negotiated before coding (L11) | The cell **is** the contract: scope, files, must_haves, verify — agreed at Gate 2 before execution |
| Process observability: plans, criteria, decision artifacts readable after the fact (L11) | `plan.md` contract with machine-checkable readiness, cell traces, reports/, decisions.jsonl |
| Cleanup loop + entropy as measured debt (L12) | `bee-grooming` entropy score + trend, hunt checklists, kill proposals with predicted→actual |
| Planner/generator/evaluator role separation (L11) | The chain itself: planning / executing / reviewing as separate skills with separate contexts |

## Adopt now (cheap, closes the environment seam and the learning loop)

### 1. Standard paths + baseline gate — the one real hole

Course, three ways: Fresh Session Test questions 3–4 ("how do I run it? how do I verify it?"), the clock-in/clock-out routine (L05), the fixed 9-step startup flow (steps 5–7: run `init.sh`, run baseline verification, **if baseline is broken fix that first**), and `RELIABILITY.md`'s Standard Paths. bee records per-cell `verify` commands but has **no repo-level record of the host project's setup/start/test/verify commands**, and no session-level baseline check — a fresh bee session can answer "where are we" perfectly and still not know how to start the app.

Adopt:
- `.bee/config.json` gains `"commands": {"setup", "start", "test", "verify"}` — captured at onboarding (prompted) or during first exploring; scribing keeps it current like any Pointers-level fact.
- `bee_status` prints them and warns when absent; `bin/lib/inject.mjs` adds them to the session preamble so both runtimes see them without discovery.
- AGENTS block Startup gains one step: run the repo's standard `verify` once per session before claiming any cell; **a broken baseline becomes its own tiny fix-first cell — never build on red**.
- Session finish gains the mirror condition: standard verify passes before ending a substantial chunk (the course's clean-state condition #1–2: build passes, tests pass).
- Unlocks 08's item 4: `verify-all` needs runnable recorded commands to sweep.

### 2. Five-layer failure attribution on friction

Course L01: every failure maps to exactly one layer — task spec / context / environment / verification / state — and a failure log aggregated by layer reveals the bottleneck. bee captures friction verbatim (good) but untyped, so grooming clusters by topic and never by *cause*.

Adopt: friction backlog entries and `trace.friction` gain an optional `layer` field with those five values; grooming's entropy report adds one line — friction count by layer, largest = bottleneck layer this cycle. Pairs naturally with 08's `interventions.jsonl` (same taxonomy, one write path).

### 3. Review-feedback promotion: executable check first, prose second

Course L10/L12 (and OpenAI's "capture taste once, enforce continuously"): every *recurring* review comment should become an automated check — lint rule, grep in a verify command, guard — because agents copy existing patterns and prose rules decay. bee's compounding promotes repeated patterns to `critical-patterns.md` — prose that costs context every session and relies on being read.

Adopt: in `bee-capturing` ("Compound"), when the same review finding or user correction appears twice, the **first-choice promotion target is an executable check** (a grep/lint line appended to the area's verify command, a helper guard, a hook denial); `critical-patterns.md` is the fallback for what can't be mechanized. Grooming's review rules already grade proposals by lane — "promote to check" proposals are tiny/small.

### 4. Fresh Session Test as a grooming audit

Course L03: a brand-new session with only repo contents must answer five questions, each mapped to an artifact. bee has the artifacts but never audits the mapping end-to-end.

Adopt into grooming's hunt checklist, one probe per question: what is this system → `docs/specs/system-overview.md` · how is it organized → `reading-map.md` · how do I run it / verify it → `config.json` commands (item 1) · where are we now → `bee_status`. Any unanswerable question = a backlog item with the missing artifact named. Five minutes per audit, catches decay the entropy formula can't see.

### 5. ERROR/WHY/FIX as the denial-message contract

Course L09/L10: agent-oriented error messages carry what failed, why, and how to fix — "Test failed" wastes a retry loop; repair instructions convert it into one correction. bee's guards and helpers refuse a lot (by design); message quality decides whether refusal teaches or thrashes.

Adopt: state the contract in [07-contracts.md](07-contracts.md) — every refusal from `bin/lib/` or a hook names the rule, the reason, and the next command/action — then audit the existing strings against it and add test assertions. Most already comply; the contract stops regression.

## Adopt later (real value, wait for the trigger)

### 6. Initialization lane for greenfield repos

Course L06 + initializer-agent playbook: the first session does infrastructure only — runnable env, one passing test, recorded standard paths, task breakdown, clean first commit — with acceptance criteria, before any feature work (31% higher multi-session completion claimed). bee onboarding initializes *bee's* machinery, not the project's. Adopt as a planning convention, not a new skill: in a greenfield repo, the first slice is an **init cell** whose must_haves are exactly the course's acceptance checklist (setup succeeds from scratch, one passing test, commands recorded per item 1, first clean commit). Trigger: the first time bee onboards a repo without a build.

### 7. Per-area quality grades

Course L12 quality document / `QUALITY_SCORE.md`: A–D grade per domain (verification, agent legibility, test stability, boundaries), new sessions fix the lowest grade first. bee's entropy is repo-level; specs carry Open Gaps but no grade. Adopt as a **computed view** — `bee_status` deriving per-area grades from entropy terms scoped by area (stale spec, unverified cells touching its files, open gaps count) — never a hand-maintained doc that forks state. Trigger: spec count reaches ~5+ areas, when "which area first" becomes a real question.

### 8. Harness simplification cadence (ablation)

Course L02/L12: periodically disable one harness component, compare, remove if nothing degrades — Anthropic dropped sprint-splitting when Opus 4.6 stopped needing it; components that outlive their model are pure overhead. bee's grooming hunts repo debt but never hunts **bee's own** components. Adopt as a grooming hunt item: any hook/guard/rule with zero firings in `logs/hooks.jsonl` across N sessions → removal proposal with predicted→actual. Trigger: enough dogfood sessions that hooks.jsonl has meaningful counts.

### 9. Fixed-category UAT scorecard

Course L11 + evaluator-rubric template: fixed categories scored 0–2 (correctness, verification, scope discipline, reliability, maintainability, handoff readiness) so different evaluators converge; evaluators need 3–5 tuning rounds against human judgment. bee-reviewing is already structured (EXISTS/SUBSTANTIVE/WIRED + P1/P2/P3); adopt the scorecard only if dogfood shows UAT drift between sessions.

### 10. Instruction metadata audit

Course L04: every rule carries source, applicability condition, expiry condition; ≤15 hard constraints in the entry file. The AGENTS block is near that ceiling now. Adopt when it next grows: a grooming-owned ledger (one line per hard rule: why added, when it applies, when it can die) audited each entropy run — the mechanism that keeps the router file from becoming the course's "add a rule every time something breaks" bloat spiral.

## Skip (with reasons)

- **`feature_list.json`, four-state enum, scope-tracker/VCR gate** — the cell is a strict superset (verify command, evidence, deps, lanes, trace); a second feature file would fork the source of truth.
- **`claude-progress.md` prose progress log** — `state.json` + `HANDOFF.json` + cells are the machine-checked equivalent; a parallel narrative file is the course's own "memo mode" anti-pattern.
- **WIP=1** — solves the no-coordination case; bee deliberately allows parallel waves *because* it has reservations + orchestrator-assigned cells. Adopting it would delete swarming's reason to exist.
- **`init.sh` as a shipped artifact** — the *record* of commands (item 1) is the primitive; a script forks per-OS (bee is Windows-first) and per-stack. Repos that want one can point `commands.setup` at it.
- **OpenTelemetry traces, Jaeger/Zipkin** — solo scale; cell traces + hooks.jsonl are the right-sized equivalent.
- **Chrome DevTools validation loop as a bee mechanism** — belongs to the host repo's tooling; reviewing's UAT already demands driving the real flow, and the capability registry (`tools.json`) is where browser tooling gets detected.
- **Three-agent $125-per-feature pipeline, high-throughput merge philosophy, benchmark-runner comparisons** — team/fleet scale; re-confirms 08's skips (benchmark coupling, H-level percentage targets).

## Sequencing

Item 1 is the priority — it is the course's single loudest lesson bee hasn't internalized (environment is the fifth subsystem), it unlocks 08's verify-all, and it is one small slice (config schema + inject + AGENTS template + one hive/exploring paragraph). Items 2–5 are each a slice of the same shape as 08's items 1–4: a field or a paragraph in the owning skill's reference. Natural bundle: item 2 rides with 08's interventions.jsonl; items 3–4 are reference-doc-only; item 5 is contract + string polish. All fit the `small` lane through bee's own chain, same dogfood posture as 08. Items 6–10 have named triggers; none is speculative work today.
