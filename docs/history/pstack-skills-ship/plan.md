---
artifact_contract: bee-plan/v1
mode: high-risk
---

# Plan: pstack-skills-ship

Mode: `high-risk`. The route carries four flags: public-contracts, multi-domain, cross-platform, and covered-contract-change.
Why this is the least workflow that protects the work: the change edits bee's
shipped contract, which is the skill set every onboarded project syncs and the
always-loaded `AGENTS.md` block. Three tests already read those files
(`pointer_integrity.rs`, `rule_index_parity.rs`, `instruction_laws.rs`), and one
wrong line reaches every host repo. A smaller shape would skip the proof
that those tests stay green after regen.

## Requirements (from CONTEXT.md)

No CONTEXT.md. The user chose option 1 in this session. Decision `1254ca49`
locks these decisions:

- D1: Ship four NEW bee skills in `skills/` under bee conventions: `bee-unslop`,
  `bee-technical-writing`, `bee-teach`, and `bee-architect`.
- D2: Five pstack skills already exist in bee, so they get NO new skill. Only
  their missing parts go into the existing homes:
  - The confidence tiers and output shape of `why` go into Provenance sweep.
  - The explainer shape of `how` goes into Trace. Both procedures live in
    `skills/bee-researching/references/trace-and-provenance.md`.
  - The variant switcher of `prototype` goes into the spike playbook.
  - `arena` and `recall` add nothing.
- D3: The candidate step of `bee-architect` runs blind lanes. It never runs an arena.
- D4: The automatic routing ships in bee's own doctrine:
  - The every-reply writing rule goes in `packages/bee/AGENTS.block.md` § Communication.
  - The architect pointer goes in `bee-planning` § Shape.
- D5: Remove the nine local `.claude/skills/` copies and the routing block in
  `CLAUDE.md`. This supersedes decision `5a3c0e98`.
- D6: Skill-touching cells run one at a time.

## Load-bearing claims

Labels: `read` means I opened the file at the anchor. `ran` means I ran the command
and kept its output. No row is `guessed`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Both sync pipes discover a skill directory by its `bee-` name prefix alone, so a `bee-*` directory ships with no Rust change | ran | `rg -n 'starts_with\("bee-"\)' packages/bee-rs/crates/bee/src` | `devtools/skill_trees.rs:448: if ft.is_dir() && !ft.is_symlink() && name.starts_with("bee-") {` and `onboard/render.rs:379: …filter(\|e\| e.name.starts_with("bee-")).collect()` |
| 2 | The render walk copies every file under a skill directory, `references/` included | read | `packages/bee-rs/crates/bee/src/devtools/skill_trees.rs:456-458` | "Depth-first walk, entries sorted by locale collation at each level." |
| 3 | The Codex copy keeps the source frontmatter byte for byte, so runtime-specific text in a body ships as is | ran | `diff <(sed -n '1,20p' skills/bee-planning/SKILL.md) <(sed -n '1,20p' .agents/skills/bee-planning/SKILL.md)` | `codex copy: frontmatter identical` |
| 4 | `AGENTS.md` carries the shipped block between markers, and the source is `packages/bee/AGENTS.block.md` | ran | `rg -n 'BEE' AGENTS.md` | `7:<!-- BEE:START -->` … `324:<!-- BEE:END -->` |
| 5 | A test pins rule markers across three copies, so the new writing paragraph goes OUTSIDE every rule marker | read | `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:118` | "`tests/rule_index_parity.rs` pins the three copies — the markers in `AGENTS.md`, the markers in `packages/bee/AGENTS.block.md`, and the rows here" |
| 6 | A test resolves every backticked `references/*.md` path and every quoted heading in `skills/` | read | `packages/bee-rs/crates/bee/tests/pointer_integrity.rs:185-195` | "Extract every live citation naming a `references/*.md` document. A live pointer is backticked" |
| 7 | Trace and Provenance sweep are one locked home with no new skill | read | `docs/history/pstack-gaps/CONTEXT.md`, D1 | "The trace flow and the provenance flow are TWO named procedures in ONE new home, `skills/bee-researching/references/trace-and-provenance.md`" |
| 8 | Blind lanes are stronger than `arena`, and HANDOFF is stronger than `recall`, so neither is ported again | read | `docs/history/research/pstack-xia.md:57-64` | "Blind lanes with byte-identical briefs, a neutrality lint and `bee blind check` beat pstack's `arena`" / "`HANDOFF.json` with two typed kinds and an `adopt` verb beats `pause-safely` / `session-pickup` / `recall`" |
| 9 | The spike playbook already holds the throwaway rule that `prototype` teaches, but it has no multi-variant step | read | `skills/bee-planning/references/planning-reference.md:275-284` | "Keep the throwaway under `.bee/spikes/<feature>/` — the ANSWER is the deliverable, the code is not" |
| 10 | Skill-touching cells cannot run concurrently because regen rewrites one shared manifest | read | `docs/history/pstack-adoption/plan.md:61` | "the regen chain rewrites ONE shared manifest on every run, which serializes skill-touching cells" |
| 11 | bee ships no writing rules today | ran | `rg -n -i 'diátaxis\|diataxis\|prose.tell\|ai tell\|em dash\|asd-ste\|simplified technical' .bee/expertise skills/` | no output |
| 12 | A new bee skill must pass the TDD-for-skills cycle, and a rule that applies to every turn must live in the always-loaded layer | read | `skills/bee-writing-skills/SKILL.md` | "THE IRON LAW: NO SKILL WITHOUT A FAILING TEST FIRST." / "Per-turn rules (chat shape, communication) are never exiled to references" |

