# Windows activity test isolation repair

The non-Linux environment fallback captured job-7 after the fixture installed that temporary identity. The ambient-job filter then suppressed the mailbox. Windows CI run 34709805168 showed three failures with this symptom.

The repair initializes ambient identity in HerdedEnv::set before environment mutation, under the existing shared lock. Production builds exclude this test setup.

The new regression starts fresh child test processes. A test-only switch selects the non-Linux fallback on Linux. One child runs all three failing mailbox cases without an inherited job. The other runs the existing inherited-job test with an inherited marker and job ID. The parent checks both test counts and process success. No tests or guards are disabled.

## Evidence

Before repair, the filtered non_linux_ambient_capture_preserves_herded_and_inherited_routing test exited 101. windows-red.log retains all three missing-mailbox failures and the parent regression failure.

After repair, cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee hooks::activity::tests exited 0. windows-green.log records 55 passed and zero failed tests. Both commands used direct Cargo PATH, TMPDIR=/var/tmp, and BEE_CODEX_PROBE_BIN=/bin/false. This is fallback-path evidence on Linux. The repaired commit still needs a Windows CI run.

## Execution correction

The worker identified the initialization change but added a nested-lock test hang. Its CLI stopped at a survey prompt. The leader cancelled that job and replaced shared-cache injection with isolated child processes. Decision aafa65ca records this method change. The leader reproduced the failure before the final fix.

Captured: cache the ambient identity before fixture mutation. The red-before-green principle required the missing-mailbox reproduction. Chesterton's fence preserved inherited-job suppression because it prevents tests from writing into actual worker mailboxes.
