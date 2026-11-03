# cpc-1 completeness check: revision required

The first attempt stalled; its same-role retry wrote code and a canary script. This is the task-miss retry for the same cell. Preserve the existing work. Never amend a sibling commit: amend only if HEAD belongs to cpc-1; otherwise make a path-scoped fixup for leader integration. Do not touch Stuart's five source files.

The script inspected by the leader does not prove the approved outcome:
- WG_CMD is extracted from the installed manifest then never executed. Tests invoke bee hook write-guard directly, bypassing runtime matcher selection.
- It prints Observed Native Tool Names: exec apply_patch unconditionally.
- It fills absent observed events with SessionStart UserPromptSubmit PreToolUse PostToolUse Stop. Those are invented observations.
- A failed codex exec is only a warning, then All Canary Verifications Succeeded prints. Missing live proof must fail or be explicitly blocked, never success.
- The live prompt only echoes text. It never attempts a denied patch/shell write or an allowed write; allowed handler return alone proves no filesystem write.
- The temporary directory trap deletes all raw evidence. Evidence must remain outside the disposable repo and be named.
- The default binary path embeds /home/thanhsmind and can select a stale binary. Resolve the build target via the existing verification helper or cargo metadata, and build the candidate.
- Dropped suspicion: onboard flags --repo-hooks and --runtime are supported; bee onboard --help confirms both. Retain them. Check the typed onboarding result and every command exit rather than suppressing the output.
- Adding only apply_patch to a BOTH matcher without measuring shell/native tool names does not close all approved tool gaps. Preserve Claude and Windows paths.

Required implementation:
1. Use the real verified binary and an isolated onboarded repo, without changing real config or persisted trust. The per-invocation vetted hook-trust option is already approved.
2. Capture actual incoming hook event and tool names in a bounded recorder. Write ONLY event/tool names, input key names, IDs needed for attribution and controlled-canary results; no credentials or arbitrary transcript text. Test fixture .codex/hooks.json may add an observation hook while retaining production hook entries. Keep evidence outside the repo cleanup target.
3. Exercise native shell and native apply_patch deny AND allow paths through Codex, assert each required tool/hook was actually observed, and inspect real file bytes. If the agent never attempts a required tool, that case is not proven.
4. Compare baseline and fixed behavior; add regression tests for exact observed input shapes and installed matcher routing. Adapt main.rs/adapter/detectors only if the real shape requires it.
5. Preserve gates/privacy/reservations and multiple-target/rename checks. Existing generated hook manifest parity and onboarding activity fixes must stay tested.
6. Run the assigned verify command and the corrected live canary. Include exact exit codes and evidence path in the mailbox report. Do not claim complete or green:live while any truth is unproven.

The leader has not capped cpc-1. No source scope or authorization changed. This is required proof and correctness work under the existing approved cell.
