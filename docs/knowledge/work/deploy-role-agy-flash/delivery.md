---
type: bee.delivery
title: deploy-role-agy-flash — delivery
description: "Delivery record proposed by bee knowledge promote for work item deploy-role-agy-flash: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-14
bee:
  id: deploy-role-agy-flash-delivery
  lifecycle: active
  required_context: [.bee/lanes/deploy-role-agy-flash.json]
  sources: [.bee/lanes/deploy-role-agy-flash.json, .bee/cells/archive/deploy-role-agy-flash/dragy-1.json]
---

# deploy-role-agy-flash — Delivery

## What shipped

- **dragy-1** — Added team.pi.deploy slot in .bee/config.json routing release work through agy-flash (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dragy-1** — `.bee/bin/bee team show --runtime pi --json | jq -e 'any(.runtimes[0].roles[]; .role == "deploy" and .transport == "herding: agy-flash" and .description == "publish an approved release from main, wait for CI, and verify release assets")'` — verified team.pi.deploy configuration

## Deviations

- **dragy-1** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work deploy-role-agy-flash` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/deploy-role-agy-flash.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
