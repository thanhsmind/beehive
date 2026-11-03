---
type: bee.pattern
title: A release's regen chain run with the vendored binary can regenerate stale artifacts against its own new source
description: scripts/release.sh regenerates with the VENDORED .bee/bin/bee — a release that lands new generator behavior regenerates with the OLD binary and writes stale artifacts, turning the pre-tag suite red
tags: [release, regen, bootstrapping]
timestamp: 2026-08-30
bee:
  id: pattern-20260830-release-regen-stale-vendored-binary
  lifecycle: active
  areas: [workflow-state]
  sources: ["release v2.25.0 first attempt, 2026-08-30 — .pi/extensions INVENTORY_ROOTS entry from pi-support regenerated stale, pre-tag suite went red"]
  polarity: pitfall
  evidence: observed
---

# A release's regen chain can bootstrap against its own stale binary

`scripts/release.sh` runs its regen chain with the VENDORED `.bee/bin/bee`
— the binary already committed in the tree, not one built from the source
at HEAD. A release that lands new generator behavior in the same release
(here: the `.pi/extensions` `INVENTORY_ROOTS` entry added by pi-support)
regenerates with the OLD binary, writes stale artifacts, and turns the
pre-tag suite red.

**It failed safe.** Nothing was tagged, the tree was restored — this is a
caught defect, not a shipped one. The remedy used: rebuild and atomically
replace `.bee/bin/bee` (write to a temp path, then `mv` — a plain `cp` fails
with "Text file busy" while hooks hold the binary open), then re-run.

## Fix direction (not yet implemented)

`release.sh` should build and use the CURRENT source's binary for regen,
or refuse outright when the vendored binary's version predates HEAD's
manifests. Filed as a friction record only — a `release.sh` change is its
own future feature.
