---
name: bee-arena
description: >-
  Fan out N parallel candidates at one task, pick a base, graft the strongest parts of the losers into it. Use when the user types /bee-arena, says arena this, or when one attempt at a non-trivial artifact would lock in the wrong shape. Not for design dossiers (blind lanes) or review waves.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: The candidates and the judge go out through the vendored bee binary's dispatch door. Absent, the leader writes the candidates itself, one after the other, and the account says so.
---

# Arena

Fan out N parallel attempts at the same task. Read every candidate end to end. Pick the strongest as the base. Graft the best ideas from the others into it. Verify the synthesized result.

## Start

Open a task list with one entry per phase before launching anything. The arena runs autonomously and the list keeps phases from silently disappearing.

1. Frame
2. Fan out
3. Cross-judge
4. Pick
5. Graft
6. Verify

## Phase A: Frame

The N candidates will receive the same prompt, so the prompt is the contract. Get it right before dispatching anything.

1. State the artifact each candidate is producing.
2. Derive the rubric. State what success looks like for *this* task, then turn it into 3-6 concrete gradeable criteria. Concrete: `Adds a --dry-run flag that skips writes`. Vague: `code is correct`. The rubric is the picker's tool in Phase D; candidates only see the task.
3. Pick the runners. Each runner goes out as `bee dispatch prepare --runtime claude --kind advisor --role lane-1 --json`, then `--role lane-2` and `--role lane-3`. The `lane-1` to `lane-3` roles sit in the bee team table, so a different model per runner is a team setting, never a choice made here. Use the payload exactly as it returns. Never pick a model or a subagent type by hand. A fourth runner or more, when the arena covers multiple design directions, reuses a lane role. Unconfigured lane roles fall through to `advisor`, which runs the same model N times: right when the work is generation-bound rather than judgment-sensitive.
4. Assign output paths. Each candidate writes to its own location (a git worktree where possible, otherwise `/tmp/arena-<slug>/candidate-<n>/`). N candidates writing to the same path is shared mutable state and fails the `bee-principle-separate-before-serializing-shared-state` test. An advisor runner writes no file: its answer is the artifact plus its rationale, and the leader saves each answer verbatim at that candidate's path.

## Phase B: Fan out

Issue all N `bee dispatch prepare` calls in the same turn and run every returned payload in the background, each with the task, the path to the shared grounding, its own output path, and instructions to produce both the artifact and a short rationale. Every candidate gets the same brief bytes.

The rationale is mandatory. Without it, the parent cannot tell whether a candidate's structure is principled or accidental, which makes Phase E grafting unreliable. Each rationale names the alternatives the candidate considered and what it rejected.

If a candidate fails to produce output, proceed with N-1 and note the dropout in the synthesis record.

When the artifact is a design question that must end in a dossier and `bee blind check`, run the blind-lane procedure in `bee-hive/references/gates-and-delegation.md`, section "Blind lanes and convergence", instead. That is the one fan-out mechanism for designs; never invent a second.

## Phase C: Cross-judge

After all Phase B candidates complete, dispatch one judge as `bee dispatch prepare --runtime claude --kind reviewer --json`, with no `--role`. The reviewer kind runs on the review role of the bee team table, a different role than the `lane-N` runners, so a judge on a different model than the runners is a team setting. Use the payload exactly as it returns. When the payload names the same model as the runners, say so in the synthesis note: a judge on the runners' model shares their blind spots.

The judge is read-only. It sees the rubric and the candidates by path label, with no lane or model names, scores each criterion, and recommends a base with rationale. It runs in parallel with the parent's reading in Phase D, not with the candidates themselves. Dispatching while candidates are still writing means the judge sees partial or empty outputs and reports them as dropouts.

## Phase D: Pick a base

Read every candidate end to end before picking. Skimming N candidates surfaces only the candidate whose surface looks most familiar.

Score each candidate against the rubric criterion by criterion, not on holistic feel. Compare against the cross-judge. Agreement on the base confirms the pick. Disagreement means one of you is biased or the rubric was ambiguous. Read both rationales before deciding.

Pick the base on which candidate a future maintainer can extend most easily without breaking invariants. Prefer the cleaner boundary or smaller surface area when two feel tied: do the least that works (`bee-principle-laziness-protocol`).

Record the pick and the reason in a short synthesis note alongside the base artifact, including the cross-judge's verdict.

## Phase E: Graft

Walk each losing candidate once more and identify what is worth porting into the base. The signal is usually one or two things per candidate, not most of it.

Fold each graft in by hand, redesigning the base from first principles (`bee-principle-redesign-from-first-principles`) around it as if the graft had been a day-one assumption. Don't paste mechanically. The result has to remain coherent under one mental model.

Record what was grafted, from which candidate, and what was rejected and why. The rejection notes are the highest-signal part of the record. Future readers learn from what you considered and dropped, not just what you kept.

When N candidates converge on the same shape, that is a strong agreement signal. Note the convergence in the record and ship the consensus shape. No graft is needed. When N candidates wildly diverge, Phase A was under-specified. Reframe and re-run rather than averaging the divergence.

## Phase F: Verify

The synthesized artifact has to hold up under the same scrutiny as any other output: run it, test it or read it against the rubric, and report what that showed, never what should happen (`bee-principle-prove-it-works`). The arena does not earn you a pass.

If verification surfaces a problem the arena did not catch, either Phase A was wrong (re-frame and re-run) or one candidate caught it and you missed the graft (go back to Phase E). Don't paper over.

## Outputs

One synthesized artifact. One short synthesis note alongside, naming the base, the grafts (with source candidate), the rejections, the dropouts if any, and the verification result.

## Handoff

Artifact and synthesis note delivered. Invoke bee-hive skill.
