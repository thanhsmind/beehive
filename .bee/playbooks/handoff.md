# Handoff

A handoff record has two
kinds — `planned-next` and `pause` — and a kindless record reads as
pause:

- A `pause` handoff is presented to the user; wait for their word —
  never auto-resume it.
- A `planned-next` handoff was written at a clean stop with
  `bee state handoff write --kind planned-next` (previous cell capped,
  the next cell's claim already owned by the writer); take its carried
  claim with `bee state handoff adopt`. Adoption fires only at the
  fresh-session boundary — a resumed or compacted session never
  adopts, it surfaces and waits.

<!-- rule: agents-context-handoff-65 -->
- The default context mark is 200000 (`handoff_context_tokens`; 0 turns
  it off). At or past it the leader hands off at the next clean stop (a
  capped cell or a merged worktree). After every green worktree merge the
  leader writes `.bee/HANDOFF.json` and continues in a fresh leader session
  through the host's leader door. It never stops mid-cell. The door, when
  one exists (waggledance: `waggledance dispatch --project <this project id> --leader --follow-up-of <current run> --handoff --request-id <id>` with the task through `--task`,
  `--task-file` or stdin; the MCP fallback follows rule
  `agents-other-project-to-its-leader`), keeps the
  successor in the same run family; task text names what is done,
  what is next, and the user's standing orders. Release the
  session after the handoff. Never tell the user to open a new session, never wait for a
  `/clear`, and never spawn the leader yourself (no `paseo
  create_agent`). Show the handoff to the user and wait only when a
  gate or decision the user owns is pending, or when the host has no
  leader door. The record's schema lives in
  `bee-swarming/references/swarming-reference.md`. A merge stores the
  merging session. `bee feature start` then refuses a second feature in
  that session (`handoff_due`) unless `--same-session` carries a reason.
<!-- /rule -->
