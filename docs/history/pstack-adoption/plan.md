---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset>
---

# Plan: pstack Adoption

Mode: `standard` — 3 risk flags: public-contracts, multi-domain, covered-contract-change
Why this is the least workflow that protects the work: one enum value with a
pinned refusal message is the only code, and everything else is prose with an
existing single home. The real exposure is drift between the four documents that
name the enum, which one fence-style parity test pins.

This is revision 3. Revision 2 folded six hat-wave blockers. Revision 3 folds
one more finding, from outside the wave: `docs/history/research/pstack-xia.md`
already studied pstack from its own tree at pinned commit `b9ddc83`, and lists
verbatim playbook todo-lists under *What must not be ported*. D1 was superseded
accordingly (`cc87b3c4` → `132551fb`): **the plan cites a playbook, it does not
copy it.** The user was told the objection is advice, not a logged decision, and
chose to keep building with the softening.

## Requirements (from CONTEXT.md)

- **D1** (as superseded by `132551fb`) — A class playbook is a named step list
  the plan **cites by name and anchor**; the steps live in one home and are read
  there, never transcribed into the plan. A skipped step stays visible and
  carries its recorded reason. Never a refusal.
- **D2** — `perf` becomes an eighth `class` enum value with its own playbook:
  baseline, change, re-measure. "It feels faster" is not a result. The enum
  change is a public-contract change **and carries a migration note** — stated
  flatly in D2, so phase 1 writes one unconditionally.
- **D3** — The investigation route is the existing `research` class. No new
  route, no new lane.
- **D4** — The herding dispatch role refuses a candidate whose CoS is not
  checkable, and names why in its skip line.
- **D5** — A review report carries its dismissed findings, each with the reason
  it was dismissed.

## Load-bearing claims

