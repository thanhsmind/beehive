# Catch it higher: static checks, rule landing, counted corrections, prose load — Context

**Feature slug:** trust-layers
**Date:** 2026-11-13
**Shaping session:** complete (owner asked for an assessment of the five-layer trust pyramid against bee, then said "do all of it")
**Scope:** Standard
**Domain types:** RUN | READ

## Feature Boundary

The owner showed a five-layer model for trusting agent work: codebase,
static analysis, verification, prose, human review. Catch a mistake at
the highest layer that can hold it, and when the same comment comes
twice, move the rule one layer up. bee already records the layer that
stops each mistake (`--fix-at`) and files a backlog row for the
mechanical ones. Four parts are missing for a project that uses bee, and
this feature adds them: onboarding offers the project's own static checks
(D1), a craft page says how a check row becomes a static rule in the
project's own tool (D2), a correction from the user or from a review is
counted with the agent's own reflections (D3), and the size of the prose
layer is reported (D4). The feature ends there: it writes no CI file,
installs no linter, and adds no refusal.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Onboarding detects a host project's static checks (lint, type check) from its own files and proposes them as members of the `commands.test` list, placed before the tests. No new `commands` key. Propose-only: bee never writes a detected command. Three cases. (a) No command recorded and a static check found: the existing "No standard commands recorded" notice also lists the static checks and says to offer them first in the `commands.test` list. (b) `commands.test` recorded, and a detected static check appears in none of its entries: one notice lists the missing checks and says to offer them, after a look through the recorded decisions for an earlier refusal. (c) `commands.test` is the `none` sentinel, or nothing is detected: no new text, and every existing notice is byte-identical to today. | `commands.test` already takes a list, and `commands.verify` was retired in 2.1.0; a second key would bring that split back. A recorded refusal is how the no-test offer already goes quiet. |
| D2 | One craft page, `expertise/static-rules.md`, says how a `fix-at: check` row becomes a rule in the project's own tool. Order of preference: a type the compiler enforces, a rule in the linter the project already runs, a test, a hook or guard. It names the usual rule homes per ecosystem and keeps the existing proof: the new rule shown red on the past mistake, then green. `expertise/INDEX.md` routes to it and step 2 of the promotion tree points to it. `FIX_AT_VALUES` stays the four words. | The sub-kind of a check already rides in `--better`; a locked decision keeps the four-value set. |
| D3 | `bee mailbox reflect` takes an optional `--source` from the closed set `self`, `user`, `review`; default `self`. A value outside the set refuses and lists the three. The stored entry carries `source` only when it is not `self`, so every existing row and every default call stays byte-identical. The letter marks a `user` or `review` reflection as such. The lesson key stays fix-at plus the first four normalized words of the what, so the three sources fold into one count. Doctrine: a correction from the user about how the agent worked, and an accepted review finding, are each recorded as a reflection with that source, at the moment they arrive. | The second-sighting rule counts only what the agent says about itself. The same owner comment twice is the signal the pyramid names, and it is not counted today. |
| D4 | `bee feedback digest` reports the prose load as a witness: the line count of each always-loaded instruction file that exists (`AGENTS.md`, `CLAUDE.md`, `.bee/LEADER.md`, `LEADER.md`) and their total, the count of critical patterns in `docs/knowledge/index.md` (0 when the file or section is absent), and the count of `fix-at:check` and `fix-at:architecture` rows in `.bee/backlog.jsonl`. The numbers go into the digest object and onto one printed line. It refuses nothing, drops nothing, scores nothing. | Same posture as the clean-run ratio: a number that is measured, never a guard that cannot pass. |
| C1 | Contract static-check-offer: a detector takes the repo root and returns the static check commands found in the project's own files, each with its source file; it reads files and runs no tool. `commands_notices` adds text only in cases (a) and (b) of D1. Proving tests carry `static_check` in their names. | |
| C2 | Contract reflect-source: `--source` absent or `self` stores no `source` key; `user` or `review` stores it and the letter bullet names it; any other value exits non-zero, appends nothing and lists the three; `--source` with `--no-mistakes` is refused; the lesson key ignores it. Proving tests carry `reflect_source` in their names. | |
| C3 | Contract prose-load: the digest object gains one `prose_load` object with the per-file line counts, their total, the critical pattern count and the two fix-at row counts of D4; the printed summary gains one line. A missing file or section counts as zero and never fails the digest. Proving tests carry `prose_load` in their names. | |
| D5 | Refines D1 and C1 after the plan check. A test command counts as recorded when `commands.test` is a non-empty string or a non-empty list; a list no longer draws the "No standard commands recorded" notice. A check is already present when any `commands.test` entry contains its marker word (the script name, or clippy, ruff, mypy, vet). At most one candidate per kind (lint, type check); a manifest script beats a convention; the script runner follows the lockfile. The notice says to run each check once and report green or the finding count before offering, and to record the owner's answer per check as a decision. With only setup or start recorded the function returns nothing, as before. | A list read as not recorded, so an accepted offer would repeat on every onboard; a whole-command match gives false offers; a red check on day one would turn CI red. |
| D6 | Refines D3 and C2. `source` is an optional field on the stored entry and on the letter item, written only when it is `user` or `review`. The letter says "your correction" and "from a review". Values are lower-case and exact. The doctrine line lives in the leader file only, covers accepted P1 and P2 findings, keeps the owner's own words in the what, and separates a correction from a new instruction. No sentence goes into the reviewing skill. | The entry must hold every fact the letter prints; the lesson key reads the first four words. |
| D7 | Refines D4 and C3. `prose_load` is a top-level key of the digest object; `bee feedback count` does not change. An absent source reads as null, never zero. The lines bee ships (`.bee/LEADER.md`) are apart from the lines the project owns (`AGENTS.md`, `CLAUDE.md`, `LEADER.md`); a symbolic link to a counted file counts once. Critical patterns are the top-level list lines of the section. Fix-at rows are counted raw, as rows filed. The printed digest sentence gains one clause. | The existing witness uses null for not measured; backlog rows carry no status. |

