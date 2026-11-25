# citation-source-check — CONTEXT

## Ask

The owner asked for a code eval on citations: search the cited file for
the related text, to see if the source an agent gives is really there.

## Findings that shaped the decisions

- `packages/bee-rs/crates/bee/src/hooks/session_close/claims.rs:61`
  `is_cited` only checks that an inline `[n]` and a Sources line `[n]`
  exist. It never opens the cited file.
- `packages/bee-rs/crates/bee/src/hooks/session_close/mod.rs:208` adds
  the claims warnings to the Stop output on the Done and Crash paths.
- `skills/bee-hive/references/routing-and-contracts.md:258` shows the
  two source forms: `path:line` and `command — result`.

## Locked Decisions

- **D1** (`70692733`, contract `citation-source-check`) The Stop hook
  checks each Sources line of the last reply against disk. A
  `path:line` or `path:a-b` source fails when the file is missing or the
  line is past the end of the file. When the source line, or a reply
  line that carries the same inline `[n]`, holds a backticked token of
  three or more characters that is not the path, at least one such
  token must appear in the file, case-insensitive, within 20 lines of
  the cited range. With no line number the whole file counts. No token
  found fails as text not found. A URL source or a source with nothing
  checkable is skipped. One advisory line names up to three failed
  sources with the reason. Stop only, never a block.
- **D2** (`b74990d9`) A command source is a source with no path and no
  URL that holds a backticked span or has the form `<command> —
  <result>`. It fails as command not run when no shell command in the
  session transcript holds its first word and, when it has more words,
  at least one other of its words.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/hook-runtime/overview.md | suspect | B2 Stop agreement warning concept |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/hooks/session_close/claims.rs` | the citation parse and agreement check |
| `packages/bee-rs/crates/bee/src/hooks/session_close/mod.rs` | the Stop warning wiring |
| `packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs` | the Stop check tests |
| `skills/bee-hive/references/routing-and-contracts.md` | the Claim labels form |