Labels are `read`, `ran`, or `guessed`. Evidence is a verbatim byte substring of
the anchored line(s); multi-line evidence joins lines with `" / "`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The `class` enum is a 7-element Rust constant whose arity is in its type, so D2 changes the type too | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:287-288` | `pub(crate) const ROUTE_CLASS_VALUES: [&str; 7] = / ["feature", "bugfix", "docs", "refactor", "research", "release", "spike"];` |
| 2 | A test asserts the refusal message listing the class values, so D2 is a covered-contract change | read | `packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs:1725` | `"route --set: invalid flag(s): --class \"nope\" (must be one of feature, bugfix, docs"` |
| 3 | No code branches on `route.class` — the check that looked like one reads `lane` | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:827` | `let lane_class = js_disp_opt(route_object.get("lane"));` |
| 4 | ...and that check's body tests only `docs` and `tiny`, never a class value | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:588-589` | `if lane.is_empty() \|\| lane == "docs" { / return false;` |
| 5 | BUT the class vocabulary IS read elsewhere: a lane record's `mode` field usually carries a workflow class, and `close.rs` guards against misreading it as a lane | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:395-397` | `` standard/high-risk). `mode` usually carries the WORKFLOW class instead / (`ROUTE_CLASS_VALUES`, same file:287-288 — "feature" is the live / shape's constant value there), which is not a lane at all; `` |
| 6 | The same leak is documented as MEASURED on the live store in the uat path | read | `packages/bee-rs/crates/bee/src/uat.rs:139` | `` /// `mode` carries the WORKFLOW vocabulary (`ROUTE_CLASS_VALUES`: `` |
| 7 | Adding `perf` is safe against that leak because `perf` is not a lane value — the actual safety argument for the contract change | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:289-290` | `const ROUTE_LANE_VALUES: [&str; 6] = / ["docs", "tiny", "small", "spike", "standard", "high-risk"];` |
| 8 | `close.rs` had to DUPLICATE the lane list because the const is module-private — the exact trap this feature's parity test must not fall into | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:403-406` | `` /// without importing it: that const is module-private there, and this / /// cell's file scope does not extend to changing its visibility. / const FEATURE_ROUTE_LANE_CLASSES: [&str; 6] = ["docs", "tiny", "small", "spike", "standard", "high-risk"]; `` |
| 9 | The enum is named verbatim in FOUR source documents, not one | read | `skills/bee-hive/references/scout-and-ticks.md:34` | `` - `class` ∈ `feature`, `bugfix`, `docs`, `refactor`, `research`, `release`, `spike` `` |
| 10 | (second site) | read | `docs/product-description/goal.md:48` | `` - Route vocabularies: class `feature\|bugfix\|docs\|refactor\|research\|release\|spike`; `` |
| 11 | (third site) | read | `docs/product-description/lifecycle/planning.md:35` | `` `class` from `feature\|bugfix\|docs\|refactor\|research\|release\|spike`; `` |
| 12 | (fourth site — and it RUNS the refusal as a verification row, so it is executable truth, not prose) | read | `docs/product-description/verification/lifecycle.md:114` | `` `class` is a closed enum: `feature bugfix docs refactor research release spike` `` |
| 13 | No test reads `docs/product-description/`, so three of the four sites are unguarded today | ran | `rg -ln 'product-description\|product_description' packages/bee-rs/crates/bee/tests/` | (no output — zero matching files) |
| 14 | Every skill file has generated copies under five plugin trees, so no two skill-touching cells are truly disjoint | ran | `ls .claude-plugin/skills/bee-herding/references/role-dispatch.md` | `.claude-plugin/skills/bee-herding/references/role-dispatch.md` |
| 15 | ...and the regen chain rewrites ONE shared manifest on every run, which serializes skill-touching cells | read | `packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:94` | `pub(crate) const MANIFEST_REL: &str = "docs/history/codex-harness-hardening/release-manifest.json";` |
| 16 | Today's bugfix proof rule already exists as one self-contained craft line that disclaims flags | read | `skills/bee-swarming/references/worker-details.md:33-35` | `the gap, and for a bugfix watch the repro fail before the fix — / red-before-green is craft, applied by judgment and enforced by review, / not by flags.` |
| 17 | `bee-planning` records the route at the step a playbook would be cited at | read | `skills/bee-planning/SKILL.md:42` | `` Record: `bee route --set --class <c> --lane <l> --flags <f> --files <n>`; `` |
| 18 | `bee-planning` already loads `planning-reference.md` at the plan-drafting step, so that file is a zero-cost home a citation resolves against with no extra read | read | `skills/bee-planning/SKILL.md:77` | `` | `standard`/`high-risk` | `docs/history/<feature>/plan.md` — `references/planning-reference.md` `` |
| 19 | `bee-hive`'s references are reached through a fixed router table, so a new file there costs a row every session pays for | read | `skills/bee-hive/SKILL.md:118-119` | `` Every heading quoted in this body resolves somewhere in `references/`; the / row that names the contract is the one to open. `` |
| 20 | The single-home-plus-pointer pattern the approach copies has an existing home | read | `skills/bee-hive/references/gates-and-delegation.md:154` | `**This section is the single home for the blind-lane PROCEDURE**` |
| 21 | The dispatch role already reads `title+cos` in its own pass, so D4 extends an existing read | read | `skills/bee-herding/references/role-dispatch.md:273` | `**Key 2 — your own reading.** Read the candidate's full title+cos yourself, in` |
| 22 | The review summary line is a fixed one-line DISPLAY shape | read | `skills/bee-reviewing/SKILL.md:72` | `<N> finding(s) — P1 <a>, P2 <b>, P3 <c> · axis: spec <s>, standards <t>.` |
| 23 | ...but the dropping happens in the reviewer's instrument, which is where D5 must also land or there is no dismissed bucket to display | read | `.bee/expertise/review.md:232-233` | `Reproduce or trace every suspected defect before you file it. Run the / failing input if you can;` |
| 24 | Cells carry a SECOND, overlapping class taxonomy the playbooks must cite rather than duplicate | read | `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:123` | `["formatting", "bugfix", "behavior", "api", "security", "migration", "refactor", "test"];` |
| 25 | The generated CLI registry carries NO enum list for `class`, so D2 has no registry regen dependency | read | `packages/bee-rs/crates/bee/src/generated/registry_payload.json` | `"class": {"type": "string", "description": "Route class."}` |
| 26 | The red found at base was UNTRACKED in the main checkout only — the worktree at the branch commit never had the file, so it never reached this feature or CI | ran | `ls docs/specs/pstack.md` inside `beehive--wt--pstack-adoption` | `ls: cannot access 'docs/specs/pstack.md': No such file or directory` |
| 27 | Class validation runs on `--set` only, so an older bee meeting `class: perf` on a read degrades safely | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:268` | `` // `--show` is a pure read through resolveMutationTarget. `--set` runs the `` |
| 28 | 37 backlog PBIs sit `proposed` — the population D4's refusal will meet | ran | `python3` over `.bee/backlog.jsonl`, folding events by id | `Counter({'done': 127, 'proposed': 37, 'declined': 36, 'parked': 2})` |
| 29 | The existing doc-fence test is a pure filesystem test with no crate import, which is the pattern the parity test must follow | read | `packages/bee-rs/crates/bee/tests/specs_fence.rs:17-18` | `use std::collections::BTreeMap; / use std::path::{Path, PathBuf};` |

## Discovery

Inspected the `class` enum end to end. Revision 1 concluded it had zero
consumers. That was too strong, and the hat wave caught it. The precise truth,
now rows 3-8: **no code branches on `route.class`** — but the class *vocabulary*
leaks into a lane record's `mode` field, and `close.rs:395-397` exists to stop
that from being misread as a lane. `close.rs:403-406` solved its own version of
this by duplicating a module-private const, which is exactly the move this
feature exists to stop.

That changes the safety argument for D2. Adding `perf` is safe not because
nothing reads the class, but because `perf` is absent from `ROUTE_LANE_VALUES`
(row 7), so a `mode: perf` record can never be mistaken for a lane. `docs` and
`spike` already sit in both vocabularies; `perf` adds no new collision.

Three more findings changed the shape:

- The enum is named in **four** source documents, not one, and no test guards
  three of them (rows 9-13). A parity test pinned only to `scout-and-ticks.md`
  would buy false confidence.
- `ROUTE_CLASS_VALUES` is `pub(crate)` (row 1), so an integration test under
  `crates/bee/tests/` cannot import it. The parity test therefore reads
  `workflows.rs` as text, the way `specs_fence.rs` already reads the tree (row
  29). Re-declaring the list in the test would reproduce the `close.rs` trap.
- Every skill file has five generated copies and the regen chain rewrites one
  shared manifest (rows 14-15). **Skill-touching cells are therefore serial**,
  not concurrent — revision 1 claimed otherwise on an incomplete file list.

Fourth finding, from running the declared suite as a base check: the tree is
already RED, on a file this feature did not author (row 26).

## Approach

**Recommended path.** The playbooks become one `## Class playbooks` section
inside `skills/bee-planning/references/planning-reference.md` — no new file, no
new router row (rows 18, 19; D1, D2, D3). `scout-and-ticks.md:34` gains a
one-line pointer beside its enum list. The pointer between the bugfix playbook
and today's craft rule runs *from the playbook to*
`bee-swarming/references/worker-details.md`, keeping that rule's existing home
and sparing a cold execution worker a planning reference (row 16). D2's enum
value is pinned across all four doc sites by one new text-reading fence test.
D5 lands in two places — the instrument that decides to drop a finding
(`.bee/expertise/review.md`) and the report shape that displays it
(`skills/bee-reviewing/SKILL.md`) — because a report cannot show a bucket
nothing was told to keep (rows 22, 23).

