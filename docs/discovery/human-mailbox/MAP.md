# human-mailbox — discovery map

## Destination

Every long unattended run leaves one letter in `.bee/human-mailbox`: a
single data file whose subject says in one sentence what happened, and
whose body tells the human — in two minutes of reading, in their own
words — what got done, where the agent departed from the plan and why,
what broke, and what is waiting on them. Arrived also means a written
handover spec has crossed to waggledance so the inbox UI can be built
against a stable record shape.

Spawned: human-mailbox — docs/history/human-mailbox/CONTEXT.md (bee's
half: writing the letters). The inbox half was handed to the waggledance
session under D17 and proceeds there as its own work.

## Notes

Long runs (usually overnight, unattended/herding) currently leave the
human nothing readable: reconstructing the night means replaying a
session. The human wants the `ak:sumup` content model — what shipped,
what failed, what was worked around, what was decided, how to use it,
what remains — narrowed to a nightly letter, with special weight on
the moments the agent decided something the plan had not anticipated,
and the reason (an unforeseen blocker, or a better route that only
appeared during the work).

Material that already exists — this is mostly a rendering and
translation problem, not a new-capture one:

- Every cell cap already stores a Result form
  `{outcome, commit, files, tests, deviations}` onto the cell trace,
  validated key-for-key by `bee cells finish`.
- `.bee/decisions.jsonl`, the capture queue, handoff records, blockers
  and waiting-on marks are all machine-readable already.
- The one real gap: `deviations[]` is written agent-to-agent, terse,
  and does not require a reason — the reason is exactly what the human
  is asking for.

Cross-repo context (confirmed by tickets/008-waggledance-consumption.md):
waggledance already reads `.bee/` through a typed pure reader
(`crates/waggledance-core/src/bee.rs`), already parses markdown
frontmatter, and already runs a notification outbox with a live delivery
channel. The handover is one more typed reader on an existing
integration, not a new bridge.

## Decisions so far

- D1 (303488de): bee owns the mailbox data and the emitting code under
  its own project; the inbox UI is waggledance's, fed by a written
  handover spec — tickets/009-handover-spec.md
- D2 (e3618e3b): every record carries a required one-sentence,
  plain-language subject — tickets/005-record-format.md
- D3 (1b079912): one markdown file per letter, typed YAML frontmatter as
  the machine contract, body as the human prose — one artifact, no twin
  — tickets/005-record-format.md
- D4 (1d56c1d2): two layers — entries appended at every clean stop, the
  letter composed when the run ends —
  tickets/001-when-the-letter-is-written.md
- D5 (e9cb4c15): a departure is recorded in three required parts (what —
  why — which kind, from a closed set), and a cell that followed its plan
  says so explicitly — tickets/003-deviation-line-shape.md
- D6 (2009bc71): read/unread is a field bee owns in the letter file,
  flipped by waggledance through a bee command so bee stays the single
  writer — tickets/007-read-state-and-retention.md
- D7 (b94381e5): five nightly sections, empty ones dropped; architecture,
  behaviour and usage only in the feature-close letter —
  tickets/006-letter-sections.md
- D8 (1c7a9d87): prose written at the moment of each event; the
  end-of-run pass may reorder and drop, never invent —
  tickets/002-who-writes-the-prose.md
- D9 (d970d6fc): every session appends entries; only an unattended run
  files a letter — tickets/004-which-sessions-produce-a-letter.md
- D10 (1fb69f4b): the explicit no-departure statement is enforced only
  while the mailbox is armed, resolving the collision with dc6a2d26 —
  tickets/010-no-departure-vs-unchanged.md
- D11 (349f25d8): one letter per run, named
  `<UTC-timestamp>-<short-run-slug>.md` —
  tickets/011-naming-and-letter-granularity.md
- D12 (05b5f964): a run that died gets its letter from the next session
  that starts, marked as unfinished —
  tickets/012-letter-for-a-run-that-died.md
- D13 (c3ece144): Needs-your-call items are addressable by id; no
  answering path in this effort —
  tickets/013-does-the-inbox-act-or-only-read.md
- D14 (a6475e2c): the feature-close letter is in scope; no weekly digest
  — tickets/014-longer-horizon-letters.md
- D17 (1660158a): bee is a harness used to build waggledance, not half of
  it; bee authors the description and the handover crosses by messaging
  the waggledance session, which owns what enters its own backlog — a bee
  session never writes another project's backlog row —
  tickets/009-handover-spec.md (replaces D16 e255fe3a and D15 4c05dde2).
  Delivered: handover message accepted by session waggledance-e1.

## Not yet specified

- (empty — nothing left to decide; every ticket is closed)

## Out of scope

- Real email delivery (SMTP, external inbox) — the mailbox is a
  directory of files; delivery beyond that is a separate effort.
- A weekly digest folding several nights together (D14).
- An answering path from the inbox back to a waiting session (D13) —
  the record stays addressable so this can return as its own effort.
