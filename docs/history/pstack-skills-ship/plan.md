---
artifact_contract: bee-plan/v1
mode: high-risk
---

# Plan: pstack-skills-ship

Mode: `high-risk`. The route flags are public-contracts, multi-domain, cross-platform
and covered-contract-change. After the hat wave the plan no longer edits the
`AGENTS.md` block. bee refuses to move a high-risk lane down to a lower lane, so
the lane stays high-risk.

Why this is the least workflow that protects the work: every onboarded project
syncs the skill set in all five skill homes, so one wrong line reaches every
host repo.

## Requirements (from CONTEXT.md)

No CONTEXT.md exists. The user chose option 1 in this session, and decision
`1254ca49` records that choice. After the hat wave, decision `1a8570fa` narrowed
it to these rules:

- D1: Ship THREE new bee skills in `skills/`: `bee-unslop`,
  `bee-technical-writing` and `bee-teach`. `bee-architect` is not a skill.
- D2: Five pstack skills already exist in bee, so they get no new skill. Only
  their missing parts go into bee's existing homes:
  - Provenance sweep gains `### Confidence tiers` and `### Provenance answer shape`.
  - Trace gains `### Trace answer shape`.
  - The spike playbook gains one step after step 4, for a question that asks
    which of several variants is best.

  `arena` and `recall` add nothing.
- D3: The new parts of architect go into
  `skills/bee-planning/references/design-sketch.md`:
  - a sketch that starts from the caller's usage;
  - the design red flags;
  - the rationale template;
  - the signs that a sketch must be scrapped.

  bee-planning § Shape cites this file when new code has an open shape. When
  several whole shapes are possible, the candidates run through blind lanes,
  under the existing lane rule. The sketch goes into the plan files. The code
  is written only after the gate, in bee-swarming cells.
- D4: The writing rule lives in the Communication contract, at
  `skills/bee-hive/references/routing-and-contracts.md`. That contract is the
  one home for chat style. The rule applies to user-facing prose and saved
  documents only. It never touches:
  - progress ticks;
  - red or refusal lines;
  - command output;
  - quoted evidence;
  - ids;
  - code.

  The project's own rules win over it, and the reply language is the user's
  language. `bee-teach` starts only when the user asks to understand something.
  The gate template and the one-next-action rule win over its shape. There is
  no `AGENTS.block.md` edit and no new Research pointer.
- D5: Remove the nine local `.claude/skills/<name>/` copies with `git rm`.
  Delete the routing block in `CLAUDE.md` and the `unslop` lines under
  `## Short responses`. Do not touch the four other non-bee directories:
  `create-verification-skill`, `maintain-verification-skill`,
  `product-description` and `verify-app`.
- D6: psh-1 to psh-3 set `regen_obligation_ack: "wave-barrier"` and run in
  parallel. psh-4 depends on all three and runs regen once, at the close of the
  wave.

## Load-bearing claims

