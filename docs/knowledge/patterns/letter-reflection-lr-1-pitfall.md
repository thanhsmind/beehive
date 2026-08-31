---
type: bee.pattern
title: A shared struct's new field is a whole-crate change, not a local one
description: Adding a field to a record shared across the crate means every struct literal that builds it needs the field, whether or not the cell's file list named that path
timestamp: 2026-08-30
bee:
  id: letter-reflection-lr-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/archive/letter-reflection/lr-1.json]
  polarity: pitfall
---

# A shared struct's new field is a whole-crate change, not a local one

## What happened

Cell lr-1 added a `better` field to the shared `Entry`/`LetterItem` records
to carry the new Mistakes & reflection section. Every struct literal in the
crate that builds one of those records had to name the field, forcing edits
to five files the cell's own file list did not name (catalog.rs,
handlers_close.rs, drivers/close.rs, work.rs, mailbox_digest.rs) — plus a
flag-ratchet bump in catalog.rs for two new flag spellings. Separately, the
cell spelled the new run flag `--session-id` instead of the plan's `--run`,
since `--session-id` is what every other mailbox stop already uses for the
same idea; and it wired the new reflection fixture into the shared
`full_run()` test helper instead of only new tests, so four existing
authorship-walk/section-position tests exercise the new section for free.

## The lesson

A new field on a struct shared across a crate is never a single-file
change — expect every construction site to need it, reserve accordingly
rather than trusting the plan's file list, and check whether a
flag-ratchet or similar closed-set guard needs its own entry. When a new
flag's name has an established sibling convention elsewhere in the same
subsystem, follow the convention over a plan's specific spelling.

## Status

Candidate only. Naming the pattern, generalizing it beyond this cell, and
moving `bee.lifecycle` to `active` are a human or agent decision.
