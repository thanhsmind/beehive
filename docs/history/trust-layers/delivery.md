# trust-layers — delivery

Feature: onboarding offers a project's own static checks, a craft page turns a check row into a rule, user and review corrections are counted as reflections, and the prose load is reported. Decisions are cited as D-numbers from `CONTEXT.md`.

## Concepts changed

- `docs/knowledge/areas/onboarding/host-project-artifacts.md` — new section on the static-check offer and the rule that a non-empty list-valued `commands.test` counts as recorded (D1, D5); anchored to `onboard/notices.rs` and `onboard/templates.rs`.
- `docs/knowledge/areas/human-mailbox/overview.md` — reflection entry gains `--source` and its letter wording (D3, D6), anchored to `verbs/mailbox.rs` and `verbs/mailbox_digest.rs`.
- `docs/knowledge/areas/human-mailbox/overview.md` — the clean-run witness rule gains the `prose_load` witness (D4, D7), anchored to `verbs/feedback.rs`.
- `docs/knowledge/areas/human-mailbox/overview.md` — "A mistake seen twice" names the static-rules craft page and the leader doctrine for user and review sources (D2, D3).

## Known limits

- A repeat counts across distinct runs only, and only when the first four words of the what match.
- The prose-load line is printed by `bee feedback digest` and no host routine prints it yet.
- The installed `.bee/bin/bee` carries the feature only after a rebuild and reinstall.
