---
artifact_contract: bee-plan/v1
mode: standard
---

# Plan: letter-digest

Mode: `standard` — 1 risk flag: covered-contract-change (D2 changes filing
behavior mailbox tests assert)
Why this is the least workflow that protects the work: three cells over one
crate, each independently testable, no new external surface — but the filing
contract change and the new digest composer both need their own tests, which
is more than a small lane carries.

## Requirements (from CONTEXT.md)

- D1 (b610a1dc): the mailbox stays a directory of files; digests are markdown
  files in `.bee/human-mailbox/` beside the letters — no email, no transport.
- D2 (aedb5be9): every `bee close` files its close letter at the moment of
  close, attended sessions included; run-end letters keep D9.
- D3 (dbbe0778): the next session that starts after a period ended and finds
  its digest missing composes it — no scheduler; inputs are the period's
  close letters and `.bee/usage/<feature>.json` records.
- D4 (b343870b): the weekly fold auto-logs a repeat error shape (2+ letters)
  as a decision tagged `lesson`, citing the letters; the human supersedes to
  retire.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = command executed. Evidence is a
verbatim byte substring of the anchored line(s).

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Close already appends its mailbox entry unconditionally; only letter FILING is gated, so D2 needs a filing call, not new data | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:3073 | UNCONDITIONAL, by D9: every session appends its entries, attended or not. |
| 2 | armed() is the only gate between stored entries and a filed letter | read | packages/bee-rs/crates/bee/src/verbs/mailbox.rs:1755 | return RunEnd::NotArmed; |
<!-- bee:not-a-deferral: "later" here describes run-end timing in the built behavior, not a promise to act later -->
| 3 | An existing letter is re-composed in place keeping filename and read state, so a close-time letter plus a later run end stays ONE letter (D11 safe) | read | packages/bee-rs/crates/bee/src/verbs/mailbox.rs:1770 | filed_at = old.filed_at; |
<!-- /bee:not-a-deferral -->
| 4 | Letter filenames are UTC-stamp-led, so a period's letters are listable from directory names alone (D3's bounded detection) | read | packages/bee-rs/crates/bee/src/verbs/mailbox.rs:177 | pub(crate) fn letter_filename(filed_at: &str, run: &str) -> String { |
| 5 | The session path that already runs mailbox recovery is `bee work set` in work.rs — the due-digest check rides beside it | read | packages/bee-rs/crates/bee/src/verbs/work.rs:300 | file_letter_at_run_end(&ctx.root, session_flag.as_deref(), status.as_deref()); |
| 6 | An internal decision-append exists for D4's lesson logging (no shell-out) | read | packages/bee-rs/crates/bee/src/verbs/cells/audit.rs:427 | pub(crate) fn log_decision(root: &Path, decision: &str, rationale: &str, tags: &[&str]) -> MR<()> { |
| 7 | Usage records carry a `closed_at` stamp, so the weekly fold can date them into a period | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:2185 | "closed_at": closed_at, |
| 8 | Usage records live under the control root at a fixed relative path | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:2227 | let rel = format!(".bee/usage/{feature}.json"); |
| 9 | The broken-work section heading is a stable constant the miner can parse by | read | packages/bee-rs/crates/bee/src/verbs/mailbox.rs:1522 | pub(crate) const SECTION_BROKEN: &str = "Broken or unfinished"; |

## Discovery

Inspected `mailbox.rs` (store paths, `file_run_letter`, `compose_letter_with`,
D12 recovery), `drivers/close.rs` (`record_feature_close_in_mailbox`,
`write_usage_record`), `work.rs` (run-end + recovery call sites),
`cells/audit.rs` (`log_decision`). Finding: everything D2–D4 needs already
exists as data; the work is one new filing call, one new composer module, one
new hook call, and one decision-append variant. Evidence: the claims table
above; code scan digest in `.bee/mailbox/job-1788098234789/report-1.md`.

## Approach

Recommended (cites D1–D4): extend the existing mailbox store in place.

1. Split `file_run_letter`'s compose-and-file core from its armed gate;
   add `file_close_letter(control, run)` that skips the gate; call it
   fail-open INSIDE `record_feature_close_in_mailbox`, right after
   `record_close_stop`, at the SAME control root it already resolved (D2 —
   advisor fix: a worktree root would file an orphan letter). Run-end
   re-compose (claim 3) keeps D11 — and `file_run_letter` re-composes an
   EXISTING letter regardless of arming (advisor P1: otherwise an attended
   run's letter freezes at close and misses the run's tail). D12 recovery
   also candidates a lettered run whose entries file is newer (mtime) than
   its letter — two stats, still zero opens — so a close-lettered run that
   <!-- bee:not-a-deferral: "later dies" describes the recovered failure case, not a promise to act later -->
   later dies still gets its unfinished mark.
   <!-- /bee:not-a-deferral -->
2. New sibling module `mailbox_digest.rs`: derive finished-and-undigested
   periods from directory names (claim 4); compose `digest-YYYY-MM-DD.md`
   (daily) and `digest-YYYY-Www.md` (weekly) into `.bee/human-mailbox/`,
   folding the period's letters via the existing letter reader plus
   `.bee/usage/*.json` records dated by `closed_at` (claims 7, 8). Renderer,
   never a summarizer: group letter subjects/sections and transcribe stored
   usage fields verbatim — no computed aggregates, no counts, no mood lines
   (mailbox D8). Digest frontmatter carries `type: digest`; letter-listing
   surfaces skip `digest-*` names (`list_letter_files`,
   `letter_run_slugs`) so digests never enter D12's lettered set or get
   folded as letters (advisor P1). A D12-recovered letter digests under its
   `filed_at` day; an already-closed period's digest is never reopened.
3. Hook `compose_due_digests` fail-open in `work.rs` beside — never inside —
   the recovery call and its armed() early-out (claim 5, D3): it fires on
   every `bee work set` and is idempotent by digest-file existence. The
   weekly fold mines repeat error shapes from the letters'
   "Broken or unfinished" bullets (claim 9) plus ONLY the
   obstacle/plan-was-wrong/fix-first departure kinds — never better-route
   lines, never plan-followed statements. Normalization: `one_line` +
   lowercase + digit-runs→`#` + strip trailing punctuation; a shape needs
   ≥4 words and 2+ DISTINCT run slugs. It logs ONE decision tagged `lesson`
   via a `log_decision` variant (claim 6), rationale citing the letter
   filenames and a stable `shape:<sha-12>` token; dedupe checks that token
   against ALL prior `lesson` decisions INCLUDING superseded ones, so a
   lesson the human retired is never re-logged (D4).

Rejected: a `bee digest` verb the human must run (D3 chose next-session);
email/send-command delivery (D1 rejects it); folding digests into the letter
record shape (a digest covers many runs — D11 maps one letter to one run).

Risk map: filing contract change in `close.rs`/`mailbox.rs` — MEDIUM, proof:
existing mailbox tests stay green plus new attended-close test. Digest
composer — LOW, pure fold over files, proof: unit tests on a temp store.
Lesson miner — MEDIUM (false-positive lessons), proof: normalization and
dedupe tests.

## Shape

One slice, three cells, in dependency order (1 → 2 → 3). All code lives in
`packages/bee-rs/crates/bee/src/verbs/` in the feature worktree.

- ld-1 (D2): close files its letter at close. Files: `mailbox.rs`,
  `drivers/close.rs`.
- ld-2 (D3): the digest composer module + period detection. Files:
  `mailbox_digest.rs` (new), `mod.rs` registration.
- ld-3 (D3+D4): the session-start hook + lesson mining. Files: `work.rs`,
  `mailbox_digest.rs`, `cells/audit.rs`.

## Test matrix

Triad per cell, writer judges existing coverage first:

- ld-1 happy: attended close → letter file exists with close sections. Edge:
  run end after close-time filing → still one file, `filed_at` and read
  state preserved, post-close entries present (unarmed re-compose); D12
  candidacy for a lettered run with newer entries mtime. Error: unreadable
  existing letter → `Failed`, close not refused (fail-open).
- ld-2 happy: two letters in a finished day → one daily digest folding both.
  Edge: period with no letters → no digest file; digest already present →
  untouched (idempotent); `digest-*` files invisible to `list_letter_files`
  and `letter_run_slugs`; body carries no counts or computed aggregates.
  Error: torn/unreadable letter → skipped with a warning, digest still
  composed from the readable rest.
- ld-3 happy: same normalized broken-shape in letters of 2 distinct runs →
  one `lesson` decision citing both filenames with its `shape:<sha-12>`
  token. Edge: shape already logged (active OR superseded) → not re-logged;
  1 occurrence, or <4 words, or better-route/plan-followed lines only →
  nothing logged. Error: decisions append fails → digest still filed,
  failure said (fail-open).

## Open Questions

(none)

## Out of scope

- Real email delivery (rejected by the user at shaping).
- Any digest viewer or rendering above the files (human-mailbox D1).
- Digesting supervisor observations (only close letters + usage records are
  in D3).
