# Full Failure Evidence — Context

**Feature slug:** full-failure-evidence
**Date:** 2026-08-14
**Shaping session:** complete
**Scope:** Quick
**Domain types:** RUN

## Feature Boundary

When a declared test command fails, its complete output is kept and the
excerpt says where. This feature ends at how a failure is recorded — it does
not touch what makes a command fail, nor the close/finish doors that read the
record.

## Feature Origin

`bee close` reported the declared suite red once, with this excerpt:

> `me("standard"), "shaping", &[]));`

Five subsequent runs passed and the failure was never identified. The excerpt
is what remained: `FAILURE_EXCERPT_MAX_CHARS = 500`
(`verbs/cells/finish_support.rs:29`) and `truncate_chars_tail`
(`:119`) keep the last 500 characters, which for `cargo test` lands inside a
source-line echo rather than the `failures:` block that names the failing
tests. The rest of the output is discarded — nothing writes it anywhere.

So a red that does not reproduce leaves no way back to what failed. That is
the defect: not the flake, which may be legitimate, but the fact that the run
which observed it kept no evidence.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The complete output of a failing declared command is written to a file under `.bee/logs/`, and the excerpt names that path. | The evidence is only useful if it survives the run that saw it. `.bee/logs/` is already gitignored, so a large log costs the repository nothing. |
| D2 | The excerpt stays bounded. `FAILURE_EXCERPT_MAX_CHARS` is NOT raised. | Widening a threshold to make a red legible is the move `docs/knowledge/patterns/20260723-clearing-a-red-by-widening-the-threshold-is-not-fixing-the-check.md` names as not-a-fix. A bounded excerpt plus a complete log is strictly better than a longer excerpt: no bound is large enough for every suite. |
| D3 | A green run writes no failure log, and leaves no stale one behind from a previous red. | A log directory that accumulates one file per failure forever is its own debt; a green run is the natural point to know the previous failure is resolved. |
| D4 | The excerpt keeps its current tail-of-output shape. Nothing tries to detect a `failures:` block or any other framework-specific marker. | `commands.test` is whatever the project declared; bee does not know it is cargo. Parsing for a framework's summary would be a guess that silently degrades on any other runner. |

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| excerpt | The bounded, human-facing tail of a failing command's output, carried in the record and quoted by the refusing door. |
| failure log | The complete captured output of one failing command run, written to a file whose path the excerpt names. |

## Existing Code Context

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:29` —
  `FAILURE_EXCERPT_MAX_CHARS`; `:100-130` — where the output is captured,
  trimmed, truncated and dropped; `:129` — the `CmdRun` the record is built
  from; `:140-160` — `tests_record_value`, which serializes it.
- `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:31` —
  `test_results_path`, the existing `.bee/logs/` writer, and the precedent for
  where a new log file belongs.
- Tests that pin the excerpt: `verbs/cells/tests.rs:1691`, `:1697`, `:1701`,
  `:1710`.

### Established Patterns

- `.bee/logs/` already holds `test-results.json`, `contention.jsonl` and
  `timings.jsonl`; it is gitignored (`.gitignore`, `.bee/logs/`).
- `write_json_atomic` / `write_text_atomic` (`src/fsutil.rs`) are the writers
  every record in this area uses.

## Canonical References

- `docs/knowledge/patterns/20260723-clearing-a-red-by-widening-the-threshold-is-not-fixing-the-check.md`
  — why D2 refuses to raise the limit.
- Backlog finding, 2026-08-14: "Declared suite failed once under bee close and
  passed on five other runs, unidentified" — the observation this feature answers.

## Outstanding Questions

### Deferred To Planning

- [ ] Does anything besides `cells finish` and `close` read
  `test-results.json`'s `failure_excerpt`? A consumer expecting the old shape
  would need the added path line to be additive, not a replacement.
- [ ] One log per command or one per run? A run can declare several commands
  and more than one can fail.

## Deferred Ideas

- Detecting a framework's own failure summary to build a smarter excerpt —
  refused by D4.
- Pruning old failure logs on a schedule; D3's green-run cleanup is the whole
  of the retention story for now.

## Handoff Note

CONTEXT.md is the source of truth. The feature answers an unreproduced red by
keeping evidence, not by chasing the flake.
