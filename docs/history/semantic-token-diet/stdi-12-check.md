# stdi-12 — check of stdi-11 (planning split)

Round 2. Checked both `cell: stdi-11` commits, `7c19ea731` (the split) and `fd64f2fe3` (the timing-rule restore), against CONTEXT.md D1, D2 and D5 and audit `reports/D-planning.md`.
Line anchors point at the tree at `2d6de1cf9`. "old" means `skills/bee-planning/references/planning-reference.md` at `7c19ea731^`.
Since the split, `G` gained two lines at `:108`, so every `G` anchor past that line moved by two. `plan-artifact.md` gained five lines at `:280`.
`R` means `skills/bee-planning`. `G` means `skills/bee-hive/references/gates-and-delegation.md`.

## Truths

- FAIL — `rg planning-reference` outside docs/history, docs/decisions, docs/discovery and CREATION-LOG files returns 4 hits in 3 files. All of them name the old file as history ("formerly", "moved out of"), not as a live pointer. The hits: `R/references/provenance.md:6`, `docs/knowledge/areas/verify-pipeline/skill-reference-pointer-integrity.md:130`, `docs/knowledge/patterns/20260728-a-derivation-the-tooling-computes-but-doctrine-forbids-where-it-is-needed.md:11` and `:30`. No live pointer is left. The literal truth still fails. See D-b.
- PASS — every section of the old file has exactly one new home. No heading appears in both new files. Old sections 1-14 map one to one below. "Tests ride the cell" (old `:640-653`) was deleted, not moved, as D2 and audit F1 order. Its live rule is the tester-cell pair at `R/SKILL.md:165-171`.
- PASS — no do-not-touch line from `reports/D-planning.md` section 4 was removed. "One reality touch" and "Existence is not evidence": `R/SKILL.md:78`, `:81`. "spends their approval on your homework": `R/SKILL.md:129-130`. Gate wording "Work shape is ready. Approve before current-work preparation?": `R/SKILL.md:135`. Match rule and "load-bearing claims table": `R/references/plan-artifact.md:129`, `:149`. "Carry the hits, not a line range" with its drift why: `R/references/cells.md:67-74`. Risk-map one-fact rule: `R/references/plan-artifact.md:204`. "Zero load-bearing claims is a real answer": `R/references/cells.md:255`. "Adapted from pstack": `R/references/design-sketch.md:11`. Greenfield did not move; it stays at `R/references/cells.md:222`.

## Section homes

- Artifact fan-out (old `:6`) → `R/references/plan-artifact.md:6`, unchanged.
- Artifact: plan.md and the claims table (old `:23`, `:129`) → `R/references/plan-artifact.md:23`, `:129`.
- Artifact: approach.md (old `:190`) → `R/references/plan-artifact.md:183`, unchanged.
- Phase plan vs epic map (old `:220`) → `R/references/plan-artifact.md:213`, unchanged.
- Playbooks (old `:229`) → `R/references/cells.md:6`.
- Cell quality rules (old `:270`) → `R/references/cells.md:46`.
- Example cell JSON (old `:341`) → `R/references/cells.md:115`, unchanged.
- Consolidation cell shape (old `:373`) → `R/references/plan-artifact.md:222`.
- Pre-flight before cells add (old `:426`) → `R/references/cells.md:147`.
- Test scoping (old `:476`) → `R/references/cells.md:193`.
- Greenfield init lane (old `:503`) → `R/references/cells.md:222`, unchanged.
- Tiny/small merged gate (old `:514`) → `R/references/cells.md:233`, unchanged.
- Plan check — the hat wave (old `:542`) → `R/references/plan-artifact.md:271`.
- Tests ride the cell (old `:640`) → deleted (F1). Rule home `R/SKILL.md:165-171`.

## Moved or cut rules

