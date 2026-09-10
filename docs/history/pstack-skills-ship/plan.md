---
artifact_contract: bee-plan/v1
mode: high-risk
---

# Plan: pstack-skills-ship

Mode: `high-risk`. The route carries four flags: public-contracts, multi-domain, cross-platform, and covered-contract-change.
Why this is the least workflow that protects the work: this work edits bee's
shipped contract. That contract is the skill set that every onboarded project
syncs, plus the `AGENTS.md` block that every session loads. Five tests read
those files: `pointer_integrity.rs`, `rule_index_parity.rs`,
`instruction_laws.rs`, `agents_block_render_parity.rs` and
`class_playbook_parity.rs`. One wrong line reaches every host repo.

## Requirements (from CONTEXT.md)

No CONTEXT.md exists. The user chose option 1 in this session. These decisions
are locked by decision `1254ca49` and refined by decision `c3b84959`:

- D1: Ship four NEW bee skills in `skills/`, following bee's skill conventions:
  `bee-unslop`, `bee-technical-writing`, `bee-teach` and `bee-architect`.
- D2: Five pstack skills already exist in bee. They get NO new skill. Only
  their missing parts go into bee's existing homes:
  - the confidence tiers and the answer shape from `why` go into Provenance
    sweep;
  - the explainer shape from `how` goes into Trace. Both procedures live in
    `skills/bee-researching/references/trace-and-provenance.md`;
  - the variant step from `prototype` goes into the spike playbook.

  `arena` and `recall` add nothing.
- D3: The candidate step of `bee-architect` runs blind lanes, never an arena.
  bee forbids source edits before the merged shape+execution gate. So the
  sketch lands in the plan artifacts, and implementation happens in
  bee-swarming cells after the gate.
- D4: The automatic routing ships in bee's own doctrine:
  - the writing rule for every reply goes in `packages/bee/AGENTS.block.md`
    § Communication;
  - the architect pointer goes in `bee-planning` § Shape;
  - there is no new Research pointer. § Research already sends
    how-and-why work to `bee-researching`, and that skill's reference table
    already routes to Trace and Provenance sweep
    (`skills/bee-researching/SKILL.md:100`).
- D5: Remove the nine local `.claude/skills/<name>/` copies with `git rm`.
  Also delete the routing block in `CLAUDE.md` and the `unslop` lines at
  `CLAUDE.md:98-100`. This supersedes decision `5a3c0e98`.
- D6: Only the regen cell runs alone, because regen rewrites one shared release
  manifest. Cells psh-1 to psh-3 write disjoint source files and never run
  regen, so they run in parallel. psh-4 runs regen once, after all three.

## Load-bearing claims

