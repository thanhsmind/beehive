# bee playbooks — Context

**Feature slug:** bee-playbooks
**Date:** 2026-09-24
**Shaping session:** complete
**Scope:** Standard
**Domain types:** ORGANIZE, RUN

## Feature Boundary

Make bee's existing class-bound playbook system (`skills/bee-planning/references/planning-reference.md`
§ "Class playbooks", built in `pstack-adoption`) a named, documented concept —
and extend it with one new non-code route-class, `content`, proving the
system generalizes past software work. It does not create a new directory,
a new router skill, or a verbatim-copy mechanism; those were already tried
and deliberately rejected (see D1/D3 below). It ends at: the vocabulary
update, a gap-check research artifact against pstack's playbook catalog,
the `content` class and its playbook body, and the backlog entries for
what stays out of this slice.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | No new `playbooks/` source directory and no new `.bee/playbooks/` vendoring. bee's playbook system already exists: `planning-reference.md` § "Class playbooks", one `##` subsection per route-class value, cited by name and anchor from every plan (per `pstack-adoption` D1, decision `132551fb`). This feature extends that one home; it does not create a second one. | Discovered mid-shaping: `docs/history/pstack-adoption/CONTEXT.md` and `docs/history/research/pstack-xia.md` already ran this exact port, deliberately rejected a new directory plus verbatim-copy playbooks (collides with bee's Direction of Truth: todo lists are projections), and built the class-bound cited system instead. Decision `c2aa0417`. |
| D2 | A playbook is the existing shape: a route-class name, its numbered steps, its proof-line rule, and the skills/expertise it cites — one `##` section under "Class playbooks", never a standalone file. This feature makes the CONCEPT first-class vocabulary (name it "playbook" in `AGENTS.md` and `bee-hive/SKILL.md`, pointing at the one home), not a content move. | Same discovery as D1: the shape already works; the real gap is that "playbook" is not yet a documented word anywhere an agent reads first (`AGENTS.md`, `CLAUDE.md`, `README.md`, `bee-hive/SKILL.md` all currently have zero hits for "playbook"). Decision `fe8ff6a5`. |
| D3 | No new `bee-hive` "Playbook match" section and no verbatim step-copying into the todo list. `bee-hive`'s existing Route step already resolves a class via `bee-planning`, which already cites the matching class playbook by name and anchor at plan-render time. This feature adds vocabulary naming that citation step "the playbook"; it does not change the citation mechanism. | `pstack-adoption` D1 (decision `132551fb`) explicitly superseded copy-verbatim for cite-by-anchor, with a documented reason (stale, transcribable todo text). Decision `82331837`. |
| D4 | The new `content` route-class's playbook is one new `##` subsection in the existing "Class playbooks" section, plus one reference-table row wherever a class is first surfaced to an agent — never a second file. | Mirrors `pstack-gaps` D1's precedent ("one new home, one reference-table row, nobody transcribes the steps"). Decision `ac42cc07`. |
| D5 | First slice: (1) a short research artifact comparing pstack's `poteto-mode` playbook catalog (22 playbooks) against bee's current 8 route-classes plus its skill catalog (`bee-herding`, `bee-writing-skills`, `bee-evolving`, HANDOFF/adopt, spike lane, `bee worktree prune`, phase-plan-vs-epic-map), naming each pstack playbook covered-stronger / covered-adequately / a genuine gap / not-applicable; (2) closing only the genuine gaps that fit a code-repo engine; (3) adding ONE new non-code route-class, `content`, the same small way `perf` was added, to prove the non-software extension for real. Further non-code classes (marketing, sales, image/video generation) are named as backlog follow-ups, not built now. | Most of pstack's 22 playbooks turned out already covered, often more strongly, once the existing class-playbook system and skill catalog were actually read. Porting 22 files plus 3 new marketing playbooks would mostly duplicate a single-source-of-truth bee already owns; the real, unmet ask is the non-code extension. split-never-shrink: the remainder goes to backlog by name, not silently dropped. Decision `da83bb92`. |
| D6 | The `content` class's proof-line rule is host-agnostic: "the artifact checked against the project's own style/fact/brand rule when the project has one (name it), otherwise a documented second read naming what was checked", recorded as `<check> — <result> — <scope reason>`, the same shape every other class already uses. The playbook body never hardcodes a specific plugin skill name (e.g. `marketing:brand-review`), since `planning-reference.md` is vendored into every bee host repo and most will not have that skill installed. | bee's class playbooks are authored once in the bee source repo and vendored everywhere; a step naming a skill only this host happens to have would silently break in every other host repo. Decision `8fd45bbb`. |

### Agent's Discretion

Which exact pstack playbooks the gap-check names as genuine gaps, and
whether any of them earn a cell in THIS slice versus a `bee backlog add`
entry, is planning's call from the research artifact's evidence — bounded
by D5 (only non-code extension is guaranteed in-slice; code-side gaps are
judged on their own merits, smallest-honest-shape). The exact wording of
the `content` class's steps is planning's, bounded by D2, D4, D6.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Playbook | The named, ordered step list plus proof-line rule bound to one `bee route --set --class` value, homed in `planning-reference.md` § "Class playbooks", cited (never copied) from every plan. Not a file of its own. |
| Route class | The `bee route --set --class` enum (`ROUTE_CLASS_VALUES`, `workflows.rs`) — currently `feature, bugfix, docs, refactor, research, release, spike, perf`; this feature adds `content` as the 9th value. Distinct from a cell's own `change_class` taxonomy, which a playbook never selects. |
| Non-code class | A route class whose proof-line rule and steps do not assume a git worktree, a test suite, or source code — `content` is the first one. |

