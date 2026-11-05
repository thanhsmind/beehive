# CONTEXT: worker-status-digest

## Ask

A leader learns a worker's progress from one status line, never from the
worker's transcript, pane log or activity stream. The pattern comes from
Claude Code, which cut background-subagent status cost by about 99% when a
small model wrote the status line from short notes instead of re-reading
the subagent's whole conversation.

## Locked decisions

- **D1** (`d9007aca`) — A leader never reads a worker's transcript, pane
  log or activity stream to learn its progress. Workers append one-line
  progress notes to their mailbox; a cheap supervisor-role model composes a
  one-line status from those notes; the leader reads only that line
  (`bee herding status`). `waggledance_await` gains a summary-only mode for
  the same reason.
- **D2** (`d981106a`) — A progress note is one JSON line `{ts, step, note}`
  in `.bee/mailbox/<job>/progress.jsonl`, note at most 120 characters.
  `bee herding notify --kind progress` writes there (not to
  `messages.jsonl`) and takes `--step` and `--job`; with `--job` it needs no
  `job.json`, so a native bee-worker passes its cell id as the job. A
  herded bee-ignorant worker appends the line itself to the absolute path
  its brief names. `messages.jsonl` keeps the notes the leader must act on
  and bee's own stall alert.
- **D3** (`28b87b5b`, supersedes `657bbe71`) — the revised herding-notify
  contract that D2 implies.
- **D4** (`b4fd9daa`) — `bee herding status --job <id> [--json]` prints one
  line and reads only `progress.jsonl` and the latest `result-N.json`.
  Order: a result file wins; no notes and no result prints
  `no progress notes yet`; 3 notes or fewer prints the last note verbatim;
  more than 3 notes asks the supervisor role through the dispatch door only
  when it resolves to a runnable Bash command, and caches the line in
  `status-digest.json` keyed by the note count; a native-model role, a
  failed run or a timeout falls back to the last note verbatim. JSON carries
  `job`, `line`, `source` (`none|result|verbatim|supervisor|cached`),
  `notes`.
- **D5** (`610b73c9`) — No plan-step hat wave (the spec is fully stated by
  the user). Merge only after release 2.75.0 has landed on main.

## Out of this repo

The `waggledance_await` summary-only mode belongs to the waggledance repo.
It is filed there as a PBI through the waggledance leader; beehive never
edits that repo.

## Measure

The delivery record carries the leader-side cost of one sample wave before
(what the leader read to learn progress) and after (the status lines).
