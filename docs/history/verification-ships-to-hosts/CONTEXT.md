# Verification Ships To Hosts — Context

**Feature slug:** verification-ships-to-hosts
**Date:** 2026-09-01
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | ORGANIZE

## Feature Boundary

A bee-harnessed agent working in a host repository can generate, install and keep
a project-local skill that drives that repo's real product and records the run as
its declared test command. The feature ends at the generated skill being visible
to every runtime and its drive command being present in `commands.test`; it does
not change what any bee door checks, and it adds no proof tier.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The verification pair enters the `bee-*` namespace as TWO skills, split by lifecycle: `bee-verifying` generates a project-local `verify-<app>` skill once, at onboard time; `bee-verify-upkeep` is the periodic audit that keeps its feature map honest. Skill count goes 13 → 15 in every host repo. | Two trigger surfaces route an agent more accurately than one body carrying two cadences. Accepted cost: two more skills of context load in every host repo. |
| D2 | The generated drive command COMPOSES into `commands.test`. A repo that already declares one gets the drive appended (`<existing> && <drive>`); a repo with none gets the drive alone. | Every cap proof and every CI push then exercises the real product path, not only unit tests. Cost accepted with eyes open — see the Consequence note below. |
| D3 | A generated `verify-<app>` skill is SOURCE under `.bee/verify/<name>/` and is rendered by bee into every runtime skill home, the same way `bee-*` skills are. One generation serves every runtime; rendered copies are never hand-edited. | bee is multi-runtime. Writing straight into `.claude/skills/` — what `verify-bee` does today — makes the skill invisible to a Codex or opencode agent in the same repo. |

**Correction to D3's home list (2026-09-01, plan step).** D3 first named two
runtime homes, `.claude/skills/` and `.agents/skills/`, citing `doctor.rs:77-78`.
There are **three**: `onboard/templates.rs:317-321` declares `REPO_SKILL_TARGETS`
as `repo-claude`, `repo-agents` and `repo-opencode` (`.opencode/skills/`).
`doctor.rs` knows only two because bee's own doctor has no opencode row — a
second, separate gap. The decision is unchanged; its home list now reads
"every runtime skill home", resolved from `REPO_SKILL_TARGETS` rather than
enumerated by hand. Cell `vsh-1` already shipped the three-home wording, and
recorded the departure rather than following D3's stale parenthetical.
| D4 | No proof tier, no `commands.verify` revival, no new door. bee keeps one declared test command and one free-text proof line. | The owner retired the proof-tier matrix on 2026-07-31 (`decisions.jsonl:1763`) because "bee's evidence machinery grew too heavy", and `commands.verify` was retired in 2.1.0 (`onboard/templates.rs:291`). This feature works inside that decision, never around it. |
| D5 | `bee onboard` OFFERS, it never generates. The binary cannot run an agent skill; onboard emits a notice pointing at `bee-verifying`, and the agent acts on it. | The decision stands as written. Its original rationale did not — see the correction below. |

**Correction to D5's rationale (2026-09-01, plan step).** The rationale first
recorded here claimed `onboard/templates.rs:293` "already fires exactly when a
repo declares no `commands.test`". That is false, and the plan-step wave caught
it. `onboard/notices.rs:203` nests that notice inside
`if commands.contains_key("verify") {`, so a repo that never carried the
legacy `commands.verify` key — every repo onboarded after 2.1.0, which is the
entire population D5 aims at — receives no notice at all. The decision is
unchanged; the work it implies grew by one new notice, independent of the
retired-verify branch. Recorded because the original claim quoted a constant's
definition and never read its call site.

### Consequence recorded against D2

`ci-owned-verify` D5+D6 (`decisions.jsonl:1384`, 2026-07-23) deliberately made
`commands.test` the impacted subset for the dev loop, because full runs were too
slow; CI owns the full run. Appending a full product drive partly reverses that.
The agent raised this before the pick; the user chose composition anyway.

Mitigation is planning's to design, not a decision here: `control-<app>` exposes a
fast scoped drive for the cell loop and a full drive for CI, matching the split
that decision already draws.

### Agent's Discretion

- The wording and structure of both skill bodies, provided `bee-verifying`
  preserves the generator's proven five steps (interview the repo, generate,
  seed the feature map, prove the generated skill by running it end to end,
  point at upkeep).
- How the renderer learns about `.bee/verify/` — a second source root, or a
  generalisation of the existing skill-tree walk.
- Whether `verify-bee` in this repo is regenerated under `.bee/verify/` or left
  where it is as the worked example.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Drive command | The single executable entry point a generated skill ships (`control-<app>`), the thing a door or CI can run. Distinct from the skill, which is instructions for an agent. |
