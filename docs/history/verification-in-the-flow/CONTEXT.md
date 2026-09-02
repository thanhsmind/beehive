# Verification In The Flow — Context

**Feature slug:** verification-in-the-flow
**Date:** 2026-09-02
**Shaping session:** complete (Lock only — every decision settled with the user in session; nothing originated here)
**Scope:** Standard
**Domain types:** RUN | READ | ORGANIZE

## Feature Boundary

A project-local verification skill stops being a thing the human remembers to
name and becomes part of bee's own flow. It gains a fixed name (`verify-app`),
a flattened source root (`.bee/verify/`), an onboard branch that routes to
generate-or-update, a read-first position for its feature map at shaping and
planning, and a home for its drive on the cap proof line.

It ends there. It does NOT add a door, a proof tier, a config key, a CLI verb,
a lane, or a route class. It does not change what any existing door checks. It
does not touch gates, worktrees, or the cell lifecycle. `bee onboard` still
only OFFERS — generating and updating stay agent work, run through the two
skills that already own them.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. IDs are bee decision-log ids (search with
`bee decisions search --feature verification-in-the-flow`).

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 · `d0e3c3a0` | The generated verification skill carries a **fixed name in every repo — `verify-app`** — never a per-project `verify-<app>`. Content differs per project; the name never does. The name is **NOT** `bee-` prefixed. | A fixed name lets every bee surface name the skill in literal text instead of a template with a hole, so a cold worker with no preamble still knows what to load, and it gives D3's onboard branch a one-path existence check. The `bee-` prefix is refused **on evidence**: `onboard/skills.rs:954` (`foreign_bee_skill_in_target_is_removed_but_non_bee_is_untouchable`) proves the `bee-*` skill sync emits `remove_skill` for any bee-named target directory absent from bee's own source tree — a `bee-verify-app` would be deleted from every runtime skill home on the next `bee onboard --apply`. `plan.rs:359` states the ownership axiom behind that: bee MINTS `bee-*` names and nobody else does. |
| ~~D2~~ · `28140420` | ~~The source root flattens to `.bee/verify/` itself.~~ **RETIRED at the plan gate by D8 (`9f4f90f0`).** | Retired on the unanimous finding of all five hat seats: D1's fixed name already yields one constant path, so flattening changed no observable behavior while rewriting a deliberately-commented renderer design, pinning a one-product limit into the file format, and stranding a phantom `features/` skill under any older binary. |
| **D8** · `9f4f90f0` | The source stays **NESTED at `.bee/verify/verify-app/`** — `SKILL.md`, `features/` and the control script sit in that directory, and the renderer's existing subdirectory walk (`plan.rs:481-575`) is unchanged. D3's branch reads `.bee/verify/verify-app/SKILL.md`. | The user's choice at the plan gate, after both shapes were presented with their costs. It keeps the source root that `verification-ships-to-hosts` D3 established — the reason the skill is not authored straight into `.claude/skills/` is unchanged: one runtime home as source makes the skill invisible to Codex and opencode agents in the same repo. |
| D3 · `65592f3f` | `bee onboard` becomes a **two-way branch on the verification skill's existence**: absent → a notice pointing at `bee-verifying` (generate); present → a notice pointing at `bee-verify-upkeep` (update). Onboard still only OFFERS; it never generates. | `verification-ships-to-hosts` D5 holds — the binary cannot run an agent skill. **Rationale corrected at the plan step — see the correction note below.** |
| D4 · `c93a6948` | The **feature map becomes a READ-FIRST artifact**, named in the AGENTS.md read-first rule beside `docs/knowledge/`. Two tiers: `features/README.md` (the index) is read at **shaping** to decide new-feature-vs-mapped-feature; the ONE matching feature file is read at **planning**, where its "How to get to it" seeds the cell file list and its "Gotchas" seed the plan risks. The execution worker carries that same feature file in its brief. | The map is compact navigation over the whole product. Loaded only at the proof door it is dead weight — the agent has already chosen the wrong surface by then. bee's doctrine already says the state layer is read first and synced when behavior changes; the feature map is a second state layer that rule never named. The two-tier load is bee's existing skill-reference pattern (index always, body on demand), so it needs no new mechanism. |
| D5 · `036e8a79` | The product drive rides the **cap proof line**, not the test command. The AGENTS.md proof-by-change-type list gains ONE row: a user-facing surface is proven by driving its mapped feature with evidence attached, recorded as a **`green:live`** result. No new door, no new proof tier. | `verification-ships-to-hosts` D4 holds. The machinery already exists: `proof-strength-and-expiry` (`cb7b14b7`) pinned `green:live` to mean exactly "the real product was driven and its result inspected", which IS the drive, and AGENTS.md already makes the agent own proof scope by change type. One row in an existing list. |
| D6 · `2a8eac15` | The generated drive command does **NOT** compose into `commands.test`. `commands.test` stays pure tests. **Supersedes `verification-ships-to-hosts` D2 (`d79baa77`).** | A test asks whether the code stayed correct; a verification asks whether the product works for a user. One command for both makes a red result ambiguous — a broken function and an app that failed to launch report identically — and forces CI to carry a live app harness. `commands.verify` was retired in 2.1.0 for exactly this class of confusion, and `ci-owned-verify` D5/D6 made `commands.test` the fast impacted subset on purpose. The D2 CONTEXT recorded that composition partly reverses that and that the agent raised it before the pick; the user has now reversed the pick. D5 gives the drive its own home, so nothing is lost. |
| D7 · `2effbe54` | bee **regenerates its own `.claude/skills/verify-bee`** into this shape: source at `.bee/verify/`, rendered as `verify-app` into all three runtime skill homes. | `verification-ships-to-hosts` D3 left this open as a discretion note. The answer is now forced: `verify-bee` sits hand-written in `.claude/skills/` only, so a Codex or opencode agent in bee's own repo cannot see it. The worked example is more useful in the shape every host will actually get. |

