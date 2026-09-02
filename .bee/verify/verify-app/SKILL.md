---
name: verify-app
description: "Drive the real `bee` CLI end to end against a throwaway onboarded sandbox repo and capture evidence. Use to prove a change to the bee binary, its verbs, gates, cells, worktrees or onboarding actually works for a user — not just that cargo tests pass."
---

# Verify bee

`bee` is a single Rust CLI binary. Its user is a coding agent (or a developer)
standing inside a project that has been onboarded, typing `bee <verb>`. So
verification means exactly one thing here: **build the binary, onboard it into a
throwaway repo, run the verbs a user runs, and read what changed on disk.**

There is no web UI and no TUI to drive. `packages/bee-rs/crates/fleet` is a
library linked into the same binary, never a second executable. The herding
cockpit drives tmux panes holding live agents — out of scope for this skill,
because it needs real agent processes; see `bee-herdr` instead.

Everything below runs through one script:

```bash
bash .bee/verify/verify-app/control-bee
```

Run it by that **literal path**, with the `bash` prefix, from the repo root (the
real checkout or a worktree of it — the script resolves the repo from its own
location, so cwd only has to make the path above resolve). The prefix is not
decoration: this source copy is executable, but bee renders `0644` copies of it
into `.claude/skills/verify-app/`, `.agents/skills/verify-app/` and
`.opencode/skills/verify-app/`, where a bare path fails with `Permission
denied`. `bash <path>` runs from either. Never put the path in a shell variable:
`"$C" put ...` is refused — see Gotchas.

Edit only this source tree. The rendered copies are bee's output and a hand edit
there is lost at the next `bee onboard --apply`; after any edit here, re-run
`bee onboard --apply` so every runtime sees the same bytes.

## Launch

`bee` is short-lived: every command starts, does its work, and exits. There is
no server to keep alive. "Launch" therefore means *build the binary once, then
create one disposable sandbox repo per run*.

```bash
bash .bee/verify/verify-app/control-bee build
bash .bee/verify/verify-app/control-bee launch
```

`build` runs `cargo build --release --manifest-path packages/bee-rs/Cargo.toml`
and prints the binary path. First build is slow (minutes); later ones are
seconds. **Never hardcode `packages/bee-rs/target/release/bee`** — this machine
sets `CARGO_TARGET_DIR`, so the binary lands elsewhere. Ask `control-bee bin`.

`launch` prints the sandbox path and is ready when it exits 0. It:

1. picks a run id and writes `$VERIFY_HOME/current-run`,
2. creates `$VERIFY_HOME/run/<id>/repo`, `git init`s it, makes a seed commit,
3. runs `bee onboard --repo-root <sandbox> --apply --json` **with cwd set to the
   real checkout** (onboard finds the engine to vendor by walking up from its own
   cwd, not from `--repo-root`),
4. commits the onboarded tree so later diffs are readable.

Paths (override `VERIFY_HOME` to move them all):

| What | Where |
|---|---|
| Sandbox repo | `${TMPDIR:-/tmp}/bee-verify/run/<run-id>/repo` |
| Evidence | `${TMPDIR:-/tmp}/bee-verify/evidence/<run-id>` |
| Run pointer | `${TMPDIR:-/tmp}/bee-verify/current-run` |

Teardown is `control-bee cleanup` (see Cleanup).

## Doctor

Run this first, and again after anything surprising:

```bash
bash .bee/verify/verify-app/control-bee doctor
```

It is read-only and answers "is this instance worth driving?" — it checks that
the binary exists, that `bee version --json` matches the version in
`.claude-plugin/plugin.json` (the release version is read from that manifest,
**not** from `Cargo.toml`, which stays at `0.1.0`), that the sandbox is a git
repo under `$VERIFY_HOME` and is not the real checkout, that
`.bee/onboarding.json` was written by the expected bee version, that
`bee status --json` inside the sandbox reports that version with no drift, and
that the evidence dir is writable. Any FAIL exits 1 — do not drive after that.

## Drive

Two verbs, plus one file-writing door:

```bash
# run a bee command inside the sandbox
bash .../control-bee cli -- <bee args...>

# run any other command inside the sandbox (git, ls, cat)
bash .../control-bee sh  -- <cmd...>

# write stdin to a file inside the sandbox
printf 'body\n' | bash .../control-bee put path/in/sandbox.md

# make a second, NOT-onboarded repo (the onboard recipe's target)
bash .../control-bee newrepo target

# run with cwd = the real checkout; only for `bee onboard`, and only when the
# command names a --repo-root under $VERIFY_HOME/run
bash .../control-bee host -- <binary> onboard --repo-root <target> --json
```

Aim any of them at a worktree bee created by setting `VERIFY_CWD` on the call:
`VERIFY_CWD=repo--wt--<feature> bash .../control-bee sh -- git status`.

`cli` and `sh` both `cd` into the sandbox, pin `BEE_SESSION_ID` and
`CLAUDE_CODE_SESSION_ID` to `verify-<run-id>` so the sandbox never inherits the
driving agent's session, capture stdout / stderr / exit code, and **always
return 0**. A bee refusal is data, not a harness failure — assert on the
recorded `.exit` file, not on the shell's status.

`cli` passes stdin through, so the pipe-a-cell form works:

```bash
printf '{"id":"demo-1", ...}' | bash .../control-bee cli -- cells add --stdin --json
```