| Fast drive | The scoped invocation a cell's cap proof reaches for. |
| Full drive | The complete feature-map sweep CI runs on every push. |

## Specific Ideas And References

- The three skills already in this repo are the working proof: `bee-verifying`
  and `bee-verify-upkeep` are renames of `create-verification-skill` and
  `maintain-verification-skill` (byte-identical copies of pstack v0.14.5 at
  commit `b9ddc83`, provenance logged 2026-09-01), and `verify-bee` is that
  generator's proven output for this repo.
- `verify-bee`'s `control-bee` is the shape D2's drive command must take: one
  executable with `doctor`, `drive`, `cleanup`, self-resolving its repo root.

## Existing Code Context

### Reusable Assets

- `.claude/skills/create-verification-skill/` — the generator, proven: it
  produced `verify-bee`, which was run end to end.
- `.claude/skills/maintain-verification-skill/` — the upkeep pass.
- `.claude/skills/verify-bee/control-bee` — the drive-command shape, executable,
  self-documenting, resolving its repo from `BASH_SOURCE`.
- `packages/bee-rs/crates/bee/src/onboard/plan.rs:232` — `copy_expertise` /
  `remove_expertise`, the existing pattern for a non-skill tree that ships.

### Established Patterns

- Per-runtime rendering — `onboard/render.rs:379` walks entries and
  `onboard/skills.rs:882` bounds deletion to the `bee-*` namespace
  (*"refusing to remove {name}: outside the bee-* namespace"*). D3 needs the
  same shape for a second source root.
- Enumerated tool surfaces asserted by test — `herding/control_loop.rs:292-302`.
- Onboard notices as the offer surface — `onboard/templates.rs:286-293`.

### Integration Points

- `packages/bee-rs/crates/bee/src/onboard/render.rs` — skill-entry walk (D3).
- `packages/bee-rs/crates/bee/src/onboard/skills.rs` — sync and the deletion
  domain (D3).
- `packages/bee-rs/crates/bee/src/onboard/templates.rs:293` — the no-test notice
  gains the pointer (D5).
- `skills/` — two new skill directories (D1), then `bee dev regen`.

## Canonical References

- `.bee/decisions.jsonl:1763` — the 2026-07-31 owner decision retiring the
  proof-tier matrix. D4 exists to stay inside it.
- `.bee/decisions.jsonl:1384` — `ci-owned-verify` D5+D6, the dev-loop/CI split
  D2's mitigation must respect.
- `docs/history/research/autopilot-harness-roadmap.md` — why this ranks first:
  `onboard` ships the binary, `AGENTS.md`, the `bee-*` skills and
  `.bee/expertise/`, and nothing else. A skill outside the `bee-*` namespace can
  never reach a host repo.
- `docs/history/research/pstack-xia.md` — the source distil and provenance.

## Outstanding Questions

### Resolve Before Planning

None. All five decisions are locked.

<!-- bee:not-a-deferral: this heading names the shaping-to-planning handoff slot, and every question under it was answered in plan.md's Discovery and Open Questions. It documents where a question travels, not a promise to act later. -->
### Deferred To Planning

- [ ] Does the renderer learn `.bee/verify/` as a second source root, or does the
      existing walk generalise? — read `onboard/render.rs` and `skills.rs`
      together; the deletion domain must stay bounded either way.
- [ ] What exactly does the fast drive scope to, for a repo the generator has
      not seen yet? — the generated skill decides per repo; `bee-verifying` needs
      to state the contract it must satisfy.
- [ ] Does Codex's runtime read `.agents/skills/` for a non-`bee-*` skill the
      same way it reads a `bee-*` one? — verify before D3 is called done.

<!-- /bee:not-a-deferral -->

## Deferred Ideas — trigger `the-three-deferred-siblings-of-this-feat__0d3e4f89`

- A progress test for `bee herding control-loop` (it counts failures, never
  progress) — separate feature; recorded in
  `docs/history/research/bee-unattended-hardening.md`.
- Narrowing the two cockpit panes' CLI wildcard to an enumerated surface, per
  locked decision herding-adopt D7-FINAL — separate feature, same brief.
- `.bee/expertise/changes.md`, the missing code-shape craft file — separate
  feature; the roadmap ranks it second.

<!-- bee:not-a-deferral: the handoff note describes which artifact downstream steps read. It names the deferred-to-planning slot as a location, never as work this feature still owes. -->
## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads the
locked decisions, the code context, and the deferred-to-planning questions.
D4 is the boundary: this feature adds no proof tier and no door.
<!-- /bee:not-a-deferral -->
