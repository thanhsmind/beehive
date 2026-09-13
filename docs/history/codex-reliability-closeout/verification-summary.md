# Codex reliability repair verification

This report covers the seven findings in the Codex workflow audit and the gate preview parser repair.

## Changes and evidence

| Finding | Result | Evidence |
| --- | --- | --- |
| F1, user settings | The canary requires a direct executable, isolates HOME and XDG directories, and fixes the probe path. Historical settings restoration remains unverified. | `isolation.md`, `isolation-green.log`, `live-canary.json` |
| F2, regression history | Three tests fail on saved source and one passes. The duplicate-token assertion fails directly. A separate synthetic mutation proves Claude test sensitivity. Original test order cannot be recovered. | `regressions.md`, retained transplant patches and logs |
| F3, temporary credential | Open. The physical-worktree guard refused removal of the authorized temporary copy. No alternate deletion method was used. | `audit-addendum.md` |
| F4, whitespace | Removed extra EOF blank lines from the transcript tests. The explicit committed source range passes the whitespace check. | `git diff --check c4122d41 HEAD -- packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs` |
| F5, evidence pointer | Appended the corrected line reference to the review record. The original review artifact remains intact. | `audit-addendum.md`, review finding `F5-correction` |
| F6, reservation order | An actual CLI fixture rejects conflicting writes before redirection. Existing bytes remain unchanged and a new file stays absent. The allowed control writes successfully. | `reservation-proof.log` |
| F7, onboarding | Fresh and refreshed onboarding emit the corrected guidance. Repository metadata and generated skill copies were regenerated. | `onboarding-cli.log`, `.bee/onboarding.json` |
| Gate preview | The shared parser accepts the documented boolean flag. Seven CLI tests verify aliases and preserve false and true approvals. | `preview.md`, `crc-4-judge-final.json` |

The installed Codex 0.154.0 run passed patch and shell denial checks and both allowed-write controls. Its evidence is in `/var/tmp/bee-codex-canary-3oMgzM/evidence`. Native spawn was not observed. Environment isolation is not an operating-system sandbox. Effective backend model selection and live Windows behavior remain unverified.

## Integration checks

`bee dev regen` completed all three steps. `bee dev release-manifest --check` matched 376 files. After regeneration, `bee knowledge index --check` reported zero stale indexes.

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" TMPDIR=/var/tmp BEE_CODEX_PROBE_BIN=/bin/false cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` exited 0. `final-suite.log` records 3,929 passed, zero failed, and 20 ignored tests across 36 test targets. The ignored tests are not passing evidence.

All four final semantic verdicts are PASS. Their artifacts are `crc-1-judge-final.json` through `crc-4-judge-final.json`. Backend model independence remains unverified. The completion records also carry the exact related verification commands, each rerun successfully after command-only normalization.

Decision `a8464d1c` records two form corrections. Verification fields now contain runnable commands with result prose in proof reasons. Initial and repair commits remain intact because the audit evidence and completion records reference them. This is an explicit exception to the one-commit-per-cell form, not a change to approval or proof requirements.

## Capture

Updated the existing hook-runtime knowledge entry and Codex verification map. The red-before-green principle required honest historical evidence. The single-source-of-truth principle kept onboarding wording in its renderer and derived the generated records.
