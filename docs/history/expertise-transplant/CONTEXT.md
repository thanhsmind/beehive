# Expertise Transplant — Context

**Feature slug:** expertise-transplant
**Date:** 2026-08-11
**Shaping session:** complete (Qualify path — headless, user directive plus three gather digests as evidence)
**Scope:** Standard
**Domain types:** READ | ORGANIZE

## Feature Boundary

Import eight craft gaps from the mattpocock engineering skill set
(`/home/thanhsmind/projects/AI/mattpocock-skill/skills/engineering/`) into bee's
expertise and skill files, rewritten in bee's own voice (trigger → move →
example); ends when the eight additions land and render — no restructuring of
existing expertise content, no new skills.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Gap analysis governs scope: only the eight gaps below are imported; craft bee already holds (expand/migrate/contract, walking skeleton, probe menus, smell catalogs, red-before-green) is NOT duplicated | Three digests confirmed `.bee/expertise/` already covers most of the external set's ground |
| D2 | Merge-conflict resolution craft becomes a new `.bee/expertise/merges.md` (+ INDEX.md row): recover intent from both sides' primary sources (commit messages, PRs, tickets); preserve both intents where possible, else pick the side matching the merge's stated goal and note the tradeoff; never invent behavior during resolution; run the project's checks after resolving; finish the merge rather than abort by default | Only area with zero coverage anywhere in bee |
| D3 | `debugging.md` gains a "Build the feedback loop first" section: ranked loop-construction ladder (failing test → HTTP/curl → CLI+fixture diff → replay trace → throwaway harness → property/fuzz → bisection harness → differential → human-in-the-loop last), loop-quality bar (red-capable, deterministic, seconds-fast, agent-runnable), hard gate "no red-capable command, no hypothesis phase", reproduction-rate targeting for flaky bugs, unique debug-tag prefix (`[DEBUG-xxxx]`) for grep-cleanup | Complements existing reproduce-first/hypothesis craft without replacing it |
| D4 | `architecture.md` gains: the deletion test (delete the module — complexity reappearing at call sites means it earned its keep), the adapter-count seam rule (one adapter = hypothetical seam, two = real), the four-way dependency taxonomy driving test strategy (in-process / local-substitutable / remote-owned / true-external), and "replace, don't layer" for tests after deepening a module | — |
| D5 | `planning.md` gains a "Design it twice" move for standard/high-risk shapes: fan out 2–3 parallel gather/review workers, each forced into a radically different interface constraint; orchestrator compares by depth/locality/seam placement and recommends one with a stated reason — fits bee's existing swarm model | — |
| D6 | `tests.md` gains the independent-oracle rule: an expected value must come from a source independent of the implementation (known literal, worked example, spec) — never recomputed the implementation's own way; tautological tests named as the anti-pattern | — |
| D7 | `planning.md` gains a "Spike craft" section giving the spike lane its HOW: one named question per spike; logic question → single-file pure-module shell, UI question → 2–3 structurally different variants; throwaway code never merges — the validated decision folds into real work, the spike itself stays on a throwaway branch as a primary source | Spike lane exists in bee-planning with no craft behind it |
| D8 | `skills/bee-hive/references/routing-and-contracts.md` gains a phase-boundary decision tree in session care: at a phase boundary evaluate in order, first yes wins — continue (next phase needs this one as primary source) → fresh start (everything disposable) → handoff (new place or person) → subagent (AFK-scoped) → compact last, because every move but continue is a lossy primary→secondary conversion | Sharpens the existing 65%-handoff rule with a decision procedure |
| D9 | `skills/bee-shaping/SKILL.md` Qualify gains verify-the-claim: a bug-shaped backlog item gets its claim reproduced (or the failed attempt recorded) before the triage verdict; dedup checks search by domain concept, not request wording | Two sentences in step 1; smallest honest change |
| D10 | All imports are rewritten in bee's expertise voice (trigger → move → concrete example), never copied verbatim; source attribution lives in the decision log, not in the files | Expertise files carry no external references today |
| D11 | Skill-file edits (D8, D9) follow bee-writing-skills discipline and re-render the vendored trees via the repo's render chain | `.claude/skills/` is generated |

### Agent's Discretion

Section placement and wording within each target file; whether merges.md
cross-links debugging.md/operations.md; exact ladder ordering inside D3 where
the source ranks methods the repo cannot express (e.g. no browser).

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| feedback loop | A repeatable command that can show the failure (red-capable) before any hypothesis work |
| deletion test | Judging a module by imagining its deletion: complexity concentrating elsewhere = it earned its keep |
| independent oracle | Expected test value derived from a source the implementation cannot influence |
| primary→secondary loss | Context moves (compact, handoff, summary) convert primary sources into lossy summaries |

## Specific Ideas And References

- Source set digested by three gather workers on 2026-08-11; digests cover
  README philosophy, 18 skills, and bee's own 9 skills + expertise layer.

## Existing Code Context

### Reusable Assets

- `.bee/expertise/debugging.md`, `tests.md`, `architecture.md`, `planning.md` —
  target files; same voice (routing table + trigger → move + example) governs the additions
- `.bee/expertise/INDEX.md` — one row per expertise file; merges.md needs a row

### Established Patterns

- Expertise entries: named rule → one-paragraph mechanism → concrete example — reuse exactly
- Skill sources live in `skills/`, vendored trees are generated — edit source, re-render

### Integration Points

- `skills/bee-hive/references/routing-and-contracts.md` — session-care contract home (D8)
- `skills/bee-shaping/SKILL.md` — Qualify step list (D9)

## Canonical References

- `/home/thanhsmind/projects/AI/mattpocock-skill/skills/engineering/` — source set (read-only)

## Outstanding Questions

### Deferred To Planning

- [ ] Which render command refreshes vendored skill trees (`bee dev render-skill-trees` vs broader chain) — check before capping the skill-edit cell

## Deferred Ideas

- Out-of-scope concept memory for rejected backlog items (external `OUT-OF-SCOPE.md` pattern, one file per concept) — bee's decision log partially covers it; revisit if parked-item dedup misses recur
- Issue-tracker abstraction layer (external setup skill) — bee's backlog is its tracker; no current need
- HTML architecture report (external improve-codebase-architecture) — presentation layer, not craft; bee-grooming covers the hunt

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
