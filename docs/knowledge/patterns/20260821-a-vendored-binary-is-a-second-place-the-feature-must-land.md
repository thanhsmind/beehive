---
type: bee.pattern
title: A vendored binary is a second place the feature must land
description: A vendored binary is a second place the feature must land
tags: [failure, verdicts, build, freshness, cli]
timestamp: 2026-08-21
bee:
  id: pattern-20260821-a-vendored-binary-is-a-second-place-the-feature-must-land
  lifecycle: active
  areas: [rust-runtime, bee-herding]
  sources: ["capture stub 446c0c54 (worker-brief-expertise live dogfood)", "live observation 2026-08-21: the vendored binary was a release behind its checkout", docs/knowledge/patterns/20260714-a-fail-open-host-swallows-fail-closed-throws.md]
  polarity: pitfall
  critical: true
---

# A vendored binary is a second place the feature must land

A repository that runs its own tool from a checked-in copy of that
tool has TWO places a change must arrive: the source, and the copy.
Merging the source is not shipping. Until the copy is rebuilt, every
caller — hooks, skills, the agent's own commands — still runs the
previous release, and the feature is simply absent at the only place
anyone uses it.

Absent, not broken. That is what makes it survive review. The merge
was green, the tests were green, the close-time verdict said the
contract was respected. Nothing was wrong with the code; the code was
not running.

Two live cases, sixteen days apart.

A worker-brief feature added a `--expertise` flag. Source merged, cell
capped, verdict "respected". The vendored binary was never rebuilt, so
the running command was the old one — and its argument parser was
fail-open on unknown flags, so the flag was neither honored nor
refused. It was swallowed. The brief rendered without its Expertise
section and looked exactly like a brief that had never asked for one.

Later the whole tool went a release ahead of its own copy: a `2.17.1`
binary in a `2.18.0` checkout. The session state reported the drift
plainly. The health check that exists to catch precisely this said
`ok` — because its version arm compared the binary's package version
against the same package version in the manifest, two values that are
pinned and never bump on a release, so the comparison can never
disagree; and its file-freshness arm never watched the manifest that
actually carries the release version.

**The rule, in three parts.**

For a verdict: a change that only takes effect through a rebuilt
artifact is "respected" only when the REBUILT artifact is demonstrated
doing the new thing. A source diff and a green suite are not evidence
that the running copy changed. Rebuild, reinstall, then run the
command and read its output.

For a freshness check: compare the version the artifact actually ships
under, and watch every file the build genuinely consumes — a manifest
that is embedded at build time is a build input, whatever directory it
sits in. A check that compares two values which cannot differ is worse
than no check: it answers `ok` with authority.

That half is now fixed and owned by a machine: the health check compares
release versions, treats the plugin manifest as a build input, calls a
binary too old to report its release version stale rather than fine, and
answers unknown when it cannot read the manifest at all — see
`areas/hook-runtime/health-checks-and-proof-surfaces.md` (cell dfv-1,
2026-08-21). What the check still cannot do is notice a REBUILT binary
that was never installed, so the verdict discipline below is not
retired.

For an argument parser: an unknown flag is a refusal, never a
shrug. A fail-open parser turns "you are running the wrong binary"
into "the feature did nothing", which is the same shape as a bug in
the feature and sends the next hour to the wrong place.

## The same rule runs backwards

A generated or spliced file is the copy, and editing it alone is worse
than useless: the next regeneration reverts the edit, silently and with
every check green. One delivery hit this on the always-loaded operating
document, whose body is spliced from a template between two markers —
the cell required a regeneration, so an edit made only to the spliced
file would have been thrown away by the very step the cell owed.

Before editing any file, ask which direction it flows. If something
generates it, edit the source and let the generator place the copy; if
you edit the copy, prove the generator agrees, in the same commit.
