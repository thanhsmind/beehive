---
name: arena
description: "Run N parallel attempts at one task, cross-judge them, pick a base, graft the best of the losers into it. Fires only when several structurally different answers are viable and no precedent settles the choice - a design, an API shape, a hard naming or structure call. Not for work with an obvious shape, a pattern already in the repo to follow, or a fix whose answer is known: it spawns parallel model runs and is the wrong tool for cheap work."
---

# Arena

Ported from pstack (cursor/plugins, arena), adapted for bee.

Fan out N parallel attempts at the same task. Read every candidate end to end. Pick the strongest as the base. Graft the best ideas from the others into it. Verify the synthesized result.

## Start

Open a todolist with one entry per phase before launching anything.

1. Frame
2. Fan out
3. Cross-judge
4. Pick
5. Graft
6. Verify

## Phase A: Frame

The N candidates will receive the same prompt, so the prompt is the contract.

1. State the artifact each candidate is producing.
2. Derive the rubric. State what success looks like for *this* task, then turn it into 3-6 concrete gradeable criteria. The rubric is the picker's tool in Phase D. Candidates only see the task.
3. Prepare candidate runners through the ONE bee door:
   ```bash
   .bee/bin/bee dispatch prepare --runtime claude --kind advisor --purpose "<one-line candidate purpose>" --json
   ```
   Run this command for each candidate runner. The door picks the model and the skill never does. Then run exactly the tool and payload the command returns. Never hand-pick a model name or subagent type. Spawn more candidates when the arena covers multiple design directions.
4. Assign output paths. Each candidate writes to its own isolated location (a git worktree where possible, otherwise a per-candidate subdirectory under the session scratchpad directory, e.g. `<scratchpad>/arena-<slug>/candidate-<n>/`) to keep state isolated per candidate before combining or serializing.

## Phase B: Fan out

Dispatch all N candidates concurrently using the prepared tool calls and payloads returned by the bee dispatch door, each with the task, the path to the shared grounding, its own isolated scratchpad output path, and instructions to produce both the artifact and a short rationale.

Each rationale names the alternatives the candidate considered and what it rejected.

If a candidate fails to produce output, proceed with N-1 and note the dropout in the synthesis record.

## Phase C: Cross-judge

After all Phase B candidates complete, prepare the cross-judge through the ONE bee door:

```bash
.bee/bin/bee dispatch prepare --runtime claude --kind reviewer --purpose "cross-judge arena candidates against rubric" --json
```

The door picks the model and the skill never does. Run exactly the tool and payload it returns.

The cross-judge sees the rubric and the candidates by path label, scores each criterion, and recommends a base with rationale. It runs in parallel with the parent's reading in Phase D, not with the candidates themselves. Do not dispatch the judge while candidates are still writing.

## Phase D: Pick a base

Read every candidate end to end before picking.

Score each candidate against the rubric criterion by criterion, not on holistic feel. Compare against the cross-judge. Agreement on the base confirms the pick. Disagreement means one of you is biased or the rubric was ambiguous. Read both rationales before deciding.

Pick the base on which candidate a future maintainer can extend most easily without breaking invariants. Prefer the cleaner boundary or smaller API when two feel tied, keeping abstractions lean with the minimal necessary interface.

Record the pick and the reason in a short synthesis note alongside the base artifact, including the cross-judge's verdict.

## Phase E: Graft

Walk each losing candidate once more and identify what is worth porting into the base. The signal is usually one or two things per candidate, not most of it.

Fold each graft in by hand, rebuilding the integrated design from foundational constraints rather than pasting mechanically. The result has to remain coherent under one mental model.

Record what was grafted, from which candidate, and what was rejected and why.

When N candidates converge on the same shape, that is a strong agreement signal. Note the convergence in the record and ship the consensus shape. No graft is needed. When N candidates wildly diverge, Phase A was under-specified. Reframe and re-run rather than averaging the divergence.

## Phase F: Verify

The synthesized artifact has to hold up under the same scrutiny as any other output, verifying that it works with concrete proof.

If verification surfaces a problem the arena did not catch, either Phase A was wrong (re-frame and re-run) or one candidate caught it and you missed the graft (go back to Phase E). Don't paper over.

## Outputs

One synthesized artifact. One short synthesis note alongside, naming the base, the grafts (with source candidate), the rejections, the dropouts if any, and the verification result.
