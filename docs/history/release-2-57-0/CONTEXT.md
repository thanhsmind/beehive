# Release 2.57.0 — Context

**Feature slug:** release-2-57-0
**Date:** 2026-10-21
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user, on 2026-10-21, after the token-thrift feature closed: "push code
rồi release" — push the code and release it.

## What will be done

Publish 2.57.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-56-0` did. The script
bumps both plugin manifests, runs the regen chain, runs the declared test
suite BEFORE anything is tagged, makes the release commit, tags, pushes
`main` and the tag, waits for `release-binaries`, and verifies the published
binaries and `SHA256SUMS`.

## Why 2.57.0

The released version is 2.56.0. This release ships the closed `token-thrift`
feature: the session-start output fits the host inline limit (budget 9,500
bytes with a named drop guard), zero-frontier discovery maps fold into one
line, the dispatch door groups roles per target, and the promote-proposal
lists in `bee orient` and the preamble name at most 3 entries.

New behavior for host repos is a minor bump.

## Risk named before the run

The script's test gate runs the whole declared suite before any tag. On this
machine two tests are red on `main` before this release
(`no_size_ceiling_on_instruction_text_survives_in_any_shipped_tooling_tree`,
`every_registered_write_or_read_capable_opencode_tool_is_mapped_or_named_as_a_gap`);
CI on `main` is green. If the gate stops on them, that is a fix-first cell,
then a re-run at the same version (idempotent).

## Locked Decisions

Active repo-scope decisions, cited not restated.

| ID | Store ID | Decision | Rationale |
|----|----------|----------|-----------|
| D1 | `11dcf251-aaf7-41b0-a1d2-5e53f9ca92db` | The release runs only through `scripts/release.sh` under an authorized deployment dispatch — never by hand-walking bump, tag or push. | A hand-walked checklist skipped tagging and pushing twice before. |
| D2 | `5d56be49-2aff-49bb-bf74-eb7857b04798` | The release is done ONLY when the script prints its final `OK` line — tag pushed, CI green, assets verified. | A release commit without that line is not a release. |

## Out of Scope

- Any source change: this lane publishes what `main` already holds.
