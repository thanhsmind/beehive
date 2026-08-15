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
