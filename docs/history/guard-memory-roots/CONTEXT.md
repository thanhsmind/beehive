# CONTEXT — guard-memory-roots (GH #71)

Mode **high-risk** · Source: GH issue #71, reported by @vantt.

## Problem

The write-guard contains every write to the physical worktree. The agent's own
persistent memory lives at `~/.claude/projects/<slug>/memory/` — outside it — so
every memory write is denied and durable learnings are lost silently. The
reporter hit this trying to save a lesson and, correctly, did not work around
the guard.

**Reproduced live in this session, twice**, while working on GH #70:

```
bee write guard denied Bash: one or more extracted targets could not be
canonically contained inside the physical worktree.
```

## Evidence

- The decisive containment check is `canonicalRelPath(workRoot, cwd, rawPath)`
  (`packages/bee/hooks/bee-write-guard.mjs:65-114`), ported 1:1 as
  `canonical_rel_path` (`crates/queen-bee/src/hooks/write_guard.rs:299-336`).
  It contains against `ctx.root`, the physical worktree — not `storeRoot` or
  `controlRoot`.
- **Both legs funnel through it.** The Write/Edit/MultiEdit leg calls it at
  `bee-write-guard.mjs:868`; the Bash extracted-target leg at `:844`. They are
  the same function with a different raw-target source, so one insertion point
  covers both, plus `apply_patch` targets via the same `relPaths` pipeline.
- **No allowlist for out-of-worktree targets exists anywhere.** Verified across
  `bee-write-guard.mjs`, `write_guard.rs`, `guards.mjs`, `bee-core/guards.rs`,
  `.bee/config.json`, and `crates/bee-core/src/config.rs`. The only guard config
  keys actually consumed are `idle_gate`, `max_read_lines`, `exclusive_paths`,
  `write_policy`, `auto_isolate` — none names a root.
- The `GATE_ALLOWED_PREFIXES` allowlist (`.bee/`, `docs/`, `plans/`,
  `AGENTS.md`) is a real ALLOW but is unreachable here: it lives in `checkWrite`,
  downstream of containment, so an out-of-worktree path never reaches it.
- The sibling-worktree enrichment (GH #31) is **message-only** — it states so
  itself and every test asserts exit 2.
- The one existing genuine escape hatch is the **companion mount**
  (`resolveCompanionMountedRelPath`, `bee-write-guard.mjs:384`), which allows a
  target only when a live, git-verified marker proves the crossing. That is the
  precedent this feature follows.
- bee has **no existing notion** of `~/.claude`, a memory directory, or an
  agent-home. This concept is entirely new.

## Locked decisions

| ID | Decision | Why |
|---|---|---|
| D1 | Extra permitted roots are **declared**, in `.bee/config.json` under a guards key holding a list of absolute paths (`~` expanded). Never auto-discovered, never taken from an environment variable | Declaration is the security boundary. A root the guard infers is a root an attacker can arrange; a root a human wrote into config is a decision |
| D2 | The default is **empty**, and with an empty list behavior is byte-identical to today — every absolute out-of-worktree target still denied, in both runtimes | The existing blanket invariants (`writeguard_core.rs:662` denying `/etc/hosts` differentially, `test_write_guard.mjs:803` `escapeRows`) must keep passing **unchanged**. A fix that requires editing those tests is the wrong fix |
| D3 | A declared root is honored with the **same canonicalization discipline as the worktree**: realpath the declared root, realpath-walk the target, then `path.relative` containment. Traversal, symlink escape out of the declared root, and foreign-platform path spellings are denied exactly as they are for the worktree | The escape hatch must not be weaker than the wall it opens |
| D4 | **Fail-closed everywhere.** An unreadable or malformed config, a declared root that does not resolve, or any error while checking it contributes nothing and the write is denied as today. No error path may produce an allow | A guard that fails open under error is not a guard. The repo already proves this discipline for a corrupt grants file |
| D5 | Declared roots are **sanity-refused** when they would swallow the wall: the filesystem root, any root that contains the worktree, and a bare home directory. A refused root is ignored with a visible reason, never silently | `extra_write_roots: ["/"]` must not be a supported way to disable the guard |
| D6 | An extra-root target is allowed and **short-circuits** — it is not passed to `checkWrite`, so the intake gate, gates, reservations, and holds do not apply to it | Those semantics are repo-scoped and have no meaning for a path with no repo-relative form. It is also the point: a learning must be recordable at phase `idle`, which is exactly when the intake gate is shut |
| D7 | Lands in **both runtimes** as a critical bugfix under the rust-port D1 freeze, with the mandated mirror artifact (a `rust-port`-tagged item naming the delta, plus any affected parity fixture updated in the same change). The mjs and Rust denial strings and allow decisions stay byte-identical, checked by the existing differential harness | The user chose this over Rust-only (which would ship nothing, since the running hook is `.mjs`) and over a local symlink workaround (which fixes one machine, not the product) |

## Out of scope

- Configuring where Claude Code puts its memory. bee honors a declared root; it
  does not manage the agent's memory location.
- Read-side guards. This feature is about writes.
- Any change to the gate/reservation/hold semantics for in-repo paths.

## Outstanding questions

None blocking.
