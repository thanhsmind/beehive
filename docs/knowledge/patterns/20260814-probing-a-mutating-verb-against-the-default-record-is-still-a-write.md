---
type: bee.pattern
title: "Probing a mutating verb against the default record is still a write"
description: "A capture-pass agent ran bee state gate with no --lane to see whether a new flag was accepted; the default record it silently targeted was the real, active feature's own workflow record, and the probe overwrote genuine actor/reason/bypass_level audit fields that no CLI flag combination can restore to null."
tags: [swarming, cli, workflow-record, default-record-target-resolution, capture]
timestamp: 2026-08-14
bee:
  id: pattern-20260814-probing-a-mutating-verb-against-the-default-record-is-still-a-write
  lifecycle: active
  sources: ["traceable-runs capture pass, 2026-08-14: .bee/runtime/workflows/wf-4605d9c6/state.json — execution gate actor/at/reason/bypass_level overwritten by a probe call, discovered by comparing against the untouched context/shape sibling entries, repaired by hand-restoring the four fields to null (no CLI flag path can write null to --actor)", "docs/history/learnings/20260814-traceable-runs-capture.md"]
  polarity: pitfall
  critical: true
---

# Probing a mutating verb against the default record is still a write

`bee state gate` (and every verb sharing its target-resolution chain) picks
its target in three tiers: an explicit `--lane`, the calling session's bound
lane, or — when neither applies — "the default `state.json` record for an
unbound session." That third tier reads like a safe, inert fallback. It is
not: it is whatever feature the repo's `.bee/state.json` currently names,
which in an active repo is very often real, live work.

A capture/documentation pass that runs a mutating verb "just to see what
flags it accepts" has no signal at the command line that the default record
is production state rather than scratch. In this incident, confirming that
`--actor auto --bypass-level normal --reason test` was accepted overwrote
the `execution` gate's genuine `actor`/`at`/`reason`/`bypass_level` fields
on the real, still-active `traceable-runs` workflow record. The damage was
recoverable only because a structurally identical sibling gate entry
(`context`, untouched) proved what the honest "before" shape looked like —
and even then, no `bee state gate` flag combination can write those fields
back to `null` (`--actor` always writes a string, defaulting to `"user"`),
so the repair was a direct hand-edit of the record file, justified only
because the prior value was provably reconstructible, not guessed.

**Rule.** Never invoke a mutating CLI verb to observe acceptance or shape —
use `--help`, a `--show`/read-only counterpart, or a status/list command
instead. When a mutating call is genuinely needed to observe an effect,
target an isolated scratch fixture (its own `.bee/`), never a checkout with
real active work. If a mutating probe against real state cannot be avoided,
capture every field it might touch beforehand so an accidental write is
repairable byte-for-byte rather than approximated.