## Specific Ideas And References

- `docs/discovery/spec-drops/sup-20260924-fb8a.md` — the originating spec-drop:
  human liked pstack's playbook concept, wants it in bee, named marketing and
  content/sales as extension examples, said "làm hoàn chỉnh luôn đi" (finish
  it completely, now).
- `/home/thanhsmind/Projects/refs/cursor-plugins/pstack/skills/poteto-mode/` —
  the source: `SKILL.md` (router + "Playbooks" section), `playbooks/*.md`
  (22 files), `../figure-it-out/SKILL.md` (bespoke-playbook fallback for
  large/cross-cutting work).
- User's second answer, verbatim intent: elevate the class-playbook concept
  into a named "playbook" vocabulary, run a fresh craft comparison against
  pstack's actual playbook catalog (not just the mechanism-level xia sweep),
  and pave the way for non-code projects (content for a marketing team,
  image/video generation) using a playbook-plus-skill combined shape. Image
  <!-- bee:not-a-deferral: names the user's own words about a possible future direction; not a promise this feature or bee will act on it -->
  and video generation are named as a *future* direction, not a concrete
  ask for this slice (see Deferred Ideas).
  <!-- /bee:not-a-deferral -->

## Existing Code Context

### Reusable Assets

- `skills/bee-planning/references/planning-reference.md:220-390` — the
  existing "Class playbooks" section: all 8 classes already have complete,
  well-formed step lists and proof-line rules. `feature`, `bugfix`, `refactor`,
  `docs`, `research`, `release`, `perf`, `spike` — nothing here is a stub.
- `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:356-368` —
  `ROUTE_CLASS_VALUES` (8 values today) plus the documented safety argument
  for adding a 9th (never collide with `ROUTE_LANE_VALUES`).
- `docs/history/pstack-adoption/CONTEXT.md` and `plan.md` — the feature that
  built the class-playbook system and added `perf` as the 8th class; the
  direct precedent for adding `content` as the 9th.
- `docs/history/pstack-gaps/CONTEXT.md` — the precedent for a second class
  sharing one home with a one-row pointer (`research` class + trace/
  provenance flow), the model for D4.
- `docs/history/research/pstack-xia.md` — the deep mechanism-level pstack
  sweep (not playbook-catalog-level); read before this feature's own D5 gap
  check to avoid re-doing its work.
- `bee-herding`, `bee-writing-skills`, `bee-evolving`, HANDOFF+adopt
  (`AGENTS.md` § "Start a session"), the spike lane, `bee worktree prune`,
  and `planning-reference.md`'s "Phase plan vs epic map" section — the
  existing bee mechanisms that already answer most of pstack's Babysit,
  Shipping, Autopilot, Orchestrate, Authoring-a-skill, Eval, Session-pickup,
  Pause-safely, Worktree-cleanup, and Multi-phase-plan playbooks respectively.
  The D5 gap check confirms or corrects this reading per playbook.

### Established Patterns

- Single-home procedure with pointers back — `pstack-gaps` D1 is the direct
  model for D4.
- Named deviation over refusal (`AGENTS.md`, "Judgment and deviation") — a
  playbook step that does not apply stays visible with a recorded reason,
  unchanged by this feature.
- Public-contract enum change plus migration note — `pstack-adoption` D2's
  `perf` addition is the direct template for adding `content`.

### Integration Points

- `AGENTS.md`, `CLAUDE.md` (bee's own, not the host's), `README.md`,
  `skills/bee-hive/SKILL.md` — none currently uses the word "playbook";
  D2 adds it, pointing at the one home.
- `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs` — D4's
  9th enum value lands here, following the `perf` precedent's safety
  argument comment.
- `skills/bee-planning/references/planning-reference.md` — the new `content`
  `##` subsection lands here (D4, D6).

## Outstanding Questions

### Resolve Before Planning

None.

<!-- bee:not-a-deferral: this section is CONTEXT.md's standard template naming what planning must still investigate — the investigation happened during planning (see plan.md's Open Questions), not a promise bee itself will act on these later -->
### Deferred To Planning

- [ ] Which specific pstack playbooks (of the 22) the D5 gap check should
      spend real research time on first, versus ones already obviously
      covered from this shaping pass's own reading — planning scopes the
      gap-check cell from the evidence already gathered here.
- [ ] Whether `content` needs its own new lane behavior or rides the
      existing lane-classification logic (which already exempts non-code-
      touching work via the `docs` lane) unchanged — investigate `workflows.rs`'s
      lane-vs-class handling before assuming either way.
<!-- /bee:not-a-deferral -->

<!-- bee:not-a-deferral: this section records already-made scope decisions (what is deliberately out of this slice) and, where a bee backlog add row is named, that row was already added by cell bpb-3 — not an open promise for bee to act on later -->
## Deferred Ideas

- Marketing and sales as their own route-classes (distinct from `content`) —
  the user named them as examples of the same pattern; `content`'s shape
  is meant to make adding them cheap later, but they are not built now.
  `bee backlog add` at capture time.
- Image/video generation as a route-class or playbook — named by the user
  as a future direction ("cũng sẽ có dạng kết hợp giống playbook và skill"),
  not a concrete ask for this slice. `bee backlog add` at capture time.
- Any genuine code-side gap the D5 research artifact finds that is judged
  too large or too speculative for this slice — named in the artifact and
  backlogged, never silently dropped (AGENTS.md split-never-shrink).
<!-- /bee:not-a-deferral -->

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads
locked decisions, code context, canonical references, and deferred-to-
planning questions. Planning's Gate 2 shape stage and reviewing use locked
decisions for coverage and UAT.
