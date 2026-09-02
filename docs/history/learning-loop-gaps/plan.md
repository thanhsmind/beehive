---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset until approval>
---

# Plan: learning-loop-gaps

Mode: `standard` — 2 risk flags: public-contracts (the offer a user is shown,
and a knowledge-area rule), covered-contract-change (a shipped skill line that
today cites a command that does not exist).
Why this is the least workflow that protects the work: the change is two skill
edits and a knowledge sync, but it changes what a user is offered and what a
promotion does, and the first draft of this same feature was wrong in a way only
a wave caught. No code changes; no cell touches `packages/`.

## Requirements (from CONTEXT.md)
- D1: no verb; `sweep-recovery-door` D3 stands whole.
- D2/D3: scout-and-ticks widens to crashed-OR-asked-for; crash path unchanged
  (`status` `recovery.candidates`), asked-for path reads the session record's
  `transcript_path` + `started_at`; both bound at the 256 KB tail.
- D4/D5: both miner prompts live in the skill; the asked-for prompt asks for
  settlements, friction, and routing candidates scoped to skills actually opened.
- D6: downstream unchanged — note under `docs/history/`, `capture add --source
  mined`, data-not-instructions, redaction, never auto-decisions.
- D7: the offer states the pane read and an over-threshold capture queue.
- D8/D9: the skill-was-used idea is a Q3 routing qualifier, not a fourth bar;
  Q4's one-line reason is written and the mechanical owner is filed.
- D10: RED-first per the Iron Law; both cells end with the render step.

