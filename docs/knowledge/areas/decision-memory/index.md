<!--
GENERATED FILE — do not hand-edit.
Rendered by `bee knowledge index` from concept frontmatter inside docs/knowledge/ (okf-foundation D21).
Regenerate: `bee knowledge index`. Check freshness: `bee knowledge index --check`.
Deterministic: byte-identical for the same bundle contents — path-sorted entries, LF endings,
never a generation timestamp or any other wall-clock value.
-->

# areas/decision-memory/

## Concepts

- [Decision Memory — the unified backlog store (event-sourced PBI records)](backlog-store.md) — How a product backlog item lives as an append-only event record in the same stream as machine friction/grooming events, how its current state is derived by folding those events, how id generation stays collision-safe under concurrent writers, and why docs/backlog.md is a generated view no session ever hand-edits.
- [Decision Memory — what the system remembers about its own decisions](overview.md) — How a decision event is classified, reversed and reconciled against its citing artifacts, recalled through a derived index, kept bounded by an explicit archive, and honored by a backlog row's own done-flip rule — all one topic at this source's size.
