# Approach: Codex harness parity repair

## Recommended path

Fix the Codex spawn payload at one constructor. Widen activity only across shared Codex events.

Run the declared suite. Add fix-first cells for failures that remain. Finish with regeneration, binary installation, doctor, and a fresh Codex session.

## Rejected alternatives

- Keep the display subject in `task_name` — the live schema rejects it.
- Add Claude-only events — Codex cannot emit them.
- Claim parity from focused tests — the declared suite was red.

## Risk map

| Component | Risk | Control | Proof |
|---|---|---|---|
| Spawn payload | High | One Codex constructor | Runtime-kind matrix and live calls |
| Activity projection | High | One catalog and explicit capability | Projection and state tests |
| Full suite | High | Exact declared command | Exit zero without ignores |
| Release | High | Regen, install, doctor, attestation | Fresh Codex task |
