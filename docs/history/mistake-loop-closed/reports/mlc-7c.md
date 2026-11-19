# mlc-7c: check report for mlc-7

Checked commit 6910acaa7 ("Name reflect --cell and fix-at close in the doctrine") against CONTEXT.md D11.

## Parity and pointer tests

| Command | Result |
|---|---|
| `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity` | green: 10 passed, 0 failed |
| `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity` | green: 5 passed, 0 failed |
| `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test agents_block_render_parity` | green: 1 passed, 0 failed |
| `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test instruction_laws` | green: 10 passed, 0 failed |
| `.bee/bin/bee dev release-manifest --check` | green, exit 0: `982 file(s) match stored manifest, 1 unhashed artifact(s) present` |

## Content checks

| Command | Result |
|---|---|
| `rg -n 'reflect --cell' packages/bee/LEADER.md` | match, line 377 |
| `rg -n 'reflect --cell' packages/bee/AGENTS.block.md` | match, line 26 |
| `rg -n 'reflect --cell' AGENTS.md` | match, line 33 |
| `rg -n 'fix-at close' expertise/static-rules.md` | match, line 77, carries `--guard`, `--guard-cmd`, `--red-proof`, `--commit` |
| `rg -n 'fix-at close' skills/bee-capturing/references/promotion.md` | match, line 129, the same flags |
| `rg -n 'type backlog-closed' expertise/static-rules.md` | no match, exit 1, as required |

## Verdict

Every check is green. The doctrine names `reflect --cell` in all three files and names `bee backlog fix-at close` with its proof flags where D11 puts it. The hand-written backlog-closed row is gone from static-rules.md.

Not checked: the full cargo suite. The cell allows only the verify filter.
