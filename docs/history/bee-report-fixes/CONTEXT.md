# Bee Report Fixes — Context

Asked: "Em issue có cái nào thì kiểm tra fix" (2026-11-04): check the open
GitHub issues and fix them.

## Feature Boundary

In scope: the four bee-report bugs #154, #155, #156 and #157. Out of
scope: #69, a cross-project dashboard. It is a new product, not a bug,
and needs its own shaping.

## Locked Decisions

- **D1 (7d3c57e8), issue #154.** A reserve by the same agent for the
  same cell over its own live lease is idempotent: it refreshes the lease
  and answers ok. `bee cells unclaim` releases the reservations the cell
  holds. A different agent or a different cell still conflicts.
- **D2 (51fe42f6), issue #155.** `bee worktree merge` succeeds when main
  carries uncommitted `.bee` store changes. The bookkeeping commit takes
  them, and no store file stays staged when `git merge` starts. A failed
  merge still restores main as found.
- **D3 (2b906517), issue #156.** `bee onboard --apply` vendors the
  running binary to `.bee/bin/bee` when the host binary differs, so the
  binary and `.bee/bin/prompts/` always come from one build. It never
  writes over the same file or through a symlink into another checkout.
- **D4 (93ec1728), issue #157.** Every deploy-stage dispatch carries the
  deployment brief on every transport. A native Agent or spawn_agent
  payload names the dispatch id, the release version and the issuer
  session to export, and a write-capable agent type. Herding and cli
  payloads stay byte-identical.

## Outstanding Questions

None.
