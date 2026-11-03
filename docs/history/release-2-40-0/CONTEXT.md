# Release 2.40.0 — Context

**Feature slug:** release-2-40-0
**Date:** 2026-09-16
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked, in one line, to push the code and cut a release
("push code để release"). Asked directly whether to release with the
`config-role-key-guard` uat door still open, they chose the full 2.40.0
release over pushing commits without a tag. That answer is recorded as their
`uat` approval for `config-role-key-guard`.

## What was found

`scripts/release.sh` refuses to run without a release permit: it requires
`BEE_DISPATCH_ID` naming a dispatch prepared under the `deploy` role at the
`deployment` stage, and `bee dispatch authorize` then validates that record
against the requested version, the current main commit, the approved plan hash,
the stage, the role, the issuing session and a two-hour lifetime. A dispatch for
that stage requires an approved `bee-plan/v2` plan carrying a role plan, so the
release needs its own plan — `config-role-key-guard`'s plan cannot carry it.

This was found by running the script directly and reading its refusal, not by
reading the code first.

## What will be done

Publish 2.40.0 from `main` through the one sanctioned command. The script owns
the whole release: bump both plugin manifests, run the regen chain, run the
declared test suite BEFORE anything is tagged, make the release commit, tag,
push `main` and the tag (which carries the 9 commits already waiting), wait for
the `release-binaries` workflow to go green, and verify the published release
carries the binaries and `SHA256SUMS`.

## Why 2.40.0

The current released version is 2.39.0. This release ships new, user-visible
guard behavior: the write guard now refuses an agent write that changes the
model table bee resolves (`team` / legacy `models`, `herding.agents`,
`herding.agent_command`) in `.bee/config.json` or `.bee/config.local.json`, or
that disables the write guard itself. Writes that previously succeeded now
refuse, so the minor number moves.

## Locked Decisions

Both decisions below are active repo-scope decisions carried from the 2.39.0
release lane. They are cited here, not restated or reinterpreted.

| ID | Store ID | Decision | Rationale |
|----|----------|----------|-----------|
| D1 | `11dcf251-aaf7-41b0-a1d2-5e53f9ca92db` | The release runs only through `scripts/release.sh` under an authorized deployment dispatch — never by hand-walking bump, tag or push. | The script exists because a hand-walked checklist skipped tagging and pushing twice before (2.6.6, 2.6.7), and the permit machinery is what makes the deploy role's authority auditable. |
| D2 | `5d56be49-2aff-49bb-bf74-eb7857b04798` | The release is done ONLY when the script prints its final `OK` line — tag pushed, CI green, assets verified. | A release commit without that line is not a release. |

## Out of Scope

- Any source change: this lane publishes what `main` already holds.
- Re-proving `config-role-key-guard`; it capped with its own proof
  (36 suites, 0 failed) plus a live run of the installed binary, and its `uat`
  gate is approved.
