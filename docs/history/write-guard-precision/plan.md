# write-guard-precision — plan

## Why

A full read of `packages/bee-rs/crates/bee/src/hooks/write_guard/` found four
guard rules that refuse work the flow permits. The user ranked them:
idle-gate git allowlist first, then the secret-read prompt, then the scratch
prefix, then the gc-2 refusal. Three are precision fixes; the fourth is a
message fix — its fail-closed behavior stays (decision 08e61612: a shared-index
commit swept 133 lines of a sibling's work; the guard exists because of it).

## Scope

Four behavior corrections, all inside the write-guard module, plus their tests.
Product files: `guards.rs`, `checks.rs`, `paths.rs`, `tests.rs`.

### 1. Idle-gate read-only git forms (checks.rs)

The intake gate's `READONLY` list misses read-only forms, so at idle these are
refused: bare `git branch`, bare `git remote`, `git stash list`, `git reflog`,
`git grep`, `git worktree list`.

Change, in `evaluate_git_invocation` — the safe-form table replaces the
flag-gated arm (checks.rs:685-694), which already sits above `push` and
`MUTATING`. Plan-check verified the ordering and found three holes in the
first draft; the predicates below carry the fixes:

- `branch`: allow when `rest` has no positional (non-flag) token AND no
  token that is a flags-only mutator — any token starting with `-u` or
  `--set-upstream` (catches the `--set-upstream-to=x` `=`-spelling and
  attached `-uorigin/main`), `--unset-upstream`, `--edit-description`.
  `--list` also allows, as today.
- `tag`: keep today's `--list` allowance (the replaced arm served `tag`
  too — dropping it would silently regress `git tag --list` to deny).
- `remote`: allow when `rest` has no positional token (bare or `-v`).
  Every mutating `remote` form carries a positional subcommand word.
- `stash`: allow when the FIRST token of `rest` is exactly `list` or
  `show` — first-positional is not enough: `git stash -- list` routes to
  `stash push` with pathspec `list` (verified against git 2.43.0).
- `worktree`: allow when the first token is `list`.
- `reflog`: allow when `rest` has no positional or the first positional is
  `show`. `reflog expire/delete` still fall to the unrecognized-deny arm.
- `grep`: allow UNLESS any token starts with `-O` or
  `--open-files-in-pager` (git-level pager spawn = arbitrary command the
  tokenizer never sees). Goes in the safe-form table, not `READONLY`.
- `stash` stays in `MUTATING`; the safe-form check runs before it, so
  `git stash` / `stash pop` still deny at idle.

Accepted precision loss, named: read-only spellings that carry an operand
(`git branch --contains HEAD`, `git reflog main`, `git remote show origin`)
stay denied — the predicate trades them for zero mutating admissions.

### 2. Secret-read prompt precision (guards.rs)

`is_secret_path` matches any basename starting with `credentials`, so source
files that implement credential handling (`src/credentials.rs`,
`credentials_test.go`) prompt the user on every read.

Change: the `credentials` prefix match skips basenames whose extension is a
code extension. `credentials`, `credentials.json`, `credentials.csv` still
prompt. The code-extension list is extracted from `docs_history_code_deny`
into one shared const (`CODE_FILE_EXTENSIONS`) so the three consumers cannot
drift.

Scope cut, named: `.pem`/`.key`/`.p12` matching stays as-is. `public.key` is a
rare spelling, the prompt is one question, and widening a key-file exemption
weakens a privacy guard for near-zero gain.

### 3. Scratch-prefix precision (guards.rs)

`scratch_shape_deny` refuses any tracked file whose basename starts with
`verdict-` / `probe-` / `digest-`, catching legitimate product files like
`src/probe-runner.rs`. The rule exists to keep bee's own scratch payloads out
of tracked dirs — those are data files, not source.

Change: the prefix rule skips basenames with a code extension (same shared
const). `probe-results.json`, `verdict-x.md` in tracked dirs still deny. The
dotfile rule and `.tmp`/`.log`/`.bak` rules are untouched.

### 4. gc-2 unresolved-count refusal names its remedy (paths.rs / checks.rs)

`WorkerCount::Unresolved` has exactly one source: a present-but-unparseable
`.bee/reservations.json`. The refusal says "treated as more than one worker"
and hands a solo session the heavy temp-index workaround — the actual fix is
restoring one file. Fail-closed stays (boundary, decision 08e61612).

Change: the Unresolved arm of the refusal names `.bee/reservations.json` and
mirrors the hold-guard corrupt-store remedy verbatim ("inspect/restore the
reservation store, then retry" — it covers unreadable as well as unparseable,
which `reservation_store_corrupt` also returns true for). The `count > 1`
arm keeps today's text. Mechanically this is a per-arm remedy parameter on
`concurrent_tree_refusal` — the current single hard-coded `FIX:` sentence
serves both arms, so a small signature change, not a format! edit.

## Shape

Two cells, sequential (named reason: both edit `tests.rs`, and cell 2's
safe-form table reads the shared const cell 1 extracts — file overlap, real
dependency):

- **wgp-1** — guards.rs precision: shared `CODE_FILE_EXTENSIONS` const,
  secret-prefix exemption (#2), scratch-prefix exemption (#3), plus tests:
  flip/extend the decision-table cases for both guards. One existing case
  flips: tests.rs:948 `deny("scripts/probe-foo.mjs")` becomes allow. The
  const is extract-only — `docs_history_code_deny` keeps its full-path
  extension extraction, the two new consumers extract from the basename.
- **wgp-2** — checks.rs/paths.rs: idle-gate safe-form table (#1), gc-2
  Unresolved refusal text (#4), plus tests: idle-gate table cases for the six
  newly allowed forms and the still-denied mutating forms; refusal-text
  assertion for Unresolved.

Verify: `commands.test` (cargo test --release) at each `bee cells finish`.

## Risks

- Every newly allowed git form must be provably read-only in ALL its
  spellings — the tests enumerate denied siblings (`stash pop`, `worktree
  add`, `reflog expire`, `branch -D x`, `remote add`) beside each allow.
- The code-extension exemption is a heuristic: a scratch script named
  `probe-x.rs` now lands untracked-denied only by review, not by the guard.
  Accepted: the guard's job is bee's own payloads, which are data-shaped.

## Smaller path

Checked: one cell would blur two independent concerns and serialize anyway;
zero cells (config-only) cannot express any of the four. Two cells is the
floor that honors the ranked scope.
