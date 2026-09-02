# Orient and status

These are the two "where am I" answers a user asks for. `bee orient` is the
routing answer — phase, gates, blockers, and the next skill to run. `bee status`
is the full store report — onboarding, gates with their approval records, cells,
lanes and debt. `bee version` is the one answer that needs no repo at all.

## Sub-features

- `orient-idle` reports a clean, unstarted repo and names the next skill.
- `orient-active` reflects a started feature, its phase and its gate states.
- `status-full` reports onboarding, drift, gate records and cell counts.
- `version-rootless` answers with no `.bee` store and no repo.

## How to get to it (user POV)

- Run `bee orient --json` at the start of a session.
- Run `bee status --json` for the full picture, or `bee status --brief`.
- Run `bee version`, `bee --version` or `bee -V` anywhere.

## Driving it with control-bee

Preconditions:

- A launched sandbox, `control-bee doctor` fully `ok`.
- No feature started yet (a freshly launched sandbox is in this state).

- **Version needs no repo.** Ask outside any bee store. Run
  `control-bee sh -- <binary> version --json`, using the path from
  `control-bee bin`. Exit code `0`, and the payload is
  `{"version": "<release>", "binary": "<crate>"}` where `version` equals the
  `version` field of `.claude-plugin/plugin.json`.
- **Orient on an idle repo.** Run `control-bee cli -- orient --json`. The payload
  reports `where.phase: "idle"`, `where.feature: null`, every entry of
  `where.gates` `false`, `work.cells` all zero, and
  `next.skill: "bee-hive"`.
- **Status on an idle repo.** Run `control-bee cli -- status --json`. It reports
  `onboarding.installed: true`, `onboarding.drift: false`, `phase: "idle"`,
  `feature: null`, and a `gate_records` object whose entries each carry
  `state: "pending"` with `actor: null`.
- **Orient reflects a started feature.** Start one — see
  [feature-gates](./feature-gates.md) — then run
  `control-bee cli -- orient --json` again. `where.feature` is the slug,
  `where.phase` is `"exploring"`, and `where.mode` is the mode passed.
- **Proof.** Run `control-bee snapshot orient`. `state.json` in the snapshot
  carries the same phase, feature and gate values the two verbs reported.

## Gotchas

- Both verbs print a `[bee] <verb> Nms` timing line to **stderr** on every call.
  It is not part of the payload. Parse stdout only.
- `bee status --json` is large. Assert on named keys; never diff the whole
  payload between runs, because counts and timestamps move.
- `bee doctor` is a different thing from a health check for this harness, and it
  requires `--runtime`. Use `control-bee doctor` instead.
- `orient` is the session-start ritual. Calling it repeatedly is harmless, but a
  recipe that calls it instead of the verb under test proves nothing.
- Version numbers come from two places. `version` is the release, read from
  `.claude-plugin/plugin.json`; `binary` is the crate version, pinned at `0.1.0`
  and meaningless. Asserting on `binary` will pass forever.
