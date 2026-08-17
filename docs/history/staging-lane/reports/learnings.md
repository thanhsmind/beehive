# staging-lane — learnings (2026-08-17)

Feature: `bee staging` (add / rebuild / status), the staging teeth, and the
topology guidance across skills. Cells sl-1..4, merged to main verify-green
after the user's uat approval — the first merge ever stopped by the new
`WORKTREE_MERGE_UAT_PENDING` door, which fired on this very feature.

## What settled

- staging-lane D0/D0a/D0b live: staging = main + Σ features awaiting UAT;
  three triggers; teeth (no staging→main, no direct commits, rebuild nudge);
  skills teach the topology (area doc
  docs/knowledge/areas/worktree-parallelism/staging-mixing-ground.md).

## Learnings

1. **The uat door proved itself in production on its own release cycle**: a
   sibling session had rebuilt `.bee/bin/bee` after the uat feature landed,
   so this feature's merge was refused until the user's approval was
   recorded — remedy text worked as written, first try.
2. **Optimistic parallelism worked as the user prefers**: two sibling
   sessions landed two features on main mid-flight; the resulting conflicts
   (bee-swarming SKILL.md, registry_payload.json) were resolved in the
   feature worktree by a three-way JSON merge (ours-added commands + theirs-
   changed commands, zero real overlaps) and a supersede on the skill text;
   full suite green (1971 passed) before re-merge.
3. **registry_payload.json is merge-hostile** (minified single line): every
   concurrent feature that adds a command conflicts textually even when the
   JSON merge is trivial. Candidate fix: store it pretty-printed one-command-
   per-line, or give it a real generator; worth a backlog row if it bites
   again.
4. **Worker stalls are recoverable by resume**: the sl-3 worker hit the
   600s stream watchdog mid-verify; a SendMessage resume completed the cell
   with its context intact — cheaper than re-dispatch.
