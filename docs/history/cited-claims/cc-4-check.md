# cc-4 check: citation doctrine text and skill pointers

Verdict: **pass**

Checked commit: `004a769f8` (cc-1, "Teach numbered source citations in the claim doctrine").
Decision checked against: D1 (`06a46ab6`, contract `cited-claims-doctrine`).

## What was checked

| Check | Result |
|---|---|
| `packages/bee/LEADER.md` rule `agents-verify-before-agreeing` teaches a numbered citation `[1]`, `[2]` in the text and a `Sources:` / `Nguồn:` block under the reply, one line per number (`[n] path:line` or `[n]` command with result), one number per reused source | pass |
| The same rule keeps the inline `[unverified]` marker for general knowledge or inference, with no number, never stated as fact | pass |
| The same rule gates agreement on a citation ("until its key assumption has a citation") and the Stop hook line names a citation, not `[verified]` | pass |
| `skills/bee-hive/references/routing-and-contracts.md` § Claim labels teaches the same form, with an example that uses `[1]`, `[2]` and a `Sources:` block | pass |
| § First reaction gates on a citation, not on `[verified]` | pass |
| No bare `[verified]` tag request remains: `rg -n '\[verified' packages/bee/LEADER.md skills/bee-hive/references/routing-and-contracts.md` returns no match | pass |
| `.bee/LEADER.md` is byte-identical to `packages/bee/LEADER.md` (`cmp` exits 0) | pass |
| Pushback text unchanged: the cc-1 diff (`git show 004a769f8 -U0`) touches no line containing "Pushback" or "you are wrong" in either doctrine file | pass |
| Rendered copies of `references/routing-and-contracts.md` under `.claude-plugin`, `.codex-plugin`, `.claude`, `.agents`, `.opencode` are byte-identical to the source (`cmp` exits 0 for all five) | pass |
| Rule index spoken line in `router-triage-and-the-agents-md-duplication-boundary.md` names the numbered citation | pass |

Side note, not a cc-1 defect: `diff -rq` over the whole `bee-hive` tree shows
`references/gates-and-delegation.md` differs in every rendered copy. The
difference is the harness-only `<!-- bee:only pi -->` and
`<!-- bee:only codex -->` blocks the renderer strips per target; cc-1 did not
touch that file and the release manifest check below is clean.

## Command output summary

- `cargo test --release ... -p bee --test rule_index_parity`: 5 passed, 0 failed.
- `cargo test --release ... -p bee --test pointer_integrity`: 10 passed, 0 failed.
- `.bee/bin/bee dev release-manifest --check`: "950 file(s) match stored manifest, 1 unhashed artifact(s) present", exit 0.