- "writer judges existing coverage" became "the tester cell judges" → `R/references/plan-artifact.md:104`, `R/references/edge-dimensions.md:11` (F1).
- edge-dimensions pointer to "Tests ride the cell" → `R/SKILL.md` ("Prep") at `R/references/edge-dimensions.md:7` (F1).
- design-sketch gate name → "the shape gate" at `R/references/design-sketch.md:34` (F11).
- `bee gate --preview` contract (old `:172-177`) → `R/SKILL.md:132` (F5).
- Role vocabulary and fall-through in pre-flight item 2 (old `:437-441`) → `R/references/cells.md:104-110` (rule 10), pointer at `R/references/cells.md:159` (F6).
- Decision ids in Playbooks and rules 4 and 13 (old `:253-254`, `:284`, `:329`) → `R/references/provenance.md:6-8` (F4).
- `commands.verify` is retired (old `:478`) → `R/references/provenance.md:16`; runtime warning `packages/bee-rs/crates/bee/src/onboard/templates.rs:383` (F4).
- Incident anecdote in rule 7 (old `:301-303`) → cut (F9). The drift why stays at `R/references/cells.md:72-73`.
- Consolidation cell when and why (old `:375-377`) → `R/SKILL.md:154-158` (F8).
- Hat wave history: plan-reviewer absorbed, dispatch kind changed, no wave on clear/tiny ids (old `:544-546`, `:558-563`, `:576-577`) → `R/references/provenance.md:11-13` (F4).
- Hat wave seats and advisor kind (old `:558-563`) → `G:211-212`, `G:296` (F3).
- SMALLER PATH one home (old `:565-568`) → `R/SKILL.md:100-103`, `G:220` (F3).
- Two mandates, never merged (old `:570-574`) → `G:239-243` (F3).
- No wave on a clear or tiny ask, once per feature (old `:576-583`) → `G:205-208`, `G:253` (F3).
- Wave timing: opens when the shape is drafted, concurrent with SMALLER PATH, costs `max(wave, planning)`, findings block nothing until the gate, the gate never happens while the wave is outstanding (old `:553-556`) → `R/references/plan-artifact.md:280-283`, restored word for word by `fd64f2fe3`. Only home: `rg -i "max\(wave|concurrent with the SMALLER|block nothing|wave is outstanding"` over `skills` and `packages/bee/LEADER.md` hits these four lines and nothing else.
- `bee feature start` internals (old `R/SKILL.md:44-49`) → `bee feature start --help` (F7).
- Mint-trap mechanism (old `R/SKILL.md:166-171`) → `docs/knowledge/areas/workflow-state/dispatch.md:189-196`; the duty stays at `R/SKILL.md:160-163` (F10).
- Inbound pointers repointed: `R/SKILL.md:56`, `:84`, `:98`, `:107`, `:143`, `:150`, `:158`, `:173`, `:201-202`; `skills/bee-hive/references/scout-and-ticks.md:44`; `skills/bee-hive/references/routing-and-contracts.md:129`; `packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs:759`; `R/playbooks/feature/multi-phase-plan.md:20`; `R/playbooks/spike/prototype.md:33`.

## Defects

<!-- bee:not-a-deferral: records where a rule now lives, promises no action -->
- D-a: FIXED by `fd64f2fe3`. The timing rule now lives at `R/references/plan-artifact.md:280-283`, the same place it held in the old file. Audit F3 had named `G` as owner; the fix keeps it on the planning side instead. That is still one home, and `G:203-266` holds no copy. Note: the paragraph sits right under "it never restates that procedure" (`:278`). A reader may want the timing moved into `G` "When hats open" (`G:197`) later. Not a defect for this cell.
<!-- /bee:not-a-deferral -->
- D-b (minor): truth 1 fails on its letter. Four history mentions of the old name remain outside the excluded trees. They are not pointers and pointer_integrity passes. Either reword them without the old file name, or read the truth as "no live pointer".
- D-c (minor): `R/playbooks/spike/prototype.md:33` was repointed from "Shape bodies by mode" to "Artifact: plan.md". The bold label "Shape bodies by mode" still exists at `R/references/plan-artifact.md:172`. The new pointer resolves but is coarser than the old one.
- D-d (minor, outside stdi-11 files): `docs/handbook/stages/planning.md:99-101` still says tests are the writer's inside each cell and "There is no trailing test cell per slice". Audit D listed it under "Found outside scope". D2 calls this text stale.
- D-e (minor): `R/references/plan-artifact.md:285-289` keeps "A hard-gate flag, a 6th product file … opens the wave at its three default seats. `high-risk` always opens it, at all five seats." The seat counts repeat `G:211-212`. F3 listed only the ≤5-file inline rule as planning-only.

## Verify

`.bee/bin/bee knowledge check && cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity`, with the cargo bin directory on PATH. Run again at `2d6de1cf9`. One file outside stdi-11, `skills/bee-knowledge-upkeep/SKILL.md`, had an uncommitted edit from another cell.

- knowledge check: 328 concepts in 349 files, 0 OKF errors, 0 profile errors, 600 profile warnings — OK.
- pointer_integrity: 10 passed, 0 failed, 0 ignored.
- Exit status 0.
