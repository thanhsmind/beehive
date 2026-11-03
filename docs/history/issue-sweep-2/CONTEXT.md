# Issue sweep 2 — Context

**Feature slug:** issue-sweep-2
**Date:** 2026-11-03
**Scope:** Standard (three bugfix cells)

## What was asked

The user asked on 2026-11-03 to check the open GitHub issues, mark the ones
already done, and fix the ones not done.

## Locked decisions

- D1 (`ab1c51bb`): fix the three open bee issues that issue-sweep did not
  reach, one cell each with red-then-green proof. #147 gets a
  `bee state session attach` alias, and the FIX lines under
  `verbs/state_group` name it. #150 makes the close routing door read a
  feature-local decision's slash, range or comma D-ID list. #151 and #152
  make `worktree merge` ignore an untracked `bee-*` skill directory that also
  exists under the same skill root in the main checkout.

## Out of scope

- #153 duplicates #137, and #148 no longer reproduces; issue-sweep settled
  both.
- #69 is a feature request for a cross-project dashboard.
