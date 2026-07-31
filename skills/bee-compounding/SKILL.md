---
name: bee-compounding
description: >-
  Capture durable learnings and decisions so future work starts smarter. Use when scribing completes, or when work is intentionally abandoned with lessons worth keeping.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: degraded
      reason: Reads cell traces and logs decisions via the vendored .bee/bin helpers.
---

# Compounding (honey)

Captures reusable lessons from completed work into future exploring,
planning, reviewing. Run after `bee-scribing`, or when work is abandoned with
lessons. "The session feels done" is not a reason to skip it.

## 1. Gather Evidence

- CONTEXT.md, plan.md, worker reports (`docs/history/<feature>/reports/`)
- cells + traces: `node .bee/bin/bee.mjs cells list --feature <feature>`
- review findings (residual-findings.md if present), feature commit history
- Incomplete history → session summary + recent git diff. Never fabricate.
- §1 gather and §8 digest refresh delegate as extraction-tier I/O workers (Delegation contract, `bee-hive/references/routing-and-contracts.md`).

## 2. Analyze — Three Parallel Analysts

| Analyst | Focus | Tier |
|---|---|---|
| pattern extractor | reusable code/process/integration patterns | extraction |
| decision analyst | choices, tradeoffs, surprises | generation |
| failure analyst | blockers, wrong assumptions, regressions | generation |

Prompts: reference ("Analyst Prompts"). Subagents return findings only —
NEVER durable files; the orchestrator synthesizes.

Spawn each analyst with the runtime's **read-only** type (Claude Code:
`Explore`), never `general-purpose` — "write no files" in the prompt is not a
safeguard while the subagent holds `Edit`/`Write`/`Bash`. Launch all three,
END THE TURN, let completions notify you — never poll. A dispatch
denied/errored at creation made no subagent: fix the cause, re-dispatch
**once**. Synthesis never needs three-of-three — synthesize from what
returned, note the gap. Never loop a failing dispatch, never wait forever.

## 3. Synthesize — One Learnings File

One dated file: `docs/history/learnings/YYYYMMDD-<slug>.md`. Template:
reference ("Learnings File Template"). Redact secrets/PII from every
snippet first; an unsafely-redactable finding is dropped and noted.

## 4. Promote Criticals — Check First, Prose Second

First choice: an **executable check** (grep/lint in the verify command, a
`bin/lib` guard, a hook denial) — a twice-seen finding almost always
qualifies. Prose is the fallback for what can't be mechanized. File the
check as a tiny/small cell if it can't ship in-feature.

Promote only when ALL three hold: multi-feature relevance, meaningful waste
prevented, generalizable. Keep critical-patterns.md high signal. Tree +
format: reference ("Promotion Decision Tree", "Critical Promotion Format").

## 5. Log Durable Decisions

`node .bee/bin/bee.mjs decisions log --decision "..." --rationale "..." [--alternatives "..."] [--confidence N]`

Log choices future planning must honor. Supersede outdated decisions
(`decisions supersede`) — never edit history. Fields: reference ("Decision
Logging").

## 6. Guard the State Layer

Learnings merge INTO the state layer (bundle concepts, else specs) — never a
parallel notes pile; contradictions replace; nothing lands in a skill body.
`bee-scribing` owns the write; compounding only verifies the handoff. Routing
+ backlog done-flip fallback: reference ("Guard the state layer").

## 7. File Unresolved Friction

`node .bee/bin/bee.mjs backlog add --type friction --severity <P1|P2|P3> --layer <layer> --title "<friction>" --detail "<predicted impact>" --feature <feature>`

So `bee-grooming` can hunt it later. Fields: reference ("Friction Backlog
Entry").

## 8. Refresh the Feedback Digest

`node .bee/bin/bee.mjs feedback digest`

Warn, never block: a throw, a missing `bee.mjs`, or an unfamiliar error is a
one-line warning, never a block/fail/delay/reversal of the close. A skipped
refresh is always disclosed, never silent. Full discipline: reference
("Feedback digest").

## 9. Sweep the Feature's Scratch

`node .bee/bin/bee.mjs tmp sweep --feature <feature>`

One of two scratch-sweep moments (other: session finish, AGENTS.md). Clears
the closing feature's `.bee/tmp/` and `.bee/spikes/` — its `<feature>/` dir
and loose `<feature>-*` files. Warn, never block: absent/failing is a
one-line warning, never a delay or reversal.

## 10. Commit the Close

Everything this skill wrote — learnings file, promoted patterns, decisions,
backlog rows, whatever scribing synced — is still uncommitted here. Commit
BEFORE §11, or `compounding-complete` claims a durability the tree lacks.

```
git add -A
git commit -m "docs(learnings): <feature> close — <one line> [<feature> close]"
```

One commit, the close's own — never folded into a cell's commit, message
names the feature + `[<feature> close]`. Unrelated dirty files stay out:
commit only close paths (`docs/history/learnings/`, `docs/knowledge/ or docs/specs/`, `docs/backlog.md`, `.bee/`), report the rest. Unresolved
refusal: warn, proceed — never set §11's phase uncommitted/unmentioned.

Register the close as a review candidate:
`node .bee/bin/bee.mjs reviews candidate add --feature <feature> --head "$(git rev-parse HEAD)" --mode <lane>`

## 11. Update State

After §10's commit lands, never before.

`node .bee/bin/bee.mjs state set --owner compounding --phase compounding-complete --next-action "<next action>" --summary "learnings: <file path>; promoted: <count>"`

Full JSON merge shape: reference ("State Update").

## 12. Suite Census

Report three counts in the run summary — informational, not a gate; zero is
a legitimate delta with no bundle/test-prune run yet:

- suites in registry, e.g. `node -e "import('./scripts/run_verify.mjs').then(m => console.log(m.SUITES.length))"`
- total test lines, e.g. `fd -e mjs 'test_' | xargs wc -l | tail -1`
- delta vs. the feature's first commit, e.g. `git diff --stat <first-commit>..HEAD -- '*test_*.mjs'`

## Hard Gates

- Never skip compounding for meaningful work; never promote without all three criteria; never write generic advice.
- Never let a subagent write durable files or spawn one write-capable — read-only only; never wait/re-loop a denied dispatch past one retry.
- Never close with a capped `behavior_change` cell unscribed — invoke bee-scribing, never sync specs inline.
- Secrets/PII never enter learnings, decisions, or backlog entries.

## Headless

`mode:headless`: gather, analyze, write the dated learnings file for
unambiguous findings; log clearly-durable decisions and friction. Critical
promotions and ambiguous calls go to `Outstanding Questions` in the terminal
report. A missing scribing record goes there too — headless never invokes
another skill on its own.

## Red Flags

- skipping compounding because the session feels done
- promoting most findings as critical; vague advice with no root cause
- inventing findings when artifacts are missing
- an analyst writing durable files directly, or spawned write-capable
- waiting past, or re-looping, a denied dispatch instead of synthesizing after one retry
- a credential in an evidence snippet
- unscribed `behavior_change` cells — and "fixing" it by editing the state layer itself
- skipping the digest refresh or scratch sweep silently, or letting either delay the close
- treating an unfamiliar digest error as exempt from warn-never-block
- setting `compounding-complete` (§11) with the close's output still uncommitted

Violating the letter of these rules is violating the spirit of these rules.

## Handoff

Compounding complete: learnings at `docs/history/learnings/YYYYMMDD-<slug>.md`,
<N> critical promotions, state-layer guard checked. Invoke bee-hive skill.

| Reference | When to Load |
|---|---|
| `references/compounding-reference.md` | analyst prompts, templates, promotion format, backlog format |
