# Remaining Codex validation

Reference record for the resumed completion request, 2026-09-13.

## Windows execution

GitHub Windows run [34709805168](https://github.com/thanhsmind/beehive/actions/runs/34709805168) tested commit `7abaee23a4ec0c61fb04f2a05bf79c6952d79485`. It ran on Windows Server 2025 and failed three activity tests. Each failure reported `no readable mailbox activity record for job-7` at `activity.rs:1159`.

The failing tests were `a_herded_pane_records_round_zero_before_the_first_brief`, `a_herded_pane_writes_the_job_mailbox_record_and_no_session_record`, and `the_state_machine_runs_unchanged_over_the_herded_sink`. The unit target reported 3463 passed, three failed, and 20 ignored tests. This result is not evidence for the current local commit.

The source still initializes the ambient job cache after the fixture sets its temporary job. Linux reads the original process environment from `/proc/self/environ`. The non-Linux fallback reads the changed environment. Cell `crc-5` repairs and tests that ordering. The actual Windows run of the repaired commit remains pending.

The configured `windows.yml` workflow runs the full Rust suite and a Windows PowerShell syntax check. This machine has no Windows runtime, Wine, or local Windows runner. The existing Windows runner can test the prepared branch after publication. No branch was pushed and no cloud run was started during this investigation.

## Native agent evidence

The earlier raw hook records remain readable. In `/var/tmp/bee-codex-canary-ZPuEAI/evidence/hook-inputs.jsonl`, line 25 records status 2 for `collaborationspawn_agent`. The preceding reproduction at `/var/tmp/bee-codex-canary-Cm0jce/evidence/hook-inputs.jsonl` records the spawn and child start and stop events at lines 24 through 34. Thus native guard denial and child lifecycle observation have retained live evidence. The newest canary did not repeat native spawning.

Successful native execution with verified role metadata is a different requirement. The installed Codex 0.154.0 hook hides the role message. The current dispatcher refuses this native execution path and the configured external workers execute through the supported route. Removing that refusal would remove role enforcement. No guard was removed or weakened.

[Official hooks documentation](https://learn.chatgpt.com/docs/hooks) describes tool-specific hook arguments. [Official subagent documentation](https://learn.chatgpt.com/docs/agent-configuration/subagents) describes configurable agent models. Neither page proves readable role metadata in this installed host. The installed observation remains the evidence for this limitation.

## User settings and temporary credential

The exact temporary credential `/var/tmp/bee-codex-auth-1qn4fO/auth.json` still exists with mode 0600. The resumed direct removal attempt was rejected by the physical-worktree guard. Its contents were not read. No alternate removal route was attempted.

The configuration backups exist, but their timestamps precede the probe by weeks. The mise backup has `gh = latest` and no Codex tool entry. The current file has `gh = latest` and `codex = latest`. These old backups do not establish the settings immediately before the probe. Restoring the entire file or deleting the Codex entry would risk removing a user change. The known disposable project trust entries are absent in the current Codex configuration.

The first GitHub lookup in this resumed turn accidentally used the PATH wrapper. Its output recorded a mise configuration write. The selected gh version remains `latest`, as in the backup, but the prior exact file bytes were not captured. The mistake was recorded immediately. Subsequent GitHub lookups used `/home/thanhsmind/.local/share/mise/installs/gh/2.100.0/gh_2.100.0_linux_amd64/bin/gh` directly.

## Capture

The Windows failure is now a specific repair, rather than a generic unverified platform statement. Native denial and child lifecycle have retained proof. Successful native role verification, current-commit Windows execution, credential cleanup, and historical setting restoration must not be described as complete without their respective evidence.

## Local repair result

Commit `2f5e951b` repairs the Windows activity fixture ordering defect.
`windows-red.log` reproduces the three failures through the non-Linux fallback.
`windows-green.log` records 55 passing activity tests after the repair.
`followup-suite.log` records 3,930 passed, zero failed, and 20 ignored across
36 targets. The full Rust test command exited with status zero.
This Linux proof does not replace Windows CI on the repaired commit.

## Judge correction

The first commit-scoped judge returned PASS but used an invalid wrapper schema.
The schema retry returned NEEDS_REVISION, claiming the fallback switch affects
production. Its narrow diff omitted the unchanged `#[cfg(test)]` attribute.
`git show 2f5e951b:packages/bee-rs/crates/bee/src/hooks/activity.rs` lines
664–665 show that attribute on `read_initial_herding_job_id`. The whole function
is excluded from production builds. The leader rejected this finding using
that exact source evidence; the raw retry verdict is in `windows-judge.json`.
The other four checks passed. Model independence remains unverified.
