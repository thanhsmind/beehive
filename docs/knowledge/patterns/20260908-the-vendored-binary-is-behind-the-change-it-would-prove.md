---
type: bee.pattern
title: The vendored binary is behind the change it would prove
description: "The vendored .bee/bin/bee predates the change under test, so a cell that changes the binary and then drives it records a green describing the state before the change."
tags: [failure, proof, tooling, worktree, regen]
timestamp: 2026-09-08
bee:
  id: pattern-20260908-the-vendored-binary-is-behind-the-change-it-would-prove
  lifecycle: active
  areas: [rust-runtime]
---

`.bee/bin/bee` is not the binary a cell just built. In a feature worktree it
is a symlink into the main checkout; in the main checkout it is whatever was
copied there last. Either way it predates the change under test.

That matters at exactly one moment: when the thing being changed is the
binary's own behavior, and the proof runs through the binary. A cell that
regenerates a manifest, re-renders an agent template, re-runs onboarding, or
asserts a new flag will drive the OLD code and record a green that describes
the state before the change. Nothing is red, nothing is wrong-looking, and the
proof is worthless.

Three cells hit this independently and each worked it out from scratch:
`grrs-2` applying onboarding, `pib-5` running the regen chain, `pis-2` proving
the release manifest. All three had to run the freshly built binary from the
cargo target directory instead of the path their own verify line named, and
all three recorded it as a deviation because the cell had told them otherwise.

## What to do

- When a cell changes the binary and then drives the binary, its verify line
  names the freshly built artifact, not `.bee/bin/bee`.
- After a release or a merge that changes the binary, re-vendor it — build,
  then copy over `.bee/bin/bee` — before trusting any door that reads it.
  `bee doctor` names this drift as `binary_freshness` and prints the exact
  build-and-copy remedy.
- The write guard will refuse an attempt to replace the symlink from inside a
  worktree, and it is right to: the vendored copy belongs to the main
  checkout. Run the build from there.

## What this is not

It is not a reason to distrust `.bee/bin/bee` generally. For every cell that
does not change the binary, the vendored copy is the correct thing to run —
it is the binary the hooks are wired to, and using something else there
proves the wrong artifact.
