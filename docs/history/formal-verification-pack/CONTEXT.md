# formal-verification-pack — Context

**Feature slug:** formal-verification-pack
**Date:** 2026-11-14
**Shaping session:** complete
**Scope:** Quick
**Domain types:** RUN

## Feature Boundary

The owner asked to bring pstack's `create-formal-verification-skill`
and `statechart` in. Research
(`docs/history/research/pstack-formal-verification-port.md`) found a
third skill they need, `maintain-formal-verification-skill`, and an
earlier decision (`d94edfa6`) that keeps `statechart` in the
skillstation hub. The owner picked the hub. The two formal skills are
now hub drafts, tracked upstream from `cursor/plugins`. This feature
only changes bee's planning playbooks so they point at the two new
skills by name, the same way they already point at `statechart`.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `create-formal-verification-skill` and `maintain-formal-verification-skill` live in the skillstation hub beside `statechart`. bee vendors none of the three. Its feature and bugfix playbooks point at them by name, and skip the step when the host lacks the skill. Decision `e9c80ef2` (supersedes `7b51f877`, `a259dd63`, `ec8a3076`, `a114ad76`). | Keeps `d94edfa6`; no TypeScript runner to resync by hand. |
| D2 | Source manifest: `https://github.com/cursor/plugins`, `pstack/`, ref `main`, read at `3dbd22ff12742fb28c9700b9760baa29ed41dc13`, MIT (c) 2026 Lauren Tan. The hub tracks the skills from upstream `main`. | Port-protocol provenance. |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/onboarding/onboarding-design-packs-the-design-role-and-the-design-gate.md | suspect | The pack route the owner did not pick. |
| docs/knowledge/areas/verify-pipeline/overview.md | suspect | Proof discipline the pointers must not weaken. |

### Code

| Path | What it decides |
|------|-----------------|
| `skills/bee-planning/playbooks/feature.md` | Step 6 points at `statechart` for an event-order flow. |
| `skills/bee-planning/playbooks/bugfix.md` | An event-order symptom takes the step 6 model. |

## Canonical References

- `docs/history/research/pstack-formal-verification-port.md` — the port research and dependency matrix.
