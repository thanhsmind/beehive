---
type: bee.pattern
title: Source that ships without reinstalling the binary the hooks call is inert
description: "A green test suite and a merged branch describe the source tree; when every hook and command invocation actually runs a built binary outside version control, neither implies the running system reflects the merge until the binary is reinstalled."
tags: [deployment, stale-binary, reinstall, proof-discipline, onboarding]
timestamp: 2026-08-05
bee:
  id: pattern-20260805-source-shipped-without-reinstalling-the-called-binary-is-inert
  lifecycle: active
  areas: [rust-runtime, onboarding]
  decisions: ["399d72e1 (hook-teeth, 2026-08-04: the installed binary still carried a route bug fixed in source by ct-1, not yet reinstalled)", "3baa41f6 (counter-teeth, 2026-08-04: the same installed-binary gap, route --set broken under any live worktree grant)"]
  sources: [".bee/bin/bee (gitignored, built by the installer, not tracked in version control)", "measured 2026-08-05: the installed `.bee/bin/bee` refused `bee knowledge promote --work promote-reach` with `unknown_work`, while `cargo run --release --manifest-path packages/bee-rs/Cargo.toml -- knowledge promote --work promote-reach` resolved it", "decision 399d72e1 (hook-teeth, 2026-08-04, same defect class one day earlier)", "decision 3baa41f6 (counter-teeth, 2026-08-04, same defect class)"]
  polarity: pitfall
  critical: true
---

# Source that ships without reinstalling the binary the hooks call is inert

A green suite and a merged branch describe the SOURCE tree. When the thing every hook and command
invocation actually runs is a built artifact outside version control, green-and-merged says nothing
about what a live session executes — only a reinstall closes that gap, and nothing about "tests
passed" or "PR merged" implies a reinstall happened.

The instance: three features shipped, merged, and tested green, and none of it reached a session.
`.bee/bin/bee` is gitignored and built by the installer, so the copy every hook and command actually
invoked was hours stale against the merged source. Proof was one command: the installed binary
still refused `bee knowledge promote --work promote-reach` with `unknown_work`, while a fresh
`cargo run --release --manifest-path packages/bee-rs/Cargo.toml` resolved it. The same gap had
already recurred twice in the two days before: `counter-teeth` and `hook-teeth` both proceeded
without a route record because the installed binary still carried a route bug already fixed in
source and not yet reinstalled.

## The rule

- A merged, green feature is a claim about the SOURCE. Whether the running system reflects it is a
  separate, unasserted claim whenever the entry point is a built artifact the source tree does not
  track.
- After merging source that changes CLI or hook behavior, reinstall (rebuild) the binary the hooks
  actually call, and verify with the SAME command against the installed path — not `cargo run` —
  before treating the feature as live.
- A recurrence — this defect landed three times in two days — is itself a signal that the reinstall
  step needs to be a checked, named part of the close or merge sequence, not a thing a session has
  to remember.
