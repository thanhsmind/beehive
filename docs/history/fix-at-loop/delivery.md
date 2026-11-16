# Delivery: Close the fix-at loop

Feature `fix-at-loop`, standard lane, decisions D1 to D10 in `CONTEXT.md`.

## What shipped

- **One mistake, one row.** `bee close` files a fix-at finding only when no finding with the same layer and normalized title exists in the current close epoch, across all features. The close line names the owning feature when it skips. `bee backlog add` keeps its per-feature dedupe (D1, D10). Commits `6e47e2033`, `351589222`.
- **Digest fold.** Backlog `finding` and `friction` rows with the same type, layer, normalized title and close epoch read as one entry with the earliest `ts`. A copy filed after a close stays separate, so rank still reports `recurred`. The store is not rewritten (D2, D10). Commit `97bd2eeff`.
- **Open and closed witness.** `prose_load` gains `fix_at_check_open`, `fix_at_architecture_open`, `fix_at_closed`, `preamble_bytes` and `preamble_lines`. The preamble is null on a render failure (D6, D8). Commit `97bd2eeff`.
- **First `clippy.toml`.** `packages/bee-rs/clippy.toml` bans `Path::strip_prefix`. 21 `src/` sites moved to `strip_root_lexical`, except the history-root check in `leader_check.rs`, which takes `relative_to`. The 4 `tests/` sites stay (D5, D9). Commits `544d42f51`, `9ce563f80`.
- **Doctrine.** A landed guard closes its row with a `backlog-closed` row naming the guard, in `expertise/static-rules.md` and `skills/bee-capturing/references/promotion.md`. The `strip_prefix` row was closed this way (D7, D8). Commit `5857cf59b`.
- **Knowledge.** The human-mailbox and feedback-digest concepts describe the new identity, fold and load fields. The path-identity concept's Pointers name `clippy.toml` and `path_identity.rs`.

## Proof

- Tests: five `fix_at_identity` (two red before `6e47e2033`), six `fix_at_open` and `prose_load` (five red before `97bd2eeff`), four `relative_to`. Each carries its marker name.
- Red for the ban: the probe clippy run listed 21 `src/` sites before the move; `cargo clippy -- -D warnings` is green after it.
- Final check: the cell's proof command and `.bee/bin/bee knowledge check`, output in the cell report.

## Not done

- Historic copies in `.bee/backlog.jsonl` stay; the fold hides them at read time.
- The other 300-odd fix-at rows are open until each gets its own guard.
- `tests/` is outside the lint.
- After merge the leader must rebuild and reinstall `.bee/bin/bee` and check a new string in the installed binary before calling this live.