**Correction to D3's rationale (2026-09-02, plan step).** The rationale first
recorded here claimed `onboard/notices.rs:203` nests the verification offer
inside `if commands.contains_key("verify")`, so no post-2.1.0 repo ever sees
it. **That is false as of today's code.** `stale_advisor_notices`
(`notices.rs:199-218`) reads `if verify-key { retirement warning } else if
!has_test { NO_TEST_VERIFICATION_OFFER }`, and its own comment states the offer
arm "is the only arm that reaches a repo onboarded after 2.1.0". The defect the
D5 correction named was FIXED by `verification-ships-to-hosts` itself; this
CONTEXT repeated the pre-fix description.

The decision is unchanged, and the real gap it closes is **larger** than the one
first cited: the offer is gated on `!has_test`, so a repo that already declares
a `commands.test` — most repos, bee's own included — receives **no verification
notice at all**, and no repo of any kind receives an upkeep pointer, because none
exists. D3's branch is therefore not a nesting repair. It is a new condition:
the notice keys off whether a verification skill EXISTS, not off whether a test
command is absent. `declares_test` (`notices.rs:224`) stops being the offer's
gate. Recorded because the original claim quoted a defect report instead of
reading the current call site — the same mistake, in the same file, that
`verification-ships-to-hosts` recorded against its own D5.

### Agent's Discretion

- The exact wording of every notice, rule line, and skill-body edit, bounded by
  D1-D7.
- Where inside `AGENTS.block.md` the read-first mention (D4) and the proof row
  (D5) sit, provided each fact keeps ONE home and every other surface points at
  it.
- Whether the preamble gains a verification line at all, and its wording if so.
  D1's fixed name means the preamble is no longer needed to carry the NAME; at
  most it carries presence. A line that only restates a constant is not worth
  its bytes — planning decides on evidence.
- How `.bee/verify/`'s flattening (D2) is migrated for a repo that already
  carries the nested shape, if any exists. bee's own is the only known instance
  and D7 covers it.
- The order of the two tiers' load points inside `bee-shaping` and
  `bee-planning`, and whether the worker's brief carries the feature file by
  path or by content.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| `verify-app` | The one fixed skill name a generated verification skill carries, in every repo, forever. |
| Feature map | `.bee/verify/features/` — a README index plus one file per user-facing feature, each answering what it is, how a user reaches it, how to drive it, and its gotchas. |
| Read-first artifact | A record the agent loads BEFORE deciding what to build, not to prove what it built. `docs/knowledge/` is one; the feature map becomes the second. |
| Drive | Running the real product through the control script the way a user would, and inspecting the result. Recorded as `green:live`. |
| Test | The declared `commands.test` run. Deterministic, fast, CI-owned. A different question from a drive. |

