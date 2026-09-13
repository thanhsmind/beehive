# Dispatch recovery

Outcome: the source revision and Linux live proof are ready for leader acceptance. Commit: `a8f94c0f6f79b0f657db4230760ec4f012f3ac8c`.

`prepare.rs` now owns the probe child, caps stdout at 64 KiB, and reads without waiting for pipe closure. It kills the Unix process group and reaps the direct child. Windows uses `PeekNamedPipe` through an added feature of the existing `windows-sys` dependency. No new crate was added.

The installed-version check refuses unverifiable native cell dispatch on the observed Codex 0.154.0 version. Explicit herding and native-to-CLI fallback configurations retain their routes. Escalated cells keep their parent-model requirement and return a refusal. The guard explains why repeating an opaque message cannot repair that request. No claim of successful native model selection is made.

The final amendment corrects a stale role-display comment. It distinguishes configuration delivery from host enforcement and changes no behavior.

The changed source files are `packages/bee-rs/crates/bee/Cargo.toml`, `src/hooks/model_guard.rs`, `src/verbs/drivers/prepare.rs`, and `src/verbs/drivers/tests.rs` under that crate. The earlier cpc-2 implementation remains in ancestor commit `33695671`; this revision does not amend cpc-1.

Tests: `cpc-2-recovery-red.log` records the detached-pipe regression taking 3.004 seconds with a 100 ms deadline. `cpc-2-recovery-green.log:2469` records 382 driver tests passed and eight ignored. The run used `BEE_CODEX_PROBE_BIN=/bin/false` and `TMPDIR=/var/tmp`. The same log at line 1363 records 70 model-guard tests passed. The release build and `git diff --check` also passed. Probe fixtures use explicit executables; they do not change global PATH. POSIX-only cases are excluded on Windows. Windows execution was not tested here.

Live proof used the leader's isolated repository, `/var/tmp/bee-verify/run/20260913-102057-3257505/repo`, and the approved private Codex home. Actual prepare JSON, stdin, and execution output are retained in `cpc-2-recovery-green.log`. Each run executed the emitted `codex exec --sandbox read-only --ephemeral -` command with its emitted stdin unchanged. The default null slot correctly reports model selection as unverified.

- Line 2471 begins dispatch `5ee47fc0-a8d6-4569-a6e3-d6ec8ac1905d`. Its tool output proves read success and patch rejection at line 2539.
- Line 2557 preserves the refused second run. No tools ran; it supplies no sandbox proof.
- Line 2619 begins dispatch `da0ce37f-88c0-4720-95e9-10adb6b66d0e`. Lines 2687–2690 contain actual shell errors and both status codes, each `1`. The shell continued after the overwrite denial and its touch attempt also failed.

The target's before and after SHA-256 values are both `6f0941bfde4f59aa76531a9fdba13781a537c45bcb8bcf3ce5d1f785f9b17bc9`. No continuation file exists. No real configuration or credentials changed. A proposed external fixture patch was refused and wrote nothing; the leader then supplied the supported fixture.

Advisor consult: the leader reused the configured advisor after a thread-limit refusal. The advisor identified the blocking pipe reader and recommended bounded reads with direct-child control. Its effective model was inherited or unknown. Arbitrary detached-process containment and Windows live behavior remain outside this proof.

Departure: added the existing Windows Pipes feature — required for bounded pipe reads, authorized by decision `7741c278-e78b-41c8-938e-68087dada897` — something else had to be fixed first.

Mistakes were sent to the leader for immediate reflection: an evidence log preceded its reservation; two test fixtures used incorrect configuration or argument shapes; one live prompt required unavailable direct tools. The corrected tests assert actual fallback and escalation results. The final live prompt uses the observed tool interface.

Capture: the timeout, opaque-input limit, and proof limits are recorded here for cpc-4. Red-before-green selected the failing deadline test; behavioral testing required actual shell errors instead of the child model's final claims.

Next action: the leader checks these artifacts and records the cell cap.
