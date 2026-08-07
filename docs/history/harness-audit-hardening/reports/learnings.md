# harness-audit-hardening — learnings

## What settled

- **Per-command help was the right instinct and it was holed**: the CLI already
  had `bee <cmd> --help`, but the text renderer hid every optional flag —
  exactly the flags skills instruct (`dispatch prepare --claim`). The fix
  (hah-4) renders the full `--flag*:type` surface, same V2 shape as the
  preamble. The preamble's 7.9KB Command surface can now be revisited
  (backlogged as a user decision — it reverses the locked
  `cli-surface-in-context` V2 choice).
- **Two recognizers, one cutover**: the R6 hook-command cutover updated the
  onboarding recognizer but not the plugin-migration one
  (`plugin_distribution.rs`), so migration cleanup silently became a no-op
  and migrated hosts double-fired every hook. Lesson: a spelling change must
  sweep every consumer of the old spelling — the repo's own "fix the fan-out"
  pattern, again.
- **Parse-failure is not absence**: `serde_json::from_str(...).ok()` turned a
  malformed host settings file into "no file", and the merge rewrote it whole.
  Any read-modify-write on a HOST-owned file must refuse on parse failure,
  never default to empty.
- **Single-session worktree lifecycle is a dead end today**: lifecycle verbs
  refuse inside a granted worktree; write-guard refuses cross-tree writes from
  main; `cells finish` run from main proves the wrong tree. Workable pattern
  used here (recorded deviation): control plane from main, edits + real test
  runs in the worktree, merge-time re-run as the binding proof. Backlogged for
  a decided model.
- **Guards must allow their own remedy**: the scratch-shape guard blocked the
  removal of the `.bak` it complained about; the plan-freeze guard blocked
  committing the frozen plan verbatim; `unsupported_argument_shape` still
  hides out-of-enum values. All three backlogged.

## Deferred (backlog, this feature's tag)

Windows Codex worktree fallback; doctor codex byte-compare vs merge;
statusline global write opt-in; renderer-duplication guard; apply-loop write
errors; submodule git-common-dir edge; skills-lock.json orphan; dead lib
group; preamble slimming (user decision).
