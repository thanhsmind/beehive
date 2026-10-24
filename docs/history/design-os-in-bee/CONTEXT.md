# design-os in bee — Context

**Feature slug:** design-os-in-bee
**Date:** 2026-10-24
**Shaping session:** complete
**Scope:** Deep
**Domain types:** SEE | RUN | ORGANIZE

## Feature Boundary

bee ships the design-os (ease-design) design taste inside itself, adds a
dedicated UI design role to its team that works from that taste, installs only
the design packs a host picks for its project type, and makes a cell that
changes UI files prove itself with `ui gate` under a baseline ratchet; it ends
at bee's own distribution and cap rules, and does not change design-os itself.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | bee bundles the design-os taste (its `knowledge/` and design skill content) so it ships with bee, instead of the host installing design-os on its own. Decision b99dde04. | Owner: the design agent is stronger with the taste built in. |
| D2 | The licensor, Jang Trinh, has agreed to bee bundling design-os (PolyForm Shield 1.0.0), per the owner on 2026-10-24. bee still carries the license text, the `Required Notice:` line, and the upstream third-party notices with what it ships. Decision b99dde04. | The Shield Notices clause binds every copy bee distributes. |
| D3 | bee's team gains a dedicated UI design role. No visual/UI design role exists today; `lane-1..3` are software-design seats. Decision 961426b5. | — |
| D4 | Design packs install by project type, akkit-style: at install the user picks which packs fit the project, and a pack not picked is never written into the host. Decision 572bc11e. | anphabe-goglbe received 23 skills and 23 `/ui:*` commands, most unused by a web dashboard. |
| D5 | UI checks run through the upstream Node tool: the design role runs `ui gate` through `npx ease-design` at a pinned version and needs Node 20 or newer. Without Node the role still designs from the bundled taste and says plainly that the check did not run. Decision d5399608. | No Rust port of the linters. |
| D6 | A cell that changes UI files must carry `ui gate` proof under a baseline ratchet: the error count may not rise above the recorded baseline, and the cap refuses a UI cell with no such proof. Decision 587bcc85. | A hard PASS would red-block honest work: the anphabe-goglbe run showed linter false positives and a dashboard-noisy rubric. |

### Agent's Discretion

- How the bundled copy is pinned to an upstream design-os commit and refreshed.
- Which design-os knowledge files and skills form each pack, and the pack
  names, as long as D4 holds.
- How the role is wired: a config-only role, or a rendered `bee-design` agent.
- Which file kinds count as "UI files" for D6, stated in the plan.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| design taste | design-os's curated design knowledge (`knowledge/` and its design skill), the part an agent reads; not the `ui` linters |
| design pack | a named subset of the design taste for one project type (for example web, native Apple); the unit the user picks at install |
| UI cell | a cell whose changed files include UI files as the plan defines them |
| baseline | the recorded `ui gate` error count a UI cell may not rise above |

## Specific Ideas And References

- akkit: the owner's model for install-time picking of skills per project
  type (not available on this machine; the owner's description is the spec).
- anphabe-goglbe already runs a design gate ratchet in CI
  (`.gitlab-ci.yml` job `design-gate`, `scripts/design-gate.mjs`,
  `design/gate-baseline.json`) — the field precedent for D6.

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/onboard/templates.rs:196-216` — default
  `team.claude` role table seeded into new hosts; `team.pi` derives from it.
- `packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:70,125,151` — role
  names are derived from `team.<runtime>` keys, so a new role is legal by config.
- `packages/bee/agents/*.md.tmpl` plus `onboard/templates.rs:423`
  `AGENT_ROLES_BY_NAME` and `drivers/prompt.rs:41-49` — the rendered-agent path.
- `onboard/plan.rs:157,169,873-894` — expertise copy into `.bee/expertise/`.
- `onboard/render.rs:377-382` and `onboard/skills.rs` — skills ship by the
  `bee-` prefix; removal refuses non-`bee-` names (`skills.rs:882`).

### Established Patterns

- Onboarding offer-only steps (`NO_TEST_VERIFICATION_OFFER`,
  `docs/knowledge/areas/onboarding/host-project-artifacts.md`) — onboarding
  offers, never builds on its own.
- Cap proof vocabulary and checks: `verbs/cells/finish_support.rs:92-99,332`.

### Integration Points

- `scripts/release.sh`, `.claude-plugin/plugin.json`, `.codex-plugin/plugin.json`,
  `devtools/release_manifest.rs:74-90` `INVENTORY_ROOTS` — what ships.
- `devtools/skill_trees.rs` — rendered plugin skill trees.

## Canonical References

- `docs/discovery/spec-drops/sup-20261024-794d.md` (main checkout) — the spec
  drop with the feasibility check and owner input.
- `/mnt/data/Projects/goglbe/design-os` @ a7ad5f1 — upstream: `LICENSE`,
  `THIRD_PARTY_LICENSES.md`, `knowledge/index.json`, `skills/design-os-designer/`.

## Outstanding Questions

### Answered In Planning

- [x] Upstream `THIRD_PARTY_LICENSES.md` says ease-design is MIT while
  `LICENSE` is PolyForm Shield 1.0.0 — bee carries both texts verbatim in
  `packages/bee/design/`; the mismatch is named to the owner.
- [x] Pack boundaries — mapped in `scripts/sync-design-os.sh`, exclusions with
  reasons in `packages/bee/design/UPSTREAM.json`.
- [x] Bundle size — one copy in `packages/bee/design/` (about 1.6 MB), outside
  every skill tree (decision f4bbb166).
- [x] How the cap detects a UI cell and reads the baseline (D6) — contract
  decision 053a14a4.

## Rejected Ideas

- Porting the `ui` linters to Rust — rejected by D5.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and the questions above.
