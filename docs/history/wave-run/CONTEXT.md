# Wave run — Context

**Feature slug:** wave-run
**Date:** 2026-11-03
**Scope:** Small (one code cell, wvr-1)

## What was asked

The user asked on 2026-11-03 which parts of the deepseek-harness repository
could make beehive more efficient, then chose two to build in order. This
feature is the second. It adapts the idea of that repository's `workflow`
tool: one call fans work out to many workers and returns one final value, so
the leader does not read each worker's payload and envelope.

## Locked decisions

- D1 (`fb99f277`): `bee dispatch wave` gains an additive `--run` flag. Without
  it the output is byte-identical. With it, bee runs every herding item of the
  wave in parallel and waits for all of them. `Agent` items are not run and
  come back unchanged under `native`.
- D2 (`39f4fbcf`): with `--run` the output is one value `{ran, native,
  skipped}` with one compact row per herding cell. The full envelope is saved
  to `.bee/workers/<cell id>-wave-run.json`. The verb exits 0 only when every
  row ended `done`. bee never retries a run and never caps a cell here.

## Why it saves context

On 2026-11-03 a herding cell payload from `bee dispatch prepare` carried a
brief of about 6.5 KB, and each worker result was 1–3 KB plus its report path.
A `ran` row is a few hundred bytes.
