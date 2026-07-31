# Provenance — bee-writing-skills body rules

The body states its rules bare (provenance exile, skill-token-diet D8, applied
here on wave-2 migration). This table maps each body rule to the decision(s)
that authorize it and the rationale in one line. Long-form records:
`docs/decisions/`, `.bee/decisions.jsonl` (via `bee.mjs decisions search`),
and `docs/history/<feature>/CONTEXT.md` for the feature that locked each ID.

| Body rule | Decision IDs / labels | Rationale |
|---|---|---|
| THE IRON LAW: no skill without a failing test first, applies to edits | TDD-for-skills methodology (Superpowers via khuym) | Untested instruction text ships the same class of bug as untested code |
| Regrowth law: learnings land in bundle/references by default; body edit only for a load-bearing invariant — a body line must change agent behavior, or it belongs in `references/` | budget-fence-removal (decision 8f63adb4) | "A size ceiling on instruction text is never a standing law in bee. A diet is a deliberate one-off optimization event that leaves no permanent gate behind." |
| Per-turn rules (chat shape, communication) are never exiled to references | comms-always-loaded CLOSE (decision 4ae4f40b) | A rule reachable only via an on-demand file is dead; home is decided by trigger frequency, not length |
| Provenance exile itself (this file's existence) | skill-token-diet D8 | Migrated bodies cite no decision IDs inline; the map lives one hop away |
