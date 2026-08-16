# write-guard-hardening — CONTEXT

## Asked
Four backlog items from review rev-backlog-fixes-20260816 and live observation,
user said do them all (2026-08-16): p-06af049e (extractor operand roles),
p-e7b82571 (mixed-target delegate fail-open), p-f82b037d (refusal message
hardening), p-2eb26c53 (brace/cd family gaps).

## Found
- Extractor: `hooks/write_guard/guards.rs:607-732` `extract_bash_targets`
  collects every non-flag token after `rm|mv|cp|mkdir|touch|tee` — for cp/mv
  the SOURCE operands are reads, not writes. Live false denies observed on
  `cp SRC DST` (named SRC) and on a `2>/dev/null` token.
- Delegate: `main.rs` per-target loop lets a containment-failing literal target
  raise `Err(Nd)` → `Outcome::Delegate` → `emit_undecidable` fail-open with a
  companion marker or `guards.memory_root` present, swallowing denials already
  decided for sibling targets (probes ADV-A/ADV-B; pre-existing since 2.0.1).
- Messages: raw token echoed unbounded/unescaped to stderr (`hook_local.rs:41-49`);
  literal-$ filename gets a FIX line about expanding a variable that does not
  exist; `main.rs:313-332` check_write loop not guarded by `denial.is_none()`,
  overwriting the resolution-failure message; two tests self-referential.
- Family: brace expansion (`.bee/sta{t,t}e.json`) and `cd` are invisible to the
  guard — the shell rewrites what the guard checked.

## Will do (locked)
- D1 (extractor, p-06af049e): `extract_bash_targets` learns operand roles for
  `cp`/`mv`: only the LAST non-flag operand (or the `-t <dir>` argument) is a
  write target; sources are never extracted. Fd-digit redirects to `/dev/null`
  (`2>/dev/null`, `2>>...`, `&>/dev/null`) never produce a refusable target.
  `rm/mkdir/touch/tee` keep all-operand extraction (correct today).
- D2 (delegate, p-e7b82571): per-target verdicts are decided BEFORE the
  delegate escape — if any target already produced a native denial, the hook
  denies; `Err(Nd)` from one target never converts sibling denials into a
  fail-open allow. Fail-closed direction only.
- D3 (messages, p-f82b037d): token echo in refusals is length-bounded and
  control-char-stripped; the unresolvable message names the literal-$-filename
  case instead of promising a variable expansion; the check_write loop respects
  an earlier denial (`denial.is_none()` guard) so the first decisive refusal
  wording survives; new-message tests pin literal fragments, not the message
  function itself.
- D4 (family, p-2eb26c53): a token carrying comma-or-range brace expansion
  (`{a,b}`, `{1..9}`) classifies unresolvable (D2-style deny naming brace
  expansion); a compound command containing `cd` makes subsequent write-verb
  targets unresolvable (deny naming the cd opacity). Plain glob characters
  (`*`, `?`, `[`) are deliberately NOT classified — globs are everyday usage
  and current behavior stands. Fail-closed only: no new allow anywhere.
- Order: D1 → D2 → D3 → D4, one cell each, same worktree — the files overlap.

## Open questions
None.