**Rejected alternatives.**
- A new `skills/bee-hive/references/playbooks.md` — rejected: unreachable by
  bee-hive's own stated routing method until it gets a router row every session
  pays for (row 19), and `planning-reference.md:6` ("Fold, don't fan out")
  refuses the fan-out.
- Rendering playbooks from the CLI — rejected: a Rust surface and a regen
  dependency for text a skill can hold.
- Repointing `worker-details.md:33-35` at the playbook — rejected: no locked
  decision requires it, and it makes an execution worker load a planning file
  for a one-sentence execution rule. Reverse the pointer instead.
- A parity test that re-declares the class list — rejected by row 8: that is
  the `close.rs` duplication trap, in the feature built to remove duplication.
- Making `ROUTE_CLASS_VALUES` `pub` so the test can import it — rejected:
  widening visibility to serve a test, when reading the file as text costs
  nothing and matches the existing fence pattern (row 29).
- Folding `perf` into `bugfix` — rejected by D2, the user's call.

**Risk map.**

| Component | Risk | Proof needed |
|---|---|---|
| `ROUTE_CLASS_VALUES` arity change | LOW | `cargo test` — the arity is in the type, so a miscount will not compile |
| The asserted refusal message | MEDIUM | the pinned assertion at `tests.rs:1725` is updated deliberately; a deleted assertion is proof-weakening, not a fix |
| The `mode`-carries-a-class leak | MEDIUM | row 7 — assert in the parity test that no class value is also a lane value, so the next added class cannot silently collide |
| Four-site doc parity | HIGH | the new fence test asserts all four sites name exactly the eight values; three are unguarded today (row 13) |
| Generated copies drifting from source | MEDIUM | `bee dev regen` runs once per cell, and the parity test reads SOURCE files only — the rendered copies are regen's output, not a second truth |
| D4's "checkable" judgment | MEDIUM | a reading rule, not a regex — the prose carries a worked checkable/uncheckable pair |
| D5 with no keep-instruction | HIGH | row 23 — the instrument change lands with the display change, in the same cell, or D5 is decorative |
| Route class vs cell `change_class` drift | LOW | the playbook section names which taxonomy it binds to, in one line |

