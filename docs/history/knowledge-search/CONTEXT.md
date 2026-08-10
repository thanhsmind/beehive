# Knowledge Search — Context

**Feature slug:** knowledge-search
**Date:** 2026-08-10
**Shaping session:** complete (headless qualify from PBI p-d7c88155; origin: deep review of knowledge-in-flow, 2026-08-10)
**Scope:** Standard
**Domain types:** CALL | READ

## Feature Boundary

A new read-only verb `bee knowledge search` lets an agent pull matching patterns and area concepts out of the bundle by symptom text at any moment mid-flow — including plain turns and off-rail work where cell dispatch never fires — and the two skills that own the debug moment point at it. It ends there: no write path, no new index files, no change to the push mechanisms (preamble digest, cell-dispatch learned context).

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Flag convention mirrors `bee decisions search`: `--text` (whitespace-split terms, case-insensitive, OR across terms), never a positional query; `json` flag supported like the rest of the surface | One search grammar across the CLI; positional args are refused by the dispatcher convention |
| D2 | Corpus is patterns + area concepts from the active bundle only. Decisions are excluded (`bee decisions search` owns them); generated index files and `docs/specs/` are excluded | One owner per corpus; no double-reporting |
| D3 | Match fields: concept title, body text, and frontmatter `sources`/`decisions` entries (where incident signatures live). Results rank by deterministic term-hit count descending, then recency descending — same ranking family as `decisions search` | Symptom strings (error text, mechanism names) often live only in the sources lines |
| D4 | Output: top-N (default 5, `--limit` to widen) rows of `path — title — one-line why-matched (which terms hit, in which field)` | The why-matched line is what makes a result checkable instead of a lucky guess |
| D5 | Zero hits is an empty result with a one-line note, never a typed refusal | A search miss is data mid-debug, not an error |
| D6 | Skill wiring in the same feature: the worker debug moment and the bee-hive scout reference name `bee knowledge search` as the pull move when a symptom appears mid-flow. Site erratum 2026-08-10: the worker debug moment lives in `skills/bee-swarming/SKILL.md` ("Execute (worker)") — `skills/bee-executing/SKILL.md` was retired into it on 2026-07-31 (commit 12ccd460); intent unchanged | The verb without the habit changes nothing — PBI's stated intent |
| D7 | Read-only contract: the verb never writes, never touches state, works identically inside a worktree and the main checkout | It must be callable from any session at any phase without gate interaction |

### Agent's Discretion

Ranking internals (whether to reuse the IDF machinery from `knowledge context` or the term-hit counter from `decisions search`), file-walk strategy, and output formatting details — constrained only by D3's observable ranking order and determinism.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| symptom | Free text an agent has mid-flow: an error excerpt, a wrong-behavior description, or a mechanism name — the query, not a structured field |
| why-matched | One line per result naming which query terms hit and in which field (title/body/sources) |

## Existing Code Context

From the deep-review gathers (2026-08-10). Downstream agents read these before planning.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/knowledge/context.rs` — IDF ranker + bundle walking (`build_context_manifest`); candidate ranking machinery to reuse or mirror
- `packages/bee-rs/crates/bee/src/verbs/decisions/` — `decisions search` term-split/rank implementation; D1/D3 mirror its grammar
- `packages/bee-rs/crates/bee/src/verbs/knowledge/routing.rs:22-41` — knowledge group dispatch table the new verb registers in

### Established Patterns

- Read-only knowledge verbs declare `writes: []` (see `verbs/knowledge/mod.rs:12-18`) — D7 rides this contract
- Every verb ships `--help` text in the dispatcher and a `json` flag

### Integration Points

- `skills/bee-swarming/SKILL.md` ("Execute (worker)") and `skills/bee-hive/references/scout-and-ticks.md` — D6's two wiring sites (bee-executing retired into bee-swarming, commit 12ccd460)
- CLI surface docs regenerate via `bee dev` render chain; `bee --help --json` shape guard tests cover new verbs

## Canonical References

- `docs/backlog.md` — PBI p-d7c88155 (story + acceptance criteria this feature delivers)
- `docs/knowledge/index.md` — bundle shape the corpus walk must respect

## Outstanding Questions

### Deferred To Planning

- [x] Reuse `context.rs` IDF ranker vs mirror `decisions search` term-hit counting — ks-1 mirrored the decisions-search term-hit counter (deterministic hit count, then timestamp recency, then path)
- [x] Whether `--area`/`--type` narrowing flags ship now — deferred to backlog; `--text`/`--limit` only in this feature

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
