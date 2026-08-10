---
type: bee.pattern
title: Locks guarding long synchronous child spawns cannot be heartbeat-renewed — probe owner liveness instead
description: A lock held across a blocking spawnSync cannot heartbeat-renew its mtime; stale takeover must probe owner liveness instead
tags: [architecture, locks, concurrency, spawnSync, liveness, stale-takeover, advisor]
timestamp: 2026-07-21
bee:
  id: pattern-20260721-locks-guarding-long-synchronous-child-spawns-cannot-be
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT47", "original feature: hardening-1-7-10"]
  polarity: pitfall
  critical: false
---

# Locks guarding long synchronous child spawns cannot be heartbeat-renewed — probe owner liveness instead

A timer refreshing a lockfile's mtime cannot fire while the holder runs spawnSync (event loop
blocked) — exactly when protection is needed (worktree merge holding worktree-admin across a
multi-minute verify). Sync-safe design: stale takeover requires the owner pid provably dead
(kill(pid,0); EPERM=alive) after 30s, with a 1h absolute ceiling as the pid-reuse guard. The
mandatory high-risk advisor consult caught this pre-execution — the original 15min ceiling would
have re-opened the exact mid-verify steal being closed — and also caught a writeCell sync→async
cascade and a judge-reopen claims-store orphan that would otherwise have surfaced mid-cell.
