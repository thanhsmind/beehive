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

Compounding captures reusable lessons from completed work and feeds them back into future exploring, planning, and reviewing. Run it after `bee-scribing` completes, or when work is intentionally abandoned with lessons. Do not skip it for meaningful work just because the session feels done.

## 1. Gather Evidence

- `docs/history/<feature>/CONTEXT.md`, `plan.md`, worker reports under `docs/history/<feature>/reports/`
- cells and traces: `node .bee/bin/bee.mjs cells list --feature <feature>`
- review findings (including residual-findings.md, if present)
- feature commit history

If history artifacts are incomplete, fall back to the session summary and recent git diff. NEVER fabricate learnings — a thin honest entry beats an invented rich one.

§1 evidence gather and the §8 digest refresh delegate as extraction-tier I/O workers per the Delegation contract (D2/D3, `bee-hive/references/routing-and-contracts.md`) — the three analysts below are already tiered.

## 2. Analyze — Three Parallel Analysts

Launch three temp-finding subagents in parallel (prompts in `references/compounding-reference.md`):

| Analyst | Focus | Tier |
|---|---|---|
| pattern extractor | reusable code/process/integration patterns | extraction |
| decision analyst | important choices, tradeoffs, surprises | generation |
| failure analyst | blockers, wrong assumptions, regressions, missing checks | generation |

Subagents return temporary findings only — they NEVER write durable files. The orchestrator synthesizes.

**Spawn read-only (D1).** Spawn each analyst with the runtime's **read-only** agent type (Claude Code: `Explore`), NEVER `general-purpose`. "Write no files" in the prompt is not a safeguard while the subagent holds `Edit`/`Write`/`Bash` — a full-tools analyst has committed unrequested source (the leak this closes). The read-only type is a runtime built-in, not a plugin agent type, so `bee-reviewing`'s "never a plugin agent type" rule still holds; its "default/general subagent type" line is reviewer-specific (reviewers may run commands) and does NOT extend to these read-only analysts.

**Wait, don't hang (D2).** Launch all three, END THE TURN, and let each completion notify you — never poll liveness. If a dispatch is denied or errors at creation (e.g. the model-guard hook denies a missing `[bee-tier: …]` marker) then NO subagent exists: surface it, fix the cause, and re-dispatch that one analyst **once**. Synthesis does NOT require three-of-three — if an analyst still fails after one retry, or never returns, synthesize from the analysts that DID return and record the missing one as a gap in the run summary. NEVER loop the same failing dispatch and NEVER wait forever: a denial that repeats identically on retry is the phantom-wait this rule exists to break.

## 3. Synthesize — One Learnings File

Write one dated file: `docs/history/learnings/YYYYMMDD-<slug>.md` with frontmatter (`date`, `feature`, `categories`, `severity`, `tags`) and sections **What Happened** / **Root Cause** / **Recommendation**. Recommendations are imperative future rules: "When X, do Y" — specific enough to act on. Template in the reference.

Before writing, redact secrets and PII from every evidence snippet. If a finding cannot be safely redacted, drop it and note the skip in the run summary. Secrets never enter learnings.

## 4. Promote Criticals — Check First, Prose Second

For a lesson that clears the bar below, the **first-choice promotion target is an executable check**: a grep/lint line appended to the affected area's verify command, a `bin/lib` guard, or a hook denial. A twice-seen review finding or user correction almost always qualifies — mechanize it and it can never recur; prose in `critical-patterns.md` taxes every session preamble and relies on being read. Prose is the fallback for what genuinely cannot be mechanized (judgment calls, product taste). File the check as a tiny/small cell if it cannot ship in the current feature.

Either way, promote sparingly — only when a lesson meets ALL three criteria:

1. **Multi-feature relevance** — it will matter beyond this feature.
2. **Meaningful waste prevented** — it would save future agents real time or real damage.
3. **Generalizable** — it is a rule, not an anecdote.

Ten findings rarely yield ten criticals. Keep critical-patterns.md high signal; a bloated file gets skipped, and then nothing compounds.

## 5. Log Durable Decisions

