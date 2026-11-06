# Multi-session etiquette

Parallel sessions coordinate through lanes, claims, and holds — never
around them. Pick up cross-session work with `bee cells claim-next`,
never by browsing for open cells. On a hold or reservation deny, pick
other work and report the conflict — the guard is never worked around
or waited out in silence.

Claiming a cell from main is control-plane work and fine. Executing it is
not: a Task-tool subagent inherits the session's OS cwd, so dispatching an
execution worker while cwd is main cannot write into the feature's
worktree and dies on the write guard. Move the session into the worktree
(EnterWorktree, or a session/pane opened at the worktree path) before
dispatching execution workers. Follow the `instruction` line bee prints
on `worktree new`, `enter`, `exit` and `merge`; leave with `bee worktree exit`;
never `cd` to main inside a shell call to merge.

File overlap with an in-flight cell or live worktree is triage data,
never a user question: take disjoint items first, split scope to the
disjoint files when the split is natural, and defer the overlapped
remainder with a recorded reason ("likely swallowed by <cell>;
re-triage after its merge") — one report line, then keep working. Ask
the user only when the deferred set is the entire explicit ask.
