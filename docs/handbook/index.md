# bee harness — index

The routing backbone. Start here, pick the **stage** your change concerns, open
its page, then cross-reference [register.md](register.md) for the state it touches.
New to the system? Read [overview.md](overview.md) first.

## Stages (the chain, in order)

| # | Stage | One line | Gate | Page |
|---|-------|----------|------|------|
| 0 | **hive** | Bootstrap, route, and keep the gates. Loaded first every session. | presents all four | [stages/hive.md](stages/hive.md) |
| 1 | **exploring** | Fuzzy request → locked decisions in `CONTEXT.md`. | Gate 1 | [stages/exploring.md](stages/exploring.md) |
| 2 | **planning** | Locked decisions → smallest believable work shape + cells. | Gate 2 | [stages/planning.md](stages/planning.md) |
| 3 | **validating** | Prove the plan against repo reality before any code. | Gate 3 | [stages/validating.md](stages/validating.md) |
| 4 | **swarming** | Orchestrate bounded workers over validated cells. | — | [stages/swarming.md](stages/swarming.md) |
| 5 | **executing** | Implement, verify, and cap exactly one cell. | — | [stages/executing.md](stages/executing.md) |
| 6 | **scribing** | Sync durable, tech-agnostic knowledge of every area. | — | [stages/scribing.md](stages/scribing.md) |
| 7 | **compounding** | Capture learnings + decisions; close the feature. | — | [stages/compounding.md](stages/compounding.md) |
| R | **reviewing** | On-demand independent review gate over a chosen scope. | Gate 4 | [stages/reviewing.md](stages/reviewing.md) |

On-demand side skills (not chain stages, so no dedicated page): `bee-briefing`
(render one implement plan), `bee-grooming` (hunt tech debt), `bee-qualifying`
(auto-triage a backlog row), `bee-xia` (research scout), `bee-writing-skills`
(author a skill), `bee-evolving` (self-improve from feedback), `bee-bypass-gate`
(toggle gate autopilot).

## Route by intent

| Your change is about… | Go to |
|-----------------------|-------|
| How a request is classified into a lane/mode; onboarding; the gates | [hive](stages/hive.md) |
| How gray-area product decisions get resolved and locked | [exploring](stages/exploring.md) |
| How the work is shaped, the mode decided, cells created | [planning](stages/planning.md) |
| How feasibility is proven before execution; the reality gate | [validating](stages/validating.md) |
| How workers are dispatched, reserved, and their results judged | [swarming](stages/swarming.md) |
| How one cell is implemented, verified, capped, committed | [executing](stages/executing.md) |
| How specs/knowledge stay current after behavior changes | [scribing](stages/scribing.md) |
| How learnings/critical patterns/decisions are captured; feature close | [compounding](stages/compounding.md) |
| How an independent review is run, findings graded, merge approved | [reviewing](stages/reviewing.md) |
| A runtime file's schema (`state.json`, `cells`, `decisions.jsonl`, …) | [register.md](register.md) |
| Multisession state: workflow records, leases, handoff mailboxes, worktrees | [register.md](register.md#beeruntimeworkflowswf-idstatejson) |
| Using this handbook to localize an edit before touching code | [using-as-planner.md](using-as-planner.md) |

## The gates (who owns each)

Gates are presented and enforced by **hive**, but each is *earned* at the end of a
particular stage:

- **Gate 1** — earned by **exploring**: "Decisions locked. Approve CONTEXT.md before planning?"
- **Gate 2** — earned by **planning**: "Work shape is ready. Approve before current-work preparation?"
- **Gate 3** — earned by **validating**: "Feasibility validated. Approve execution?" *(no source edits before this)*
- **Gate 4** — earned by **reviewing** only: P1>0 → "P1 findings block merge. Fix before proceeding?"; P1=0 → "Review complete. Approve merge?"

Tiny/small lanes merge Gates 2+3 into one shape+execution question. The docs lane
has no gates. `gate_bypass` (in [config.json](register.md#beeconfigjson)) can
auto-approve gates by level (`normal` / `full` / `total`).

## Lanes (how much of the chain runs)

| Lane | Trigger (from the request alone) | Stages that run |
|------|----------------------------------|-----------------|
| `docs` | every touched file is knowledge, not runtime | announce → write → format-check → capture |
| `tiny` | 0–1 risk flags, ≤2 product files, one direct task | hive → merged gate → one execution worker |
| `small` | 0–1 flags, ≤3 product files, no gray areas | hive → merged gate → serial execution worker(s) |
| `standard` | 2–3 flags, or story-sized behavior | full chain, Gates 1–3 |
| `high-risk` | 4+ flags **or any hard-gate flag** | full chain + brief + persona panel, Gates 1–3 |

Risk flags (counted mechanically): auth · authorization · data model ·
audit/security · external systems · public contracts · cross-platform · changing
behavior a test asserts · weakening/deleting existing proof · multi-domain.
**Product files** exclude `.bee/**`, `docs/**`, plans, and generated renders.
