# Knowledge freshness mechanic — Context

**Feature slug:** knowledge-freshness-mechanic
**Date:** 2026-11-07
**Shaping session:** complete (consumes docs/discovery/knowledge-from-code/MAP.md D1–D8)
**Scope:** Standard
**Domain types:** READ | RUN | ORGANIZE

## Feature Boundary

Ship the portable mechanic that lets a host project's knowledge bundle stay
honest beside its code: every area concept carries machine-readable anchors
and a verified commit, `bee knowledge check` grades each concept confirmed,
suspect or orphan from git, every read verb and the session preamble print
that grade, the cap sync door makes the worker re-confirm or edit concepts
anchored to the files it touched, promote stops generating delivery
records, and `bee knowledge report` adds pull rate per feature. It ends
before any content change to bee's own areas (that is the next, bee-only
feature) and before glossary or event catalogue.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Knowledge is pulled on demand; nothing new is pushed into briefs. (map D1, 2b11f025) | Preamble and verbs print grades; brief.rs is not touched. |
| D2 | Document homes rule: OKF holds vision, glossary, area map/boundaries/invariants, decisions, patterns; behaviour lives in verify features and tests, contracts are generated, runbooks are scripts; every area entry anchors to code or a verify feature. (map D2, 93e1001d) | This feature ships the anchor contract and the check; the homes rule text lands in the OKF profile area and bee-capturing. |
| D3 | Freshness is derived from git: concept carries `bee.verified_at` (ISO date) and `bee.verified_commit` (sha); check grades confirmed (no anchor changed after verified_commit), suspect (an anchor changed after it), orphan (an anchor path is gone); search, show, context and the preamble print the grade. (map D3, 27ad96a5) | A concept with anchors but no verified_commit grades `unverified`, a fourth visible state, never silently confirmed. |
| D4 | Updates ride existing doors: the cap sync door lists concepts whose anchors intersect the cell's touched files; the cap is refused until each is either re-confirmed (verified_commit bumped) or edited in the same cell, with `--sync-ack` as the recorded escape; promote proposes pattern candidates and area invariant bullets only and no longer writes `work/<feature>/delivery.md`. (map D4, e7e80d78) | Existing `sync_refusal` gains check (d); promote's delivery section is removed, its tests updated. |
| D5 | Delete before write: this feature moves `docs/knowledge/work/` out of the bundle (to `docs/history/knowledge-work/`), excluded from index, context and check; it authors no glossary and no event catalogue. (map D5, 28fb074d) | Index, context and check walks must not see the moved tree. |
| D6 | Pull rate is the measure: `bee knowledge report` gains per-feature counts of `knowledge search` and `knowledge context` invocations read from `.bee/logs/timings.jsonl` joined to the active feature at the time. (map D6, e7e133dc) | Timings rows carry no feature today; the join uses the lane record's active window or a feature field added to the row. |
| D7 | The mechanic is portable: it lives in the bee binary, the OKF profile area, and the bee-capturing skill; hosts receive it through onboarding with no host-side configuration. (map D7, 6a502ad0) | No bee-repo-only path. |
| D8 | An anchor is an entry in a new `bee.anchors` frontmatter list of repo-relative paths; a one-shot `bee knowledge anchors seed` fills it from the concept's Pointers paths that resolve on disk; Pointers stays prose. (map D8, 4148025d, agent's named guess) | Seeding is a verb, re-runnable, writes only concepts with no anchors list. |

### Agent's Discretion

- Exact flag and field names, output row layout, and the grade's wording in
  text output, as long as `--json` carries `grade`, `verified_at`,
  `verified_commit`, and the changed anchor paths.
- Whether `verified_commit` is compared by `git log` ancestry or by
  last-touch timestamp of each anchor; must be correct for a squashed or
  rebased history and must degrade to `unverified` with a named reason when
  git is unavailable.
- How the preamble shows grades without growing its byte budget: a count
  line (`knowledge: N confirmed, M suspect, K orphan`) is acceptable.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| anchor | A repo-relative path in `bee.anchors` that a concept's truth depends on. |
| verified_commit | The commit sha at which a human or worker last confirmed the concept against its anchors. |
| confirmed / suspect / orphan / unverified | The four freshness grades `knowledge check` derives (D3). |
| pull rate | Count of knowledge search and context invocations attributed to a feature. |
| re-confirm | The cap-time act of bumping verified_commit without editing the concept body. |

## Specific Ideas And References

- Owner's framing: knowledge is a new hire's onboarding layer beside code,
  pulled when needed, with freshness visible so the agent can weigh it.
- Measured 2026-11-07: 95 of 139 area concepts pointed at code changed after
  the concept; search invoked 89 times in 35,478 CLI calls; bundle 524k words.

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs` — the bundle
  validator with its WARN/ERR finding rows; `dangling_source_candidate`
  already resolves path-shaped entries against two roots.
- `packages/bee-rs/crates/bee/src/verbs/knowledge/frontmatter.rs` — the
  canonical emitter (`is_plain_safe`, `emit_scalar`); any new field must
  round-trip through it or `not_canonical` fires.
- `packages/bee-rs/crates/bee/src/verbs/knowledge/search.rs` — `SearchHit`
  rows (`path`, `title`, `hits`, `why`) to extend with the grade.
- `packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs` — existing
  report verb to extend with pull rate.
- `packages/bee-rs/crates/bee/src/verbs/knowledge/promote.rs` — the
  delivery draft section to remove.
- `packages/bee-rs/crates/bee/src/hooks/session_preamble/knowledge.rs` —
  concept collection for the preamble.
- `.bee/logs/timings.jsonl` — `{ts, cmd, ms, ok}` rows, the pull-rate source.

### Established Patterns

- Finding rows in `knowledge check` carry a code, a path, and a remedy
  sentence; new grades follow that row shape.
- `--json` output is a manifest, never file content (context.rs).

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/cells/sync_door.rs` —
  `sync_refusal(root, cell, touched)` gains check (d) for anchored
  concepts; `--sync-ack` already records the escape.
- `packages/bee-rs/crates/bee/src/verbs/knowledge/index.rs`,
  `context.rs`, `walk.rs` — must exclude the moved `work/` tree.
- `docs/knowledge/areas/okf-profile/` — the homes rule and grade semantics
  are documented here (sync door will require it).
- `skills/bee-capturing/SKILL.md` — the capture discipline names
  re-confirm and the homes rule.

## Canonical References

- `docs/discovery/knowledge-from-code/MAP.md` — the decisions this feature
  consumes.
- `docs/knowledge/areas/okf-profile/overview.md` — OKF v0.1 profile.
- `docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md`
  — the cap door and its sync checks.

## Outstanding Questions

### Deferred To Planning

- [ ] How timings rows learn their feature — a new field written by the
  dispatcher from the bound lane, or a join on lane start/close windows.
- [ ] Whether `work/` moves by `git mv` in this feature or is left in place
  and excluded, given 453 files — planning picks by what the index and
  context walks can exclude cheaply.
- [ ] Seed coverage: how many area concepts have zero resolving Pointers
  paths after seeding (they grade `unverified` with no anchors).

## Deferred Ideas

- Cut bee's own areas to map + invariants (map ticket 002) — next feature,
  bee-only content work.
- Glossary (ticket 004) and event catalogue (ticket 003) — wait on pull
  rate (D5).
- Pattern freshness — patterns keep a date only (agent-suspected, on the map).

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