## Shape

Two phases, **serial** — rows 14-15 mean any two skill-touching cells share the
regen chain's output, so they do not run concurrently. (Revision 2's phase 0 is
gone: the red it fixed was an untracked file in the main checkout, never in this
branch — row 26. Cell `psa-0` was dropped with that reason and the file was
moved as main-checkout housekeeping.)

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | `ROUTE_CLASS_VALUES` 7→8 with `perf`; the pinned refusal at `tests.rs:1725`; the four doc sites (rows 9-12); the new parity fence test; **the migration note D2 requires** | The only Rust and the only covered-contract change. Demoable alone | `--class perf` accepted; `--class nope` refuses naming `perf`; the parity test pins four docs and asserts no class is also a lane value | the `perf` playbook body has a legal class to bind to |
| 2 | Three serial cells: (a) the `## Class playbooks` section in `planning-reference.md` for perf/bugfix/refactor/research + bee-planning's CITATION step at `SKILL.md:42` + the `scout-and-ticks.md:34` pointer (D1 as superseded, D2, D3); (b) D4 in `skills/bee-herding/references/role-dispatch.md`; (c) D5 in `.bee/expertise/review.md` AND `skills/bee-reviewing/SKILL.md` + `skills/bee-reviewing/references/reviewing-reference.md` | Each is single-file-group prose; serial only because of the shared regen chain, not because of a real dependency | a plan for each class names its playbook and anchor, and a reader following the anchor lands on the steps; a dry-run dispatch skips a vague-CoS candidate naming why; a review report shows a dismissed finding with its reason | — |

Cell (a) needs phase 1 only so the `perf` body names a legal class. Cells (b)
and (c) need nothing from phase 1 and may take any order among themselves.

## Test matrix

The triad, at its smallest demonstrating size. Each cell's writer judges
existing coverage first (`.bee/expertise/tests.md`) and authors only the gap.

- **Happy path** — `bee route --set --class perf --lane standard --flags "" --files 1`
  is accepted and reads back `class=perf`. Rust test beside the existing route
  tests in `state_group/tests.rs`.
- **Edge** — a NEW test file under `crates/bee/tests/`, following
  `specs_fence.rs`'s pure-filesystem shape (row 29). It reads
  `workflows.rs` as TEXT to extract `ROUTE_CLASS_VALUES` and `ROUTE_LANE_VALUES`
  — never re-declaring either (row 8) — then asserts (i) all four source doc
  sites (rows 9-12) name exactly the class values, and (ii) `perf` is not a lane
  value, so the `mode` leak (rows 5-7) stays harmless. Rendered plugin copies
  are regen output and are not read.
- **Error** — `--class nope` still refuses with a typed message, and that
  message now lists `perf`. The pinned assertion at `tests.rs:1725` is updated,
  never deleted.

Phase 2's cells change prose only and carry parity/pointer checks as proof, not
Rust tests; `commands.test` still runs to prove nothing else moved.

## Open Questions

(none. The older-bee degradation question is answered by row 27 — validation
runs on `--set` only, so a read of an unknown class prints it and never
validates it. D2's migration note is written regardless, because D2 says so.)

## Out of scope

- **An authoring-time checkable-CoS check.** D4 fires at dispatch time only, so
  a vague PBI is accepted at filing and refused later, with the person who could
  reword it out of the loop. Of the 37 `proposed` PBIs (row 28), roughly 4 refuse
  outright and up to 8 under a strict reading. CONTEXT.md's boundary ends at the
  four named surfaces, so this is a known gap filed to the backlog, not silently
  absorbed.
- **Making `research` routes actually read-only.** D3 reuses the `research`
  class, but nothing in the repo makes such a route refuse a source edit — the
  playbook step is new law, not a promotion of existing craft the way the bugfix
  step is (row 16). The playbook states the step; enforcing it is separate work.
- `/automate-me`, `/teach`, `/bro`, Graphite stacking, and the Benny automation
  pack — skipped with reasons in `docs/history/research/pstack-distill.md`.
- **The four gaps `pstack-xia.md` ranks above these.** Code-shape blindness,
  proof with no expiry or strength vocabulary, no rule for a quietly-dead
  worker, and questions that reach the human when running something would have
  answered them. That study is better sourced than this feature's own; its
  items are separate work and are not started here.
- Playbooks for `feature`, `docs`, `release`, and `spike`. Four classes get
  bodies; the rest keep today's behavior. Adding them later is additive.
- Any change to gates, worktrees, or the cell lifecycle.
