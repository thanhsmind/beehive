# Paseo wait deadline — Context

**Feature slug:** paseo-wait-deadline
**Date:** 2026-11-03
**Scope:** Small bugfix (one code cell, pwd-1)

## What was asked

The user asked on 2026-11-03 to fix the 900-second idle timeout that ended two
healthy herding workers (cells cos-1 and wvr-1) as `timed_out_idle`.

## What was found

- Both workers lost their heartbeat at about the same minute (02:23 and
  02:24 UTC) and never got it back. Both kept working and each wrote a valid
  result at the end of its run.
- In the paseo wait loop, the heartbeat is refreshed only when an armed
  `paseo agent wait --timeout N` returns. A wait child that does not return
  blocks every refresh and every re-inspect, so the idle timeout fires while
  `paseo inspect` still reports `running`.
- At that minute the paseo daemon log shows several `ws_slow_request` lines,
  including `wait_for_finish_request`. The daemon ran 0.13.2 and the CLI ran
  0.14.0. A hung wait is the likely trigger; no hung wait was observed
  directly.
- A first guess, that a long `cargo` build gave no heartbeat, was wrong: a live
  test showed the pi shell tool ends each call after about 25 seconds.

## Locked decisions

- D1 (`917fa9e4`): an armed wait that has not returned 30 seconds after its
  own timeout counts as a failed wait. bee cancels it and switches to the
  existing inspect fallback, so `paseo inspect` decides liveness. A running
  agent refreshes the heartbeat and gets a new wait. The deadline uses the
  loop's injected clock.
