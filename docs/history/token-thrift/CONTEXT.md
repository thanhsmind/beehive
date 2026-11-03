# Token thrift — Context

**Feature slug:** token-thrift
**Date:** 2026-10-21
**Scope:** Standard
**Domain types:** RUN

## What was asked

On 2026-10-21 the user asked for an analysis of the token-saving methods in
AICoworker (`/mnt/data/Projects/refs/AICoworker`). Then the user asked for
bee strategies that help host projects save tokens, and then said "implement
them in full".

## What bee can change

bee runs inside a host (Claude Code, Codex, Pi). The host builds the request
to the model. bee cannot trim or drop history that is already in the request.
bee controls only the text it injects and the text its commands print. So
"save tokens" here means: inject less, keep the injection readable by the
host, and print less.

## What the scout found

1. The session preamble on this repo is 11,041 to 11,161 characters. Claude
   Code keeps hook output above 10,000 characters out of the context. It
   saves the output to a file and shows a 2 KB preview. So most of the
   preamble never reaches the agent. The current budget
   (`PREAMBLE_BUDGET_BYTES = 14336`) is above that host limit.
2. Nine "Open discovery map(s)" lines name maps with 0 frontier tickets. This
   is a list that grows with the store, which the preamble budget rule
   forbids.
3. The dispatch door prints one line per configured role (20 lines) and cuts
   each line at 60 characters, so the herding agent names end in `…`.
4. `bee orient` prints one blocker line that names every feature with an
   unapplied promote proposal (74 features on this repo).
5. Already in place, no change needed: the per-turn reminder is deduped by
   hash with a 30-minute re-inject (`hooks/prompt_context.rs` `should_inject`);
   per-turn text is appended, so the cached prefix stays the same; each cell
   runs in a fresh dispatched worker.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `PREAMBLE_BUDGET_BYTES` drops to 9,500. A named constant records the Claude Code inline limit (10,000 characters), and a test asserts the budget stays below it. | Bytes are never fewer than characters, so a byte budget under the limit keeps the whole preamble inline. |
| D2 | When the assembled preamble is over budget, the renderer drops whole optional sections in this fixed order until it fits: Recent decisions, Critical patterns (digest), Command surface, Project map. It then adds one line before the closing trailer that names the dropped sections and how to fetch them (`bee status --json`, `bee --help --names`). The header, gates, HANDOFF, debt counts, dispatch door, knowledge context and discovery maps are never dropped. | A cap must hold for any store, and these four sections can be fetched again with one command. |
| D3 | In the preamble, a discovery map with 0 frontier tickets no longer gets its own line. All such maps fold into one count line that points to `bee discovery list`. A map with frontier tickets keeps its own line. | The count is the signal. A map with nothing to resume is not something to act on. |
| D4 | The dispatch door prints one line per distinct rendered target, listing every role that resolves to it, in full and without the 60-character cut. The compaction re-inject reads the same function. | Fewer lines, and the agent names are no longer cut. |
| D5 | The `bee orient` promote-proposal blocker names at most 3 features with their clear commands, then `+N more`. | A number that grows, not a list that grows. |
| D6 | A knowledge concept records the token economy of the injected context: the host inline limit, the budget and drop order, the reminder dedupe, append-only injection, and a fresh worker per cell. Under decision `8f63adb4` (budget-fence-removal D1), budgets apply only to generated payloads, never to authored instruction text (skills, AGENTS.md). | One home for why these limits exist. |

## Out of Scope

- Trimming or dropping host history. The host owns the request.
- Wait timing tuned to the cache TTL. The host owns its wakeup tool, and the
  TTL differs per host and plan.
- Byte budgets on skills or AGENTS.md. Decision `8f63adb4` forbids this.
- RTK-style compression of shell output. The user excluded it.
