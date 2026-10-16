# pstack-0-15-port — plan-step hat wave, synthesis

Three seats, one draft of `plan.md`, 2026-10-16. All three returned inside
the ceiling; none dropped. The leader folded the findings into the second
draft of `plan.md`; this record names what changed and what did not.

## MANDATE 1 — Structure (facts-gaps)

- BLOCKER, fixed: three docs cells shared the exclusive release-manifest path
  and each ran `bee dev regen`. Resolved by merging p15-3 and p15-4 into one
  cell with `deps: ["p15-2"]`; two waves instead of one.
- BLOCKER, fixed: claims 6 and 12 quoted prettified bytes. Replaced with the
  verbatim lines. Claims 1 and 7 anchors re-pointed to the drifted lines.
- WARNING, fixed: `run_git` cannot feed `git patch-id` (stdin null). p15-1 now
  adds `run_git_stdin` in `git.rs` and names the single-stream compute.
- WARNING, fixed: "no_code_comments is on" was false; the real guard is the
  comment ratchet `bee dev comment-baseline --check`. p15-1 names it.
- WARNING, fixed: p15-2's `rg -vc` verify could never read green; now
  `rg -v '—' | wc -l` prints 0.
- WARNING, fixed: bee-capturing had no `fix-at` anchor; the pointer lands at
  step 4 ("already-promoted pattern recurs").
- Decision coverage: D1–D6 each land in one cell; D7 and D8 are provenance
  and land in no cell, said so in Requirements.

## SMALLER PATH (alternatives)

- Taken: three cells, p15-1 ∥ p15-2 → p15-3. One regen cycle saved, one
  exclusive-path conflict removed, every D held.
- Taken: branch-side patch-ids computed once, lazily, only when the first
  pass flagged a cap.
- Taken: the reason-less playbook steps are found by one `rg` the worker runs
  as a checklist (about 37 of 48).
- Not taken: one docs cell for everything — too much in one brief.

## User impact

- Fixed: `proof_carried` would have been invisible in the printed merge text;
  p15-1 adds the `proof-carried:` line in `handlers.rs`, with a message ending
  "Nothing to re-run."
- Fixed: the 140-character bound now covers only the seven touched lines, and
  the reason goes last, under 12 words, because `bee orient` prints the line.
- Fixed: the judge-tier row names its diff base (`<capped sha>..HEAD`,
  cumulative), the leader as the one who applies it, and the slice-close tick
  that records the skip.
- Fixed: the playback bullet fires only when a question follows.
- Fixed: the twice rule names the two surfaces where a second sighting is seen
  and the reproduce-fails fallback to `doctrine`.
- Open, backlogged: `bee mailbox reflect` could echo the prior count for a key
  (code, outside D4).

## Seats and confidence

facts-gaps 92 · alternatives 85 · user-impact 88. Falsifiers recorded in each
seat's return; none run (all three are "dispatch as written and watch it
fail", which the redraft makes moot).