### Agent's Discretion

- Which files mark each static check, provided the detection reads only the project's own manifest and config files and never runs a tool.
- The exact wording of the two notices, provided it follows the posture of the existing offers: say what was found, ask, never assume a yes.
- The key names inside the digest object.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| static check | A command that reads code without running the product: a linter or a type checker. |
| prose load | The always-loaded instruction text an agent reads at session start, plus the critical pattern digest. |
| witness | A reported number that never refuses work. |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/human-mailbox/overview.md | suspect | Holds the reflection entry, the fix-at layer, the lesson key and the "a mistake seen twice" rule that D3 and D4 extend. |
| docs/knowledge/areas/doctrine-layer/overview.md | suspect | The doctrine layer whose size D4 reports. |
| docs/knowledge/patterns/20260713-promote-an-order-to-the-always-loaded-layer.md | pattern | D3 adds a line to the always-loaded leader file; its transport must ride along. |
| docs/knowledge/patterns/20260825-a-guard-that-cannot-pass-teaches-agents-to-ack-it.md | pattern | Why D1 is an offer and D4 is a witness. |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/onboard/notices.rs` | `detect_commands` and `commands_notices`: what onboarding finds and proposes. |
| `packages/bee-rs/crates/bee/src/onboard/templates.rs` | `COMMAND_KEYS` and the offer texts. |
| `packages/bee-rs/crates/bee/src/verbs/mailbox.rs` | The `reflect` verb, `FIX_AT_VALUES`, the stored entry. |
| `packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs` | `reflection_key` and the letter render. |
| `packages/bee-rs/crates/bee/src/verbs/feedback.rs` | The digest and its clean-run witness. |
| `skills/bee-capturing/references/promotion.md` | The promotion decision tree. |
| `packages/bee/LEADER.md` | The source of the leader doctrine line on reflections. |

## Existing Code Context

### Reusable Assets

- `detect_commands` in `onboard/notices.rs` — reads `package.json`, `Makefile`, `composer.json` and convention markers; the static detector reuses its readers.
- `NO_TEST_VERIFICATION_OFFER` in `onboard/templates.rs` — the offer posture and the recorded-refusal opt-out.
- The clean-run counts in `verbs/feedback.rs` — the witness shape D4 copies.

### Established Patterns

- A notice proposes; only a confirmed value is written to `.bee/config.json`.
- A new flag is registered in `packages/bee-rs/crates/bee/src/generated/registry_payload.json`.
- Vendored skill and doctrine trees change only through `bee dev regen`.

## Outstanding Questions

### Resolve Before Planning

(none)

### Deferred To Planning

(none)

## Deferred Ideas

- A CI workflow file that bee writes for a host project — bee does not write a host's CI today, and a host's CI is its owner's choice.
- A `fix_at` field on review findings — D3 counts them through the reflection key; a second counter would split the count.
- A hard line budget for the prose layer — a guard that cannot pass teaches agents to ack it.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, the read set, code context, canonical references, and deferred-to-planning questions.
