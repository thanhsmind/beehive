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
  a merged worktree); never stop mid-cell. When a waggledance door is
  present and `handoff_auto` is on, the next claim (`cells claim`,
  `cells claim-next`, `dispatch prepare --claim`), wave (`dispatch wave`),
  or `feature start` past the mark refuses `[handoff_due]`. Run
  `bee state handoff send --task "<owner's request in their words; standing orders>"`
  at once (`feature start` also accepts `--handoff-task`), and on
  `[handoff_sent]` tell the owner in one line and stop. Each handoff
  send is a new attempt with its own request id. `[handoff_deduped]`
  means waggledance returned an old run and sent nothing, so rerun the
  send (add `--follow-up-of <run>` to continue a finished run). Every
  sent handoff records what it must deliver (the feature branch merge,
  plus `--expect-merge`). `bee state handoff check`, which the
  SessionStart and Stop hooks also run, dispatches a follow-up leader
  when a finished run left a merge undone, and after 3 follow-ups names
  the row `needs-owner`. When a waggledance door exists, the Stop hook
  refuses a turn end that asks the owner to open a session, run
  `/clear`, or run a step the agent can run. When no
  waggledance door exists or `handoff_auto` is `false`, write
  `.bee/HANDOFF.json` with `--verified`, `--tree`, `--key-files`,
  `--gotchas`, `--standing-orders`, and `--ruled-out`, probe it once with
  a cold read (`bee dispatch prepare --runtime claude --kind gather --role read --json`
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
