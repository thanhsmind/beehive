---
type: bee.pattern
title: A cell's named file may not be where the text it targets actually lives
description: Help/prose text can live only in a hand-edited generated payload that regen does not overwrite from the source file the plan named
timestamp: 2026-08-29
bee:
  id: slp-advisor-nudge-an-5-pitfall
  lifecycle: draft
  sources: [.bee/cells/archive/slp-advisor-nudge/an-5.json]
  polarity: pitfall
---

# A cell's named file may not be where the text it targets actually lives

## What happened

Cell an-5's plan named `catalog.rs` as the home for a `--kind`/`--signal`
help-prose fix. The actual help text lives only in the hand-edited
`packages/bee-rs/crates/bee/src/generated/registry_payload.json` — `bee dev
regen` does not write that file (an earlier finding, bln-3, already
recorded this). `catalog.rs` held only a flag-ratchet comment tied to a
different cell (sup-2) and stayed untouched; the ratchet count itself
(199 -> 199) still needed its ledger entry since no new flag name was added.

## The lesson

Before editing the file a plan names for a prose/help-text change, confirm
that file is actually the live source — a generated or hand-edited payload
can be the true home while the "obvious" source file only carries unrelated
history.

## Status

Candidate only. Naming the pattern, generalizing it beyond this cell, and
moving `bee.lifecycle` to `active` are a human or agent decision.
