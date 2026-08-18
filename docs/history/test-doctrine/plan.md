---
artifact_contract: bee-plan/v1
mode: standard
---

# Plan: Agent-Owned Test Scope (test-doctrine)

Mode: `standard` — 2 risk flags: proof-weakening, covered-contract-change
Why this is the least workflow that protects the work: the feature
replaces bee's proof mechanism itself, so it needs a frozen plan and a
review wave, but no hard-gate flag applies — CI stays untouched (D4).

## Requirements (from CONTEXT.md)

- D1: agent owns test scope end to end, including the close/merge boundary.
- D2: docs-only diffs skip the suite; parity checks are their proof.
- D3: the session-start full-suite red check is dropped.
- D4: CI full suite stays exactly as is — the one deterministic net.
- D5: DoD = proof-per-change-type, taught as principle.
- D6: scoped-green-but-CI-red = fix-first cell + mandatory captured learning.
- D7: no boundary auto-run — close/merge require a recorded proof line.
- D8: cap report `tests` becomes the proof string `<command> — <result> — <scope reason>`.
- D9: one feature ships CLI + preamble + skill text + stale AGENTS.md lines with hash regen.

## Discovery

Full mechanism inventory pre-exists at
docs/discovery/test-doctrine/research/002-findings.md (gather-tier sweep,
anchors verified): cap vocab validator finish_support.rs:125-139, close
door close.rs:134/1657-1706, merge door phases.rs:681-732 +
handlers.rs:530, preamble line budget.rs:597-617 (byte-pinned by
session_preamble/tests.rs:145,157), ~16 skill refrains, stale
AGENTS.md:86-89 = AGENTS.block.md:79-82 (hash-pinned block).

## Approach

Two phases: mechanism first (CLI accepts and demands proof strings —
D7/D8), then doctrine text (every teaching surface says the same thing —
D2/D3/D5/D6/D9). Phase 1 is the walking skeleton: after it, a cap
carries real proof and both doors check proof instead of running tests.
Phase 2 cannot precede it (text would teach a contract the CLI refuses).
Rejected: text-first waves (rejected by D9 — text and machine must not
disagree mid-rollout); keeping one auto-run door (rejected by D7).
Door proof-check contract (answers CONTEXT's first open question the
lightest way, within CONTEXT's discretion): `--report` becomes REQUIRED
at `cells finish`, so every new cap carries a proof string. At the
doors, a cap with a present-but-empty or malformed proof string refuses,
naming the cell and the remedy (re-cap with a real proof line); a cap
with NO report record at all is a legacy cap — it passes with a named
note, never a refusal, so pre-contract features still close. The doors
add a new read of `trace.report` from the cells store (they do not read
it today — close reads capped cells for scribing debt only, merge reads
grants); the merge door resolves the store root the same way its
scribing-debt sibling does. The proof-string check lives in ONE shared
helper owned by the first door cell; the second door reuses it.
`bee test` remains the only writer of `.bee/logs/test-results.json`;
the D2 red-base claim door keeps reading it unchanged, and its refusal
remedy already names the refresh path (`bee test`) — the close-door
cell confirms this with a pinned test. The hand-maintained CLI help
rows (`generated/registry_payload.json`, per decision 3358743e) for
`cells finish`, `close`, and `worktree merge` change in the same cells
as their code — help prose is machine surface under D9. D6's
enforcement is skill text only (no CLI check) — the lightest honest
mechanism. No release ships between slice 1 and slice 2 (D9: text and
machine may not disagree in a shipped artifact).

Risk map: finish_support validator MEDIUM (contract change, worker-cell.md
must move in the same cell — proof: cargo test + c4_embedded_prompts_match_disk);
doors MEDIUM (refusal path replaces run path — proof: drivers/worktree
suites); text sweep LOW (regen chains are idempotent — proof: regen +
render-parity tests); AGENTS.block hash MEDIUM (managed-block hash must
be regenerated — proof: onboard parity test).

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 — mechanism | finish_support.rs proof-string vocab + worker-cell.md Result-form (D8); close.rs and phases.rs/handlers.rs doors check recorded proof, never run commands.test (D7) | Walking skeleton: the new contract must exist before any text teaches it | A cap records `cargo test -p bee — green — touched close.rs` and `bee close`/`bee worktree merge` accept it without running the suite | Phase 2 |
| 2 — doctrine text | Preamble red-check line removed (D3) + its byte-pin test; ~16 skill refrains rewritten to proof-per-change-type (D2, D5) incl. the D6 capture-on-miss rule; AGENTS.md + AGENTS.block.md stale lines + hash regen; skill-tree + manifest regen (D9) | Text follows machine in the same feature, per D9 | Preamble, skills, and AGENTS.md all state the new doctrine; regen parity green | Feature close |

Current slice: Phase 1 (3 cells). Phase 2 stays a headline until slice 1 caps.

## Test matrix

Triad at smallest demonstrating size, judged against existing suites first
(drivers/tests.rs, cells/tests.rs, worktree suites, session_preamble/tests.rs
already pin the current behavior — the writers repoint them, then add only):
- Happy: cap with a well-formed proof string accepted; door accepts a
  feature whose caps all carry proof.
- Edge: legacy `boundary`/`undeclared` values refused with the new remedy
  text; empty proof segment refused; a report-less legacy cap passes the
  door with a named note; no-test sentinel repo caps with command segment
  `none` and a reason naming the parity/docs proof used (existing sentinel
  tests migrate to this form); a proof string whose reason contains the
  ` — ` separator parses (split on the first two separators only).
- Error: door refusal names the proof-less cell; a `red` result segment
  refuses the cap (D6's spirit: a red is fix-first, never a done); refusal
  is a red line, never silent.

## Out of scope

- CI pipeline changes (D4).
- Per-test selection tooling (nextest filters, impact analysis).
- bee test verb and test-results.json (unchanged; confirmed in phase 1).
