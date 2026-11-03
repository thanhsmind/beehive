# Codex workflow audit addendum

This addendum supplements the frozen workflow review. It does not rewrite the previous judge or approve a merge.

## F5 — corrected evidence pointer

The installed write check in `docs/history/codex-parity-completion/reports/cpc-1-judge.json:8` points to summary line 22. That line is a whitespace check.
The correct anchor is `docs/history/codex-parity-completion/verification-summary.md:26`.
Retained raw evidence: `/var/tmp/bee-codex-canary-VVDuwM/evidence/hook-inputs.jsonl`, lines 13 and 16 return 2 for forbidden patch and shell writes; lines 19 and 26 return 0 for allowed writes.
The leader read these records during this repair. Review record `codex-workflow-20260913` now contains appended finding `F5-correction` through `bee reviews record`.

## F6 — reservation-before-output proof

The leader reserved this feature's evidence paths before output. Real CLI fixture run `/var/tmp/bee-crc-verify/evidence/20260913-140407-3937321` records reservations in 006–007 and the successful probe in 009.
`reservation-proof.log` preserves the exact command, stdout, stderr, and exit 0. A foreign reservation refused both overwrite and creation with exit 2. The existing file retained ORIGINAL, the absent file remained absent, and an uncontested control wrote CONTROL.
This is a real-binary fixture check through a guard-before-command boundary. It is not a new live Codex native-hook claim. Earlier setup refusals and exit 127 remain in the external evidence and session record.

## F3 — cleanup still blocked

The leader owns cleanup of `/var/tmp/bee-codex-auth-1qn4fO/auth.json` only. Metadata confirmed mode 0600. The exact rm operation was refused again by the physical-worktree guard during this repair. No alternate deletion mechanism was attempted. Credential contents were not read.
F3 remains open. No new credential copy was created for this feature.

## F1 — prevention and historical restoration are separate

The prior non-secret execution record identifies `mise use -g codex` in the PATH wrapper and ephemeral Codex trust probes. It does not preserve original global setting values.
The repair must not infer a rollback from a new isolated success. Only exact proven probe-created entries can be removed; unrelated settings remain outside scope.

## Remaining integration results

F1 prevention, F2, F4 and F7 receive their results from worker evidence and the final verification summary. Until then they remain pending.

Captured: corrected F5 pointer and verified F6 refusal behavior; F1 restoration uncertainty and F3 guard blocker remain explicit.

Historical trust check: a metadata-only selection of the three exact known probe project keys found zero matching entries in the current Codex config. This does not prove every prior global side effect was restored. No config values were changed.
