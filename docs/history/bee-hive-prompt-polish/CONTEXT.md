# bee-hive-prompt-polish — CONTEXT (short brief)

## Asked

Tighten prompt quality of `skills/bee-hive` (SKILL.md + 5 references) to
the agent-skills style: imperative rules, trigger framing, constraint
before rationale, token economy. Scope: exactly those six files.

## Found

The files are dense doctrine; honest gains are surgical trims only —
duplicated sentences, decorative rationale, passive voice, filler
lead-ins. Hard guardrails for the pass: semantics preserved, headings
frozen (pointer_integrity), citations/pointers verbatim, no new numeric
limits (instruction_laws), no cross-file moves, net lines down or flat.

## Will be done

Apply the 14-edit patch in `reports/bee-hive-prompt-polish.patch`
(net -3 lines; validated on a patched clone: applies clean, and after
the regen chain the FULL suite is green — 2003 passed, 18 suites).
The patch alone leaves render-parity red: after `git apply -p1`, run
`bee dev render-skill-trees` and `bee onboard --repo-root . --apply`
to re-render the committed mirror trees, or the parity tests fail.
The editing session was cwd-pinned to main, so the source write
happens in the feature worktree `beehive--wt--bee-hive-prompt-polish`
(already created): from a session at that path, approve the merged
shape+execution gate, apply the patch, run the regen chain, run
`commands.test`, commit, land via `bee worktree merge`.
