# steer-recipes

One-line prompts you can type to steer a session by name. Each routes to a
situation file under `playbooks/` or to a principle slug that bee loads on its
own. This page is a lookup for the owner, not a situation: no step list, no
proof line.

| Say | Routes to |
|---|---|
| "babysit this", "get it green" | `session/babysit.md` |
| "land the stack", "ship it" | `session/shipping.md` |
| "open a PR" | `session/opening-a-pr.md` |
| "going to bed, keep going until X" | `session/autonomous-run.md` |
| "take over from the last session" | `session/session-pickup.md` |
| "pause safely, I'm going offline" | `session/pause-safely.md` |
| "clean up the worktrees" | `session/worktree-cleanup.md` |
| "reproduce it first" | `bee-principle-reproduce-first`, then `bugfix/bug-fix.md` |
| "write the failing test first" | `bee-tdd`, `bee-principle-red-before-green` |
| "is that number real?" | `bee-principle-explain-the-number`, `perf/perf-issue.md` |
| "prove it works" | `bee-principle-prove-it-works` |
| "write the README first" | `feature/feature.md` step 4 |
| "try two designs" | `bee-arena`, `bee-principle-exhaust-the-design-space` |
| "who else calls this?" | `bee-blast-radius` |
| "how does this run?" / "why is it like this?" | `bee-how` / `bee-why` |
| "tear this change apart" | `bee-interrogate` |
| "delete it, don't wrap it" | `bee-principle-the-deletion-test`, `bee-principle-subtract-before-you-add` |
| "don't ask, just do it" | `bee-principle-never-block-on-the-human` |
| "same bug again, stop patching" | `bee-principle-attack-the-premise` |
| "no comments, strip them" | `bee-comment-review` |
| "sounds like a machine wrote it" | `bee-unslop` |
| "nothing fits, this is big" | `bee-figure-it-out` |
