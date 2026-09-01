# Expertise Principles — Context

**Feature slug:** expertise-principles
**Date:** 2026-09-02
**Shaping session:** complete
**Scope:** Standard
**Domain types:** ORGANIZE, READ

## Feature Boundary

Give bee's craft layer speakable, harness-supported handles: a thin
principle skill per craft principle, sitting on top of the ten craft
guides in `expertise/`, selected by `bee orient` from class and flags,
and named in the reply beside the decision each one changed.

It ends at those three surfaces — the principle skills, the router's
selection, and the naming obligation. It does NOT delete or rewrite an
expertise guide, add a route class or lane, change gates, worktrees, or
the cell lifecycle, and it does not touch the six domain guides.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | The 16 `expertise/` guides stay as the body of the craft layer. A new thin principle layer sits on top: one skill per principle, each a short page naming the principle, when it fires, and the rule, with a pointer into its guide section for depth. No guide is deleted or shattered. | `expertise/` holds ~250 rules across 16 guides, so a 1:1 conversion to skills is impossible and a full shatter would strand most rules. Keeping the guides preserves every rule while the principle skills supply the speakable handles and the harness support (description index, model invocation) that a plain markdown guide cannot get. Decision `1d6ff0dc`. |
| D2 | `bee orient` selects the principles. The router already knows class, lane and risk flags; it names the candidate principles for the task in the session preamble, and the agent reads only those. The principle index is NOT always-loaded. | bee's doctrine is progressive disclosure — route, then read exactly what the task needs (`expertise/INDEX.md`: "never load all of them"). An always-loaded index spends context on every turn whether or not the task triggers a principle, and duplicates a routing job `bee route` already does. Decision `11221f5b`. |
| D3 | The naming obligation extends from user-invoked rule ids to routed principles: on a multi-step task the reply MUST name each principle it applied and the decision that principle changed, or state plainly that it changed nothing. | The obligation on the answer, not the list, is what makes a principle steer. `pstack-gaps` decision `f051746e` already established this law for the 10 `AGENTS.md` rule ids on user invocation; this widens the same law to router-selected principles, so a routed principle cannot be read and silently ignored. The "changed nothing" branch keeps it honest. Decision `c101be56`. |
| D4 | The first cut covers the TEN craft guides only: `thinking`, `planning`, `architecture`, `decisions`, `tests`, `review`, `documentation`, `knowledge`, `debugging`, `merges`. The six domain guides (`data`, `apis`, `security`, `operations`, `performance`, `frontend`) keep their present shape and gain no principle skills in this feature. | Craft guides fire on nearly every task, so they carry the leverage; domain guides fire only when the work is domain-shaped. Shipping the craft half first proves the layer under real use before the surface doubles. Decision `f1b48dac`. |

### Agent's Discretion

Planning owns, bounded by D1–D4:

- **Which principles, and how many.** The set is derived from the ten
  craft guides' headings. Not every heading earns a principle; a
  principle earns its skill only if a task can be steered by speaking
  its name. Roughly 12–18 is the expected shape, not a target.
- **Naming and location.** The `principle-<slug>` convention and where
  the skills live in the tree (`skills/`, a subdirectory, or a rendered
  home), provided one home holds each principle and every other surface
  points at it.
- **The class-to-principle mapping's storage.** Whether `bee orient`
  reads the mapping from a manifest, from skill frontmatter, or from
  the existing class playbooks — provided the mapping has ONE home and
  a parity test covers it (the rule-living-in-N-places pattern).
- **Where D3's law sits** inside `packages/bee/AGENTS.block.md`,
  provided it is one place and it does not duplicate the `f051746e`
  invocation law — it extends it.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| **principle** | One named craft rule, small enough that speaking its name redirects the work. Ships as a skill; its depth stays in an `expertise/` guide section. |