Prefer stable handles: the `--json` payload keys and the `error` / `kind` strings
in a refusal, never the human wording and never the `[bee] <verb> Nms` timing
line on stderr.

The per-feature recipes live in [`features/`](./features/README.md). Read the
index, then the feature file, before driving.

## Evidence

Every `cli` / `sh` call writes four files into
`$VERIFY_HOME/evidence/<run-id>/NNN-<slug>.{cmd,out,err,exit}` — the command as
run, its stdout, its stderr, and its exit code. `control-bee snapshot <label>`
adds a `state-<label>/` folder holding the sandbox's `.bee/state.json`,
`config.json`, `onboarding.json`, `decisions.jsonl`, `backlog.jsonl`,
`reservations.json`, its `cells/` directory, and `git log` / `git status`.

Proof standards:

- **Drive the real user path.** Run the verb a user runs. Do not hand-write
  `.bee/*.json` to reach a state — that proves the fixture, not bee. (The Rust
  integration tests already do fixture-shaped setup; this skill exists to cover
  what they do not.)
- **Capture the action and the resulting state.** A `cells cap` that exits 0 is
  half a proof. Snapshot `.bee/cells/<id>.json` and show `status: "capped"` with
  the recorded proof line on `trace.report.tests`.
- **Verify side effects.** bee's whole job is durable state, so read it back:
  files written, `state.json` gates flipped, decisions appended, cells archived,
  git commits and worktrees created.
- **Prove refusals too.** Half of bee's user-visible behavior is a typed refusal.
  A refusal proof is the `error` / `kind` in the JSON body plus a non-zero
  `.exit`. Both, or it is not proven.
- **No mocks.** Everything here is the real binary against a real git repo on a
  real filesystem. There is nothing to stub.
- **Check what a dry-run skips.** `bee onboard` without `--apply`, and
  `bee close --dry-run`, claim to mutate nothing. Prove that by snapshotting
  before and after and diffing, not by trusting the flag name.

## Cleanup

```bash
bash .bee/verify/verify-app/control-bee cleanup
```

It removes `$VERIFY_HOME/run/<run-id>` — the sandbox *and* any sibling worktrees
`bee worktree new` created inside it, since bee places them at
`../<basename>--wt--<feature>` — then clears the run pointer. It kills nothing by
name and touches no process it did not start (there are none: bee always exits).

**Evidence is never removed.** It lives at
`$VERIFY_HOME/evidence/<run-id>`, outside the run dir, on purpose. After cleanup,
confirm it is still there before reporting a proof.

Run cleanup after a failed attempt too, so a broken run leaves no stray sandbox
or registered worktree behind.

## Gotchas

- **bee's own write guard fights inline shell.** This repo's `PreToolUse` hook
  refuses a Bash command whose write target still carries an unexpanded `$VAR`,
  refuses a write that follows a `cd` in the same compound command, and refuses
  a redirect to an absolute path outside the worktree. That is why `control-bee`
  exists and why you must call it by its literal path — `"$C" put x`
  gets refused on the `$C`. Use `control-bee put` instead of `> file`.
- **`bee cells cap --no-mistakes` does not work in either form.** Bare, it is
  refused as `unsupported_argument_shape`; with a value, the cap succeeds and
  silently records nothing. `bee mailbox reflect --no-mistakes` (bare) is what
  settles the close door. Details in `features/cells-and-proof.md`.
- **A refusal can still exit 0 at the shell.** With `--json` the error body goes
  to **stdout**, not stderr. Read the `.exit` file the harness recorded.
- **`bee doctor` needs `--runtime`.** Bare `bee doctor --json` is a
  `missing_required_argument` refusal. That verb is not the health check for this
  skill — `control-bee doctor` is.
- **`--files` on `bee route --set` is a count, not a list.** It wants an integer.
- **A `small`-lane cap needs a registered worker.** `bee cells cap` refuses
  unless `trace.worker` also appears in `state.json` `workers[]` for that cell.
  Register it with `bee state worker add --nickname <w> --cell <id>`, or use a
  `tiny` lane, which runs inline.
- **The close `mistakes` door checks the cells first, the mailbox second.** The
  cell half is unreachable from the CLI (see the `--no-mistakes` gotcha above), so
  in practice only `bee mailbox reflect --no-mistakes` clears it.
- **Onboard must run from the real checkout.** `--repo-root` says where to
  install; the engine it vendors is found by walking up from cwd, stopping at the
  first `.git`. `control-bee launch` and `control-bee host` handle this; plain
  `control-bee cli -- onboard` returns `blocked_no_engine`.
- **Version confusion.** `bee version --json` returns
  `{"version": "<release>", "binary": "<crate>"}`. The release version comes from
  `.claude-plugin/plugin.json`; the crate version is pinned at `0.1.0` and means
  nothing. Assert on `version`.

## Helpers

`control-bee` is the only script. It is executable in this source tree and
self-documenting:

```bash
bash .bee/verify/verify-app/control-bee --help
```

Subcommands: `build`, `bin`, `paths`, `launch`, `doctor`, `cli`, `sh`, `put`,
`newrepo`, `host`, `snapshot`, `cleanup`.

## Keeping this honest

The feature map rots as bee changes. Load the `bee-verify-upkeep` skill to audit
it: it reads each feature from source, drives every feature live, and ships one
PR of proven corrections.
