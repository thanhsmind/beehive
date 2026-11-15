# eval

You own the experiment design. Plan, blind, run, synthesize. An eval tests how
a change affects agent behavior before it is promoted: a new skill variant, a
structural change, a prompt tweak. The failure mode is the observer effect: an
agent that knows it is being evaluated behaves differently, so the candidates
must run blind, and a result from candidates that knew proves nothing.

Non-negotiables for blinding:

- Keep `eval`, `test`, `judge`, `experiment`, `rubric`, `score`, `compare`,
  `benchmark`, `candidate` and `arena` out of every directory, file and prompt
  the candidate sees, because each word tells the candidate it is measured.
- The candidate prompt looks like an organic user request. State the goal, not
  the meta: "build me a small todo cli", not "show me how you follow the
  principles chain". A meta prompt measures how well the candidate performs for
  an examiner.
- Ask for no chain-eliciting cue. Do not ask the candidate to list which
  skills, principles or files it applied: that meta prompt inflates citation
  behavior. Ask for design notes generally and grade chain-following from the
  code's shape, not from self-report.
- Sanitize directory and slug names. Use project-shaped names a user might
  pick, not labels like `candidate-1` or `agent-a`, because a label is a cue.
- Do not tell a candidate that other candidates exist, because rivalry changes
  what it does.
- The judge may know it is judging, but sees outputs by sanitized label only,
  never by model name, because a model name pulls the verdict toward what the
  judge expects of that model.
- When comparing two variants, one judge scores both sets in a single pass on
  one scale, blind to which set each came from. Two judge runs with different
  prompts do not compare, because the calibration drifts between them.

Steps:

1. Frame. State what variant is under test and what behavior counts as
   success. Write the rubric (3 to 6 concrete criteria) for the judge only and
   hold it back from the candidates, because a candidate that sees the rubric
   writes to it.
2. Set up sanitized environments: one working directory per candidate with the
   variant in place. Plant any context an organic task would have: a project
   skeleton, and the skills the candidate would naturally read, because a bare
   directory is itself a cue that this is a test.
3. Author one organic prompt, what a user would type, with no leakage of what
   is measured, because one prompt held constant makes the candidates differ
   only by the variant.
4. Run N parallel candidates, each in its own sanitized directory with the
   same prompt, dispatched through `bee dispatch prepare`. The door returns
   each worker's model; never hand-pick one. Parallel runs share nothing, so
   one candidate's output cannot leak into another's.
5. Run one blinded judge through `bee dispatch prepare --kind reviewer`, which
   returns the review tier's model. The judge sees outputs by sanitized label
   and the rubric, never a model name. A judge on the same model as the
   candidates shares their blind spots, so when the door returns the same
   model, say so in the reply.
6. Verify the chain from transcripts, not self-report. Read each candidate's
   own transcript under the active workspace and no other: never read
   transcripts across workspaces, because that crosses into private work from
   unrelated projects. Look at which files each candidate actually opened.
   Citing a principle is not reading its leaf skill, and reading it is not
   applying it. Grade chain-following from the files it really read plus the
   shape of the code, never from the candidate's own claims.
7. Read every candidate output yourself, end to end, and compare it to the
   judge's verdict. Disagreement means a model is biased or the rubric is
   ambiguous. Synthesize, because the judge's score is evidence and the
   decision to promote stays yours.

Reply: variant under test, rubric, per-candidate notes, the judge's verdict,
your synthesis, and a recommendation for whether to promote the variant.

Proof line: the rubric, the judge's per-label scores, and the transcript files each candidate opened.