Labels: `read` means I opened the file at the anchor. `ran` means I ran the command
and kept its output. No row is `guessed`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Both sync pipes find a skill directory only by its `bee-` name prefix, so a `bee-*` directory ships with no Rust change | ran | `rg -n 'starts_with\("bee-"\)' packages/bee-rs/crates/bee/src` | `devtools/skill_trees.rs:448: if ft.is_dir() && !ft.is_symlink() && name.starts_with("bee-") {` and `onboard/render.rs:379: …filter(\|e\| e.name.starts_with("bee-")).collect()` |
| 2 | The shipped trees carry each skill's `references/` subdirectory | ran | `ls .agents/skills/bee-researching/references/` | `trace-and-provenance.md` is listed (see ## Discovery) |
| 3 | The render copies skill bytes unchanged, except `<!-- bee:only claude\|codex -->` blocks, which it keeps or drops per runtime. So a shipped body must name the runtime generically, or wrap runtime-specific lines in a `bee:only` block | read | `packages/bee-rs/crates/bee/src/devtools/skill_trees.rs:843-860` | `fn render_is_byte_identity_without_markers()` … `render_skill_bytes(src.as_bytes(), "claude")` → `"top\nC\nend\n"`, and for `"codex"` → `"top\nX\nend\n"` |
| 4 | `AGENTS.md` carries the shipped block between markers, and the source is `packages/bee/AGENTS.block.md` | ran | `rg -n 'BEE' AGENTS.md` | `7:<!-- BEE:START -->` … `324:<!-- BEE:END -->` |
| 5 | A test pins rule markers across three copies, so the new writing paragraph goes OUTSIDE every rule marker | read | `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:118` | "`tests/rule_index_parity.rs` pins the three copies — the markers in `AGENTS.md`, the markers in `packages/bee/AGENTS.block.md`, and the rows here" |
| 6 | A test resolves every backticked `references/*.md` path in `skills/`. A plain markdown link is not checked | read | `packages/bee-rs/crates/bee/tests/pointer_integrity.rs:180-182` | "Extract every live citation naming a `references/*.md` document. A live pointer is backticked; a retired name written unquoted reads as history and is not a promise." |
| 7 | A test pins the `AGENTS.md` block bytes to the render of `AGENTS.block.md`, so the block changes only through regen | read | `packages/bee-rs/crates/bee/tests/agents_block_render_parity.rs:1-15` | "the bytes between `<!-- BEE:START -->` and `<!-- BEE:END -->` in `AGENTS.md` equal what `render_agents_block` … produces from `packages/bee/AGENTS.block.md`" |
| 8 | Every `### ` heading under `## Class playbooks` must be a real class value, so the spike change adds a numbered step, never a new `###` heading | read | `packages/bee-rs/crates/bee/tests/class_playbook_parity.rs:16-19` | "2. Every `### ` section under that heading names a real class value." |
| 9 | Regen cannot delete a non-`bee-` directory, so the nine local copies need `git rm` | read | `packages/bee-rs/crates/bee/src/onboard/skills.rs:881-883` | `if !name.starts_with("bee-") { return Some(format!("refusing to remove {name}: outside the bee-* namespace"));` |
| 10 | Trace and Provenance sweep form one locked home, with no new skill | read | `docs/history/pstack-gaps/CONTEXT.md:25` | "The trace flow and the provenance flow are TWO named procedures in ONE new home, `skills/bee-researching/references/trace-and-provenance.md`" |
| 11 | That home has no confidence tiers and no answer shape today | ran | `rg -n -i 'confidence\|tier\|explain' skills/bee-researching/references/trace-and-provenance.md` | one hit, `20: hand-pick a \`subagent_type\`, a \`model\`, or a tier marker` (a model tier, not a confidence tier) |
| 12 | Blind lanes beat `arena`, and HANDOFF beats `recall`, so bee ports neither again | read | `docs/history/research/pstack-xia.md:60-64` | "Blind lanes with byte-identical briefs, a neutrality lint and `bee blind check` beat pstack's `arena`" / "`HANDOFF.json` with two typed kinds and an `adopt` verb beats `pause-safely` / `session-pickup` / `recall`" |
| 13 | The spike playbook holds the throwaway rule, but it has no step for choosing among several variants | read | `skills/bee-planning/references/planning-reference.md:275-284` | "Keep the throwaway under `.bee/spikes/<feature>/` — the ANSWER is the deliverable, the code is not" |
| 14 | Regen rewrites one shared manifest, so cells that run regen cannot run at the same time | read | `docs/history/pstack-adoption/plan.md:61` | "the regen chain rewrites ONE shared manifest on every run, which serializes skill-touching cells" |
| 15 | bee ships no writing rules today, in skills, expertise, or the AGENTS block | ran | `rg -n -i 'diátaxis\|diataxis\|prose.tell\|ai tell\|em dash\|asd-ste\|simplified technical' .bee/expertise skills/ packages/bee/AGENTS.block.md` | no output (see ## Discovery) |
| 16 | A new bee skill must pass the TDD-for-skills cycle, and a rule that applies to every turn must live in the always-loaded layer | read | `skills/bee-writing-skills/SKILL.md:18,43` | "THE IRON LAW: NO SKILL WITHOUT A FAILING TEST FIRST." / "Per-turn rules (chat shape, communication) are never exiled to references" |

## Discovery

I read the earlier pstack studies before the gate. They show that five of the
nine ported skills already exist in bee, in a stronger form (claims 10 to 13).
The sync takes any `bee-*` directory, so the four new skills need no Rust change
(claims 1 and 2).

The render copies skill bytes unchanged, except `bee:only` runtime blocks
(claim 3). So the shipped skills write `--runtime <rt>`, as the other bee
skills do, and never `--runtime claude`.

The hat-facts-gaps seat found four more tests that read these files:
`agents_block_render_parity`, `class_playbook_parity`, the prefix refusal in
`apply_remove_skill`, and the backtick-only scope of `pointer_integrity`.
Claims 6 to 9 now carry them.

`pointer_integrity` does not check plain markdown links. So the leader check
below resolves links in the four new skills itself.

Two environment facts shape the execution:
- The `agy-flash` herding agent is stuck at an Antigravity OAuth consent
  screen, and the `docs` role resolves to that agent.
- The Fable model has hit its usage limit.

Execution cells therefore run on the session model through `bee cells escalate`.
The remaining hat seats run on the `plan` role, whose job description names plan
checks. Decisions `dbc5677c` and `1c407980` record both reasons.

## Approach

The work runs in four steps:

1. Pressure-test the new skills.
2. Write the four new skills and the merges into bee's homes, in parallel.
3. Make the doctrine edit and run one regen.
4. Prove the result.

**TDD for skills (claim 16).** Five pressure scenarios ran WITHOUT the new
skills before any skill text was written:

- S1 asks for a PR body.
- S2 checks that architect does not fire on a patterned fix.
- S3 checks that architect fires on an open shape.
- S4 asks for a teaching reply.
- S5 checks that dispatch still goes through the door under pressure.

S4 failed on teaching rules. The reply used a preview line, a Lego metaphor, a
long wall of text, and a menu at the end. The other four passed. S1 and S2 are
contaminated, because the subagents inherited this repo's `CLAUDE.md`, which
already routes to the local ports.

GREEN reruns the same scenarios WITH the new skill text. S4 runs both RED and
GREEN on the same model, so the pair is a fair comparison.

Named deviation: the Iron Law says to delete content and rewrite it from
observed failures. That step does not apply to a port of content already used
in the field. RED tests the triggers and the bee changes, and REFACTOR answers
any RED rationalization that the ported body does not.

**Skill conventions.** Every new skill follows the `bee-writing-skills`
checklist:

- The `name` equals the directory name.
- The description has one purpose clause, then "Use when …", then "Not for …".
  It never summarizes the steps.
- `metadata.version: '0.1'` and `metadata.ecosystem: bee`.
- `metadata.dependencies` is `[]` for the two writing skills. For
  `bee-teach` and `bee-architect` it is the bee-cli mapping, because both call
  `.bee/bin/bee`.
- The skill has a `Headless` section and ends with a handoff sentence.
- Every dispatch writes `--runtime <rt>`, and every cross-reference uses a bee
  name.

**The four new skills, and the bee changes each one needs:**
- **`bee-teach`** routes a how-does-this-run question to § "Trace" and a
  why-is-this-so question to § "Provenance sweep". It no longer uses the `how`
  and `why` skills.
- **`bee-architect`** runs three phases:
  - Phase A grounds the problem with Trace and Provenance sweep.
  - Phase B runs blind lanes. Its procedure lives in
    `skills/bee-hive/references/gates-and-delegation.md` ("Blind lanes and
    convergence"). That procedure covers `--kind advisor --role lane-N
    --brief-file`, one LaneBrief built from `references/runner-prompt.md`, a
    dossier, and a green `bee blind check`.
  - Phase D and after run in bee-swarming cells after the gate.

  A scrap returns to bee-planning, and the old sketch is superseded through
  `bee decisions log`. The plain markdown link to the arena skill in
  `references/rationale-template.md` is deleted.
- **`bee-technical-writing`** loses the one sentence that is true only in this
  repo. It gains a line saying that a project's own writing standard wins.

**The merges into existing homes (psh-3):**
- Provenance sweep gains `### Confidence tiers` and `### Answer shape`.
- Trace gains `### Answer shape`.
- The spike playbook gains one numbered step for choosing among several
  variants, with no new heading (claim 8). The step names the project's own
  verification skill, not a path that exists only in this repo.
- `bee-planning` § Shape gains one `bee-architect` sentence.

No heading is renamed, re-levelled, or deleted.

**Where each skill routes:**

| Skill | When it starts | Home of the routing line |
|---|---|---|
| `bee-unslop` | every reply | `AGENTS.block.md` § Communication, always loaded, outside every rule marker |
| `bee-technical-writing` | every document bee writes | the same paragraph |
| `bee-teach` | the user asks to understand something, or a gate needs plain words | its own description |
| `bee-architect` | the shape of new code is open on the standard or high-risk lane | `bee-planning` § Shape |

**Rejected alternatives:**
- Ship all nine. That gives two homes each to Trace, Provenance sweep, blind
  lanes, handoff and spike, and it reverses two locked records. The user
  rejected this.
- Put the writing rule in a `references/` file. The checklist forbids that for a
  per-turn rule.
- Put the writing rule inside a rule marker. The parity test would then demand a
  new rule id and an index row.
- Merge psh-1 and psh-2 into one cell. The hat-alternatives seat offered this as
  optional. It saves nothing, because D6 now lets those cells run in parallel.

**Risk map:**

| Component | Risk | Proof needed |
|---|---|---|
| `AGENTS.block.md` paragraph | HIGH, because every host loads it | regen is green; the declared suite is green, including `rule_index_parity`, `agents_block_render_parity`, `instruction_laws` and `pointer_integrity` |
| the four new skills | MEDIUM, because a wrong trigger costs model runs | GREEN runs pass, and the leader frontmatter and link check is green |
| the merges into `trace-and-provenance.md` and the spike playbook | MEDIUM, because other files cite those headings | `pointer_integrity` and `class_playbook_parity` are green, and no cited heading changes |
| removing the local copies | LOW | `git rm` of nine directories; afterwards `.claude/skills/` holds the regenerated `bee-unslop`, `bee-technical-writing`, `bee-teach` and `bee-architect` and no copy of the nine |

## Shape

Phase 0 ran before the gate: the RED baseline, scenarios S1 to S5. Slice 1 has
four cells. All four are escalated to the session model.

| Cell | Deps | Writes |
|---|---|---|
| psh-1 | none | `skills/bee-unslop/` and `skills/bee-technical-writing/` (SKILL.md and CREATION-LOG.md) |
| psh-2 | none | `skills/bee-teach/` and `skills/bee-architect/` (SKILL.md, CREATION-LOG.md, and the three architect reference files) |
| psh-3 | none | `skills/bee-researching/references/trace-and-provenance.md`, the spike step in `skills/bee-planning/references/planning-reference.md`, and one Shape sentence in `skills/bee-planning/SKILL.md` |
| psh-4 | psh-1, psh-2, psh-3 | `packages/bee/AGENTS.block.md`, `README.md`, `CLAUDE.md`, `git rm` of the nine local copies, and GREEN in the four CREATION-LOG.md files. It then runs `bee dev regen` and the declared suite |

## Test matrix

- **Happy path.** `bee dev regen` is green. The four new skills appear under
  `.claude/skills/`, `.agents/skills/` and the plugin trees.
- **Edge case.** The declared `commands.test` is green, including
  `pointer_integrity`, `rule_index_parity`, `instruction_laws`,
  `agents_block_render_parity` and `class_playbook_parity`.
- **Error path.** These leader checks run over the four new skill directories
  and the three merged files, and each prints nothing:
  - `rg -n -e '--runtime claude' skills/bee-unslop skills/bee-technical-writing skills/bee-teach skills/bee-architect`
  - `rg -n -w -e arena -e recall skills/bee-teach skills/bee-architect`
  - `rg -n 'disable-model-invocation' skills/bee-unslop skills/bee-technical-writing skills/bee-teach skills/bee-architect`

  A Python check confirms each new skill's frontmatter. The `name` must equal
  the directory. The description must carry "Use when" and "Not for". The
  metadata must hold `version`, `ecosystem` and `dependencies`. The same check
  resolves every relative markdown link in the four new skills.

  After psh-4, `test ! -e .claude/skills/<name>` holds for each of the nine
  local names. RED and GREEN are recorded for all four new skills.

## Open Questions

(none)

## Out of scope

- The release. It runs through `scripts/release.sh` after the uat stop, on the
  user's word.
- `docs/07-contracts.md` does not list `bee decisions search`,
  `bee dispatch prepare` or `bee blind check`. Shipped skills already quote all
  three. This is a doc-drift follow-up.
- Any change to `bee orient`, the principle index, or the Rust crate.
- Fixing the `agy-flash` login or the Fable quota. Both belong to the user.