Labels: `read` means I opened the file at the anchor. `ran` means I ran the command
and kept its output. No row is `guessed`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Both sync pipes find a skill directory only by its `bee-` name prefix, so a `bee-*` directory ships with no Rust change | ran | `rg -n 'starts_with\("bee-"\)' packages/bee-rs/crates/bee/src` | `devtools/skill_trees.rs:448: if ft.is_dir() && !ft.is_symlink() && name.starts_with("bee-") {` |
| 2 | The shipped trees carry each skill's `references/` directory | ran | `ls .agents/skills/bee-researching/references/` | `trace-and-provenance.md` is listed |
| 3 | Regen writes five skill homes | ran | `ls -d .claude/skills .agents/skills .opencode/skills .claude-plugin/skills .codex-plugin/skills` | all five paths are listed |
| 4 | The render copies skill bytes unchanged, except `bee:only` runtime blocks. So a shipped body names the runtime as `--runtime <rt>` | read | `packages/bee-rs/crates/bee/src/devtools/skill_trees.rs:843-860` | `fn render_is_byte_identity_without_markers()` … `render_skill_bytes(src.as_bytes(), "claude")` → `"top\nC\nend\n"` |
| 5 | The Communication contract is the one home for chat style | read | `skills/bee-hive/references/routing-and-contracts.md:209-211` | "## Communication contract" / "One home — chat style is never governed from anywhere else." |
| 6 | The gate question has a fixed template, and every turn closes on one next action, never a menu | read | `skills/bee-hive/references/routing-and-contracts.md:238-240` and `packages/bee/AGENTS.block.md:223-225` | "the Gate Presentation Contract is the template" / "close on exactly ONE next action … never a menu" |
| 7 | A test resolves every backticked `references/*.md` path in `skills/`. It does not check a plain markdown link or a skill name | read | `packages/bee-rs/crates/bee/tests/pointer_integrity.rs:180-182` | "Extract every live citation naming a `references/*.md` document. A live pointer is backticked" |
| 8 | Every `### ` heading under `## Class playbooks` must be a class value, so the spike change adds a numbered step, never a heading | read | `packages/bee-rs/crates/bee/tests/class_playbook_parity.rs:16-19` | "2. Every `### ` section under that heading names a real class value." |
| 9 | Regen cannot delete a directory whose name lacks the `bee-` prefix, so the nine local copies need `git rm` | read | `packages/bee-rs/crates/bee/src/onboard/skills.rs:881-883` | `refusing to remove {name}: outside the bee-* namespace` |
| 10 | `wave-barrier` moves regen to the orchestrator, which owes one full regen when the wave closes | read | `packages/bee-rs/crates/bee/src/verbs/cells/obligation.rs:219` | "the recognized value \"wave-barrier\" defers the regen to the orchestrator, which owes the full regen chain once at wave close" |
| 11 | Trace and Provenance sweep are one locked home, with no new skill | read | `docs/history/pstack-gaps/CONTEXT.md:25` | "TWO named procedures in ONE new home, `skills/bee-researching/references/trace-and-provenance.md`" |
| 12 | That home has no confidence tiers and no answer shape today | ran | `rg -n -i 'confidence\|tier\|explain' skills/bee-researching/references/trace-and-provenance.md` | one hit only, `20: … a tier marker` (a model tier) |
| 13 | Blind lanes beat `arena`, and HANDOFF beats `recall` | read | `docs/history/research/pstack-xia.md:60-64` | "Blind lanes with byte-identical briefs … beat pstack's `arena`" |
| 14 | The spike playbook has the throwaway rule but no step to choose among several variants | read | `skills/bee-planning/references/planning-reference.md:275-284` | "the ANSWER is the deliverable, the code is not" |
| 15 | bee ships no writing rules today | ran | `rg -n -i 'diátaxis\|diataxis\|prose.tell\|ai tell\|em dash\|asd-ste\|simplified technical' .bee/expertise skills/ packages/bee/AGENTS.block.md` | no output |
| 16 | A new bee skill needs a failing RED test first | read | `skills/bee-writing-skills/SKILL.md:18` | "THE IRON LAW: NO SKILL WITHOUT A FAILING TEST FIRST." |
| 17 | No RED run showed a failure that architect prevents; the one clear failure was teaching | ran | RED scenarios S3 and S4 through `bee dispatch prepare --kind advisor --brief-file` | S3 chose B: "The two special-case branches … are the streaming shape telling you it does not fit the problem" / S4: "Think of it as one Lego brick" |
| 18 | A high-risk lane cannot move down to a lower lane | ran | `bee route --set --lane standard …` | "rule violated: high-risk lanes never demote." |

## Discovery

The earlier pstack studies show that five of the nine ports already exist in bee
(claims 11 to 14). The five-seat hat wave then reshaped the plan. It found
bee-teach's gate trigger to be a BLOCKER, because it would turn every gate
question into a menu (claim 6). It found that a new `AGENTS.block.md` paragraph
would be a second home for chat style (claim 5). It also found that bee-architect
has no RED failure that justifies a skill (claims 16 and 17).

Two facts about the environment shape the execution:
- The `agy-flash` herding agent behind the `docs` role waits at an OAuth consent
  screen.
- The Fable model has used up its quota.

So the cells run on the session model through `bee cells escalate`, and the hat
seats ran on the `plan` role. Decisions `dbc5677c` and `1c407980` record both.

## Approach

**TDD for skills (claim 16).** Five pressure scenarios ran WITHOUT the new
skills. S4, the teaching reply, failed: it had a preview line, a Lego metaphor,
a wall of text and a menu at the end. S2 and S3 show that the model already
makes the right design decision under pressure. That is why architect becomes a
reference, not a skill. S1 and S2 are contaminated, because this repo's
`CLAUDE.md` already routes to the local ports.

GREEN reruns S1 and S4 with the new skill text. It runs S4 again WITHOUT the
skill too. All GREEN runs use the same role, so the S4 pair compares like with
like.

Named deviation: the Iron Law says to delete content and rewrite it from
observed failures. That step does not apply to content that was already in use
in the field.

**Skill conventions (bee-writing-skills checklist):**
- `name` equals the directory name.
- The description has one purpose clause, then "Use when …", then "Not for …".
- `metadata.version: '0.1'` and `metadata.ecosystem: bee`.
- `metadata.dependencies` is `[]` for the two writing skills. For bee-teach it is
  the bee-cli mapping, because teach cites bee procedures.
- Each skill has a `Headless` section and ends with a handoff sentence.
- Every dispatch reads `--runtime <rt>`.

**The three skills:**
- **bee-unslop.** It keeps every numbered rule. It gains a scope line:
  user-facing prose and saved documents only. It never touches ticks, red or
  refusal lines, command output, quoted evidence, ids or code. The project's
  own tone, length and language rules win. The English word lists apply only to
  English text.
- **bee-technical-writing.** It drops the one line that is true only in the bee
  source repo. A new line says the project's writing standard and code style win
  over its defaults, including over the tabs rule. Quoted evidence, ids, commands
  and code are never rewritten.
