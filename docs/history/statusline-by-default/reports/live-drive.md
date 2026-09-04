# statusline-by-default — live drive (`green:live`)

Real `bee onboard` runs against throwaway host repos, driven with the binary
built from this branch. Scripts:
`scratchpad/live-verify.sh` and `scratchpad/live-optout.sh` (session scratchpad,
reproduced below as their observed output).

The first attempt at this drive **found a real defect and then disproved it**:
the settings file was not written and the recheck stayed `changes_needed`. The
cause was not the code — it was a stale binary, built before the `apply.rs`
edit. Pattern `20260805 source that ships without reinstalling the binary is
inert`, hit again. Rebuilt, re-driven, and the run below is the real one.

## The happy path — a virgin host

```
=== BEFORE: does the host declare a statusLine?
settings.json ABSENT

=== PLAN (writes nothing)
status: changes_needed
statusline actions: [{'action': 'copy_statusline', 'path': '.claude/statusline-command.sh'},
                     {'action': 'merge_statusline_settings', 'path': '.claude/settings.json'}]

=== AFTER: the host's own settings entry
{
  "type": "command",
  "command": "[ -f \"${CLAUDE_PROJECT_DIR:-.}/.claude/statusline-command.sh\" ] && bash \"${CLAUDE_PROJECT_DIR:-.}/.claude/statusline-command.sh\""
}
script present: .claude/statusline-command.sh  (4018 bytes)

=== RECHECK (the one scripts/install.sh fails the install on)
status: up_to_date
remaining plan items: 0

=== the status line, actually run in that host
/tmp/…/sandbox-host | master | Opus 5 | ctx: 72%
(exit 0)
```

One apply, then `up_to_date` with zero remaining items — the one-pass
convergence `scripts/install.sh` turns into a pass/fail on every install.

## The ways out, and the ways in that must not fire

```
=== CASE A: --no-statusline on a virgin host
settings.json: absent
script: absent
recheck: up_to_date

=== CASE B: a host that already declares its OWN statusLine
settings.json: byte-identical (left alone)
script: absent

=== CASE C: a host whose settings.json does not parse
settings.json: byte-identical (left alone)
apply exit code: 0 (0 = the run was not blocked)

=== CASE D: .bee/config.json statusline:false
settings.json: absent
script: absent

=== CASE E: --repo-hooks + the statusline default, one settings file, one .bak
.bak count: 1
.bak holds the ORIGINAL file
final keys: ['model', 'hooks', 'statusLine']
```

Case C is the one that matters most for a default: a host with a broken
settings file is left untouched and the install still succeeds. Case E is the
backup-of-a-backup fix, observed rather than argued — two writers, one settings
file, one `.bak`, and it still holds the pre-bee bytes.
