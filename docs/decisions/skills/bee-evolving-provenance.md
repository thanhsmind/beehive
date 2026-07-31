# Provenance — bee-evolving body rules

The evolving body states its rules bare (provenance exile, skill-token-diet D8). This table maps
each body rule to the decision(s) that authorize it and the rationale in one line. Long-form
records: `docs/decisions/`, `.bee/decisions.jsonl` (via `bee.mjs decisions search`), and
`docs/history/<feature>/CONTEXT.md` for the feature that locked each ID.

| Body rule | Decision IDs / labels | Rationale |
|---|---|---|
| This loop modifies bee itself, so it carries two human gates (Gate A, Gate B) ordinary work does not | D5 | Self-modifying work needs a heavier approval bar than normal execution |
| Invoked by the human only, never triggered automatically | D3 | Bee improving itself must never start on its own initiative |
| Never dispatched to an external CLI executor; self-modifying work stays on native tiers | decision 0019 | The orchestrator's goal-check only applies on native tiers — an external executor would run the loop outside that check |
| §0 HARD-GATE — bee-repo guard must pass before any other step | D3 | A host repo's vendored `.bee/bin/` copy is not the bee repo; the loop must never run there |
| `mergeDigests` revalidates and datamarks every foreign field before ranking/clustering | D2b | Foreign digest content is untrusted until revalidated and datamarked; ranking on raw foreign data would reopen injection paths |
| Hand the chosen fix to bee-writing-skills; no mechanical-edit exemption exists | D4, decision ff26725d | Even a "trivial" self-fix gets the full RED-first discipline — there is no shortcut tier for bee editing itself |
| Push is a named, manual step — never automatic, to any ref | D5 | An unreviewed self-modification leaving the machine is the harm this loop exists to prevent |