- **bee-teach.**
  - It starts only when the user asks to understand something, never on every
    gate.
  - At a gate, the Gate Presentation Contract and the one-next-action rule win
    over teach's shape.
  - It closes on one next action, never a menu.
  - It answers in the user's language.
  - A how question uses § "Trace", and a why question uses § "Provenance sweep".

**Folds (psh-3):**
- `trace-and-provenance.md` gains `### Confidence tiers`,
  `### Provenance answer shape` and `### Trace answer shape`. Each heading name
  is unique, and no existing heading changes.
- The spike playbook gains a step after step 4 and before the closing sentence.
  It names the project's own verification skill.
- `design-sketch.md` is new. bee-planning § Shape gains one sentence that cites it.

**Routing:**

| What | Home |
|---|---|
| `bee-unslop` on user-facing prose, `bee-technical-writing` on saved documents | Communication contract § "Everything else is craft" — the one home for chat style |
| `bee-teach` | its own description, triggered when the user asks to understand something |
| design sketch | `bee-planning` § Shape → `references/design-sketch.md` |

**Rejected alternatives:**
- Ship all nine skills. This gives two homes to five procedures. The user
  rejected it.
- Ship bee-architect as a skill. No RED failure supports it, and it would add a
  second consult wave.
- Put a new paragraph in `AGENTS.block.md`. That creates a second home for chat
  style.
- Trigger teach on every gate. That turns the gate into a menu.

**Risk map:**

| Component | Risk | Proof needed |
|---|---|---|
| Communication contract line | MEDIUM: every bee session reads it | `pointer_integrity` is green; the carve-out words are present |
| the three new skills | MEDIUM: a wrong trigger costs runs | GREEN runs pass; the leader frontmatter and link check is green |
| folds | MEDIUM: other docs cite these headings | `pointer_integrity` and `class_playbook_parity` are green; no existing heading changes |
| removal of the local copies | LOW | nine directories removed with `git rm`; the four other non-bee directories are still there |
| host that already has a `bee-*` directory with a new name | LOW, and it existed before this plan | residual risk, named in the Out of scope list |

## Shape

Slice 1 is four cells, each escalated to the session model.

| Cell | Deps | Writes |
|---|---|---|
| psh-1 | none | `skills/bee-unslop/` and `skills/bee-technical-writing/`, each with SKILL.md and CREATION-LOG.md |
| psh-2 | none | `skills/bee-teach/` with SKILL.md and CREATION-LOG.md |
| psh-3 | none | `skills/bee-researching/references/trace-and-provenance.md`; `skills/bee-planning/references/planning-reference.md` (the spike step); `skills/bee-planning/references/design-sketch.md` (new); `skills/bee-planning/SKILL.md` (one Shape sentence) |
| psh-4 | psh-1, psh-2, psh-3 | `skills/bee-hive/references/routing-and-contracts.md`, `README.md`, `CLAUDE.md`; `git rm` of the nine copies; GREEN recorded in the three CREATION-LOG.md files; `bee dev regen` across the five skill homes and the manifest; the declared suite |

## Test matrix

**Happy path.** `bee dev regen` is green. `.bee/bin/bee dev release-manifest --check`
is green. All three new skills appear in all five skill homes.

**Edge case.** The declared `commands.test` is green, including
`pointer_integrity`, `class_playbook_parity`, `rule_index_parity` and
`instruction_laws`.

**Error path.** These leader checks each print nothing:

```bash
rg -n -e '--runtime claude' skills/bee-unslop skills/bee-technical-writing skills/bee-teach skills/bee-planning/references/design-sketch.md
rg -n -w -e arena -e recall skills/bee-teach skills/bee-planning/references/design-sketch.md
rg -n 'disable-model-invocation' skills/bee-unslop skills/bee-technical-writing skills/bee-teach
```

A Python check runs over the three new skills and `design-sketch.md`:
- Each frontmatter has a `name` equal to its directory, a description that holds
  "Use when" and "Not for", and `version`, `ecosystem` and `dependencies`.
- Every relative markdown link resolves.
- Every `bee-[a-z-]+` token in the touched doctrine files names a real
  `skills/` directory.

After psh-4:
- The nine local names are gone.
- `create-verification-skill`, `maintain-verification-skill`,
  `product-description` and `verify-app` still exist.

## Open Questions

(none)

## Out of scope

- The release. It runs `scripts/release.sh` after the uat stop, on the user's word.
- `docs/07-contracts.md` does not list `bee decisions search`,
  `bee dispatch prepare` or `bee blind check`. That doc drift is a follow-up.
- A host that already has its own `bee-unslop`, `bee-technical-writing` or
  `bee-teach` directory gets it overwritten by sync. This is existing sync
  behavior at `onboard/skills.rs:842-876`, and the risk remains.
- bee's own doctrine uses em dashes, and `bee-unslop` bans them in new prose.
  This plan does not rewrite existing doctrine.
- The `agy-flash` login and the Fable quota belong to the user.
- `bee orient`, the principle index, and the Rust crate do not change.
