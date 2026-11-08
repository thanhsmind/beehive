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
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/mise-shim-stdout.json, .bee/cells/archive/mise-shim-stdout/msh-1.json]
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
