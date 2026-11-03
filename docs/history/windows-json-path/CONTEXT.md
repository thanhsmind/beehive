# Windows path escaping in the herding ledger test — Context

**Feature slug:** windows-json-path
**Date:** 2026-09-22
**Scope:** Tiny (fix-first)

## What was asked

The user asked "fix lỗi Windows" after the Windows CI job on `main` stayed
red once the `windows-cfg-unix` compile fix landed.

## What was found

GitHub run 35716941630 (`verify-windows`) now compiles and runs the suite:
3789 passed, 1 failed. `herding::run::tests::ledger_outcome_blocked_with_report_fills_evidence`
panics at `crates/bee/src/herding/run.rs:9654` with
`result-1.json is not valid JSON: invalid escape at line 1 column 92`. The
test formats a temp-dir path straight into a raw JSON string
(`run.rs:9645`); on Windows that path holds backslashes, which JSON reads as
escapes. No other test in the file formats a path into raw JSON.

## What will be done

Build the result document with `serde_json::json!` and `.to_string()`, so
the path is escaped. Test-only change, no behavior change.

## Locked Decisions

| ID | Store ID | Decision |
|----|----------|----------|
| D1 | `63e42e22-0d65-4940-bb9a-f0072052b587` | A test that puts a path into JSON builds it with `serde_json::json!`, never a raw format string. |
