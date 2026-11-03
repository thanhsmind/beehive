---
artifact_contract: bee-context/v1
feature: codex-harness-parity-repair
status: locked
locked_at: 2026-09-12
---

# Codex harness parity repair

## Outcome

Bee completes the same gated workflow on Codex and Claude. Each runtime keeps its supported event vocabulary.

## Decisions

### D1. Match workflow results, not raw event lists

Codex and Claude enforce the same write, gate, dispatch, proof, and release results. Unsupported host events remain explicit gaps.

Decision: `a4a9ab6a-4dca-4364-8d25-96bd78f704f0`.

### D2. Emit the live Codex spawn schema

Each Codex spawn payload uses `task_name`, `message`, and `fork_turns`. Optional model fields stay valid.

The task name matches `^[a-z0-9_]+$`. The complete assignment stays in `message`.

Decision: `f94e2341-3621-4f66-bcab-dfbf9e26490c`.

### D3. Require runnable release proof

The declared Rust command exits zero without weaker tests. The installed binary matches source.

Claude doctor is ready. Codex doctor is ready only with current trust attestation.

Decision: `5f4f9c7b-f917-40d4-b9af-5084c7f3cbf1`.

## Acceptance

- Every Codex worker kind starts from the exact prepared payload.
- Every task name contains only lowercase letters, digits, and underscores.
- Each worker message contains the complete purpose.
- Codex records activity for supported shared events.
- Unsupported Claude-only events remain declared gaps.
- Concurrent herding jobs keep unique ids.
- The declared Rust suite exits zero.
- The installed binary matches source.
- Claude doctor is ready.
- Codex doctor is ready after trust review and attestation.
- A fresh Codex task completes start, dispatch, guarded write, proof cap, stop, and resume.

## Constraints

- Keep gate bypass behavior.
- Keep independent review user-invoked.
- Do not weaken a guard or test.
- Do not invent a Codex host event.
- Do not read secret-shaped files.

## Out of scope

- Raw event-list equality.
- Pi behavior already merged under `pi-harness-workflow-parity`.
