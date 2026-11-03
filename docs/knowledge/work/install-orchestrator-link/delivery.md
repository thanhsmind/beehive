---
type: bee.delivery
title: install-orchestrator-link — delivery
description: "Delivery record proposed by bee knowledge promote for work item install-orchestrator-link: 2 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: install-orchestrator-link-delivery
  lifecycle: active
  required_context: [docs/history/install-orchestrator-link/CONTEXT.md]
  sources: [docs/history/install-orchestrator-link/CONTEXT.md, .bee/cells/archive/install-orchestrator-link/iol-1.json, .bee/cells/archive/install-orchestrator-link/iol-3.json]
---

# install-orchestrator-link — Delivery

## What shipped

- **iol-1** — README § Install carries the orchestrator (waggledance) subsection with the one-line installer and update note (1 file(s) changed)
- **iol-3** — Orchestrator install section moved verbatim README -> INSTALL.md (own ## section after Update/uninstall); README mention removed (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **iol-1** — `README § Install carries the orchestrator subsection; the linked raw URL resolves (HTTP 200).`
- **iol-3** — `README no longer mentions the waggledance installer; INSTALL.md carries the section; the raw install.sh URL resolves 200.`

## Deviations

- **iol-1** — followed the plan
- **iol-3** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work install-orchestrator-link` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/install-orchestrator-link/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
