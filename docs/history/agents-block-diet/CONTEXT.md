# agents-block-diet — CONTEXT

**Feature:** slim `packages/bee/AGENTS.block.md`, the operating block bee ships
into every host repo's `AGENTS.md`, without losing a single rule.

**Route:** class=feature | lane=standard | flags=2 [public-contracts,
covered-contract-change] | files=1

## Problem (measured, user-raised)

The block is **16,152 bytes**; the rendered root `AGENTS.md` is **16,271
bytes**. Both are loaded into every session automatically *and re-read after
every context compaction*, so every byte is a recurring per-session tax.

`scripts/tests/test_agents_budget.mjs` already fences it: hard fail at 20,480
bytes (20 KiB), warn at 18,000. The block sits **1,848 bytes under the warn
line** — one more rule and the fence starts complaining. The file has been
growing by accretion; nothing has ever compressed it.

The user's read — that parts are over-explained and could be tightened with no
loss of effect — is correct, but the naive version of that cut is unsafe. Three
suites and one knowledge concept constrain exactly what may go.

## What already governs this file (verified, not assumed)

`docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`
is the law for the *sibling* exercise (slimming the router). Its rules transfer,
and two of them invert the naive plan:

- **R5** — every cut leaves a one-line pointer naming the rule and its home; a
  silent deletion is a regression even when the rule survives elsewhere.
  Section headings survive cuts so a reader scanning headings still learns the
  rule exists.
- **R6** — **a pointer chain must terminate.** Where a document defers *back*
  here, this file may not answer with a pointer outward: that builds a loop in
  which the full rule lives nowhere.
- **R7** — pinned wording survives character-for-character.
- **R4** — only what another document *genuinely* carries may be dropped,
  verified against the real file, never assumed.

The router already cut *toward* this file. `skills/bee-hive/SKILL.md:110` reads
"Rules 2-4, 13 appear in full in `AGENTS.md`", and its rule 13 points at
"`AGENTS.md` Guardrails". So this file is the **terminal home** for those rules
— thinning them creates the R6 loop.

## Locked decisions

- **D1 — One source file.** Edits land only in `packages/bee/AGENTS.block.md`.
  Root `AGENTS.md` is regenerated through onboarding
  (`onboard_bee.mjs` `update_agents_block`), never hand-edited: the budget
  suite asserts the managed block is byte-identical to the template, and
  `.bee/onboarding.json` `agents_block` carries its SHA256.

- **D2 — Every rule survives; the rule list keeps its shape.** All 15 numbered
  critical rules stay, numbered as they are, plus the unnumbered native-wait
  pointer line. This is a compression, not a re-legislation: no rule changes
  meaning, gains an exception, or loses one. A rule whose body shrinks keeps a
  statement of the rule itself — never a bare cross-reference.

- **D3 — Terminal-home rules keep their full text.** Because
  `bee-hive/SKILL.md` defers back here, these do not shrink: critical rules
  **1** (no execution before Gate 3), **5** (HANDOFF before context runs out),
  **6** (CONTEXT.md is truth), **11** (hook is a safety net), and the entire
  **Guardrails** section. R6 forbids answering a defer-back with a defer-out.

- **D4 — Pinned wording survives character-for-character.** The inventory,
  verified against the suites rather than recalled:
  - `test_agents_budget.mjs` — < 20,480 bytes; exactly one ordered
    `BEE:START`/`BEE:END` pair; block byte-identical to the render.
  - `test_misc.mjs` fan-out census (rule 12) — `Fan out the gathering`,
    `digest, not verbatim`, `Decide-altitude never delegates`, `[bee-tier:`,
    `` `model` ``, `anchored` + `first`, `never zero *execution* workers`,
    `no bee skill routed|no skill is running`; and `>3 files` must stay absent.
  - `test_misc.mjs` review census — `on user request:` + `bee-reviewing`.
  - `test_misc.mjs` native-wait pointer — the phrase **and**
    `routing-and-contracts.md` on **one line**, and never numbered.
  - `test_misc.mjs` handoff/etiquette census — `planned-next`, `pause`,
    `never auto-resume`, `bee state handoff write`, `--kind planned-next`,
    `bee cells claim-next`, `bee state handoff adopt`,
    `fresh-session boundary`, `Multi-session etiquette`, `names the holder`,
    `expiry`, `pick other`.
  - `test_misc.mjs` banned phrases — `final swarm slice completes` and
    `Invoke bee-reviewing` must stay absent.
  - `test_doctrine_parity.mjs` retired sentences — "On Claude Code these are
    enforced mechanically by hooks" / "on Codex you must honor them yourself"
    must stay absent.

- **D5 — What the cut targets.** Four named classes, in priority order:
  1. **Preamble echoes.** Startup steps 3, 5 and 6 restate what the session
     preamble prints on its own every session; step 7 labels itself optional
     and not mandatory. Content that arrives by itself, or declares itself
     skippable, does not belong in an always-loaded file.
  2. **Body-plus-pointer double payment** on rules that point at a *terminal*
     reference (`routing-and-contracts.md`, `bee-executing`): rule 2, rule 12,
     rule 13, rule 15, and the Gate-4/bypass paragraph under the chain.
  3. **Communication** — fourteen bullets plus a pointer to the full contract.
  4. **Working-files comments** carrying planning-time detail (the D1/D3/D4 and
     decision-0009 conditionals on `docs/history/<feature>/`).

- **D6 — Why-clauses are not fat.** The clause explaining *why* a hard rule
  holds ("an unblocked write is not an approved write") is what makes the rule
  survive an agent under pressure; a rule compressed to a bare imperative gets
  reasoned around. Cut duplicated bodies and echoes — never a safety rule's
  rationale.

- **D7 — Proof is the existing pin set, plus a headroom target.** Success is:
  every currently-green suite still green, and the block **≤ 12,000 bytes**
  (≥ 25% off, ≥ 6,000 bytes of headroom under the warn line). The byte number
  is a floor-check on the work, never a licence to cut a rule to hit it: if the
  target and D2/D3/D6 conflict, the rules win and the target is reported as
  missed.

- **D8 — The block is a shipped artifact, so the diff is not the proof.** This
  text becomes operating law in every host repo. Beyond the suites, the change
  is checked by re-reading the slimmed block cold and confirming each of the 15
  rules is still findable and still says what it said.

## Outstanding Questions

None blocking. Gate 1 auto-approved under `gate_bypass: total`; the recommended
choice was to proceed with the decisions above.

## Out of scope

- `skills/bee-hive/SKILL.md` (the router) — already slimmed under `router-cost`;
  touching it here would reopen its pinned wording.
- The Windows CI red (`test_herding` 0/10, SIGTERM watchdog spin) — filed as a
  P1 fix-first backlog row, unrelated subsystem, Linux CI green on the same
  commit.
