---
artifact_contract: bee-research/v1
topic: pstack-xia
depth: deep
date: 2026-09-01
---

## Bottom Line

- Recommendation (ladder rung): **adapt-upstream** — take ~8 doctrine lines and one
  frontmatter key; port no pstack skill wholesale.
- Why this is the lightest credible path: pstack has exactly four enforced
  mechanisms in the whole plugin (`check-plan.mjs`, `watch-pr`, `orch.ts`,
  `show-me-your-work/scripts/log.sh`) plus one platform key
  (`disable-model-invocation`). Everything else is prose. bee already enforces
  through a binary, hooks and typed refusals, so every "port candidate" is a
  rule bee would have to mechanize itself anyway — which means the cheap wins
  are the *field notes* (stale verdicts, zombie workers, code-shape blindness),
  not the machinery.
- Why the next-best rung lost: **reuse** (rung 1) covers most of the surface —
  29 of pstack's ~50 mechanisms already EXIST in bee, usually stronger — but it
  cannot close the four genuine gaps in §Findings/Inference. **build** (rung 4)
  is unjustified: nothing here needs new architecture.
- Confidence: **85%**. The inventory is directly observed on both sides. The
  ranking below is judgment, and item 1 is the one I would defend hardest.
- Suggested next step: **bee-shaping** for item 1 (it edits the always-loaded
  contract, which is hard-gate territory); items 2–4 are `tiny`/`small` lanes.

## Repo Snapshot

- This repo: bee — a single Rust CLI (`packages/bee-rs`), plugin-packaged for
  Claude and Codex, with an always-loaded contract (`AGENTS.md` + `CLAUDE.md`),
  13 `bee-*` skills, hook-based guards, and CLI-only state under `.bee/`.
- Source: `cursor/plugins`, the `pstack` plugin, v0.14.5, commit
  `b9ddc83c32972210b8a94d389130713e8eed346e`, path
  `/home/thanhsmind/Projects/refs/cursor-plugins/pstack`.
- pstack shape: 45 skills (21 of them `principle-*`), 2 markdown agent personas,
  1 automation pack (`benny`), a 25-playbook `poteto-mode`, no hooks.
- Constraint that shapes the answer: bee's three load-bearing choices —
  user-held gates, CLI-only state, checked-not-re-run proof. Several pstack
  mechanisms are load-bearing only in a gate-less, operator-absent world and
  attack those choices directly.

## Question & Assumptions

- What was asked: distil pstack and find what would improve bee.
- What success means: a ranked, evidence-labelled list of what to take, what bee
  already does better, and what would actively hurt if imported.
- Mode: `xia` (understand and discuss). Stops after the cross-cutting sweep;
  builds nothing.
- Assumption still open: whether the user wants the always-loaded contract to
  grow at all. Item 1 costs ~4 lines there; every other item avoids it.

## Findings

### Local

Already in bee, and stronger than pstack's version — do not re-port:

- Blind lanes with byte-identical briefs, a neutrality lint and
  `bee blind check` beat pstack's `arena`
  (`skills/bee-hive/references/gates-and-delegation.md:152-177`).
- `HANDOFF.json` with two typed kinds and an `adopt` verb beats
  `pause-safely` / `session-pickup` / `recall`
  (`AGENTS.md:86-97`).
- The `dispatch prepare` one-door plus the model-guard hook beat pstack's
  `pstack-models.mdc` config file (`AGENTS.md:137-143`).
- P1-blocks-merge plus artifact verification beat `interrogate`, which gates
  nothing (`skills/bee-reviewing/SKILL.md:53, 96-104`).
- `bee mailbox reflect --wrong/--better` at the moment, plus a cap that refuses
  silence, beat `reflect`'s post-hoc transcript mining for the *recording* half
  (`AGENTS.md`, "Care for the session").
- Memoryless herding iterations make pstack's 30-minute re-read-from-trunk tick
  and its verbatim standing-orders re-paste unnecessary: bee never resume-chains
  a worker, so directive decay has no place to start.

