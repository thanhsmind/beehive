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

## Cell

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| tt-5 | Cap the promote-proposal line in the session preamble | hooks/session_preamble/render.rs, hooks/session_preamble/tests.rs, the token-economy concept | tt-1, tt-4 | the preamble names 3 clear commands and `+N more` | cargo test session_preamble; live render under 9,500 bytes |