| **the craft half** | The ten `expertise/` guides named in D4. Distinct from the domain half (the other six). |
| **routed principle** | A principle `bee orient` named for the task at hand, as opposed to one the user spoke by name mid-run. D3's obligation covers both. |
| **the naming obligation** | The reply-side law: name each applied principle and the decision it changed, or say plainly it changed nothing. |

## Specific Ideas And References

- pstack ships 21 principles as individual skills, and `/poteto-mode`
  reads their index at the start of every multi-step task. Its leverage
  is not the list — it is the obligation to name each applied principle
  and the choice it drove. bee takes the obligation (D3) and the small-
  skill shape (D1), but routes instead of always-loading (D2).
- Reference tree, read-only: `/home/thanhsmind/Projects/refs/cursor-plugins/pstack`
  — `skills/poteto-mode/SKILL.md` (the index and the citation rule),
  `skills/principle-prove-it-works/SKILL.md` (a leaf's shape and size).

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `expertise/INDEX.md` — today's router: one row per guide, situation →
  entry. The principle layer replaces its *selection* job with D2's
  router output; the file itself stays as the guide index.
- `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`
  (§ AGENTS.md rule homes) — the existing spoken-rule registry from
  `pstack-gaps` D4. The principle set is the same idea one layer down.
- `packages/bee-rs/crates/bee/tests/rule_index_parity.rs` — the
  text-reading parity test pattern for a registry that must match its
  markers. The principle mapping needs the same shape of proof.
- `skills/bee-planning/references/planning-reference.md` — holds the
  per-class playbooks from `pstack-adoption` D1. The playbooks are the
  natural carrier for a class-to-principle pointer.

### Established Patterns

- Single home, pointers everywhere else (`pstack-adoption` D1,
  `pstack-gaps` D1). A principle's rule text lives in one place; the
  guide and the playbook point at it, never transcribe it.
- Progressive disclosure — `expertise/INDEX.md` states the law
  explicitly: route, then read exactly one.
- A rule living in N places needs one test that reads all N
  (`docs/knowledge/patterns/20260826-a-rule-living-in-n-places-needs-one-test-that-reads-all-n.md`).

### Integration Points

- `packages/bee-rs/crates/bee/src/router.rs` — computes class, lane and
  flags. D2's selection reads from here.
- `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs` and
  `packages/bee-rs/crates/bee/src/hooks/session_preamble/` — where the
  named principles surface to the agent.
- `packages/bee-rs/crates/bee/src/onboard/skills.rs` (`compute_skill_sync`)
  and `.claude-plugin/plugin.json` (`skills: ./.claude-plugin/skills/`)
  — the two pipes that already carry skills to host repos. Principle
  skills ride them unchanged; no new distribution path is needed.
- `packages/bee-rs/crates/bee/src/onboard/plan.rs:235` —
  `("expertise", Some(".bee/expertise"))`, the guide vendoring that
  stays exactly as it is under D1.
- `packages/bee/AGENTS.block.md` — home of D3's law.

## Canonical References

- `docs/history/pstack-adoption/CONTEXT.md` — the class playbooks this
  feature connects principles to.
- `docs/history/pstack-gaps/CONTEXT.md` — D4/D5, the spoken rule ids and
  the invocation law that D3 extends.
- `docs/history/research/pstack-distill.md` — the pinned study of
  pstack, including what must NOT be ported.

## Outstanding Questions

### Resolve Before Planning

None.

### Deferred To Planning

- [ ] Does adding ~15 skills to a host repo's skill index cost enough
      always-loaded description budget to matter? — measure the current
      skill-description total, then the projected one, before choosing
      how terse each principle's `description` must be.
- [ ] Do the principle skills need `disable-model-invocation: true`
      (pstack's choice, so the model never auto-fires a leaf) or does
      bee want them model-invocable? — decide from how D2's router
      output actually reads in a live session.

## Deferred Ideas

- The six domain guides (`data`, `apis`, `security`, `operations`,
  `performance`, `frontend`) get their own principle layer. Deliberately
  out of scope per D4; revisit once the craft half has lived through
  real tasks.
