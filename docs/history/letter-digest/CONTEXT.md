# Letter Digest — Context

**Feature slug:** letter-digest
**Date:** 2026-08-30
**Shaping session:** complete
**Scope:** Standard
**Domain types:** READ | ORGANIZE

## Feature Boundary

After every `bee close` a close letter is filed in `.bee/human-mailbox/`;
the next session folds finished days and weeks into digest files in the
same directory, and the weekly fold auto-logs repeat errors as decisions
tagged `lesson`. It ends there: no email, no viewer, no scheduler.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 (b610a1dc) | The mailbox stays a directory of files. No email transport, no SMTP, no send command. Daily and weekly digests are markdown files filed in `.bee/human-mailbox/` beside the letters. | "Mail" here means a file the human reads in place, not a delivery system. Touches human-mailbox D14 (a6475e2c), which left the weekly digest open over an unchanged record shape. |
| D2 (aedb5be9) | Every `bee close` files its close letter at the moment of close, attended sessions included. Run-end letters keep the D9 rule (only an unattended run files); the feature-close letter no longer waits for run end or arming. | The entry data is already appended at close; filing is the missing step. Touches human-mailbox D9 (d970d6fc). |
| D3 (dbbe0778) | The daily and weekly digest is composed by the next session that starts after the period ended and finds the digest missing — the same recover-on-next-session pattern as dead-run letters. No scheduler, no cron. Inputs: the period's close letters and `.bee/usage/<feature>.json` records. | Touches human-mailbox D12 (05b5f964), which rejected schedulers; this reuses its pattern with zero new moving parts. |
| D4 (b343870b) | Lesson mining is automatic: when the weekly fold finds the same error shape in two or more letters, bee logs it as a decision tagged `lesson`, citing the letters as evidence. The human retires a wrong lesson by superseding it; no pre-approval step. | Evidence citation keeps the authorship ban intact: a lesson states only what the letters carry. |

### Agent's Discretion

- Digest file naming, frontmatter shape, and section layout — constrained
  to match the letter conventions already in `mailbox.rs` (readable
  subject, typed frontmatter, empty sections dropped).
- What counts as "the same error shape" for D4 — constrained to be
  deterministic and explainable in the logged decision's rationale.
- UTC day/ISO week boundaries and where the "already composed" marker
  lives — constrained to directory/file evidence only, per the D12
  detection pattern.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| mail | A markdown file in `.bee/human-mailbox/`. Never an email. |
| digest | One markdown file folding one finished period (one UTC day, or one ISO week) from that period's close letters and usage records. |
| lesson | A decision tagged `lesson`, auto-logged by the weekly fold, citing two or more letters showing the same error shape. |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/mailbox.rs` — the store, letter
  record, composing rules, `write_letter`; the digest reuses these
  conventions.
- `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs` —
  `record_feature_close_in_mailbox` (close entry append, ~line 3075) and
  `write_usage_record` (~line 2196); D2 hooks in here.
- `packages/bee-rs/crates/bee/src/verbs/work.rs` — session-start
  silent-run recovery; D3's due-digest check follows this pattern.
- `.bee/usage/<feature>.json` (`bee-usage/v1`) — written at close, read
  by nothing yet; the digest is its first reader.

### Established Patterns

- Recover-on-next-session, directory-evidence-only detection (human-mailbox D12) — reuse for due digests.
- Composing pass is a renderer, never a summarizer (human-mailbox D8) — the digest folds letter content; it never states a fact no letter carries.

### Integration Points

- `.bee/human-mailbox/` — letters and digests live side by side; the
  inbox listing and `bee mailbox mark` must keep working.

## Canonical References

- `docs/knowledge/areas/human-mailbox/overview.md` — letter record, authorship ban, filing rules.
- `docs/history/human-mailbox/CONTEXT.md` — the locked D1–D17 this feature touches.

## Outstanding Questions

<!-- bee:not-a-deferral: both questions were answered during execution — digests carry frontmatter type: digest (ld-2), and the miner sources SECTION_BROKEN bullets plus stored trouble-kind departures (ld-3); kept for the record, nothing is pending -->
### Deferred To Planning

- [x] Whether digests need their own frontmatter `type` so the inbox
  listing can tell a digest from a letter — read `mailbox.rs` listing
  code to answer.
- [x] Where the error shapes for D4 come from in the letter body
  (Broken/unfinished section vs departures) — read the entry fields to
  answer.
<!-- /bee:not-a-deferral -->

<!-- bee:not-a-deferral: these are rejected scope, not promises to act later — the user refused email delivery at shaping, and human-mailbox D1 bans a viewer -->
## Deferred Ideas

- Real email delivery (SMTP or send command) — the user rejected it: the
  mailbox is files, not a delivery system.
- A digest viewer or rendering — human-mailbox D1 bans anything above
  the data.
<!-- /bee:not-a-deferral -->

## Handoff Note

<!-- bee:not-a-deferral: template boilerplate naming the deferred-to-planning section, not a promise to act later -->
CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
<!-- /bee:not-a-deferral -->
