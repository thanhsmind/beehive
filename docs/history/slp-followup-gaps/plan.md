# slp-followup-gaps — plan

Lane: standard. Class: feature. Two slices, no file overlap between them.

## Slice 1 — the guard resolves the acting record from the session's claim

**sfg-1 — `resolve_write_record` falls back to the claimed cell's lane before
the default record** (role: code)

Files: `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs`,
`packages/bee-rs/crates/bee/src/hooks/write_guard/store.rs`,
`packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs`.

Today `resolve_write_record` (`checks.rs:45`) has three arms that fall straight
to the control-root default record. Two of them stay exactly as they are (no
session id, no session file). The third — a session record with no non-empty
`lane` — gains one step first: read this session's own live claims, map the
single claimed feature to `.bee/lanes/<feature>.json`, and use that record with
`source: "claim"`.

Every narrowing condition in CONTEXT D1 is a test, not a comment:

- a bound session's resolution is untouched, refusals included;
- no claim at all -> the default record, byte-identical to today;
- one claim, lane record present -> the lane record answers, and a `git commit`
  of a source path at default-record `idle` is ALLOWED when that lane is not
  terminal;
- one claim whose lane record is missing or corrupt -> the default record
  answers; no new refusal is invented;
- claims naming two different features -> the default record answers;
- a claim owned by a different session is never read.

Then D2: when the acting record resolved from the default AND the session
carries no lane binding, `intake_refusal` names the binding as the remedy.

Verify: `cargo test --release --manifest-path packages/bee-rs/Cargo.toml
write_guard`

## Slice 2 — dissent rides the herding mailbox and one writer records it

**sfg-2 — the mailbox result carries a dissent and the run verb transcribes it**
(role: code)

Files: `packages/bee-rs/crates/bee/src/herding/mailbox.rs`,
`packages/bee-rs/crates/bee/src/herding/run.rs`,
`packages/bee-rs/crates/bee/src/verbs/cells/dissent.rs`.

Three surfaces, mirroring `options` / `leaning` exactly (CONTEXT D3):

1. `render_brief`'s result-schema block gains a `"dissent"` object with
   `claim`, `alternative`, `severity`, plus one sentence saying when to fill
   it. The byte-exact brief pin moves with it. The `dissent`-substring ban is
   retargeted to a bee-command ban (CONTEXT D6), red-first.
2. `MailboxResult` gains `dissent: Option<MailboxDissent>`; the parse is
   lenient — absent, wrong-typed, or partial reads as absent, and the severity
   string is passed through unchecked (CONTEXT D4).
3. `result_envelope` re-emits `dissent` only when present, so a result carrying
   none keeps today's exact key set.

Then the transcription (CONTEXT D4/D5): after `read_result` yields a result
carrying a dissent, the run verb calls `record_dissent` — the same function
`bee cells dissent` routes to — against `opts.cell_id`, and stamps
`dissent_recorded` (and `dissent_error` on failure) into the envelope. No cell
id, or a refusal from the writer, is reported, never swallowed.

Verify: `cargo test --release --manifest-path packages/bee-rs/Cargo.toml
herding`

## Proof at cap

Each cell records its own scoped run. Before the merge, the declared suite:
`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release
--no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`