## Load-bearing claims
Labels: `read` (opened that file at that line), `ran` (executed it, hold the output).
Evidence is a verbatim byte substring of the anchored line(s).

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The skill today tells the agent to mine with a command that does not exist | read | `.claude/skills/bee-hive/references/scout-and-ticks.md:95` | `dispatch one down-tier worker with the code-generated \`recovery window\` prompt` |
| 2 | `recovery window` is unbuilt and the binary says so | ran | `.bee/bin/bee recovery window --help` | `NOT BUILT INTO THIS BINARY — the recovery group was never ported off Node.` |
| 3 | `sweep-recovery-door` D3 keeps it unbuilt — the decision D1 declines to touch | read | `docs/history/sweep-recovery-door/CONTEXT.md:35` | `` `bee recovery window` stays unbuilt and keeps its registry marker. `` |
| 4 | The session record carries the transcript path, written by the hook | read | `packages/bee-rs/crates/bee/src/hooks/activity.rs:458` | `transcript_path` |
| 5 | The only transcript bound the binary applies is a 256 KB tail | read | `packages/bee-rs/crates/bee/src/verbs/status_full/mod.rs:143` | `const DEFAULT_TAIL_MAX_BYTES: u64 = 262144;` |
| 6 | The crash path's window start is computed from every decision in the repo, unfiltered by lane | read | `packages/bee-rs/crates/bee/src/verbs/status_full/recovery.rs:204-207` | `pub(crate) fn last_durable_settlement(` / `lane: Option<&Value>,` / `decisions: &[Value],` / `capture_events: &[Value],` |
| 7 | …and the result is broken in the live store: two candidates dead in August get a September window start | ran | `bee status --json` → `recovery.candidates` | `49e28775 \| lane compound-release-knowledge \| hb 2026-08-25T05:41:57.859Z \| since 2026-09-02T06:16:58.963Z` |
| 8 | …which is exactly the newest decision in the repo | ran | `tail -1 .bee/decisions.jsonl` | `2026-09-02T06:16:58.963Z 2df5f472` |
| 9 | Mining is already offered-not-forced, digest-only, redacted, workspace-fenced — nothing to re-invent | read | `docs/knowledge/areas/workflow-state/recovery.md:41-43` | `Recovery itself never runs automatically:` / `when candidates are found the agent offers to recover them, the same way it` / `offers to drain a pending capture queue, and acts only if the human agrees.` |
| 10 | Candidate settlements already land as mined stubs a human confirms at flush | read | `docs/knowledge/areas/workflow-state/recovery.md:51-53` | `Candidate settlements are appended as capture stubs` / `marked as mined rather than confirmed; they become real knowledge only after the` / `normal capture-flush review a person already performs, never automatically.` |
| 11 | The three promotion bars are filters that reject; the body states them | read | `.claude/skills/bee-capturing/SKILL.md:123-124` | `4. Promote a learning only when it clears all three bars:` / `   multi-feature relevance, meaningful waste prevented, generalizable.` |
| 12 | Q3 is the "promote as prose" branch — the slot the qualifier belongs on | read | `.claude/skills/bee-capturing/references/promotion.md:118-119` | `3. Not mechanizable (judgment, taste, product intent) → promote as` / `   prose per the format below.` |
| 13 | Q4 already requires a one-line reason when prose survives instead of a mechanism (D9's home) | read | `.claude/skills/bee-capturing/references/promotion.md:124-127` | `→ that durable owner is the promotion, filed as a tiny/small cell` / `when it cannot ship in-feature. No → prose survives only with a` / `one-line recorded reason (in the learnings file or a decision log` / `line) naming why no mechanical owner exists yet.` |
| 17 | The bar count is stated in two places and both must move together | read | `.claude/skills/bee-capturing/references/promotion.md:111` | `AND it clears all three promotion criteria — multi-feature` |
| 14 | The Regrowth law puts a new learning in references by default, the body only for a load-bearing invariant | read | `.claude/skills/bee-writing-skills/SKILL.md:44` | `**Regrowth law:** a new learning lands in the knowledge bundle or` |
| 15 | This host's read slot is an external pane, so D7's disclosure is not hypothetical | ran | `bee models show` | `read              [configured]  {"kind":"herding","agent":"agy-flash"}` |
| 16 | The capture queue is already past its blocker threshold, so D7's second disclosure fires today | ran | `bee orient` | `capture queue: 50 pending stub(s)` |

## Discovery

Ran the four `dispatch prepare` shapes and `bee status --json` against the live
store; read the recovery resolver, the registry payload's `recovery.window`
entry, both target skills, and the two prior pstack briefs. The hat wave
(`reports/hat-wave.md`) rejected the first draft from both sides and the
`since` check (claims 6–8) settled it: the verb would have inherited a broken
window start. Findings that survived became D1–D10; the defect became its own
P2 backlog row rather than a silent fix inside a widening.

## Approach

Recommended: deliver both follow-ups in the skill layer, reading only facts that
already ship (D1–D10). Rejected: (a) build `bee recovery window` — 7 blockers,
11 surfaces asserting it unbuilt, and it blesses the broken `since`; (b) one
fused prompt for both purposes — the recover digest's shape is pinned by
`recovery.md` B33 and fusing it costs a second supersede for no gain; (c) a
fourth promotion bar — the three bars reject, this one routes, and shelving a
router among filters mis-teaches all four; (d) a mechanical "was this skill
opened" check — nothing in `hooks/` records a Skill invocation, so Q4's recorded
reason is the honest answer today.
Risk map: the widened offer — LOW (never auto-runs; proof is the skill's own
pressure scenarios) · the Q3 qualifier — LOW (authoring-time only, invisible at
runtime) · the render step — LOW (`bee dev regen` clean) · the `since` defect —
MEDIUM but **not this feature's**, filed P2.

## Shape

Playbook: `docs` + `bugfix` (`planning-reference.md` "Class playbooks") — the
skill line at claim 1 is a live defect, so its cell is red-first in the
skill sense: a pressure scenario that follows the current text and reaches for a
command that does not exist.

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 (parallel) | `scout-and-ticks.md` § Crash recovery → both triggers, both prompts inline, the dangling citation fixed; knowledge sync of `recovery.md` R52/B33 | the skill cites a command that does not exist | a pressure run picks the asked-for path and dispatches a real worker | close |
| 1 (parallel) | `promotion.md` Q3 gains the routing qualifier + Q4's recorded reason; `SKILL.md:123` and `promotion.md:111` keep one bar count | the idea is worth having and has no home | a Compound dry-walk routes a never-opened skill to `tune description:` | close |

Current slice: both cells, concurrent — file-disjoint (`bee-hive/**` +
`docs/knowledge/**` vs `bee-capturing/**`). Both run `bee dev regen`; that
invocation serializes, the cells do not.

## Test matrix
- Happy (cell A): a session with a resolvable transcript and no crash → the
  procedure offers mining in one line, the human agrees, one down-tier worker is
  dispatched, a note lands, stubs append `--source mined`.
- Happy (cell B): a finding whose target skill the run never opened → routed
  `tune description: <path>`, not a body edit.
- Edge (A): a crash candidate → today's path, byte-identical, `since` from
  `status` (defect untouched and named).
- Edge (A): the user declines → nothing runs, nothing is written.
- Edge (A): a pane read slot → the offer says the transcript leaves the runtime.
- Edge (A): capture queue over threshold → the offer says so.
- Edge (B): a finding whose target skill WAS opened → body edit, unchanged.
- Edge (B): a mechanizable finding → Q2 still wins; the qualifier never fires.
- Error (A): no resolvable transcript → the procedure says so and offers nothing.
- Proof both: RED pressure scenarios recorded verbatim before any edit (D10);
  after, the same scenarios re-run with the text present; `bee dev regen` clean;
  every anchor cited in the edited files still resolves (`rg` each).

## Open Questions
(none — the wave's blockers are answered by D1–D10 or recorded as Known gaps in
CONTEXT.md; record: `docs/history/learning-loop-gaps/reports/hat-wave.md`)

## Out of scope
- Building `bee recovery window` (D1) and the eleven surfaces that assert it unbuilt.
- Fixing `last_durable_settlement`'s global decision scan — filed P2.
- A named gesture or slash command for the ask (Known gap).
- Stub dedupe, a pane transport rule, and `collect_feedback`'s blind spot.
