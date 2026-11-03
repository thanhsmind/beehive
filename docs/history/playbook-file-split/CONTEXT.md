# Playbook file split — Context

**Feature slug:** playbook-file-split
**Date:** 2026-09-25
**Shaping session:** complete
**Scope:** Standard
**Domain types:** ORGANIZE

## Feature Boundary

Split the single `## Class playbooks` section in
`skills/bee-planning/references/planning-reference.md` (9 `###` subsections:
perf, bugfix, refactor, research, feature, docs, release, spike, content)
into one standalone Markdown file per route class under a new
`skills/bee-planning/playbooks/` directory, and update every citer, the Rust
parity fence, and the generated-file source so the split is the one home
going forward — a clean break, no transitional stub. It does not add a new
route class, does not change what a playbook's steps say, and does not
touch the cite-by-anchor citation mechanism itself (a plan still cites the
playbook by name, never copies its steps).

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. Changing one requires the user, a new D-ID or an explicit
supersession note, never a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | One playbook = one file. Each of the 9 route classes gets its own file at `skills/bee-planning/playbooks/<class>.md`, named exactly the class value (`perf.md`, `bugfix.md`, `refactor.md`, `research.md`, `feature.md`, `docs.md`, `release.md`, `spike.md`, `content.md`). This **supersedes** `docs/history/bee-playbooks/CONTEXT.md` D1 (decision `c2aa0417`), D2 (`fe8ff6a5`), and D4 (`ac42cc07`), each of which locked "one `##` section, never a standalone file." | User's explicit direction, after being shown the prior rejection and its reasoning: per-file layout serves adding future non-code classes (marketing, sales, media) better than one growing section. Logged via `bee decisions log --relation supersedes:c2aa0417` (also touches `fe8ff6a5`, `ac42cc07`). |
| D2 | Clean break, no compatibility stub. `## Class playbooks` is deleted from `planning-reference.md` in the same change that adds the new directory. Every citer (AGENTS.md's generated source, `bee-planning/SKILL.md`, `bee-hive` references, the Rust parity test) is updated in the same change — no host repo keeps resolving the old anchor after this ships. | User's explicit choice, given directly: "Cắt luôn" over a transitional index page, once told the file is vendored into every bee host repo and a split changes what host repos receive. |
| D3 | The citation mechanism is unchanged: `docs/history/bee-playbooks/CONTEXT.md` D3 (`82331837`, cite-by-name-and-anchor, never verbatim-copy into `plan.md`) still holds. A plan now cites the playbook by **file path** (`skills/bee-planning/playbooks/<class>.md`) instead of by file-plus-heading-anchor within one file; the steps are still read live from that file, never transcribed. | — |
| D4 | Host-repo vendoring changes from copying one file to copying a directory. This **partially supersedes** `docs/history/bee-playbooks/CONTEXT.md` D6 (`8fd45bbb`) only on the "one file" phrasing — its actual requirement (a playbook body is host-agnostic and never hardcodes a skill name only this repo has) is unchanged and still holds for every file in the new directory. | Confirmed no separate vendoring manifest names this file by path — `bee dev release-manifest` walks `skills/**` recursively (`packages/bee-rs/crates/bee/src/devtools/release_manifest.rs`), so the new directory is picked up automatically; no manifest code change is required. |
| D5 | `packages/bee-rs/crates/bee/tests/class_playbook_parity.rs` is real code, not docs, and hardcodes the one-file/one-`###`-per-class shape (constants `PLAYBOOKS_MD` and `PLAYBOOKS_HEADING`). It is rewritten in this feature to check one `skills/bee-planning/playbooks/<class>.md` file per `ROUTE_CLASS_VALUES` entry, in both directions (every route class has a file; every file in the directory names a real class) — the same two-directions guarantee it gives today, over the new shape. | Read at shaping: the test's own header explains the fence exists because a class with no playbook fails silently otherwise; that guarantee must survive the split. |
| D6 | `AGENTS.md` is a generated file rendered from `packages/bee/AGENTS.block.md` (`bee dev regen`). The Deep contracts pointer sentence naming the class-bound playbook is edited in the **source** (`AGENTS.block.md`), never in the generated `AGENTS.md` copy directly. | A prior cell in `bee-playbooks` (bpb-1) hand-edited the generated copy by mistake and `bee dev regen` reverted it; recorded as a real bug found at that feature's close (`docs/knowledge/work/bee-playbooks/delivery.md`, "Follow-on"). Naming it here so planning does not repeat it. |

### Agent's Discretion

- Whether the new `skills/bee-planning/playbooks/` directory gets its own
  short human-readable index/README file (a convenience for a reader
  browsing the directory) is left to planning — no product behavior depends
  on it.
- The exact wording of each updated pointer sentence (AGENTS.block.md,
  `bee-planning/SKILL.md`, `bee-hive/references/scout-and-ticks.md`,
  `bee-hive/references/routing-and-contracts.md`, `bee-planning/references/edge-dimensions.md`)
  is planning's to draft, as long as each still names "one playbook per
  route class" and points at the new directory instead of the old anchor.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Playbook | A route-class name, its numbered steps, its proof-line rule, and the skills/expertise it cites — now one Markdown file under `skills/bee-planning/playbooks/`, one file per route class (was: one `##` subsection in `planning-reference.md`). |
| Clean break | This split ships with no transitional alias, stub, or compatibility index — every citer is updated in the same change, and no old-shape reference is left resolvable. |

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `skills/bee-planning/references/planning-reference.md:220-420` — the current `## Class playbooks` section and its 9 `### <class>` bodies; this is the content that moves, verbatim, into the 9 new files.
- `packages/bee-rs/crates/bee/tests/route_class_parity.rs` — the sibling fence pinning the class *vocabulary* (unaffected by this split; only `class_playbook_parity.rs` assumes the one-file shape).

### Established Patterns

- `docs/knowledge/work/<slug>/index.md` + one concept file per topic — this repo already has a working "one topic, one file, one generated index" pattern to mirror for the new playbooks directory, if planning chooses an index file.

### Integration Points

- `packages/bee/AGENTS.block.md:350-351` (renders into `AGENTS.md:357-358`) — the Deep contracts pointer sentence naming "the class-bound playbook ... (`bee-planning/references/planning-reference.md`, "Class playbooks")"; must point at the new directory instead.
- `skills/bee-planning/SKILL.md:45` — cites `references/planning-reference.md ("Class playbooks")` by anchor; needs updating to the per-file citation shape.
- `skills/bee-hive/references/scout-and-ticks.md:36` and `skills/bee-hive/references/routing-and-contracts.md` — same pointer, same update.
- `skills/bee-planning/references/edge-dimensions.md` — cites the section; check and update.
- `packages/bee-rs/crates/bee/tests/class_playbook_parity.rs` — rewritten per D5.
- `docs/02-architecture.md`, `docs/04-skills-spec.md` — living docs that mention the section; check whether they need a pointer update (not confirmed at shaping depth).
- `docs/decisions/0009-artifact-scaling-and-cap-before-state.md`, `docs/decisions/skills/bee-planning-creation-log.md`, `docs/discovery/model-role-split/tickets/002-role-candidates.md`, `docs/discovery/test-doctrine/research/002-findings.md`, `skills/bee-technical-writing/CREATION-LOG.md` — historical citations of the old shape; these are records of past decisions and are **not** edited by this feature (a decision record is never reinterpreted after the fact) unless planning finds one that is a living, still-authoritative doc rather than a dated log.

## Canonical References

- `docs/history/bee-playbooks/CONTEXT.md` — the feature that locked the shape this one supersedes (D1, D2, D3, D4, D6).
- `docs/history/pstack-adoption/CONTEXT.md` and `docs/history/research/pstack-xia.md` — the earlier attempt at this exact port, and the "Direction of Truth: todo lists are projections" reasoning it was rejected under; planning should read why that reasoning applied then and confirm it no longer blocks a **file-per-class** split (as opposed to the verbatim-copy-into-todo mechanism that reasoning was actually about).
- `docs/knowledge/work/bee-playbooks/delivery.md` ("Follow-on: a real bug this delivery found") — the `AGENTS.md`-is-generated gotcha, restated as D6 above.

## Outstanding Questions

### Deferred To Planning

- [ ] Confirm whether `docs/02-architecture.md` and `docs/04-skills-spec.md` are living docs needing a pointer update, or historical snapshots — a quick read of each, not asked of the user.
- [ ] Confirm no other host repo currently vendors `planning-reference.md` by exact filename in its own tooling (outside `bee dev release-manifest`'s generic `skills/**` walk) before treating the clean break as zero-migration for host repos.

## Deferred Ideas

- Splitting non-code classes (marketing, sales, image/video generation) beyond `content` — already backlogged from `bee-playbooks` (per its D5); unaffected by this feature, still out of scope here.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads
locked decisions, code context, canonical references, and
deferred-to-planning questions. Planning's Gate 2 shape stage and reviewing
use locked decisions for coverage and UAT.
