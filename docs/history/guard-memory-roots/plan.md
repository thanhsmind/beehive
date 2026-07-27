# plan — guard-memory-roots (GH #71)

Mode **high-risk**. Decisions: `CONTEXT.md` D1–D7.

## Goal

Let a human declare extra roots the write-guard will permit, so the agent's
memory directory outside the worktree becomes writable — without weakening
containment for anything undeclared.

## Shape

One new concept, honored at one seam, in two runtimes.

**Config.** `guards.extra_write_roots`: a list of absolute paths, `~` expanded.
Read through the existing `readConfig(storeRoot)` the hook already calls
(`bee-write-guard.mjs:765`), so the tracked `.bee/config.json` and the gitignored
overlay `.bee/config.local.json` (`.gitignore:20`) both work. A machine-specific
home path belongs in the **overlay** — document that, do not enforce it.

**Seam.** `canonicalRelPath` (`bee-write-guard.mjs:65-114`) and
`canonical_rel_path` (`write_guard.rs:299-336`) are the single funnel both the
Write/Edit leg (`:868`) and the Bash extracted-target leg (`:844`) call. The
extra-root check goes beside it, consulted by both call sites — mirroring how
`resolveCompanionMountedRelPath` is already wired as the fallback at those exact
two sites (`:849`, `:869`).

**Decision shape.** `canonicalRelPath` fails → try the companion mount (today's
behavior, unchanged) → then try the declared extra roots. A hit is a real ALLOW
that **short-circuits**: the target is not passed to `checkWrite`, so gates,
intake, reservations and holds do not apply (D6). A miss falls through to
today's denial, message unchanged.

**Containment for a declared root** is the same discipline as the worktree
(D3): realpath the declared root; reject the target on a foreign-platform path
spelling or a lexical `..`; walk up through ENOENT segments to the first
existing ancestor and realpath that; then require `path.relative(rootReal,
target)` to be non-empty, not `..`, not `..`-prefixed, not absolute.

**Sanity refusals** (D5), applied per declared root at read time: the filesystem
root; any root that contains the worktree; a bare home directory. A refused root
contributes nothing and the reason is visible, not silent.

**Fail-closed** (D4): unreadable or malformed config, a declared root that does
not resolve, or any error while checking → contributes nothing, write denied as
today. No error path may produce an allow.

## Why this cannot regress today's behavior

The default list is empty, and with an empty list the code path is a no-op.
`writeguard_core.rs:662` (`/etc/hosts` denied differentially in both runtimes)
and `test_write_guard.mjs:803` `escapeRows` (traversal, absolute-main, symlink,
windows-separator, case-alias) run against fixtures that declare nothing, so
they must keep passing **unedited**. If they need editing, the implementation is
wrong.

## Proof

Extend the existing guard suites rather than inventing a parallel harness:
- With nothing declared: every current denial row unchanged (the existing
  suites already assert this — they must stay green untouched).
- With a root declared: a write inside it is allowed on the Write/Edit leg
  **and** on the Bash leg; a traversal out of it is denied; a symlink escaping
  it is denied; a sibling path outside it is denied.
- Sanity refusals: declaring `/`, the worktree's parent, or a bare home
  contributes nothing and the target is still denied.
- Fail-closed: a malformed config value (string instead of list, a relative
  path, a non-existent root) denies.
- Differential: the mjs and Rust hooks agree on allow and on deny, byte-identical
  stderr, through the existing `assert_allow_conformant` /
  `assert_deny_conformant` harness.

## Cells

| id | scope | verify |
|---|---|---|
| `gmr-1` | The extra-root check in both runtimes, plus the suites above, plus the rust-port mirror artifact required by the freeze (D7) | full suite (`node scripts/run_verify.mjs`) — not an impacted run |

One cell, one commit. `splr-1` capped false-green on an impacted run that
selected zero suites; this cell's verify is the full suite for that reason.

## Risks

- **This widens a security boundary.** Mitigated by: empty default, declaration
  required, same canonicalization as the wall, sanity refusals, fail-closed.
- **Short-circuiting `checkWrite` (D6)** means an extra root is exempt from the
  intake gate. That is intended and is the point of the feature, but it must be
  stated plainly in the config documentation: a declared root is a place bee will
  let the agent write at any phase.
- **The mjs is under the rust-port freeze.** This lands as a critical bugfix with
  the mandated mirror artifact (D7); the two runtimes must not diverge, and the
  differential harness is what proves it.
