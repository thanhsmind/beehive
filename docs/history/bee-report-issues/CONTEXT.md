# Bee Report Issues — Context

**Feature slug:** bee-report-issues
**Date:** 2026-10-06
**Shaping session:** complete
**Scope:** High-risk
**Domain types:** CALL | RUN

## Feature Boundary

When bee itself misbehaves while an agent works in a host repository, the agent packages the problem as one clear report and files it as a GitHub issue on `thanhsmind/beehive`. In the beehive repository, the feedback pipeline reads those open issues as one more feedback source, and `bee-evolving` ranks and fixes them through its two existing gates. The feature ends there: the host agent never edits bee, and nothing in beehive fixes an issue without the `bee-evolving` gates.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | When bee misbehaves inside a host repo, the agent does not fix bee there. It packages the problem (symptom, evidence, suspected root cause) as a GitHub issue on `thanhsmind/beehive`, and beehive pulls those issues in and fixes them through its own chain. (store `b58c1cef`) | Keeps the `bee-evolving` rule that bee is never edited in a host repo. A live local patch in the AICoworker Self-Patch style was rejected. |
| D2 | The host agent files the issue by itself, without asking, after a mandatory scrub of secrets, absolute paths and host source excerpts. The issue carries the `bee-report` label. (store `c3be1e3f`) | `thanhsmind/beehive` is a public repo; the scrub is the only guard, so it is a refusal path, never best-effort. |
| D3 | In beehive, open `bee-report` issues enter the feedback pipeline: `bee feedback collect` and `bee feedback rank` read them as hostile input under the same trust boundary as dogfood digests, and `bee-evolving` ranks and fixes them through its two gates. (store `d45b1e6b`) | One ranked intake. A one-PBI-per-issue path was rejected. |

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| bee report | One packaged problem about bee itself: symptom, evidence, suspected root cause, bee version, and the command that showed it. |
| scrub | The outbound check that removes or refuses secrets, absolute paths and host source excerpts before anything reaches the public issue tracker. |
| host repo | Any repository that runs bee but does not develop it. |

## Specific Ideas And References

- AICoworker `aicoworker-self-patch` skill (`~/aicoworker/openclaw/skills/aicoworker-self-patch/SKILL.md`): a patch record with `symptom` and `rootCause` as one plain sentence each, evidence in notes, and a check after each app update whether the fix already shipped. Bee takes the record shape and the "root cause in source, not in the symptom" discipline; it does not take the live patch.

<!-- bee:not-a-deferral: section heading of the template; nothing was deferred -->
## Deferred Ideas

None.
<!-- /bee:not-a-deferral -->
