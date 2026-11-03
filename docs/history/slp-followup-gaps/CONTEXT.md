# slp-followup-gaps — locked context

Two gaps the SLP cluster named and deliberately carried out of its own
boundary, now closed together. They share an origin (the SLP work) and touch
no common file, so they run as one feature in two slices.

## Origin

- `p-221e6d0e` — "A worker session bound to no lane is refused at the commit
  intake gate because the gate reads the control-root default record."
  Reproduced live from `slp-supervisor-heartbeat` cell `sup-5`.
  Pattern record: `docs/knowledge/patterns/20260828-a-session-bound-to-no-lane-commits-against-the-default-record.md`.
- `p-05d2a4f4` — "A herding-lane worker cannot record a dissent, because
  herding workers are told never to run a bee command." Carried out of
  `slp-dissent-stop-and-ask` by decision `6a6b9975`, which names the shape the
  fix must take.

## D1 — The claim is the lane the guard was missing

`resolve_write_record` (`hooks/write_guard/checks.rs:45`) resolves the acting
record in three steps today: no session id, no session file, or no non-empty
`lane` field each fall straight back to the control-root default record
(`.bee/state.json`). A dispatched worker that was never bound is therefore
judged against a record about some other feature — at `idle` it loses every
source write and every commit.

The fix resolves the acting record from the session's OWN LIVE CLAIM before
that fallback: the claim names a cell, the cell names a feature, and the
feature's lane record is the record this session is actually working under.
This is not a guess. A claim is a fact the store already holds, written by
`bee cells claim` under this same session id.

Four conditions bound it, all of them narrowing:

1. It fires ONLY when the session has no `lane` binding. A bound session's
   answer is unchanged, including every one of its refusals.
2. It reads only claims whose owner is THIS session.
3. Claims naming two or more different features are ambiguous and resolve to
   nothing — the default record answers, exactly as today.
4. A claimed feature whose lane record is missing or corrupt resolves to
   nothing, and the default record answers. The bound-session path keeps its
   loud typed refusals; the derived path never invents one.

The derived record is therefore never MORE permissive than the lane the worker
was legitimately handed, and a session holding no claim sees byte-identical
behavior to today.

## D2 — The remedy is named where the refusal is read

The intake refusal (`hooks/write_guard/paths.rs:532`) tells the caller to route
the request through the workflow. For an unbound session holding no resolvable
claim that is the wrong remedy: the work IS routed, the session just is not
bound to it. When the acting record came from the default record AND the
session carries no lane binding, the refusal names the binding as the remedy.

## D3 — Dissent reaches a herding worker as DATA, never as a command

Decision `6a6b9975` fixed the shape: "the herding mailbox result grows dissent
fields and the control loop transcribes them through the one existing verb."
That is exactly what StopAndAsk already does (`options` / `leaning`), and the
dissent fields mirror it on the same three surfaces:

1. the brief's result schema (`herding/mailbox.rs` `render_brief`),
2. the parsed `MailboxResult` (optional at parse, membership never enforced),
3. the re-emitted JSON envelope (`herding/run.rs` `result_envelope`), present
   only when the worker filled it.

The brief still names no bee command. The worker writes JSON into the file it
already writes.

## D4 — One writer keeps the dissent record

The control loop transcribes a carried dissent through `record_dissent` —
the SAME function `bee cells dissent` calls — so the record, the closed
severity set, the secret scan, the blocker tooth and the claim release all
have exactly one implementation. The severity is passed through unvalidated by
the parser on purpose: the writer owns that check, and a second copy of a
closed set is the drift a boundary listed twice always earns.

## D5 — A failed transcription is loud, never silent

A dissent that does not land is a voice lost, so the outcome is reported, never
swallowed:

- transcription succeeded -> the envelope carries `dissent_recorded: true`;
- it failed, or there was no cell to record against -> `dissent_recorded: false`
  plus `dissent_error` naming the reason, and the raw `dissent` object still
  rides the envelope so the orchestrator can record it by hand.

The run's exit code is unchanged: a blocked result already exits non-zero.

## D6 — The brief's "never name dissent" pin is retargeted, not deleted

`mailbox.rs` pins that the rendered brief contains neither `bee cells` nor the
substring `dissent`. The second half pinned decision `6a6b9975`'s OLD boundary
— herding dissent was out of scope, so the word had no business in the brief.
This feature IS that boundary moving. The pin is retargeted red-first: the
brief must still never name a bee command (`bee cells`, `bee ` anywhere), and
the `dissent` field name in the result schema is now expected, not banned.

## Out of boundary

- No change to `bee cells dissent`'s own flags, record shape, or doors.
- No change to the intake gate's allowlists, its terminal-phase set, or the
  bound-session lane refusals.
- No auto-binding of sessions anywhere. `bee cells claim-next` still writes no
  `lane` field; the guard derives, it does not mutate.
