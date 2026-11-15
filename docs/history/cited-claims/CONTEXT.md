# cited-claims — CONTEXT

## Ask

The owner asked to replace the bare `[verified]` claim tag with a
cite-the-source standard: number each checked claim `[1]`, `[2]`, `[3]`
in the text, and list the sources under the reply so the user can open
one when they need to.

## Findings that shaped the decisions

- `packages/bee/LEADER.md:199` (rule `agents-verify-before-agreeing`)
  asks for a `[verified]` or `[unverified]` tag on each technical claim.
- `skills/bee-hive/references/routing-and-contracts.md:311` holds the
  full form, with an inline `[verified: <file>:<line> ...]` example.
- `packages/bee-rs/crates/bee/src/hooks/session_close/claims.rs:37`
  counts a reply as labelled only when it contains `[verified`, so the
  Stop hook would warn on a reply that uses the new citation form.
- `.bee/LEADER.md` is the onboarded copy of `packages/bee/LEADER.md`.

## Locked Decisions

- **D1** (`06a46ab6`, contract `cited-claims-doctrine`) A technical claim
  the session checked carries a numbered citation `[1]`, `[2]` in the
  reply text. The reply ends with a Sources block (heading `Sources:` or
  `Nguồn:`), one line per number: `[n] path:line`, or `[n]` the command
  run with its result. One source keeps one number on reuse. A claim
  from general knowledge or inference keeps the inline `[unverified]`
  marker and gets no number. Agreement waits until the key assumption
  has a citation.
- **D2** (`4c6f834c`, contract `cited-claims-stop-check`) The Stop
  hook's agreement check counts a reply as cited when it holds an inline
  `[n]` and a line that starts with `[n]` after a `Sources:` or `Nguồn:`
  heading, or the legacy `[verified` marker. The warning names the
  citation form. It stays advisory, Stop only, never a block.

## Read set

### Knowledge

| Path | Why |
|---|---|
| docs/knowledge/areas/hook-runtime/overview.md | Stop hook agreement warning concept |
| docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md | rule index row for agents-verify-before-agreeing |

### Code

| Path | Why |
|---|---|
| packages/bee-rs/crates/bee/src/hooks/session_close/claims.rs | the agreement check |
| packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs | its tests |
| packages/bee/LEADER.md | the doctrine rule |
| skills/bee-hive/references/routing-and-contracts.md | the full Claim labels form |
