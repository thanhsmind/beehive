# Issue sweep — Context

**Feature slug:** issue-sweep
**Date:** 2026-11-02
**Scope:** Standard (bugfix sweep)
**Domain types:** RUN

## What was asked

The user asked on 2026-11-02: "kiểm tra xem có issue nào không thì fix" — check
the open GitHub issues and fix them.

## What will be done

Fix the 12 open `bee-report` issues (#137-#144, #146-#149). Each one was traced
to live code on 2.70.0. Close #145 as stale: main CI is green since it was
filed. Leave #69 open: it is a feature request, not a defect.

## Locked decisions

- D1 (`4cb2e63a`): one standard lane, one cell per disjoint cluster of issues;
  every fix is proven by a red-then-green unit test that asserts the reported
  behavior.
