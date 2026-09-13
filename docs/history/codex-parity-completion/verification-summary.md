# Codex parity verification

Document mode: reference.

## Scope and result

The four implementation cells are capped. Source acceptance ends at `c47f73be6eecdf2bd49e4903bdee9c579b0a8b54`.
Final checklist verification returned PASS for all four cells. Merge remains pending at this record's update.
The verdicts are retained in `reports/cpc-1-judge.json` through `reports/cpc-4-judge.json`.
Dispatch `da54825f-bc1f-4f68-8c81-4d057a64eca0` used the configured review role through a read-only CLI.
Model independence is unverified because the external builder and judge records declare models without proving effective models.
Commit consolidation preserved the exact tested tree. The backup branch is `backup/codex-parity-pre-consolidation-20260913`.

The full command `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` exited 0.
The run used `TMPDIR=/var/tmp`, the configured Cargo PATH, and `BEE_CODEX_PROBE_BIN=/bin/false`.
It passed 3,920 top-level tests, with zero failures and 20 ignored tests.
The nested subprocess test result is excluded from that total.
Evidence: `final-green-suite.log` in this directory.

`bee dev regen` completed all three steps with the built candidate binary.
The hook check found three matching projections. The release check found 376 matching files and one present unhashed artifact.
`git diff --check` exited 0.

## Live evidence

- Final-code canary: `/var/tmp/bee-codex-canary-VVDuwM/evidence`. Forbidden patch and shell writes returned 2. Approved writes returned 0. A persisted parent transcript produced the turn-end mark. Native spawn was skipped, not proved by this run.
- Earlier native guard evidence: `/var/tmp/bee-codex-canary-ZPuEAI/evidence`. The actual installed model guard denied `collaborationspawn_agent` with exit 2.
- Earlier child isolation evidence: `/var/tmp/bee-codex-canary-Cm0jce`. Child lifecycle events did not end the parent turn. This run preceded the native alias repair.
- Final guide execution: `/var/tmp/bee-verify/evidence/20260913-112318-3463334`, records 001 through 005. Prepared dispatch `a3600d96-c439-433d-90dc-4186da0994fd` ran the emitted command and stdin through the corrected `bash -c` helper. Both raw filesystem denials returned 1. The enclosing script returned 0. The target hash was unchanged and no continuation file existed.
- Earlier read-only patch proof: `cpc-2-recovery-green.log`. The separate shell proof does not substitute for this patch probe.
- Instruction test: main mailbox `job-1789272594951-3409518-1/report-1.md`. All three original scenarios passed. `CREATION-LOG.md` retains the original failures and the failed first rerun.

## Limits

Codex 0.154.0 sends opaque native spawn messages to hooks. Native cell preparation refuses that unsupported path. Configured external routes remain available.
Native requested model settings do not prove the effective model. Read-only CLI execution supplies the filesystem restriction.
Windows live execution, privacy and reservation canaries, and PreCompact live behavior are not proved here.
SessionEnd is an observer event in the canary, not a new production activity hook.

## Test isolation and cleanup

The user approved a private temporary sign-in copy. No credential contents appear in this report.
The original sign-in file was not changed by the copy operation.
An earlier PATH probe could reach the user's mise wrapper. Global configuration remained outside the approved change scope; untouched global configuration is not claimed.
Subsequent probes used the direct executable.
Removal of the temporary sign-in copy was refused by the physical-worktree guard. The copy remains at `/var/tmp/bee-codex-auth-1qn4fO/auth.json`, mode 600, pending a permitted cleanup path.

## Applied principles

Red-before-green retained the actual timeout, native alias, instruction, and recipe failures before their repairs.
Test-behavior-not-structure required raw tool errors and resulting bytes, not worker summaries.
Verify-before-reporting rejected the broken guide and the false native model-pinning claim.
Single-source-of-truth derived runtime copies from the owning skills and hook catalog.
One-fact-one-home updated existing runtime concepts. The approved scope was not reduced to hide host limitations.

Captured: runtime behavior and limits are recorded in the existing hook-runtime concepts and the feature decision log.