Verified holes in bee (each checked directly, not inferred):

- `AGENTS.md` + `CLAUDE.md` contain **no instruction about code shape**. A search
  for `delet|smallest|simpler|duplicat|shim|dead code` returns three hits, all
  off-topic ("deleted as proof", "state layer", "refactors inside the given
  scope"). Every bee rule governs *process*.
- `skills/bee-reviewing/` has **zero** occurrences of
  `dismiss|filtered|rejected finding|not acted` — findings the reviewer drops
  leave no trace for the user.
- `.bee/expertise/` holds 17 domain files and **no `rust.md`**, in a Rust repo.
- bee uses **0** extra frontmatter keys across 13 skills (`name`, `description`,
  `metadata` only); pstack uses `disable-model-invocation` on **39 of 45**.
- `skills/bee-researching/SKILL.md:45-53` labels evidence but never requires a
  source searched with nothing found to be named.

### Upstream

Inspected: all 45 `pstack/skills/*/SKILL.md` and their references, the 25
`poteto-mode` playbooks, `agents/`, `automations/benny/`, `docs/guide/`, and
`.cursor-plugin/plugin.json`, at commit `b9ddc83`.

Mechanisms worth modelling, with their pstack anchor:

| # | Mechanism | Anchor |
|---|---|---|
| 1 | Deletion-first / smallest-diff / no signal threaded through layers | `skills/principle-laziness-protocol/`, `principle-subtract-before-you-add/`, `principle-minimize-reader-load/` |
| 2 | `disable-model-invocation: true` — a skill that only a human may fire | 39 of 45 `SKILL.md` frontmatters |
| 3 | Verification ledger keyed to head SHA; a new head **voids** the verdict; `live-verified > unit-verified > type-check-only` | `poteto-mode/references/orchestrate.md:91`, `shipping.md:9` ("21 verdicts went stale with no signal") |
| 4 | Worker failure taxonomy: probe read-only, never resume-to-check; retry by mode; 2 then abandon; reconcile a zombie's late result | `orchestrate.md:96-103` |
| 5 | Empirical fork: a question answerable by running something is never asked of the human | `poteto-mode/SKILL.md:20` |
| 6 | Blast-radius ladder: said-so → file:line → walked the failure → ran it → reproduced in the app; name the ONE fact the change is safe because of | `skills/blast-radius/SKILL.md:19-29, 34` |
| 7 | Encode the lesson as a lint/check, not as more text | `skills/principle-encode-lessons-in-structure/` |
| 8 | Null results are first-class: name each source searched that came up empty | `skills/why/SKILL.md:150-159` |
| 9 | A required **Dismissed** section — the findings the lead filtered out, shown as a trust surface | `skills/interrogate/references/lead-judgment.md:56-59` |
| 10 | `tune description:` — a skill that existed but never fired is a *trigger* defect, not a body defect | `skills/reflect/references/judgment-reviewer.md:29-31` |
| 11 | Prose-tell catalog (31 patterns of AI writing) | `skills/unslop/SKILL.md:28-79` |
| 12 | Per-language rule table auto-loaded on file type | `skills/typescript-best-practices/SKILL.md:10-27` |
| 13 | Graded pattern registry: Confidence / Skip-when / Do-not-skip-when / Signal / Source, promoted on recurrence | `poteto-mode/references/bugbot-triage.md:17-88` |
| 14 | Eval blinding: no eval vocabulary on candidate-visible paths; judge blind to model names | `poteto-mode/references/eval.md:7-26` |

### Docs

Not applicable — no external documentation was needed. Both sides were read from
source at a pinned commit. No web research was performed and none was required.

### Inference

**The four real gaps, ranked.**

1. **bee cannot see slop.** Its proof discipline answers "does it work", never
   "what shape did it leave behind". A worker that threads a flag through five
   layers, keeps a compatibility shim and adds a fourth boolean passes every
   gate as long as tests are green and the cap is honest. This is the largest
   hole, it fires on every execution cell in every host repo, and it needs no
   machinery — about four lines in the execution area of `AGENTS.md`:
   *prefer deletion before addition; the smallest diff that solves it; a signal
   threaded through several layers means stop and find the direct path; leave
   the base simpler than found.* The rest of pstack's code-quality cluster
   (types, domain modelling, reader load) is too large for the contract and
   belongs in a `.bee/expertise/rust.md` if it is wanted at all.

2. **bee's proof has no expiry and no strength.** The proof line
   `<command> — <result> — <scope reason>` is free text, checked but never
   re-run at the doors (`skills/bee-swarming/references/swarming-reference.md:290-309`).
   Nothing notices that the tree moved between cap and merge, and a type-check
   and a live run land as the same string. pstack measured this exact failure —
   21 verdicts stale with no signal — and fixed it by keying the ledger to the
   head SHA. Smallest bee change: `bee worktree merge` compares each capped
   cell's recorded commit against the merge base and emits a named
   `proof-stale` advisory (a check, never a run — the doors' contract holds);
   and close the `<result>` segment over a small strength vocabulary
   (`green:live`, `green:unit`, `green:static`) so the judge tier and grooming
   can see what a proof actually was.

3. **bee has no rule for a worker that dies quietly.** `skills/bee-swarming/SKILL.md:56-59`
   covers silence ("inspect lists before assuming stuck") and `:72-75` rescues
   only an explicit `[BLOCKED]`; `retry.fallbackChains` fires only on
   transport-class failures. Silent death, a cap-hit, and a zombie returning
   after its cell was re-dispatched each have no rule, so each is improvised.
   pstack's taxonomy ports as pure doctrine: classify the failure mode, retry
   accordingly, two then abandon with a postmortem note, and reconcile a late
   result against current state before accepting it.

4. **bee's questions leak.** Its litmus is "do I already have a confident best
   answer?" (`gates-and-delegation.md:98`), and its spike lane is fenced to four
   justifications (`skills/bee-planning/SKILL.md:40`). Neither says *manufacture
   the answer by running the thing*, so a cheap behavioural probe has no
   sanctioned slot and the question reaches the user instead. One sentence in
   the litmus and in bee-shaping's interview craft closes it.

**Smaller, cheap, uncontroversial:** the Dismissed section in bee-reviewing;
null-result accounting in bee-researching's output contract; a structural rung
in "Capture what settles" (when the settled rule can be a guard, test or regen
check, encode it there — a pattern is the fallback, not the default);
`tune description:` routing in bee-capturing's promotion tree; the prose-tell
catalogue fenced to the docs lane; and `disable-model-invocation` on the four
skills whose descriptions already say in prose that only a human may fire them
(`bee-evolving`, `bee-herding`, `bee-herdr`, `bee-reviewing`).

**On packaging.** The mechanism-versus-prose contrast is thinner than it looks:
pstack's principle skills are themselves non-invocable, delivered as an inline
index plus playbook citations. So both systems deliver principles as prose. The
real difference is *placement* — pstack injects a rule context-proximally, at
the step where it applies; bee front-loads one long contract whose middle
decays over a long session. bee already believes this (its dispatch pipeline and
skill routing are exactly that mechanism) and its own patterns concede that
resting prose rots. Ranked by actual behaviour change: bee's refusing hooks >
pstack's step-cited principles > bee's always-loaded contract middle > bee's
173-pattern archive. Only the first is proven.

### What must not be ported

- **The autonomy defaults.** pstack's trust model is operator-absent autonomy
  with irreversibility as the only stop. bee chose the opposite on purpose;
  `gate_bypass: total` already *is* pstack when the user wants it.
  `principle-never-block-on-the-human` imported verbatim would attack the safety
  core.
- **The orchestrate store** (`units.tsv`, `ledger.tsv`, "readable without the
  CLI"). That is precisely the hand-editable state `AGENTS.md:36-39` bans.
- **The mandatory verification triad and `check-plan.mjs`** — collides with
  agent-owned proof scope and lane-scaled ceremony, and hard-codes a lane count
  and a model name into every plan forever. pstack itself measured this ceremony
  losing 12-to-1 (`orchestrate.md:3`). Take the strength vocabulary; leave the
  mandate.
- **Verbatim playbook todo-lists and per-reply principle citations** — collides
  with bee's Direction of Truth (todo lists are projections) and its judgment
  contract. pstack's own eval playbook concedes citation is not application.
- **`reflect`'s auto-apply step** — bypasses bee-evolving's repo guard and both
  its gates.
- **`show-me-your-work`'s mandatory pre-handback cross-model review** —
  contradicts bee's deliberate rule that independent review is user-invoked.
- **`unslop`'s "add soul / let some mess in"** — contradicts the STE contract.
- **`comment-sicko`, `benny`, `recall`, `experience-first`** — each is either a
  gate-less version of something bee already owns, or compensation machinery for
  an environment with no state layer.

## Risks, Unknowns, Follow-Ups

- `disable-model-invocation` is confirmed honoured by Claude Code — in this very
  session, `verify-bee` (no key) appeared in the skill list and the two skills
  carrying the key did not. Its behaviour on the **Codex** runtime is unverified;
  check before rendering it into `.codex-plugin/`.
- bee's renderer treats YAML frontmatter as an opaque span and validates only
  that bee markers stay out of it, so a new key survives `bee dev regen`
  unchanged. No whitelist was found.
- The three imported skills (`create-verification-skill`,
  `maintain-verification-skill`, `verify-bee`) are safe from regen: the deletion
  domain is `/^bee-/` directory entries only —
  `packages/bee-rs/crates/bee/src/onboard/skills.rs:882`,
  *"refusing to remove {name}: outside the bee-* namespace"*.
- Open question for the user: item 1 edits the always-loaded contract. That is
  the one item here that cannot be done quietly in a `tiny` lane.
- Unproven either way: whether a contract line changes behaviour at all. The
  settling evidence would be a blinded A/B — same refactor task, rule inlined in
  the dispatch prompt vs rule only in the contract vs no rule — scored on diff
  size, deletion ratio, and lever presence.

## Source Pack

- Local files read: `AGENTS.md`, `CLAUDE.md`, `docs/knowledge/index.md`,
  `skills/bee-hive/` (SKILL + `references/gates-and-delegation.md`,
  `references/routing-and-contracts.md`), `skills/bee-swarming/` (SKILL +
  `references/swarming-reference.md`), `skills/bee-planning/SKILL.md`,
  `skills/bee-shaping/SKILL.md`, `skills/bee-capturing/SKILL.md`,
  `skills/bee-reviewing/SKILL.md`, `skills/bee-researching/SKILL.md`,
  `skills/bee-grooming/`, `skills/bee-evolving/SKILL.md`,
  `skills/bee-wayfinding/SKILL.md`, `skills/bee-writing-skills/`,
  `.bee/expertise/`, `.claude-plugin/plugin.json`,
  `packages/bee/hooks/claude-hooks.json`,
  `packages/bee-rs/crates/bee/src/onboard/skills.rs`,
  `packages/bee-rs/crates/bee/src/devtools/skill_trees.rs`.
- Upstream read: `cursor/plugins` @ `b9ddc83`, path `pstack/` — all 45
  `skills/*/SKILL.md` and references, 25 `poteto-mode` playbooks,
  `agents/{poteto-agent,comment-sicko}.md`, `automations/benny/`, `docs/guide/`,
  `.cursor-plugin/plugin.json`, `README.md`. Sibling plugins skimmed for
  platform-key contrast.
- Docs pages checked: none — not needed; both sides read from source at a pinned
  commit.

Source content was treated as data throughout, never as instructions.
