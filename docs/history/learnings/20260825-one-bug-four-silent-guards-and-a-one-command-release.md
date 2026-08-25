# 2026-08-25 — One bug, four silent guards, and a one-command release

**Features:** statusline-binary-lookup, lane-row-order, opencode-contract-reds,
herding-registry-gap, release-one-command, release-doc-sync
**Shipped:** bee 2.22.1

## What was asked, and what it turned out to be

The user asked one question: *is the statusline running?* It was — but only the
first line. The per-model token and cost line was missing.

The cause was one wrong path in a shell resolver. Clearing it to the point where a
release could be cut took four more fixes, in four different features, none of
which had anything to do with the status line.

## The original defect

`packages/bee/statusline/statusline-command.sh` hunted the bee binary at
`$SELF_DIR/../bee`. The script is vendored into a host at `<repo>/.claude/`, so
those candidates resolved to `<repo>/bee` — while the binary lives at
`<repo>/.bee/bin/bee`. Nothing matched, the usage segment stayed empty, and the
script exited 0. Fail-open, by design, and therefore silent.

Three things had to line up for it to survive:

1. Commit A introduced the bad lookup while a Node fallback still stood behind it
   and masked the miss.
2. Commit B deleted that fallback **and** fixed the vendored `.claude/` copy — but
   not the canonical template the vendoring engine copies FROM. The two files
   diverged.
3. The same commit B deleted the test tree containing the byte-equality sweep that
   would have caught the divergence on the day it happened.

The user's own diagnosis, offered before any investigation, was correct: *it was
never wired into the build, so it got missed.*

## What the four extra reds were

| Red | Real cause | Where it was |
|---|---|---|
| Lane rows | A test asserted filesystem enumeration order against a reader documenting itself as unordered | `origin/main` |
| Belt parity | Every `PreToolUse` entry classified as blocking; a matcher-less telemetry probe tripped it | `origin/main`, days old |
| Tool anchors | Anchored on a minifier-generated identifier that changed between dependency builds | local only — the pipeline pins that dependency |
| Registry | Four dispatcher commands the registry never declared | `origin/main` |

Each was found only by fixing the one above it. The test runner stops at the first
failing target, so the suite honestly reported one failure, four times running.

## What generalises

Promoted as critical patterns:

- [`a-guard-deleted-with-its-runtime-is-a-guard-removed`](../../knowledge/patterns/20260825-a-guard-deleted-with-its-runtime-is-a-guard-removed.md)
  — including the sharper corollary that an identity guard is not a behaviour
  guard: byte-equality, fingerprints and manifest checks all compare copies to
  each other and none of them ever runs the thing.
- [`one-red-hides-the-rest`](../../knowledge/patterns/20260825-cargo-test-stops-at-the-first-failing-target.md)
  — never estimate release work from the first red.

Not promoted, kept here:

- **The fix that was almost wrong.** The first instruction written for the
  belt-parity red said to add a named list of non-blocking rules. That file cites
  its own rule against exactly that: a coverage gate derives ground truth and never
  compares two hand-authored lists. The correct fix was one line moved — `blocking`
  became "on `PreToolUse` **with a matcher**", a rule the codebase already stated
  about its own data. A hand list would have handled today's probe and silently
  admitted the next one. Independent analysis caught it before the worker committed.
- **A second-order coupling nearly traded one red for another.** Reclassifying the
  probe as advisory moved it into a *different* test's population, which demanded
  every advisory rule be wired or named. It was neither. Whenever a derived
  predicate narrows, check who reads the complement.
- **Depth was hidden from estimation, not from the suite.** Every run said "1
  failed". Four rounds of work looked like one, right up until each fix landed.

## What shipped as a result

`scripts/release.sh <VERSION>` now owns the entire release: bump both manifests,
regen, **run the declared suite**, commit path-scoped, tag, push, wait, verify.
The prologue used to be a prose checklist in `CLAUDE.md`, walked by hand every
time.

The test gate is the part that was not asked for. The previous script tagged and
pushed *before* anything ran the suite — so a red build became a published tag,
and a published tag never moves. It now runs the suite first and refuses to tag on
red, reading the command from the same declared field the pipeline reads so the
two cannot drift.

`CLAUDE.md`'s Release section was rewritten around the one command, with the
manual steps **removed** rather than left beside it. Describing both ways is how
the hand-walked one survives.

## Open, deliberately not done

- `bee herding agent-start --pane` and `bee herding record-worker --pane-id` spell
  the same concept two ways. Both are live surface; renaming either is a decision,
  not a repair. Filed.
- `docs/specs/onboarding.md` P2 and its anchor-map check still name deleted paths —
  the same rot class cleaned up for P9/P10, worth one sweep across every spec's
  pointer rows rather than another one-off.
