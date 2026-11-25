# srcchk-4 — Claim labels source-check paragraph check

Verdict: **pass**

## What was checked

Commit `682f29869` (srcchk-3) adds one paragraph at
`skills/bee-hive/references/routing-and-contracts.md:274`, in Claim
labels and pushback, between the second Claim-label example and
**Pushback**.

| Paragraph says | `citations.rs` does | Match |
|----------------|---------------------|-------|
| opens each `path:line` source | `find_path_token` + `check_path_source` read the file under the root | yes |
| warns when the file is missing | `std::fs::read` fails gives `file not found` | yes |
| the line is past the end | start or end of 0 or past `line_count` gives `line N past end` | yes |
| no backticked token of the claim or source line within 20 lines | `collect_tokens` takes backticked spans of 3+ chars, not the path, from the source line and reply lines with the same inline `[n]`; window is `lo-20 .. hi+20`; none found gives `text not found` | yes |
| warns when a cited command was never run in the session | `find_command_source` + `command_was_run` over the session's shell commands gives `command not run` | yes |
| warns (advisory) | `mod.rs:212` chains the result into the Stop warnings, never a block | yes |

The paragraph names `path:line` only; the code also checks a bare path
against the whole file (D1). The paragraph is narrower, never wrong, so
this is not a fail.

Pushback text and both examples are unchanged: `git show 682f29869
--numstat` on the source file gives `2 0` (paragraph plus blank line,
no deletions), and no commit after it touches the file.

## Rendered copies

Each copy holds the paragraph once (`rg -cxF`) and is byte-identical
to the source file (`cmp`):

- `.claude/skills/bee-hive/references/routing-and-contracts.md` — count=1, identical
- `.agents/skills/bee-hive/references/routing-and-contracts.md` — count=1, identical
- `.claude-plugin/skills/bee-hive/references/routing-and-contracts.md` — count=1, identical
- `.codex-plugin/skills/bee-hive/references/routing-and-contracts.md` — count=1, identical
- `.opencode/skills/bee-hive/references/routing-and-contracts.md` — count=1, identical

## Command output summary

- `cargo test --release -p bee --test pointer_integrity` — 10 passed; 0 failed
- `bee dev release-manifest --check` — exit 0, `994 file(s) match stored manifest, 1 unhashed artifact(s) present`
