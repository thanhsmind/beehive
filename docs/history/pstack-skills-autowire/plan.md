---
artifact_contract: bee-plan/v1
mode: standard
---

# Plan: pstack-skills-autowire

Mode: `standard` — 1 risk flag: multi-domain
Why this is the least workflow that protects the work: the nine ported skills already
exist and pass their parity check. Making them automatic is one frontmatter switch plus
nine sharpened trigger descriptions plus one routing block. The risk is not breakage,
it is misfire — a skill that spawns parallel model runs firing on a typo fix. So the
ceremony that pays is the trigger wording, not the plumbing.

## Requirements (from CONTEXT.md)

No prior CONTEXT.md. The decisions this plan locks:

- D1: Remove `disable-model-invocation: true` from all nine ported skills so the model
  invokes them itself. This SUPERSEDES pstack-part2-skills D6, which kept them
  user-invoked only.
- D2: Rewrite each of the nine `description` fields as an auto-fire trigger — when the
  model should reach for the skill, not what the user types. The three expensive skills
  (`arena`, `architect`, `prototype`) each carry an explicit negative guard, because
  each spawns parallel model runs and a misfire is the named ceremony-capture failure.
- D3: The routing block — which bee lifecycle step reaches for which skill — lives in
  this repo's `CLAUDE.md`, beside the ASD-STE100 rule that already governs how bee
  writes here (`bee-principle-one-fact-one-home`).
- D4: Do NOT touch `AGENTS.md`, `packages/bee/AGENTS.block.md`, or `skills/bee-*`. Those
  ship to every host repo, and a host repo has none of these nine skills. Wiring them
  into the published contract is a separate feature behind its own gate.
- D5: The hat wave stays the plan-step consult. `arena` and `architect` do not replace
  it; they fire on a different condition — the shape itself is open, not the plan needs
  critique (`bee-principle-single-source-of-truth`).

## Load-bearing claims

Labels: `read` = I opened the file at the anchor. `ran` = I ran the command and kept its
output. `guessed` = no evidence yet. No row may stay `guessed` past the gate.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | `disable-model-invocation: true` is the switch: bee's own skills omit it and are model-invocable, the nine ports carry it | ran | `for f in skills/bee-*/SKILL.md; do grep -q 'disable-model-invocation' "$f" && echo DISABLED || echo auto-invocable; done` and `rg -n 'disable-model-invocation' .claude/skills/ -l` | every `bee-*` printed `auto-invocable` except `bee-verifying` and `bee-verify-upkeep`; all nine ports listed as carrying the flag |
| 2 | A rule added to `AGENTS.md` is pinned to two more copies by a test, so it cannot stay local | read | `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:118` | "`tests/rule_index_parity.rs` pins the three copies — the markers in `AGENTS.md`, the markers in `packages/bee/AGENTS.block.md`, and the rows here" |
| 3 | `skills/` is synced into host repos by onboarding, so a pointer there reaches repos with no pstack skills | read | `README.md:621` | "it syncs the bee skill set into the host repo's own managed roots (`<repo>/.claude/skills/bee-*` …) from this repo's `skills/` tree" |
| 4 | bee-planning's Research step is the place a rationale read belongs | read | `skills/bee-planning/SKILL.md:52-62` | "Remove uncertainty at the lowest cost: cite a pattern or verify one fact; unfamiliar territory or competing approaches dispatch `bee-researching`, findings merged in." |
| 5 | bee-planning's Shape step already carries the hat wave as the plan check | read | `skills/bee-planning/SKILL.md:86-93` | "Standard/high-risk add the hat wave before the gate: the plan-step wave IS the plan check now" |
| 6 | The hat wave's own threshold warns against firing consults on small work | read | `skills/bee-hive/references/gates-and-delegation.md:204-208` | "five dispatches on a typo fix is the named ceremony-capture failure" |
| 7 | AGENTS.md § Communication is a rule home with markers, so plain prose cannot be added inside it | ran | `rg -n 'rule: agents-' AGENTS.md` | `225:<!-- rule: agents-one-next-action -->` |
| 8 | The nine skills are installed and currently pass their check | ran | `python3 check_port.py` in main | `OK  9 skills: frontmatter parses, links resolve, no pstack token` |

