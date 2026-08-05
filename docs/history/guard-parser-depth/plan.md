# guard-parser-depth — plan

**Lane:** high-risk (hard-gate flag `audit-security`)
**Origin:** review session `unreviewed-batch-20260805-a`, findings
`p1-guard-compound-bypass` and `p1-guard-shell-wrapper-bypass`.

## What is broken

The write guard decides from a token list. Two holes in how that list is built
let any Bash command walk past every check the guard makes.

**1. Only the first git invocation is classified.** `find_git_invocation`
(`packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs:165`) walks tokens,
finds the first one whose basename is `git`, and **returns**. Later
separator-delimited segments are never examined. Confirmed against the binary
in this checkout:

```
git stash                  exit=2  refused (2 workers live)
git status && git stash    exit=0  ALLOWED
```

**2. A shell wrapper hides the command entirely.** `tokenize`
(`hooks/write_guard/guards.rs:206`) folds a quoted span into one token, and no
consumer knows `sh` / `bash` / `eval` execute their argument. The payload never
becomes tokens, so there is no target to extract, no separator to scan and no
git verb to classify:

```
echo x > .bee/state.json              exit=2  refused (CLI-owned)
sh -c 'echo x > .bee/state.json'      exit=0  ALLOWED
bash -c 'git stash'                   exit=0  ALLOWED
eval 'git stash'                      exit=0  ALLOWED
```

Together these lift the CLI-only-state boundary (including hand-editing
`approved_gates`), the reservation conflict check, the secret-shaped-path deny,
and the concurrent-worker git rule that exists because a whole-tree revert
already deleted a live worker's edits twice in one wave.

## Shape — fix the parser, not the doors

Both holes are in how tokens are produced and consumed, not in any individual
check. Patching `checks.rs` would fix the git rule and leave target extraction
and the secret deny still blind — the failure the repo's own critical pattern
*"Fix the law, not the line the report cited"* names. So the fix lands at the
two shared helpers, and every consumer inherits it.

**S1 — see through shell wrappers.** A new `tokenize_deep` wraps `tokenize`:
after tokenizing, expand any wrapper payload in place, then re-scan. A wrapper
is a token whose basename is `sh`, `bash`, `dash`, `zsh` or `ksh` followed by
`-c` and a payload token, or the token `eval` followed by its arguments up to
the next separator. The payload is tokenized and spliced in, fenced by `;` so a
wrapper can never join two segments into one. Recursion is bounded (depth 4,
then the command is treated as opaque and refused rather than allowed — a
wrapper nested deeper than that is not something an honest worker types).

**S2 — classify every git invocation.** `find_git_invocation` becomes
`find_git_invocations`, returning every invocation in the token list. The one
caller (`checks.rs:599`) loops; the first refusing invocation wins, so an
allowed leading command can no longer shadow a denied one.

**S3 — wire and prove.** Route the three write-guard consumers
(`guards.rs:304` target extraction, `detectors.rs:226`, `checks.rs:598`)
through `tokenize_deep`. `cli_shape.rs:586` keeps plain `tokenize` — it parses
bee's own argv shape, where a wrapper is not a thing.

## What must not break

The guard already fails open on an undecidable payload, and that stays. The
danger of this change is the other direction — over-blocking:

- `echo "git stash"` and `git commit -m "wip && git stash"` must stay allowed.
  A quoted span is only re-tokenized when it is a wrapper's payload, never
  because it merely contains shell-looking text.
- `git -C sub status` and `git --git-dir=.git log` keep parsing as today.
- A literal path containing `sh` (`scripts/shell/thing.sh`) is not a wrapper —
  matching is on the token's basename equal to a shell name, not a substring.

## Cells

| Cell | Does | Verified by |
|---|---|---|
| `gpd-1` | S1 + S2 + S3 in one cell — they are one parser and splitting them ships a half-fixed guard | new cases in the write-guard test module: compound git after `&&`, `;`, `\|\|` and `\|`; `sh -c` / `bash -c` / `eval` wrapping both a git verb and a state-file redirect; nested wrapper; depth-limit refusal; plus the four negative controls above. `cargo test --release` |

One cell, not three: an intermediate state where `tokenize_deep` exists but
`checks.rs` still reads the first invocation only is a guard that reports
itself fixed and is not.

## Smaller path

Considered: deny any command containing a shell wrapper outright, no parsing.
Rejected — `sh -c` appears in legitimate build and CI invocations, and a guard
that refuses honest work gets switched off, which costs more than it saves.
Parsing the payload is barely more code and keeps the guard usable.
