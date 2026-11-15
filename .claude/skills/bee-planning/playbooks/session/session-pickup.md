# session-pickup

You own the resume point. Read the prior trail, don't redo it.

A pickup is inheritance. The prior agent already paid the cost of reading the
code, running the repros and making the design choices. Redoing loses the bias
check and burns context. Resist the urge to re-derive; read.

1. Locate the prior trail, starting with `.bee/HANDOFF.json`
   (`bee state handoff show --json`). A `pause` record waits for the owner's
   word: present it and continue only after confirmation. A `planned-next`
   record is adopted with `bee state handoff adopt`, and only in a fresh
   session; a resumed or memory-compacted session never adopts, because
   auto-resume authority exists only at a fresh-session boundary. When the
   owner names a transcript or a pushed branch instead, read that one file or
   branch and nothing around it: a transcript from another project is a
   private chat. Read the overview and the last messages first, then scan back
   for the decision points. Parse a long transcript in a gather
   (`bee dispatch prepare --kind gather`) and keep the reduced timeline in the
   main thread, because a transcript is bulk and the lead's context is the
   scarce thing.
2. Reconstruct operational state: the branch and worktree, what already landed
   (`git log`, `git diff` against the base), the open cells and the decisions
   made. The prior trail is authoritative input, and re-deriving it is the bias
   this playbook exists to prevent.
3. Diff done against pending. Compare what shipped with what was planned, name
   the resume point, and do not re-run the prior repro or redo completed work.
   A "let me verify from scratch" pass is the tell that you treat the trail as
   untrustworthy when it is authoritative.
4. Route the remaining work through `bee orient` to the matching situation and
   pick the verdict: continue the execution, ship a finished recommendation,
   ratify or override a prior conclusion, or postmortem a failed run. This
   playbook ends here and the routed situation owns the rest, so the two never
   restate each other.
5. Verify the inherited claims against the original goal on the real artifact,
   driven through the project's `verify-app` skill when it has one, otherwise
   through the real interface. A passing prior self-report is not the proof:
   the prior agent graded its own work.

Reply: where the prior agent stopped, what you inherited versus redid (ideally
nothing redone), the resume point, and the outcome.

Proof line: the real-artifact run that confirmed or refuted the inherited
claim, with its output.
