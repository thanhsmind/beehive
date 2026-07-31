# expertise/

Craft guides for doing software work well — how to plan, decide, test,
review, document, and debug. Skills reference these files; the files never
reference skills, tooling, or workflow state. Every guide is written in
universal terms and must read as sound guidance dropped into any software
repo.

One file per discipline. Voice: direct, rule-stating, judgment-teaching,
with concrete micro-examples where a bare rule would be ambiguous.

## Distribution

This directory is the SOURCE. Onboarding
(`packages/bee/scripts/onboard_bee.mjs`) vendors every `*.md` here to
`.bee/expertise/` in every host repo — and in this repo via self-onboard —
exactly like the engine is vendored to `.bee/bin/`. Skills always reference
the vendored path (`.bee/expertise/<name>.md`), never this directory, so the
pointers resolve identically in a host repo and in the bee checkout. Edit
guides here only; a copy under `.bee/expertise/` is managed output —
onboarding refreshes drifted files and removes stale ones on every apply.

## Guides

| Guide | Discipline |
|---|---|
| [planning.md](planning.md) | How to shape work |
| [decisions.md](decisions.md) | How to make and record decisions |
| [tests.md](tests.md) | What to test and how to judge coverage |
| [review.md](review.md) | Finding quality, severity, verification |
| [documentation.md](documentation.md) | Specs a human can rebuild from |
| [debugging.md](debugging.md) | Repro-first, instrument before guessing |
