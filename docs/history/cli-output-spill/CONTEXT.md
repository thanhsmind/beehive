# CLI output spill — Context

**Feature slug:** cli-output-spill
**Date:** 2026-11-03
**Scope:** Small (one code cell, cos-1)

## What was asked

The user asked on 2026-11-03 which parts of the deepseek-harness repository
could make beehive more efficient, then chose two to build in order. This
feature is the first: the spill policy of `packages/spill/spill-policy` in that
repository, adapted to the bee CLI. A large text output goes to a file, and the
caller sees the head, one marker line, the tail and the file path.

## Locked decisions

- D1 (`f0aa3d67`): spill applies only to an allowlist of read verbs (`status`,
  `cells list`, `decisions list`, `backlog list`, `knowledge context`,
  `knowledge search`, and every `--help` form), only to human-text stdout, and
  only when stdout is not a terminal. A `--json` run never spills. The
  allowlist has one home, the `SPILL_VERBS` const in `output_spill.rs`.
- D2 (`8b0803c8`, supersedes `347abd01`): spill is opt-in per repository. The
  config key `output_spill_bytes` sets the threshold. An absent key or `0`
  means no spill. `BEE_OUTPUT_FULL=1` prints the full output for one run.
  Above the threshold, bee writes the whole stdout to
  `.bee/tmp/output/<verb>-<utc-stamp>.txt` and prints the leading lines up to
  two thirds of the threshold, one marker line, and the trailing lines up to
  one quarter of the threshold. The exit code and stderr do not change. This
  repository sets `8000`.
- D3 (`41814204`): the capture redirects file descriptor 1 on Unix only. On
  Windows the output passes through unchanged.

## Measured sizes that set the allowlist

On 2026-11-03 in this repository: `bee status` printed 12160 bytes,
`bee cells list` 17499, `bee status --json` 39247 and `bee --help --all`
156584. `bee orient` printed 3703 and stays off the list.
