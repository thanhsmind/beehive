# thin-worker-boot — CONTEXT

## Feature Boundary

A dispatched peer (herding pane or native subagent) starts with only what
its job needs: a short peer role text, its brief, and the project's hard
boundaries. The bee flow doctrine stays with the leader. Token use of
every worker is measured, so the change is judged on numbers.

Out of scope: the leader's own flow, gates, and the review chain.

## Evidence (2026-11-06)

- Each pane worker boots with 28-51k fresh cache-write tokens; only
  10-14k (tool definitions) are reused across workers.
- `.bee/usage/claude-sub-session-dispatch.json` records 39k Sonnet
  tokens; the pane transcripts of that feature hold about 9.2M.
- Every worker loads AGENTS.md + CLAUDE.md (~32k chars), the user
  memory, about 100 skill descriptions, and MCP server instructions.
- `bee herding run` reads only `.bee/config.json`; agents declared in
  `.bee/config.local.json` fail at spawn (issue 166).

## Locked Decisions

- **D1** (`16cccbb2`) A dispatched peer knows no bee flow: peer role
  text plus brief, do the work, prove it, hand back.
- **D2** (`48021ff8`) One thin shared instruction file holds only the
  hard boundaries; the flow doctrine moves to a leader-only file peers
  never load.
- **D3** (`8bc5d1ee`) The peer ends with the one finish command its
  brief names. Consistent with `7152ebab` (one runnable finish
  instruction, last in the prompt).
- **D4** (`9e4bcc76`) Pane peers launch trimmed: auto-memory off, no
  skill listing, no extra MCP, per-machine prompt sections out of the
  system prompt; project-level guard hooks stay on. Rejected: `--bare`
  (skips guard hooks, no OAuth), a per-role config dir (breaks OAuth).
- **D5** (`16eb7a31`) Pane tokens count in the feature usage record;
  each capped cell records its own token totals.
- **D6** (`2d911143`) The thin boot ships behind a config switch, is
  piloted on 3-5 cells, and becomes default only on a measured saving.
- **D7** (`60186f26`) `bee herding run` reads the merged tracked + local
  config, the same view `bee dispatch prepare` reads.

- **D8** (`3173e22f`) The trimmed launch has two layers: an env layer
  always on with the switch, and an optional flag layer through a
  user-installed paseo provider. paseo run passes only `--env`.
- **D9** (`3f97d8d1`) The leader doctrine file is never `@`-imported;
  the hook injects it for leaders; the thin block names it in one line
  for hosts without the hook (Codex).
- **D10** (`458619d9`) Env layer: `CLAUDE_CODE_DISABLE_CLAUDE_MDS=1`,
  `CLAUDE_CODE_DISABLE_AUTO_MEMORY=1`, `ENABLE_CLAUDEAI_MCP_SERVERS=false`,
  `CLAUDE_CODE_DISABLE_BUNDLED_SKILLS=1`, plus the role-aware preamble.
  Partial by design: plugin and user skills and local MCP servers still
  load; the pilot reports what stays.

## Prior decisions cited

- `b340d3f9` a herding or cli dispatch carries the bee agent body at
  the head of its stdin.
- `7152ebab` one finish instruction per worker; the runner never caps
  for the worker.

## Open Questions

- Closed: paseo passes only `--env` (D8); Codex leaders (D9).
- Open, blocks the default flip: does the project SessionStart hook
  still fire in a pane launched with `CLAUDE_CODE_DISABLE_CLAUDE_MDS=1`?
  If not, the peer preamble never reaches the worker. The pilot checks
  it on a real pane first.
- Open: the leader doctrine file name (slice 5).
