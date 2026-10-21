# Token thrift — slice 2

## Why a second slice

After tt-1 to tt-3, the leader built the branch binary and rendered the
session-start output on this repo:

```
bee hook session-init | wc -c  →  11444
### Omitted to fit the host limit: Recent decisions → …; Critical patterns (digest) → …; Command surface → …; Project map → …
```

All four droppable sections were gone and the output was still over the
9,500-byte budget. The `### Unapplied promote proposal(s)` line carried 74
`bee deferred-queue complete` commands (about 5 KB). D5 capped the same list in
`bee orient` only; the plan did not see the preamble copy, because the scout
measured the line with the installed 2.55 plugin, which printed no commands.

Decision D7 (store `691fcac4-14a8-40b1-8600-d720ebc47e6f`, contract
`preamble-promote-section`): the preamble line keeps the count and the newest
path, and names at most 3 clear commands, then `+N more`.

## Where it landed

The approved cell packet is frozen, so no new cell was added. tt-1 was
reopened: its outcome (fit the host limit on this repo) was not met, and the
missing piece sits on that outcome's path. The fix landed in
`hooks/session_preamble/render.rs` (`PROMOTE_CLEAR_COMMANDS_SHOWN = 3`) with
the test `five_unapplied_promote_proposals_name_three_clear_commands_and_the_rest_as_a_count`.
tt-4 added the D7 sentence to the token-economy concept.

Live render after the fix: 9,224 bytes; dropped Recent decisions and Critical
patterns (digest), each named with its fetch path.