## Discovery

Two findings shaped this. First, the switch is one line: bee's own skills are already
model-invocable and the nine ports are not, so "automatic" costs a frontmatter edit, and
the real work is the trigger wording that decides when each one fires. Second, every
doctrine home I would naturally reach for — `AGENTS.md`, `skills/bee-*` — ships to host
repos that do not have these nine skills, and `AGENTS.md` rule blocks are pinned by
`tests/rule_index_parity.rs` to two other copies. So the routing block goes in
`CLAUDE.md`, which is this repo's alone.

## Approach

Recommended path: one switch, nine triggers, one routing block.

The trigger contract every rewritten description follows:

1. Say **when the model should reach for it**, in the words a task arrives in — not the
   slash command the user types.
2. Keep the user's own phrasings too, so `/how` still works.
3. The three expensive skills carry a negative guard naming what must NOT fire them.
4. Nothing else in the body changes. The port is already verified.

The routing block in `CLAUDE.md` maps bee's existing lifecycle steps to the skills:

| bee step | Skill that fires | Condition |
|---|---|---|
| every reply, every doc | `unslop` | always |
| a doc, plan, spec, README, PR or commit body | `technical-writing` | always, before saving |
| session start on a resumed topic | `recall` | history not already in bee state |
| planning research | `how` | the work touches a subsystem nobody read this session |
| planning research, and before locking a decision | `why` | existing shape is a constraint the plan must honour |
| planning shape, standard/high-risk | `architect` | the shape is open and no precedent fits |
| inside architect, or any single-shot artifact | `arena` | several viable designs, choice genuinely open |
| planning, spike lane | `prototype` | a build answers the question cheaper than an argument |
| explaining work to the user | `teach` | the user asks to understand, or a gate needs plain words |

Rejected alternatives:

- Put the routing in `AGENTS.md` as a new rule. Rejected per claim 2 and D4: the parity
  test would force it into the shipped block and into every host repo, where none of the
  nine skills exist.
- Leave the descriptions alone and rely on the routing block. Rejected: with
  `disable-model-invocation: true` the model cannot invoke the skill at all, so a routing
  line pointing at it is dead.
- Replace the hat wave with `arena`. Rejected per D5.

Risk map:

| Component | Risk | Proof needed |
|---|---|---|
| the frontmatter switch | LOW — one line per file | no skill still carries the flag |
| `unslop`, `technical-writing`, `teach`, `how`, `why`, `recall` triggers | LOW — cheap to fire, cheap to be wrong | frontmatter parses, description non-empty |
| `arena`, `architect`, `prototype` triggers | MEDIUM — each spawns parallel runs; a misfire is real cost | each description carries a negative guard |
| `CLAUDE.md` routing block | MEDIUM — read every session, so a wrong line misroutes every task | the block names only steps that exist in bee today |

## Shape

Slice 1 is the whole feature. Two cells, disjoint files.

| Cell | Writes | Why separate |
|---|---|---|
| A | the nine `.claude/skills/*/SKILL.md` frontmatter blocks | mechanical: apply the exact descriptions this plan carries, drop one line |
| B | `CLAUDE.md` | the routing block; this repo's own voice and ASD-STE100 rules |

## Test matrix

The nine trees are markdown an agent loads. The declared `commands.test` is
`cargo test` and does not reach them (pstack-part2-skills claim 7). Proof is the same
parity check, extended by two assertions this feature adds:

- **Happy path** — every one of the nine `SKILL.md` files still parses, still carries
  `name` and `description`, and NO LONGER carries `disable-model-invocation`.
- **Edge case** — `arena`, `architect` and `prototype` each carry a negative guard in
  their description (a "Not for …" clause), so a cheap task cannot trigger them.
- **Error path** — the de-pstack scan still returns zero hits, and every relative link in
  the nine trees still resolves.

## Open Questions

(none)

## Out of scope

- `AGENTS.md`, `packages/bee/AGENTS.block.md`, `skills/bee-*` (D4). Shipping this wiring
  in the bee plugin, so host repos get it too, is a separate feature behind its own gate.
- Any change to the nine skills' bodies. The port is verified; only frontmatter moves.
- Any new bee CLI verb, hook, or Rust change.
