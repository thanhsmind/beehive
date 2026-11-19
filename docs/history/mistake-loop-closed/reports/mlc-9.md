# mlc-9: test-role check behind mlc-7, mlc-7c and mlc-8

Checked HEAD cd6cbfab9 on wt/mistake-loop-closed. Merge base with main is 0b0f6d3bb.

## Tests

| Command | Result |
|---|---|
| `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test instruction_laws` | green: 10 passed, 0 failed |
| `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity` | green: 10 passed, 0 failed |
| `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity` | green: 5 passed, 0 failed |
| `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test agents_block_render_parity` | green: 1 passed, 0 failed |
| `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- no_shipped_command_spelling` | green: 1 passed, 0 failed, 5498 filtered out (`hooks::cli_shape::documented_invocations::no_shipped_command_spelling_is_refused_by_the_widened_guard`) |

## Knowledge check

`/mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check` is red, exit 1:

```
ERROR [missing_decision_link] areas/decision-memory/decision-memory-the-leader-triages-proposed-pbis-the-owner-gets-only-owner-only-calls.md: area concept "areas/decision-memory/decision-memory-the-leader-triages-proposed-pbis-the-owner-gets-only-owner-only-calls.md" carries no decision link — run bee decisions search --text "<topic>", then bee decisions log, and put the decision id in this concept's bee.decisions list
knowledge check: 330 concept(s) in 351 file(s), 0 OKF error(s), 1 profile error(s), 603 profile warning(s) — FAIL
```

This error comes from main, not from this feature:

- The concept was last changed in df30cfc21, which is an ancestor of the merge base 0b0f6d3bb.
- The same check run in the main checkout at 0b0f6d3bb shows the same single error: `1 profile error(s), 600 profile warning(s) — FAIL`.
- This branch changes four concepts and none of them is the failing one.

The branch adds three warnings over main:

- `long_concept` on `areas/human-mailbox/overview.md`, which mlc-8 edited: the body is 307 lines, past the 300-line limit.
- `not_canonical` on `areas/human-mailbox/overview.md`, which mlc-8 edited: the frontmatter does not match its re-emitted form.
- `stale_anchor` on `areas/decision-memory/audit-log.md`, because its anchor `packages/bee/LEADER.md` changed in mlc-7.

Seven other `stale_anchor` warnings that main already has now list more changed anchors.

## Concept content

| Command | Result |
|---|---|
| `rg -n -F 'fix-at close'` over the four concepts | match: `feedback-digest/ranking-and-self-improvement.md:151` and `:156`, naming `bee backlog fix-at close` as the only door (D7) |
| `rg -n -F 'reflect --cell'` over the four concepts | no literal match. `human-mailbox/overview.md:244` names the D2 behavior as `--cell <id>[,<id>...]`: only the cells a closing-run answer names are cleared |
| `rg -n -F 'Fix-at debt'` over the four concepts | match: `hook-runtime/token-economy-of-injected-context.md:57`, `:74`, `:79`, `:80` (D6 preamble block, DROPPABLE, `fix-at next` pointer) |
| `rg -n -F 'evidence_ref_missing_path'` over the four concepts | match: `okf-profile/conformance-check.md:90` (D10 split rule) |

Each of the four concepts describes the behavior its decisions shipped. Only `reflect --cell` fails the literal search. The mailbox concept describes the same behavior with the flag on its own.

## Verdict

Blocked. The cell says any red check means BLOCKED. `bee knowledge check` is red because of one profile error that main already had before this branch. Every test filter passed, and every concept content check passed. The only exception is the literal `reflect --cell` spelling noted above.

Not run: the full cargo suite and clippy. The cell allows only the named filters.
