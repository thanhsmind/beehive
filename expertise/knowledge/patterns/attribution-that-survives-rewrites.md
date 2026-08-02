# Attribution that survives rewrites

## Context

When an entry states a claim that came from somewhere else — a document, a
run, a person's answer — and that entry lives in a layer agents rewrite.
The attribution has to survive edits nobody reviews line by line: a source
inserted at the top of a list, two sources merged, a paragraph moved.

## Mechanism

Give every source a **stable key**, and attach claims to the key. Never
attach a claim to a position.

```markdown
---
sources:
  - id: retry-policy
    resource: https://wiki.example/ops/retry-policy
    title: Retry policy
  - id: incident-4417
    resource: reports/incident-4417.md
    title: Incident 4417 postmortem
---

The cap is enforced inside the loop, not by the caller.[^retry-policy]
Three callers were found bypassing it during the outage.[^incident-4417]

[^retry-policy]: Retry policy
[^incident-4417]: Incident 4417 postmortem
```

The label is the join key; a reader resolves attribution through the
matching entry, not by reading the footnote's prose. Prose in the footnote
is a courtesy for humans and carries no meaning for a consumer.

**Why positional references fail silently.** `sources[0]` is correct until
someone prepends an entry — and then it is wrong, points at a real source,
and reads as perfectly well-formed. Nothing errors. The claim now cites a
document that does not support it, and the only way to notice is to
re-check every claim by hand. A key that no longer resolves, by contrast,
is a detectable broken reference: a check can find it, and a reader sees
immediately that something moved.

```markdown
Bad — breaks silently the first time the list is reordered:
  The cap is enforced inside the loop (see sources[0]).

Bad — unfalsifiable; nothing can check it and nothing can follow it:
  The cap is enforced inside the loop (per the ops docs).

Good — resolvable, checkable, reorder-proof:
  The cap is enforced inside the loop.[^retry-policy]
```

## Notes

The same rule governs references between entries: link by a stable
identifier or a path that a check can resolve, never by "the third item
under that heading" or by a title that the next author will reword. When an
entry is superseded, leave its key resolvable rather than deleting the
file — every claim that cited it becomes unverifiable the moment the key
stops resolving, and the citing entries are exactly the ones nobody is
looking at.

Attribution also has a direction worth preserving: an entry citing a source
that is itself an entry in the layer creates a derivation edge. Follow it
and the signals attached to the upstream entry are available to a reader
judging the downstream one — which is why keys should point into the layer
where an internal source exists, rather than restating the upstream content
inline.
