---
type: bee.pattern
title: Windows Git Bash /tmp is invisible to node
description: Windows Git Bash /tmp is invisible to node
tags: [failure, windows, paths, environment]
timestamp: 2026-07-08
bee:
  id: pattern-20260708-windows-git-bash-tmp-is-invisible-to-node
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT8", "original feature: harness09", docs/history/learnings/20260708-harness09.md]
  polarity: pitfall
  critical: false
---

# Windows Git Bash /tmp is invisible to node

Shell redirection into `/tmp` works under Git Bash, but handing that `/tmp/...` string to
a node API fails — node cannot resolve MSYS paths. Pipe the file through stdin
(`cat file | node -e ...`) or use a Windows-style absolute path (the session scratchpad).

**Full entry:** docs/history/learnings/20260708-harness09.md