```
node .bee/bin/bee.mjs decisions log --decision "..." --rationale "..." [--alternatives "..."] [--confidence N]
```

Log choices future planning must honor. Supersede outdated decisions (`bee.mjs decisions supersede`) — never edit history.

## 6. Guard the State Layer

Learnings merge INTO the state layer (bundle concepts, else specs) — never into a parallel notes pile; contradictions replace; nothing lands in a skill body (thin-body law). Full routing rules and the what-goes-where table: the reference ("Guard the state layer").


## 7. File Unresolved Friction

Unresolved friction from cell traces or the session → `node .bee/bin/bee.mjs backlog add --type friction --severity <P1|P2|P3> --layer <layer> --title "<friction>" --detail "<predicted impact>" --feature <feature>`, so `bee-grooming` can hunt them later. Field guidance in the reference.

## 8. Refresh the Feedback Digest

Warn-never-block: regenerate the ranked friction digest when stale; a broken digest never blocks the close. Mechanics: the reference ("Feedback digest").


## 9. Sweep the Feature's Scratch (tree-hygiene D2)

Feature close is one of the two moments D2 names for sweeping scratch (the other is session finish, AGENTS.md). Run it for the finishing feature:

```
node .bee/bin/bee.mjs tmp sweep --feature <feature>
```

This clears the finished feature's scratch under `.bee/tmp/` and `.bee/spikes/` — both a `<feature>/` directory and the loose `<feature>-*` files agents write straight into the scratch root. It is the one documented override that sweeps a named feature's scratch even though compounding still treats it as "current" at the moment this runs. **Warn, never block** — same discipline as the digest refresh (§8): a failing or absent sweep (the command throws, the feature had no scratch dir, `bee.mjs` predates this verb) is a one-line warning in the run summary and nothing more. It never blocks, fails, delays, or reverses the feature close.

## 10. Commit the Close (issue #48)

**Everything this skill wrote is still uncommitted at this point.** The dated learnings file, the promoted critical patterns, the logged decisions, the backlog rows, and whatever scribing synced are all sitting dirty in the tree. Commit them **before** the state update below — otherwise the phase says `compounding-complete` while the close's own output exists nowhere but the working tree, and the next session (or a crash, a `git checkout`, a worktree merge) loses it with no trace that anything was lost.

```
git add -A
git commit -m "docs(learnings): <feature> close — <one line> [<feature> close]"
```

Rules:
- **One commit, and it is the close's own commit** — never fold the close into a cell's commit, and never leave it for "the next commit to pick up". Per-cell commits (critical rule 7) already landed during execution; this one carries the compounding artifacts.
- **The commit message names the feature and the close**, so the close is findable in the log: `[<feature> close]`.
- **Nothing outside the close belongs in it.** If `git status` shows unrelated dirty files, commit only the close's paths (`docs/history/learnings/`, `docs/knowledge/` or `docs/specs/`, `docs/backlog.md`, `.bee/`) and report the rest in the run summary rather than sweeping it in.
- **Warn, never block, on a refusal you cannot resolve** (a hook rejects the commit, the repo is mid-rebase, nothing is dirty because a cell already committed it): one line in the run summary naming the reason, and the close proceeds. What is never acceptable is setting the phase in §11 while silently leaving the close's artifacts uncommitted and unmentioned.
- **Register the close as a review candidate** (SPEC 7.1 step 6): `node .bee/bin/bee.mjs reviews candidate add --feature <feature> --head "$(git rev-parse HEAD)" --mode <lane>` so an on-demand review session can find this feature later.

## 11. Update State

The phase is set **after** the commit in §10 has landed, never before: `compounding-complete` is the claim that the close is durable, and it is only true once the close's artifacts are in a commit.

Record the completed compounding run: `node .bee/bin/bee.mjs state set --owner compounding --phase compounding-complete --next-action "<next action>" --summary "learnings: <file path>; promoted: <count>"`.

## 12. Suite Census (test-economy D4)

Report three counts in the close's run summary, so suite growth has a visible ledger instead of climbing unnoticed (test-economy D4 — the counterweight to auto-discovery's monotonic growth; no bundle, no test-prune has run yet, is a legitimate delta of zero):

