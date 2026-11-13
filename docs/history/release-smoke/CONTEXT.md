# The release runs a live check of the built binary — Context

**Feature slug:** release-smoke
**Date:** 2026-11-13
**Shaping session:** complete (the owner said verify should run at release, then chose the scripted step)
**Scope:** Quick
**Domain types:** RUN

## Feature Boundary

`scripts/release.sh` runs the regen chain and the declared test suite, and
nothing drives the real `bee` binary before a tag is pushed. After this
feature the release script builds the binary, onboards it into a throwaway
repo, drives a short fixed list of first-contact commands, and stops the
release when any of them fails. The feature ends there: it adds no rule
for agents, no new skip flag, and no change to CI.

## What was asked, what was found, what will be done

- Asked: verify should run on its own at release.
- Found: no line of `scripts/release.sh` or `.github/workflows/` calls the
  verify driver. The driver, `.bee/verify/verify-app/control-bee`, already
  builds the binary, launches an onboarded sandbox and has a doctor, but it
  has no one-shot run.
- Done: one `smoke` subcommand on the driver, one call to it in the
  release script after the test gate, and the docs that name the step.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The release script runs a scripted live check of the built binary in a throwaway onboarded repo before anything is tagged; a red check stops the release. A scripted step, not a rule for the agent. | A step the script runs cannot be forgotten. |
| D2 | The live check is one new subcommand `smoke` of `.bee/verify/verify-app/control-bee`. It uses a run id of its own and never moves the `current-run` pointer. It builds the release binary, launches an onboarded sandbox, runs the driver's doctor, then drives this fixed list: the version equals the plugin manifest, status as JSON, orient, the JSON help surface, a decision written then found by search, and a second onboard that reports up to date. Each step prints one `ok` or `FAIL` line. The first failure exits non-zero, names the step and the evidence folder, and keeps the sandbox. A clean run removes the sandbox and keeps the evidence. | The driver already owns the sandbox and its isolation check; a private run id keeps a release from clobbering a verify run in flight. |
| D3 | `scripts/release.sh` runs the smoke right after the test gate and before the release commit. A red smoke stops the release with nothing committed, tagged or pushed, and the existing restore-on-abort path puts the tree back. `--no-test` skips the smoke too, with one log line that says so; the confirm checklist text does not change. The idempotent re-run of an already committed version does not run it again. | Before the commit is the last point where a red result costs nothing; the checklist text is pinned by `scripts/tests/release-confirm-gate.sh`. |

### Agent's Discretion

- The exact bee calls behind each step of the fixed list, provided each one is deterministic, needs no network and no live agent, and passes on the tree as it stands.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/verify-pipeline/overview.md | suspect | Owns the verify driver and the proof discipline the smoke joins. |
| docs/knowledge/patterns/20261112-verify-harness-sandbox-leaks-into-the-real-environment.md | pattern | Why the smoke must keep the driver's isolation and strip the driver's environment. |

### Code

| Path | What it decides |
|------|-----------------|
| `scripts/release.sh` | The order of the release steps and the restore-on-abort path. |
| `.bee/verify/verify-app/control-bee` | The sandbox, the build, the launch and the doctor the smoke reuses. |
| `.bee/verify/verify-app/SKILL.md` | The driver's documented commands. |
| `LEADER.md` | The project's own description of what a release runs. |

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
