---
type: bee.area
title: "Hook Runtime — guard precision: code exemptions, the idle-gate git safe-form table, and a named worker-count remedy"
description: "Why the secret- and scratch-file write guards stand down for a path whose extension marks it as ordinary source code, how the idle-gate git check admits a table of safe, non-mutating command forms instead of refusing by subcommand alone, and why a refusal caused by an unresolvable worker count names its own remedy instead of failing opaquely."
timestamp: 2026-08-11
bee:
  id: hook-runtime-guard-precision-exemptions-and-remedies
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md, areas/hook-runtime/write-guard-request-shapes.md, areas/hook-runtime/governed-paths-and-the-intake-gate.md]
  decisions: ["write-guard-precision D1 (guard precision over blanket strictness, 2026-08-11)"]
  sources: ["write-guard-precision cell wgp-1 (trace .bee/cells/archive/write-guard-precision/wgp-1.json, commit 23ad32e, worker suite 1529 passed / 7 ignored, 2026-08-11)", "write-guard-precision cell wgp-2 (trace .bee/cells/archive/write-guard-precision/wgp-2.json, commit fbcc40d7, worker suite 1422 passed, 2026-08-11)", docs/history/write-guard-precision/plan.md]
  authoritative_for: "hook-runtime: write-guard code-extension exemptions, the idle-gate git safe-form table, and the worker-count-unresolved remedy"
---

# Hook Runtime — guard precision: code exemptions, the idle-gate git safe-form table, and a named worker-count remedy

Three write-guard checks were found refusing legitimate work that a blanket rule
could not tell apart from the thing it actually exists to stop. Each was
narrowed to the precise shape it needs to catch, never widened past it — the
guard still refuses everything it refused before, it just stops refusing the
ordinary work sitting next to it.

## Behaviors & Operations

**A source-code extension exempts a path from the secret- and scratch-file
heuristics.** Two write-guard checks flag a path by its NAME alone: a
credentials-shaped name that would otherwise force a question to the human
before it is read, and a scratch-shaped name — a `verdict-`/`probe-`/`digest-`
prefix, or a `.tmp`/`.log`/`.bak` extension — that would otherwise be refused
as disposable. Both heuristics share one exemption: when the path's own
extension marks it as a real source file (the common shell, script, and
program-language extensions), the heuristic does not fire, because a file that
merely starts with a matching word — `src/credentials.rs`,
`probe-runner.py` — is ordinary source work, not the secret or scratch payload
the heuristic was built to catch. The narrower `.pem`/`.key`/`.p12` secret
match is untouched by this exemption on purpose: widening it to cover a
matching key file was considered and rejected as a privacy loss for
near-zero gain.

**The idle-gate git check admits a table of safe, non-mutating command forms
instead of refusing by subcommand alone.** At the terminal, gate-controlled
phase, a fixed list of read-only subcommands always passes. Beyond that list,
a second table recognizes specific SAFE SPELLINGS of subcommands that can
also mutate: a bare or listing-only branch or remote inspection, a stash
`list`/`show` (checked by the first token rather than by "any positional",
because a pathspec fallthrough routes a differently-placed `list` to a push
instead), a worktree listing, a bare or `show` reflog, and a grep that never
asks git to spawn a pager. Every admitted form was proved read-only in every
spelling it accepts. The corresponding MUTATING spelling of the same
subcommand — a branch with an upstream-setting flag, a stash push carrying a
pathspec — still falls through to the ordinary deny; inspection is never
blocked at idle, only a form that could rewrite the tree is.

**A refusal caused by an unresolvable worker count names its own remedy.**
The concurrent-worker check treats a tree-rewriting git verb as safe only
when it can prove at most one worker is live. When the live count cannot be
resolved at all, the check still refuses — exactly as it does when it counts
more than one worker — but the refusal does not stop at "unresolved": it
names the reservation store as the thing to inspect or restore before
retrying, rather than only handing a solo session the heavier remedy built
for a genuine multi-worker conflict.

## Business Rules

- The code-extension exemption applies identically to the secret-prefix
  check and the scratch-prefix check; a `.pem`/`.key`/`.p12` match is never
  exempted by it (write-guard-precision D1).
- An idle-gate git safe form is recognized by its exact, proven-read-only
  spelling; the corresponding mutating spelling of the same subcommand is
  never covered by the same table entry (write-guard-precision D1).
- A worker-count-unresolved refusal always names the reservation store as
  its remedy; a resolved count above one keeps the existing multi-worker
  remedy unchanged (write-guard-precision D1).

## Pointers (implementation)

- Code-extension exemption: `CODE_FILE_EXTENSIONS`, `is_secret_path`, and
  `scratch_shape_deny` in
  `packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs`. Provenance:
  cell `wgp-1`, trace `.bee/cells/archive/write-guard-precision/wgp-1.json`,
  commit 23ad32e.
- Idle-gate safe-form table: `idle_gate_safe_form`, consulted from
  `evaluate_git_invocation` after the fixed read-only subcommand list, in
  `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs`. Provenance:
  cell `wgp-2`, trace `.bee/cells/archive/write-guard-precision/wgp-2.json`,
  commit fbcc40d7.
- Worker-count remedy: the `WorkerCount::Unresolved` arm of
  `evaluate_git_invocation`, `checks.rs`, naming `.bee/reservations.json` as
  the inspect/restore target. Same provenance as above.
