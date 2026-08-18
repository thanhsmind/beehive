---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

Does the coordination core live inside the existing `bee` binary as a
new command group, or as its own crate and binary?

## Answer

Its own crate in the same workspace, compiled into the existing `bee`
binary as a library dependency — not a second shipped binary, and not
loose modules inside the `bee` crate.

This takes both halves of the trade. The crate boundary is what
enforces D02's genericness: the core cannot see bee's types, so
bee-shaped assumptions cannot leak in by accident, only by a deliberate
dependency edit that shows up in review. Linking it into `bee` keeps
one binary to build, version, checksum, ship and install — the release
matrix (`release-binaries.yml:44-51`) and the Windows install path stay
exactly as they are.

Consequence: the workspace `members` list grows by one; the core crate
takes no dependency on the `bee` crate, ever. A later reuse outside bee
is then a matter of adding a thin binary target, not of untangling.

Logged as D05.
