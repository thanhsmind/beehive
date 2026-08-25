---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

Does a letter carry a lifecycle — unread / read / archived — and if so,
who owns it? Under D1 the state is data, so bee owns the field and
waggledance's inbox writes through to it. And what happens to old
letters: kept forever as history, or aged out like other bee stores?

➡️ Recommendation (revised after ticket 008): the letter file is
**immutable** once filed, and read/unread lives in waggledance's own
database, not in bee's store. Ticket 008 established that
`waggledance-core/src/bee.rs` is a deliberately *pure reader* of
`.bee/`; giving the inbox a write-back path would break that property
and put two processes in contention over the same files for a field
that is purely about one reader's relationship to the letter. D1 still
holds — bee owns the letter as data; "have I read it" is not part of
the letter. Keep letters indefinitely: they are the only
human-readable record of nights that are otherwise unrecoverable, and
they are small.

## Answer

Read/unread is a field bee owns **inside the letter file**, and the
waggledance inbox flips it by calling a bee command rather than writing
the file — logged as **D6 (2009bc71)**. The human chose this over the
agent's revised recommendation (state in waggledance's own database,
letters immutable); routing the flip through a bee command keeps bee the
single writer over `.bee/`, so the pure-reader property of
`waggledance-core/src/bee.rs` survives.

Retention: letters are kept indefinitely — uncontested, folded into D3's
record shape rather than logged separately.
