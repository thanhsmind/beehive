---
name: why
description: "Use for 'why does X work this way', 'why we picked Y', design rationale, regressions, postmortems, or historical decisions. Queries source control and bee project records (decisions, context, knowledge areas, learnings) in parallel, then returns a cited read on decisions and tradeoffs. Use how for runtime behavior."
disable-model-invocation: true
---

# Why

Ported from pstack (cursor/plugins), adapted for bee.

Investigate the motivation and intent behind code.

Companion to the `how` skill. `how` answers what the code does and how it works. `why` answers what forces led to its shape.

## Operating Posture

Operate as a **careful, cautious, and precise investigator**. Be honest about what you know vs what you're inferring. Read `references/epistemics.md` for the full confidence framework and phrasing guide. The synthesizer must follow it.

## Step 1. Understand the Target and the Question

Parse what the user is asking. The **target** is usually a chunk of code, a pattern, a feature, or a named design decision. The **question** is usually a design rationale, a tradeoff, a motivating edge case, an external constraint, dead code, or a broad history sweep.

If the target is vague ("why do we do it this way?" with no clear referent), make your best guess from conversation context (open files, recent edits, cursor location, what was just discussed). State your interpretation briefly so the user can redirect if you're off, then proceed.

## Step 2. Establish the Code Anchor

Before spawning investigators, anchor the investigation in concrete code. You need:

- The relevant file path(s) and line range(s)
- The key symbols (function names, class names, constants)
- An initial commit list: the last few commits touching the target
- PR numbers from merge commits (pattern `(#1234)` in the subject line)

Build this inline:

```bash
# Blame target lines for last-touch commits
git blame -L <start>,<end> <file>

# Full file history, with patches, through renames
git log --follow -p -- <file>

# Last N commits touching the file, PR numbers visible
git log --oneline -20 -- <file>

# Extract PR numbers from a commit message
git log -1 --format=%B <commit>
```

Pull PR bodies and discussion via `gh` for any substantive commits:

```bash
gh pr view <number> --json title,body,author,createdAt,mergedAt,labels,closingIssuesReferences,comments,reviews
```

Capture this as seed context (file paths, symbols, commits, PR numbers, linked feature/issue IDs). Pass it to the investigators.

## Step 3. Spawn Parallel Investigators

Default to running both evidence categories concurrently.

### The Two Evidence Categories

This environment provides two authoritative sources for 'why':

1. **Source control history** (`references/sources/code-archaeology.md`). Git history, `gh` for PRs, code comments, tests. Surfaces implementation-time rationale captured during code authoring and review.
2. **Bee store and project records** (`references/sources/bee-store.md`). Decision log (`bee decisions search`), locked feature context (`docs/history/<feature>/CONTEXT.md`), area specifications (`docs/knowledge/areas/`), and captured learnings (`docs/history/learnings/`). Surfaces explicit design decisions, system invariants, and historical retrospectives.

See `references/source-playbook.md` for the playbook index.

### Dispatching Investigators

Prepare each investigator dispatch through the ONE bee door:

```bash
.bee/bin/bee dispatch prepare --runtime claude --kind gather --purpose "investigate <source>" --json
```

Run exactly the tool and payload it returns.

Each investigator receives:
1. Investigation instructions: gather concrete evidence with verbatim quotes and citations, track search queries and negative results, note contradictions and gaps, and avoid forming premature conclusions.
2. The category playbook: `references/sources/code-archaeology.md` or `references/sources/bee-store.md`.
3. The code anchor from Step 2 (file paths, symbols, commit hashes, PR numbers, feature slugs, ticket IDs).
4. The user's original question.

Spawn both investigators concurrently in a single message.

### When to skip an investigator

Both sources exist in this repository. Only skip with an **explicit, written justification** recorded in the final "Sources Consulted" section. Valid reasons:
- The source is **provably irrelevant** to the question (e.g. source control skipped for an abstract architecture question where no code has been authored, or bee store skipped in a repository or branch that carries no `.bee` store and no `docs/history/` records).
- If a single-commit trivial target already provides the complete, unambiguous answer in its PR description, you may answer inline **only after** confirming both source searches would be redundant. Say so explicitly.

## Step 4. Synthesize

Once both investigators return, prepare the synthesizer dispatch through the ONE bee door:

```bash
.bee/bin/bee dispatch prepare --runtime claude --kind advisor --purpose "synthesize why investigation for <question>" --json
```

Run exactly the tool and payload it returns.

The synthesizer receives:
1. The investigator findings, including any null results and any categories skipped with justification.
2. The code anchor from Step 2 (file paths, symbols, commit hashes, PR numbers, feature/decision references).
3. The user's original question.
4. The epistemics framework from `references/epistemics.md`.
5. The output format instructions below.

The synthesizer reconciles overlapping findings, surfaces contradictions, calibrates confidence tiers, and spot-checks citations using read-only access to the repository.

## Step 5. Present

Take the synthesizer's output and present it to the user. You may lightly edit for clarity or add context from the conversation, but **do not rewrite the confidence language**.

## Output Format

Use this structure, keeping the confidence separation intact:

### The Question
Restate the user's question in one or two sentences.

### The Code in Question
File paths, line ranges, key symbols.

### What We Found
Claims with direct evidence, one per bullet:
- **[Direct]** {Claim}. Source: {PR # / commit hash / decision ID / file:line}. {Quote or paraphrase.}
- **[Supported]** {Claim}. Evidence: {list of items and what each contributes}.

### What We Can Reasonably Infer
Claims supported by indirect evidence where the inference chain is explicit:
- **[Inferred]** {Hedged claim}. Reasoning: {the specific evidence and the inference step}.

### Competing Hypotheses
If the evidence fits multiple interpretations, present each with evidence for and against. Skip if there is a single clear answer.

### What We Don't Know
Explicit gaps: questions unanswered, searches that returned nothing, sources unavailable.

### Sources Consulted
One line per source:
- **Source control history**: {files, commits, PRs, comments searched}
- **Bee store and project records**: {decision queries, CONTEXT.md files, knowledge areas, learnings searched}

### Confidence Summary
One or two sentences summarizing overall confidence.

After the Sources Consulted block, if the user's `why` question is a precursor to changing the code, convert the findings into a Preserve / Change / Avoid / Risk constraint set suitable for planning the change.

## Reference Files

- `references/epistemics.md`. Confidence tiers and phrasing guide.
- `references/source-playbook.md`. Index of source playbooks.
- `references/sources/code-archaeology.md`. Playbook for git and in-repo history.
- `references/sources/bee-store.md`. Playbook for bee decision log, context, and knowledge records.
