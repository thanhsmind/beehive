---
artifact_contract: bee-plan/v1
mode: standard
---

# Plan: Wayfinding Flow

Mode: `standard` — 1 risk flag: public-contracts (status/preamble output
shape, new CLI verb group)
Why this is the least workflow that protects the work: one slice of five
bounded cells covers a new skill, three Rust surfaces, and one skill
edit; anything less drops a locked activation mechanism (D6).

## Requirements (from CONTEXT.md)

- D1: separate `bee-wayfinding` skill, not a bee-shaping move.
- D2: map = markdown under `docs/discovery/<effort>/` (MAP.md +
  tickets/NNN-<slug>.md); no new state store.
- D3: name `bee-wayfinding`.
- D4: `bee status` + session preamble show open maps with frontier
  counts from v1.
- D5: `bee orient` recommends `skill=bee-wayfinding` deterministically
  when an open map has frontier tickets and no feature work is active.
- D6: four activation mechanisms (resume scan, explicit invocation,
  Qualify park-for-vagueness → map stub, shaping entry check with Gate 1
  backstop).
- D7: wayfinder ticket model — 4 types, destination-first, one HITL
  ticket per session, convention-only claim/block lines.
- D8: resolved tickets log via `bee decisions log`; map only gists;
  exit hands decisions to bee-shaping Lock without re-asking.

## Discovery

Gather report (this session) anchored the integration points:
- Status text: `render_status_text` in
  `packages/bee-rs/crates/bee/src/verbs/status_full/render.rs:100`
  (push-lines pattern; insert a guarded block before `Recommended next`
  at render.rs:427); status fields built in `build_status`
  (status_full/build.rs — grep `fn build_status`).
- Orient: `build_orient` in
  `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:298`;
  `next.skill` via `ORIENT_PHASE_SKILL` lookup (orient.rs:471-481),
  `next.command` via `orient_next_command` (orient.rs:34-42); blockers[]
  assembled orient.rs:334-409 (report-only lines plug in there).
- Preamble: `build_session_preamble` in
  `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:448`
  (same push-lines shape); SessionStart hook calls it in-process via
  `hooks/session_init.rs:75`, NOT via `bee status`.
- Park verdict: NO Rust path exists — Park lives in bee-shaping prose
  (skills/bee-shaping/SKILL.md:66-88); `bee route` lane enum
  (`workflows.rs:289`) has no "park" value.
- Skill registration: automatic — `list_bee_skill_dirs` read_dirs
  `skills/` (devtools/skill_trees.rs:435); `bee dev render-skill-trees`
  regenerates `.claude-plugin/skills/` and `.codex-plugin/skills/`.

## Approach

Recommended path: add a small `discovery` module + verb group in bee-rs
owning MAP.md scanning and stub creation; status/preamble/orient consume
it (D4, D5). Qualify's park-for-vagueness calls `bee discovery stub`
(D6.3) — the stub command is the code-hard part; adding a "park" value
to the route enum is rejected (breaks the route contract for no gain;
Park today never calls route at all — Discovery). The skill is authored
from docs/history/wayfinding-flow/design-draft.md (D1, D7, D8); shaping
gets its entry check and Lock hand-off (D6.4, D8).

Risk map: discovery module LOW (new, isolated, unit-testable);
status/preamble insertion LOW-MEDIUM (public output shape — additive
line only); orient recommendation MEDIUM (must not misfire while a
feature is active — condition tested both ways); skill prose LOW.

## Shape

One slice, five cells (phases would be forced — this is one coherent
capability):

| Cell | What changes | Proof |
|---|---|---|
| wf-1 | `skills/bee-wayfinding/` (SKILL.md, references, agents/openai.yaml) + bee-hive route-table row + rendered skill trees | render-skill-trees clean, marker grammar valid |
| wf-2 | bee-rs `discovery` module: scan `docs/discovery/*/MAP.md` + tickets, frontier calc; verbs `discovery list`/`discovery stub`; router wiring | unit tests: scan, frontier, stub create/refuse, malformed-map remedy line |
| wf-3 | status text + status JSON + session preamble show open maps (deps wf-2) | tests asserting the section appears/absents |
| wf-4 | orient: `next.skill=bee-wayfinding` + blocker line when map open, frontier>0, no active work (deps wf-2) | tests both ways (fires idle, silent mid-feature) |
| wf-5 | bee-shaping SKILL.md: entry fog check, Qualify park→`bee discovery stub`, Lock consumes map decisions; regen trees (deps wf-1, wf-2) | render-skill-trees clean; prose cites D6/D8 |

## Test matrix

Triad, smallest demonstrating size:
- Happy: stub creates `docs/discovery/<slug>/MAP.md`; scan reports it
  with frontier count; status/preamble line renders; orient recommends
  bee-wayfinding when idle with open frontier.
- Edge: no discovery dir → no section, no recommendation; map with zero
  open tickets → listed, no frontier, orient silent; ticket with
  blocked-by/claimed-by lines excluded from frontier.
- Error: stub onto existing effort slug → typed refusal, nothing
  written; unreadable MAP.md → visible "unreadable … — remedy" line,
  never a crash.

## Out of scope

- `bee triggers` nudge for stale maps; orient route-table entry for
  brand-new fog; CLI guard for claim/block lines (all deferred in
  CONTEXT.md).
- Any change to `bee route`'s lane enum.