## Specific Ideas And References

- `docs/history/verification-ships-to-hosts/CONTEXT.md` — the parent feature.
  D1-D5 there still hold except D2, which D6 here supersedes. Its "Consequence
  recorded against D2" paragraph is the written record that this reversal was
  foreseen.
- `docs/history/research/pstack-xia.md`, `docs/history/research/pstack-distill.md`
  — the study this feature's shape came from. The Feature Map's value as
  "materialized memory" read up front is the source of D4.
- pstack's own answer is that the human types the skill name in every prompt.
  D1 + D3 are bee's answer: bee has a planner, a store and a renderer, so the
  name is a constant and the routing is machinery.

## Existing Code Context

From the session's scout. Downstream agents read these before planning.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/onboard/plan.rs:349-450` — the whole
  `.bee/verify/` render path: `verify_source_root`, `verify_render_targets`,
  the root preflight, and `verify_skill_drifted`. D2 changes what it walks, not
  how it renders. Its "Render, never prune" design is load-bearing for D1.
- `packages/bee-rs/crates/bee/src/onboard/skills.rs:954` — the prune test that
  is D1's evidence. It must stay green and unchanged.
- `packages/bee-rs/crates/bee/src/onboard/notices.rs:188-221` — the stale-key
  notice arm holding the misplaced verification offer. D3's branch lands here.
  `NO_TEST_VERIFICATION_OFFER` is the existing constant and already names
  `bee-verifying`.
- `packages/bee-rs/crates/bee/src/onboard/templates.rs:317-321` —
  `REPO_SKILL_TARGETS`, the three runtime homes. Never hand-enumerate them.
- `skills/bee-verifying/SKILL.md` — the generator. Step 3 seeds the feature map
  (D4's producer); step 5 is the compose-into-`commands.test` step that D6
  removes; the `verify-<app>` naming runs through the whole body (D1) and the
  `.bee/verify/verify-<app>/` path likewise (D2).
- `skills/bee-verify-upkeep/SKILL.md:49` — step 0, "Locate the target". D1+D2
  delete this step: the path is a constant.
- `packages/bee/AGENTS.block.md` — the rendered source for `AGENTS.md`. Holds
  both the read-first rule (D4) and the proof-by-change-type list (D5).
- `.claude/skills/verify-bee/` — the hand-written instance D7 regenerates. Its
  `features/` directory is a working example of the map shape.

### Established Patterns

- Single home plus pointers — every playbook and procedure in this repo. D4's
  two-tier load and D5's single row both follow it.
- Index-always, body-on-demand — every bee skill's reference table. D4 is that
  pattern applied to a host artifact.
- Text-reading parity tests (`tests/route_class_parity.rs`,
  `tests/specs_fence.rs`) — the shape for asserting a constant name appears
  where the doctrine says it does.

### Known Constraints

- `AGENTS.md` renders from `packages/bee/AGENTS.block.md`; both carry rule
  markers and both move together through `bee dev regen`.
- Every skill file has generated copies under the plugin trees and the regen
  chain rewrites one shared release manifest — **skill-touching cells are
  SERIAL, never concurrent** (`pstack-adoption` plan rows 14-15).
- The rendered skill copies have their executable bit stripped, so every
  invocation a skill body shows is written `bash <path> …`.
- A repo with two separate products cannot have two `verify-app`. The feature
  map indexes multiple features inside one skill; a genuine two-product host is
  a later decision, recorded here as a known limit, not a blocker.

## Outstanding Questions

### Deferred To Planning

- [ ] Does any host repo outside this one already carry `.bee/verify/<name>/`?
      If none, D2 needs no migration path at all — only D7's regeneration.
- [ ] Does the preamble earn a verification line once the name is a constant?
      Decide from what a cold worker actually lacks, not from symmetry with
      other preamble fields.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable and are bee
decision-log ids. Planning reads the locked decisions, the code context, and
the deferred-to-planning questions. D6 retires `verification-ships-to-hosts`
D2 (`d79baa77`) in the active decision set — cite D6, never the retired one.