- **suites in registry** — total distinct suites `run_verify.mjs` discovers, e.g. `node -e "import('./scripts/run_verify.mjs').then(m => console.log(m.SUITES.length))"` (adjust to the registry's actual export if the script's shape has moved on)
- **total test lines** — summed line count across test files, e.g. `fd -e mjs 'test_' | xargs wc -l | tail -1`
- **delta for the feature just closed** — the same two counts compared against the feature's first commit, e.g. `git diff --stat <feature-first-commit>..HEAD -- '*test_*.mjs'`, so the report shows whether this feature grew, held, or shrank the suite

These are read-only shell one-liners — no new `bee.mjs` CLI verb. Fold the three numbers into the run summary text; they are informational context for the human, not durable evidence and not a gate on the close.

## Hard Gates

- Do NOT skip compounding for meaningful work. "The session feels done" is the rationalization, not a reason.
- Do NOT promote everything as critical — apply all three criteria.
- Do NOT write generic lessons ("test more carefully" is banned-grade advice). Concrete situation, root cause, imperative rule.
- Do NOT let subagents write durable files; the orchestrator synthesizes.
- Do NOT spawn analysts with a write-capable agent type — read-only only (D1). Do NOT wait on, or re-loop, an analyst dispatch that was denied/errored at creation; synthesize from what returned after one retry (D2).
- Do NOT close out while `behavior_change` cells were capped but scribing never ran — invoke bee-scribing; never sync specs inline. A spec older than the behavior it describes is measured entropy, not a detail.
- Secrets and PII never appear in learnings, decisions, or backlog entries.

## Headless

`mode:headless`: gather, analyze, and write the dated learnings file for unambiguous findings; log clearly-durable decisions and friction. Critical promotions and ambiguous calls are NOT applied — they go to an `Outstanding Questions` section of the structured terminal report for the human. A missing scribing record is reported there too (headless compounding never invokes another skill on its own).

## Red Flags

- skipping compounding because the user left or the session feels done
- promoting most findings as critical
- vague advice with no situation or root cause
- inventing findings when artifacts are missing
- an analyst subagent writing to `docs/history/learnings/` directly
- an analyst spawned as `general-purpose` (or any write-capable type) instead of the runtime read-only type — "write no files" is a prompt string, not a tool restriction (D1 §2)
- waiting past a denied/errored analyst dispatch, or re-looping the same failing call, instead of synthesizing from what returned after one retry (D2 §2)
- an API key, token, or credential in an evidence snippet
- `behavior_change` cells capped but no scribing record — and compounding "fixing" it by editing the state layer itself (`docs/knowledge/` or `docs/specs/`)
- closing without running the digest refresh because "the skill/teammate didn't ask for it" — it is a step, not an optional extra (Scenario 1)
- blocking or failing a host project's feature close because `bee.mjs feedback digest` threw — telemetry never stops the line; warn and file friction (Scenario 2)
- treating an *unfamiliar* digest error as exempt from warn-never-block — "I must understand this throw before I can close" is the loophole; a digest error never gates a close, understanding it is post-close cleanup (Scenario 2 REFACTOR)
- skipping the digest refresh under context/exhaustion pressure and saying nothing — a silent skip is a violation; disclose it in the summary and Handoff (Scenario 3)
- skipping the scratch sweep (§9) silently, or letting it block/fail/delay the close — same warn-never-block discipline as the digest refresh (tree-hygiene D2)
- setting the phase to `compounding-complete` (§11) with the close's own output — learnings file, decisions, backlog rows, synced state layer — still uncommitted: the phase claims the close is durable, and a dirty tree is not durable (issue #48, §10)

Violating the letter of these rules is violating the spirit of these rules.

## Handoff

Compounding complete: learnings at `docs/history/learnings/YYYYMMDD-<slug>.md`, <N> critical promotions, state-layer guard checked. Invoke bee-hive skill.

| Reference | When to Load |
|---|---|
| `references/compounding-reference.md` | analyst prompts, learnings template, promotion format, backlog entry format |
