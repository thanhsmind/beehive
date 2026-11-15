# pause-safely

You own a clean stop. Leave a checkpoint a cold-start agent can resume from.

This is explicit only: the owner's word starts it. On "keep going", "going to
bed, keep going" or "don't stop", do not pause; those mean continue, and
`autonomous-run` already checkpoints per iteration. Context filling up is not
the owner's word either: a context mark never stops work, so when compaction
is the only trigger, write the note in step 4 to a file and keep working.

1. Stop at a safe boundary. Finish the current atomic step or back out of it.
   Never stop mid-edit in a known-broken state: a cold-start agent cannot tell
   a half-edit from a design. Start nothing new, and let any dispatched worker
   finish or cancel it, so no writer outlives the checkpoint.
2. Don't cross an irreversible line to pause. Open no new PR and push nothing
   beyond what `ship_visibility` already pushes on a cap, because pausing is
   not a ship decision.
3. Make the work durable. Commit uncommitted edits as one clear `wip:` commit
   on the current branch so nothing is lost. It carries no `cell:` trailer, so
   it is not a cap and the cell stays open. If the tree is broken, say so in
   the commit body in one line, so the next agent does not trust it.
4. Write the resume note off-context. Run `bee state handoff write --kind
   pause` with the cell, the files, what is `--done`, what is `--remaining` and
   the `--next-action`, and put the rest in the note: intent, what you were
   doing, progress and what is verified, current state, key files and gotchas.
   For the compaction trigger write the note to a file like
   `/tmp/<slug>-resume.md` instead and write no handoff, because the in-context
   plan will not survive summarization and a handoff happens only on the
   owner's word or at feature close. If the feature keeps a decision trail
   (`bee-show-me-your-work`) under `docs/history/<feature>/`, point at it instead of duplicating it, so
   one fact keeps one home.

Reply: where you are in the loop, what is on disk versus still in your head
(paths, no diff dumps), the commits you made and whether the tree is clean, and
the first action on resume. This is a pause, not a final report. Resume is the
`session-pickup` situation reading this note.

Proof line: `git status` clean or the named exceptions, and the handoff
(`bee state handoff show --json`) read back.
