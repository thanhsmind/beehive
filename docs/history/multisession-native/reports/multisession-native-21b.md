# multisession-native-21b

[DONE] — passive keepalive hooks (bee-state-sync.mjs, bee-prompt-context.mjs) now
thread `ctx.controlRoot` into `claims.heartbeatTouch`, fixing a granted-worktree
session's heartbeat renewing against the wrong store; the msn-21
canonical-containment scratchpad deny was investigated and confirmed deliberate
pre-existing behavior (reported, not fixed).

Files touched: `hooks/bee-state-sync.mjs`, `hooks/bee-prompt-context.mjs`,
`hooks/test_hook_contracts.mjs`, `.bee/bin/hooks/bee-state-sync.mjs`,
`.bee/bin/hooks/bee-prompt-context.mjs`, `.bee/onboarding.json`,
`docs/history/codex-harness-hardening/release-manifest.json`.

Full trace/evidence: `.bee/cells/multisession-native-21b.json`.

Commit: `8952116`.
