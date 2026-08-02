# expertise/

Craft guides for doing software work well — how to plan, decide, test,
review, document, and debug. Skills reference these files; the files never
reference skills, tooling, or workflow state. Every guide is written in
universal terms and must read as sound guidance dropped into any software
repo.

One file per discipline. Voice: direct, rule-stating, judgment-teaching,
with concrete micro-examples where a bare rule would be ambiguous. Every
rule carries a citable name (its heading, or a bolded handle in prose),
and each guide opens with a routing table — situation → entry — so a
reader jumps straight to the rule they need.

## Distribution

The authoring home for these guides is the bee source repo's
`expertise/` directory. Onboarding vendors every `*.md` there to
`.bee/expertise/` in each host repo — exactly like the engine is
vendored to `.bee/bin/` — so if you are reading this under
`.bee/expertise/`, you are reading managed output: onboarding refreshes
drifted files and removes stale ones on every apply; edits belong in the
bee source repo. Skills always reference the vendored path
(`.bee/expertise/<name>.md`), never the source directory, so the
pointers resolve identically in a host repo and in the bee checkout.

## Guides

[INDEX.md](INDEX.md) is the router: one row per guide — what it covers
and when to load it. Pick by task; never load all the guides at once.

| Guide | Discipline |
|---|---|
| [thinking.md](thinking.md) | Choosing a reasoning approach, stress-testing arguments |
| [planning.md](planning.md) | How to shape work |
| [architecture.md](architecture.md) | Structure, boundaries, dependencies, anti-patterns |
| [decisions.md](decisions.md) | How to make and record decisions |
| [tests.md](tests.md) | What to test and how to judge coverage |
| [review.md](review.md) | Finding quality, severity, verification |
| [documentation.md](documentation.md) | Specs a human can rebuild from |
| [knowledge.md](knowledge.md) | Building the project's own knowledge layer |
| [debugging.md](debugging.md) | Repro-first, instrument before guessing |
