---
type: bee.pattern
title: A required field added to a shared params struct ripples to every literal call site — pinned counts included
description: "Adding a required field to a params struct that call sites construct literally forces a repo-wide sweep: every literal construction (tests included) fails to compile until updated, and pinned-count or allow-list assertions (flag catalogs, CLI-shape guards) shift too. Scope the cell for the sweep up front — the fallout is mechanical but it is not free, and it lands in files the cell never named."
timestamp: 2026-08-17
bee:
  id: pattern-20260817-required-field-ripples-to-call-sites
  lifecycle: active
  areas: [decision-memory]
  sources: ["knowledge-distill-trigger cell kdt-3 deviation (2026-08-17): LogParams gained required relation/trigger fields; decisions/tests.rs, triggers/mod.rs, catalog.rs, and hooks/cli_shape.rs all changed though none sat in cell.files", "docs/history/knowledge-distill-trigger/promote-proposals.md pattern candidate"]
---

A cell added two required fields (`relation`, `trigger`) to a shared
params struct (`LogParams`). The cell's own scope named the verb's
source file — and the change then touched four files the cell never
listed, because the fallout of a required field is mechanical and
repo-wide:

- every literal construction of the struct (dozens of test call sites)
  stopped compiling until it passed the new fields;
- the pinned CLI flag count in the catalog shifted and its assertion
  had to move with it;
- the CLI-shape guard's schema for the verb had to learn the new
  required flags.

**The rule:** when a plan adds a REQUIRED field to a struct that call
sites construct literally, the sweep of those call sites is part of
the cell's real scope — tests, pinned-count assertions, and shape
guards included. Either name the sweep in the cell's files up front or
expect the worker to record the fallout as a deviation; an optional
field with a default is the cheaper shape when the sweep is not worth
it.
