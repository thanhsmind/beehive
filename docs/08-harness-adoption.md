# 08 — repository-harness Deep Dive: What Else bee Should Adopt

A second, closer read of repository-harness (HARNESS_MATURITY.md, FEATURE_INTAKE.md, IMPROVEMENT_PROTOCOL.md, SYMPHONY_QUICKSTART/SCOPE.md, TRACE_SPEC.md, CLAUDE.md) against bee v0.1 as built. Verdict per mechanism: **already covered / adopt now / adopt later / skip**.

## Already covered in v0.1 (no action)

| Harness mechanism | Where bee has it |
|---|---|
| Risk lanes + hard gates, mechanical checklist | Mode gate in `bee-planning` (same 10 flags, same 0–1/2–3/4+ mapping) |
| Policy ≠ operations | markdown in `docs/` (incl. `docs/history/`) vs JSONL/JSON in `.bee/` |
| Verify gate on close | **Stronger in bee**: harness's pre-close gate is advisory ("ADVISORY: verification has not passed"); bee's `capCell` mechanically refuses |
| Friction triggers → backlog with predicted→actual | `bee-swarming` ("Execute") trace + `bee-grooming` close-the-loop |
| Entropy audit | `bee-grooming` formula |
| Decision records with lifecycle | `.bee/decisions.jsonl` + `docs/decisions/` (event-sourced — stronger than harness's mutable status column) |
| Context phase × lane matrix, token budgets | `bee-hive` scout contract |
| Capability-based tool registry | `.bee/tools.json` (designed; helper still minimal) |
| Trace tiers by lane | Cap-time enforcement in `cells.mjs` (high-risk requires files+outcome; behavior_change requires evidence) |

## Adopt now (cheap, directly serves the learning loop)

### 1. Durable intake records + input types

Harness's core stance: *"The human does not need to classify risk. The harness does"* — and even tiny work **records the intake row before implementation** ("tiny skips story overhead, not durable task classification"). bee classifies in planning but keeps no evidence of the classification: which flags fired, why this lane.

Adopt:
- `intake` block in `plan.md` frontmatter **and** an appended row in `.bee/intake.jsonl`: `{date, input_type, summary, risk_flags:[], lane, reason, feature}`.
- Harness's **input-type table** into `bee-hive` routing (before the lane choice): new spec / spec slice / change request / new initiative / maintenance / harness improvement → each names its target artifact. bee's routing currently only distinguishes vague-vs-clear; input type decides *where work lands* before *how risky it is*.
- Payoff: grooming can later audit **lane accuracy** (misclassified work is a top source of harness failure), and repeated misclassification becomes a proposal.

### 2. Intervention log

Harness records every human/reviewer/CI correction as a typed durable row (`correction | override | escalation | approval`) and feeds it to `propose`. bee has gates but forgets what the human actually did at them — the highest-signal training data for "bớt sai sót" is currently discarded.

Adopt: `.bee/interventions.jsonl` — `{date, type, source: human|ci|agent, description, feature, cell|gate ref}`. Writers: `bee-hive` logs gate rejections and mid-flight corrections; `bee-reviewing` logs P1 acknowledgments and UAT failures; `bee-swarming` logs escalations. Reader: `bee-grooming`'s hunt ("repeated intervention patterns") and `bee-capturing`'s Compound section (a decision the user reversed twice is a critical-pattern candidate).

### 3. Rule-based `propose`

Harness's improvement loop is explicitly rule-based, not vibe-based: `repeated trace friction + repeated intervention patterns + non-zero audit categories → proposal {title, component, evidence, predicted impact, risk, suggested action, validation plan, confidence}`. And its **review rules** grade by lane: tiny doc-clarification proposals may be implemented directly; high-risk proposals (changing source hierarchy, validation requirements, risk policy) require a durable decision record first.

Adopt: name these three sources explicitly as grooming's proposal inputs, adopt the proposal field set (add `validation_plan` and `confidence` to bee's pain/predicted/lane format), and adopt the lane-graded review rules verbatim into `grooming-reference.md`.

### 4. Re-verification sweep (`verify-all`)

Harness stories carry `verify_command` + `last_verified_at/result`, with `story verify-all` re-running every configured proof. bee verifies once at cap time — after that, regressions against capped cells are invisible.

Adopt: `bee cells verify-all [--feature F]` re-runs the recorded `verify` command of capped cells and stores `last_verified_at/last_verified_result`; entropy adds `capped_but_failing × 10`. Optional mirror for decisions: a `verify_command` field + `bee decisions verify --id`, counted in entropy as unverified/failing decisions. This turns grooming's "unverified verify-commands" hunt from a static check into a live regression sweep.

### 5. CLAUDE.md `@import` fallback

Harness solves "Claude Code doesn't auto-load AGENTS.md" with a minimal CLAUDE.md whose bare `@AGENTS.md` / `@docs/FEATURE_INTAKE.md` lines import the must-read set at context-load time. bee's session-init hook covers this — but when plugin hooks don't fire (unsupported environment, disabled), bee has no Claude Code bootstrap at all.

Adopt: `bee onboard --claude-md` writes a 5-line CLAUDE.md with `@AGENTS.md` (belt three, trivially cheap).

## Adopt later (phase 4+, real value, real cost)

### 6. Maturity ladder (B0–B4)

Harness's H0–H5 makes adoption *inspectable*: each level has file/record criteria and expected compliance numbers. bee onboarding is currently all-or-nothing. Define: **B0** bare → **B1** policy (AGENTS block + skills, no state) → **B2** durable state (cells/intake/decisions/traces in use) → **B3** automation (hooks armed, guards firing) → **B4** self-improvement (grooming loop closing with actual outcomes). `bee_status` reports the current level and what's missing for the next. Docs-only + ~30 lines in `bee_status`; do it when a second repo adopts bee.

### 7. Symphony-style worktree isolation for swarming

Symphony's run model: isolated worktree + copied state + `RUN_CONTRACT.json` (explicit scope) + required `SUMMARY.md`/`RESULT.json` + reviewable changeset + idempotent `sync`. bee's same-checkout reservations are simpler and fine for a solo developer — but for high-risk lanes or wide-blast parallel waves, worktree isolation is categorically safer (a worker cannot even see files outside its run).

Adopt selectively: a `--isolation worktree` swarming mode (Claude Code's Agent tool supports worktrees natively; Codex path documented as manual `git worktree`). bee's **cell already is the RUN_CONTRACT** (scope, files, must_haves, verify) — no new artifact needed. **Skip** the copied-DB + semantic-changeset + sync machinery: bee's state is per-file JSON inside the repo, so git merge does what Symphony's changeset replay does.

### 8. Context-accuracy measurement (`score-context`)

Harness scores whether `files_read` in a trace matches CONTEXT_RULES. Would require capturing `files_read` per cell trace and diffing against the scout matrix. Real observability, but cost > value at solo scale. Backlog; revisit if scout-matrix violations show up as repeated friction.

## Skip (re-confirmed)

Rust CLI + platform binaries, SQL schema migrations, the 20-command CLI surface, external benchmark coupling, H-level benchmark percentage targets, full Symphony sync/changeset engine, 17-doc bulk.

## Sequencing

Items 1–4 are one small slice each (a JSONL + helper extension + a paragraph in the owning skill) — natural first dogfood targets for bee itself, run through bee's own chain in `small` lane. Item 5 is a one-flag onboarding change. Items 6–7 wait for phase 4 as already listed in [05-roadmap.md](05-roadmap.md).
