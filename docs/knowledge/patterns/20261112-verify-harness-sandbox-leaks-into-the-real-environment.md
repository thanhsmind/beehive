---
type: bee.pattern
title: A verify sandbox that inherits the driver's environment registers real paseo workspaces, and a binary copied onto the linked .bee/bin/bee overwrites the build
description: "control-bee must strip PASEO_* from every sandbox command, claim each evidence stem with a no-clobber create, and link .bee/bin/bee instead of copying onto it."
tags: [verify-app, control-bee, paseo, sandbox-isolation, symlink]
timestamp: 2026-11-12
bee:
  id: pattern-20261112-verify-harness-sandbox-leaks-into-the-real-environment
  lifecycle: active
  areas: [verify-pipeline]
  anchors: [.bee/verify/verify-app/control-bee, packages/bee-rs/crates/bee/src/herding/paseo.rs]
  sources: ["verify-map-upkeep-2 cell vmu2-1, live run 20261112-123209-529324 (2026-11-12)"]
  polarity: pitfall
  critical: false
---

# A verify sandbox that inherits the driver's environment registers real paseo workspaces

Read when: a verify-app drive runs from inside paseo, parallel `control-bee cli` calls share one
evidence dir, or a recipe wants to put a binary at the sandbox's `.bee/bin/bee`.

`bee worktree new` registers a paseo workspace whenever `PASEO_AGENT_ID` or `PASEO_CLI` is set.
A driving session inside paseo passed both into the sandbox, and two throwaway worktrees showed up
in the user's real paseo daemon. `control-bee` now unsets every `PASEO_*` variable at start, so no
sandbox command sees one.

The evidence index was a check-then-create loop, so five parallel `cli` calls could pick one stem
and write two JSON lines into one `.out`. The stem is now claimed with a no-clobber create and the
loop steps on collision.

`launch` links `.bee/bin/bee` to the built binary, because onboard from the source checkout leaves
none and `doctor --runtime pi` reports `hook_handler not_ok` without it. A recipe that still
vendors with `put .bee/bin/bee < <binary>` would write through that link onto the build, so those
recipes now say nothing is vendored.

Rejected: an `env -u` list in `record_in` only. `launch` runs onboard outside `record_in`, so the
strip lives at the top of the script.
