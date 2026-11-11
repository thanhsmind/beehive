---
type: bee.delivery
title: cross-project-guard — delivery
description: "Delivery record for feature cross-project-guard: the write-guard refuses reads and shell commands that touch another project's tree, and the leader doctrine sends that work to the project's leader."
timestamp: 2026-11-11
bee:
  id: cross-project-guard-delivery
  lifecycle: active
  areas: [hook-runtime, doctrine-layer]
  required_context: [docs/history/cross-project-guard/CONTEXT.md, docs/history/cross-project-guard/plan.md]
  sources: [docs/history/cross-project-guard/CONTEXT.md, docs/history/cross-project-guard/plan.md, cell cpg-1, cell cpg-2, cell cpg-4]
---

# cross-project-guard — Delivery

## Concepts changed

- **docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md** — New cross-project refusal section: how another project is recognized, what stays allowed, which shell tokens are checked (including `..` and `cd` chains), the refusal remedy, and why `guards.cross_project_allow` is governed (cross-project-guard D1-D4).
- **docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md** — Rule index gains `agents-other-project-to-its-leader` with its home and spoken line (cross-project-guard D5).

## What shipped

- **cpg-1** — The write-guard refuses Read, Glob, Grep and Bash that touch another project's tree in every phase, with the waggledance remedy; `guards.cross_project_allow` is a governed key (commit c7bd39648).
- **cpg-2** — Leader doctrine rule `agents-other-project-to-its-leader` and the `guards.cross_project_allow` entry in the config reference (commit ba483cc2a).
- **cpg-4** — Bash relative tokens with `..` and `cd` chains resolve against the hook cwd before the same check (commit 5c43f7e96).
- **cpg-3** — Checked the feature's knowledge deltas and wrote this delivery record.

## Acceptance notes

- A user who wants a cross-repo research read adds the path to `guards.cross_project_allow` first; `ls` of a folder of repos is refused, and `waggledance_projects` lists projects instead.
- Non-Claude runtimes whose hooks do not send Read, Glob and Grep to the write-guard get the doctrine rule only.
