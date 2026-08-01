# 00 — Vision & Principles

## Why bee exists

Every upstream framework solves the same problem — AI agents write code faster than humans can verify intent, feasibility, and quality — but each solves it at a different layer and with different overhead:

- **khuym** proved the shape: a distilled, opinionated 7-stage chain for one developer, with hard gates and file-based state. bee inherits khuym's skeleton directly.
- **gsd-core** proved that plans can be *executable prompts* and that verification must work *backwards from the goal*, not forwards from the task list.
- **superpowers** proved that skills are code — they must be pressure-tested — and that a workflow only holds if the chain is enforced, not suggested.
- **claudekit** proved context isolation: subagents that receive ~100 tokens of task context outperform subagents fed session history.
- **repository-harness** proved that risk should be *mechanical* (a checklist, not a feeling) and that harness health can be measured (friction, entropy, predicted-vs-actual).
- **gstack** proved that knowledge must be event-sourced (decisions are superseded, never edited) and that a second model's opinion is a gate feature, not a gimmick.

bee reassembles these into one opinionated chain, sized for a single developer running Claude Code and Codex.

## Principles

1. **Validate before execute. Always.** No source-editing execution before the feasibility of the current work is proven with *concrete evidence* — code inspection, command output, or a spike. "This should work" is not evidence. (khuym, gsd)

2. **CONTEXT.md is the source of truth.** Decisions get locked with stable IDs (D1, D2…) during exploring; every downstream stage executes against locked decisions and cites them, instead of reinterpreting intent. (khuym, gsd)

3. **The smallest honest workflow wins.** Every piece of work passes a mode gate first: `tiny` → `small` → `standard` → `high-risk`, plus `spike` when one yes/no proof decides the plan. Risk classification is a mechanical flag checklist, not judgment. Tiny work must not generate ceremony. (khuym modes + repository-harness lanes)

4. **Goal-backward, adversarial verification.** Checkers start from "this plan/claim will fail until evidence proves otherwise." Task completion ≠ goal achievement. Artifacts must be EXISTS + SUBSTANTIVE + WIRED. Claims require fresh command output. (gsd, superpowers, khuym)

5. **Fresh context, minimal context.** Subagents receive one task, the interfaces it touches, and global constraints — never session history. Scouting gathers *just enough*: phase-and-lane-scoped reading lists with token budgets, and research depth levels 0–3. (claudekit, superpowers, repository-harness, gsd)

6. **A cell is capped only after verification.** One worker, one cell, one commit. Workers never pick their own work, never edit outside reservations, and never wait silently — they return `[DONE]`, `[BLOCKED]`, `[HANDOFF]`, or `[NOOP]`. (khuym)

7. **Knowledge compounds or the system decays.** Every meaningful feature ends in compounding: dated learnings, critical-pattern promotion, and event-sourced decisions (`decide` / `supersede` / `redact`, append-only). Learnings and active decisions are injected at the *start* of future sessions, not archived to be forgotten. (khuym, gstack, gsd)

8. **The hive cleans itself.** Friction observed during work is captured in a structured backlog with *predicted* impact; grooming runs kill tech debt and measure *actual* outcomes. Hive health is a computed entropy score, not a feeling. (repository-harness, gstack)

9. **Skills are code. Test them.** No bee skill ships without a failing pressure test first (the Iron Law). Descriptions state *when to use*, never summarize the workflow — a workflow summary in the description causes agents to skip the skill body. (superpowers, khuym)

10. **Humans decide at exactly four gates.** Approve decisions, approve the work shape, approve execution, approve merge. Models recommend; the user decides. When two models disagree at a gate, surface the disagreement — never auto-resolve. (khuym, gstack)

11. **The meaning outlives the stack.** This holds for the entire development process — backend, frontend, integrations, pipelines, processes alike. Vibe-driven development locks knowledge into code and the current technology: business rules, field meanings, and behaviors agreed in discussion vanish when the session closes. So every settled outcome of the discuss → build → test → adjust loop — a rule agreed, a behavior confirmed by test, a value tuned — is recorded technology-agnostically in the state layer (`docs/specs/`) the moment it settles, at BA grade. Code is one *rendering* of the spec; the spec must survive a full rewrite on another stack (the rebuild bar). Lanes scale ceremony, never memory: a tiny fix that changes behavior still updates the spec. (owner requirement, decisions 0001/0002)

## Non-goals

- **Not a daemon, not a database.** No background service, no schema migrations. State stays JSON/JSONL + markdown, readable and diffable by hand. *(Revised at the R6 cutover: this non-goal used to read "not a binary — no Rust CLI", and the helpers WERE small vendored Node scripts. They are one native binary now (plans/rust-port.md): hook latency, not architecture taste, is what moved it — a PreToolUse hook fires on nearly every tool call and paid ~50–120 ms of Node cold start each time. The parts this bullet actually protects — plain-text state, no daemon, no migrations — are unchanged.)*
- **Not 20 runtimes.** Claude Code and Codex only. The abstraction must make a third runtime cheap later, but bee does not pay for it now.
- **Not 40 skills.** Twelve skills today; additions are decision-gated, not counted — a new skill requires a decision record in `docs/decisions/` naming the workflow gap no existing skill covers (decision 0002 lifted the original ten-skill hard cap this way). Domain skills (frontend, deploy, DB…) remain out of scope — that's what other plugins are for.
- **Not a benchmark rig.** Health is measured by internal signals (entropy score, friction backlog, predicted-vs-actual), not an external benchmark harness.
- **Not autonomous merging by default.** P1 findings block. Gates never auto-approve unless the user opts in via the `gate_bypass` switch (decision 0010) — and even then Gate 4 UAT/P1, high-risk/hard-gate work, and secret reads always stay human.

## Success criteria

bee succeeds when, for its owner:

1. A vague feature request reliably becomes locked decisions, a validated plan, and capped cells without a single "wait, that's not what I meant" late in execution.
2. Small fixes complete in minutes with near-zero ceremony (tiny lane works).
3. A session can pause at ~65% context and resume the next day from `HANDOFF.json` without re-explaining anything.
4. `docs/history/learnings/critical-patterns.md` and `decisions.jsonl` visibly change agent behavior in later features (fewer repeated mistakes).
5. Grooming runs find and kill real debt, and the entropy score trends down.
6. Any long-lived area can be understood by a human — and rebuilt by an agent on a different stack — from its `docs/specs/` entry alone, without reading the code (the rebuild bar, decision 0002).
6. The same skills run under both Claude Code and Codex with no divergence in the workflow contract.
