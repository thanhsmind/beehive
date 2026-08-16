# guard-refusal-wording — CONTEXT

## Asked
Backlog p-aa37ec95: the Bash write-guard refusal names a shell token instead of a
path when the target cannot really be resolved — observed as `writing "cp" is
blocked` (compound command) and `writing "$WT/..." is blocked` (unexpanded
variable). Operator cannot tell a gate refusal from a target-resolution miss.

## Found
- Extractor: `hooks/write_guard/guards.rs:607-732` (`extract_bash_targets`) collects
  raw tokens after `rm|mv|cp|mkdir|touch|tee`; no resolution there.
- Resolution: `hooks/write_guard/main.rs:189-207` maps each token via
  `canonical_rel_path`; failures take the containment branch (main.rs:218-247,
  `GENERIC_BASH_CONTAINMENT_MESSAGE` / `describe_cross_worktree_target`).
- Mechanism of the bug: a bogus token like `cp` or `$WT/foo` resolves as a literal
  relative path under cwd, so it lands in `rel_paths` and reaches the gate
  sentences "writing X is blocked" (checks.rs:486,501,565) naming a fake path.

## Will do (locked)
- D1: A token carrying unexpanded shell syntax (`$`, backquote) is classified
  unresolvable — never treated as a literal path.
- D2: The unresolvable-target refusal says the target could not be resolved and
  quotes the raw token, clearly distinct from the gate "writing X is blocked"
  sentence.
- D3: Regression tests: (a) a Bash command with an unexpanded `$VAR/...` target is
  refused with the resolution-failure wording, not a fake path; (b) existing
  resolved-path refusals keep their wording.
- Out of scope: rewriting the tokenizer/extractor beyond D1's classification.

## Open questions
None.

## P1 fix round (review rev-backlog-fixes-20260816, approved by user 2026-08-16)

Review found two P1s in grw-1's implementation; user said fix.

- D4: The shell-syntax classification applies ONLY to the Bash surface. It moves
  out of the shared resolvers (`canonical_rel_path`, `resolve_target_realpath`)
  so Edit/Write/MultiEdit `file_path` and apply_patch targets — literal strings
  no shell expands — resolve exactly as before grw-1 (a file named `Foo$Bar.java`
  is a valid literal there).
- D5: On the Bash surface an unresolvable shell-syntax token DENIES outright,
  before any companion/delegate branch. With `.bee/companion-session.json`
  present, a `$`/backquote Bash target must still deny — never Delegate into the
  dispatcher's fail-open allow.
- D6: Tests: (a) edit()/patch() twins proving a literal `$`-named file_path is
  allowed again on those surfaces and denied paths keep old wording; (b) a
  companion-marker + `$VAR` Bash case asserting deny with the D2 wording;
  (c) the three existing Bash regressions stay green unchanged.