## Discovery

Before the gate I read the earlier pstack studies. They found that five of the
nine ported skills are already in bee, in a stronger form (claims 7 to 9). The
sync code picks up any `bee-*` directory, so the four new skills need no Rust
change (claims 1 and 2). One problem remains. The local ports write
`--runtime claude` in nine places, and the Codex tree copies each body as it is
(claim 3). The shipped skills must name the runtime generically, as
`--runtime <rt>`, in the form the other bee skills already use.

## Approach

I recommend this order: pressure-test first, then four new skills, then the
merges into existing homes, then doctrine and regen, with every step serial.

**TDD for skills (claim 12).** Five pressure scenarios run WITHOUT the new
skills before any cell writes skill text. Two scenarios produce real output: a
PR body and a plain explanation for the user. Three force a choice: architect
must not fire on a patterned bug fix, architect must fire on an open shape,
and dispatch must still go through the door under time pressure. Each
scenario has three or more pressures. The same scenarios run again WITH the
skill text. Both runs go into each new skill's `CREATION-LOG.md`.

Named deviation: the Iron Law's "delete the content and rewrite from observed
failures" does not apply to a port of field-used content. The RED runs test
the triggers and the bee adaptations. A RED rationalization that the ported
body does not answer is fixed in REFACTOR.

**Skill conventions.** Every new skill follows the `bee-writing-skills`
checklist:
- `name` equals the directory name.
- The description has one purpose clause, then "Use when …", then "Not for …".
  It never summarizes the steps.
- `metadata.version: '0.1'` and `metadata.ecosystem: bee`.
- A `Headless` section.
- A handoff sentence at the end.
- Every dispatch is written as `--runtime <rt>`.
- Every cross-reference uses a bee name.

**Where each skill routes:**

| Skill | When it starts | Home of the routing line |
|---|---|---|
| `bee-unslop` | every reply | `AGENTS.block.md` § Communication, always loaded |
| `bee-technical-writing` | every document bee writes | the same paragraph |
| `bee-teach` | the user asks to understand something, or a gate needs plain words | its own description |
| `bee-architect` | the shape of new code is open on the standard or high-risk lane | `bee-planning` § Shape |

**Rejected alternatives:**
- Ship all nine. Trace, Provenance sweep, blind lanes, handoff and spike would
  each get two homes, and two locked records would be reversed. The user
  rejected this.
- Put the writing rule in a `references/` file. The checklist forbids this for
  a per-turn rule, because an agent never opens a reference file that nothing
  points to.
- Put the writing rule inside a rule marker. The parity test would then demand
  a new rule id and a row in the doctrine index. A plain paragraph carries the
  rule with less weight.

**Risk map:**

| Component | Risk | Proof needed |
|---|---|---|
| `AGENTS.block.md` paragraph | HIGH. It is always loaded in every host. | regen is green, and `cargo test` passes `rule_index_parity`, `instruction_laws` and `pointer_integrity` |
| the four new skills | MEDIUM. A wrong trigger costs model runs. | GREEN runs pass; the frontmatter follows the checklist |
| merges into `trace-and-provenance.md` and the spike playbook | MEDIUM. The headings are cited. | `pointer_integrity` is green, and no cited heading is renamed |
| removal of the local copies | LOW | after regen, `.claude/skills/` holds `bee-*` directories only for these skills |

## Shape

Phase 0 runs before the gate: RED baseline, scenarios S1 to S5, with no skill.
Slice 1 holds four serial cells:

| Cell | Writes |
|---|---|
| psh-1 | `skills/bee-unslop/` and `skills/bee-technical-writing/` (SKILL.md and CREATION-LOG.md) |
| psh-2 | `skills/bee-teach/` and `skills/bee-architect/` (SKILL.md, CREATION-LOG.md, and the three architect reference files) |
| psh-3 | `skills/bee-researching/references/trace-and-provenance.md`, `skills/bee-planning/references/planning-reference.md` (spike), and `skills/bee-planning/SKILL.md` (Shape and Research pointers) |
| psh-4 | `packages/bee/AGENTS.block.md`, `README.md` (skill table), `CLAUDE.md`, and deletion of the nine `.claude/skills/<name>/` copies. This cell also records GREEN in the four CREATION-LOG.md files, then runs `bee dev regen` and the declared suite |

## Test matrix

- **Happy path.** `bee dev regen` is green. The four new skills appear under
  `.claude/skills/`, `.agents/skills/` and the plugin trees. Each frontmatter
  passes the checklist.
- **Edge case.** The declared `commands.test` is green, including
  `pointer_integrity`, `rule_index_parity` and `instruction_laws`.
- **Error path.** No shipped skill writes `--runtime claude`. No shipped text
  routes to a `how`, `why`, `arena`, `recall` or `prototype` skill. No
  `.claude/skills/<name>/` copy of the nine survives. RED and GREEN are
  recorded for all four new skills.

## Open Questions

(none)

## Out of scope

- The release. It runs through `scripts/release.sh` after the uat stop, on the
  user's word.
- `docs/07-contracts.md` does not list `bee decisions search` or
  `bee dispatch prepare`. Shipped skills already quote both. This is a doc-drift
  follow-up, not part of this work.
- Any change to `bee orient`, the principle index, or the Rust crate.
