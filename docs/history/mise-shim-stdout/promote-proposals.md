promote proposal for work item "mise-shim-stdout" (.bee/logs/scribing-runs.jsonl + .bee/lanes/mise-shim-stdout.json) — 1 capped cell(s): msh-1
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/mise-shim-stdout.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/mise-shim-stdout/delivery.md

---
type: bee.delivery
title: mise-shim-stdout — delivery
description: "Delivery record proposed by bee knowledge promote for work item mise-shim-stdout: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: mise-shim-stdout-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/mise-shim-stdout.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/mise-shim-stdout.json, .bee/cells/msh-1.json]
---

# mise-shim-stdout — Delivery

## What shipped

- **msh-1** — MISE_QUIET=1 exported once in each script; the install failure message now names an unparseable probe file too (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **msh-1** — `bash -n scripts/release.sh && bash -n scripts/install.sh && .bee/bin/bee dev release-manifest --check`

## Deviations

- **msh-1** — Ran the tiny cell inline in the MAIN checkout instead of a feature worktree — AGENTS.md exempts a solo tiny fix when no other session is live, and bee state session list showed only this session live — found a better route

## Provenance

Proposed by `bee knowledge promote --work mise-shim-stdout` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/mise-shim-stdout.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "mise-shim-stdout" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-30T07:47:06.649Z), the work item declares no bee.areas.

area bee-herding:
  - [msh-1] MISE_QUIET=1 exported once in each script; the install failure message now names an unparseable probe file too — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/msh-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell msh-1 — save as docs/knowledge/patterns/mise-shim-stdout-msh-1-pitfall.md

---
type: bee.pattern
title: mise-shim-stdout cell msh-1 — pitfall candidate
description: "Pitfall candidate mined from cell msh-1's capped trace: Ran the tiny cell inline in the MAIN checkout instead of a feature worktree — AGENTS.md exempts a solo tiny fix when no other session is live, and bee state se…"
timestamp: 2026-08-30
bee:
  id: mise-shim-stdout-msh-1-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/msh-1.json]
  polarity: pitfall
---

# mise-shim-stdout cell msh-1 — pitfall candidate

## What the cell did

MISE_QUIET=1 exported once in each script; the install failure message now names an unparseable probe file too

## Recorded evidence (verbatim from .bee/cells/msh-1.json)

- **deviation** — Ran the tiny cell inline in the MAIN checkout instead of a feature worktree — AGENTS.md exempts a solo tiny fix when no other session is live, and bee state session list showed only this session live — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.