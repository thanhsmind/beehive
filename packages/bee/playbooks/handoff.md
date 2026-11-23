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
- Default context mark: 120000 (`handoff_context_tokens`; `0` disables).
  At or past the mark, hand off at the next clean stop (a capped cell or
  a merged worktree); never stop mid-cell. Write `.bee/HANDOFF.json`
  with `--verified`, `--tree`, `--key-files`, `--gotchas`,
  `--standing-orders`, and `--ruled-out`, probe it once with a cold read
  (`bee dispatch prepare --runtime claude --kind gather --role read --json`
  showing `bee state handoff show` and asking for the next action and
  ruled-out paths), rewrite once on a miss, then dispatch the fresh
  leader through the host's leader door (`waggledance dispatch --project
  <id> --leader --follow-up-of <run> --handoff --request-id <id>` with
  `--task`, `--task-file`, or stdin; MCP fallback follows rule
  `agents-other-project-to-its-leader`) and release this session. Never
  tell the user to open a new session, wait for `/clear`, or spawn a
  leader with `paseo create_agent`. Show the handoff and wait only when a
  user-owned gate or decision is pending, or when no leader door exists.
  Schema: `bee-swarming/references/swarming-reference.md`.
- After a green worktree merge, `bee feature start` refuses a second
  feature in that session (`handoff_due`). Rerun `bee feature start`
  with `--handoff-task "<owner's request in their words; standing orders>"`:
  bee writes the task file, dispatches the fresh leader itself, writes
  no `.bee/HANDOFF.json`, and releases the session. On `[handoff_sent]`,
  tell the owner in one line and stop. `--same-session "<reason>"` is
  accepted only when bee has no waggledance door or the owner set
  `handoff_auto` to `false` by hand; an owner asking for more in the
  same chat is never a reason to waive.
<!-- /rule -->
