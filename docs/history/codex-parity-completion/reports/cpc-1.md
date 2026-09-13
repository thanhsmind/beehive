# Installed Codex hook proof

Outcome: installed hook proof completed. The leader owns acceptance and cap.

The contract tests execute the selected manifest command. The source uses one renderer. The test script requires a private Codex home. User decision `5a5da73c-660c-49f7-9527-440028bc016d` approved the sign-in copy; the leader supplied it. No new credential copies or real configuration changes occurred during this resumed work.

Tests: `cpc-1-checks.log` records the failed installed-spawn regression, followed by 13 passing hook contracts, 7 manifest tests, and 23 onboarding hook tests. The failure was `manifest must select model-guard`. The new binary regenerated both Codex manifests; the Claude manifest stayed unchanged.

Live proof: `CANARY_CODEX_HOME=<approved private home> CODEX_BIN=<installed 0.154.0 executable> BEE_BIN=<new release binary> bash scripts/codex-parity-canary.sh` exited 0. Its retained records are:

- `/var/tmp/bee-codex-canary-ZPuEAI/evidence`: final proof. Both protected writes were denied. Backlog bytes stayed unchanged. Both history writes succeeded. Installed and source manifests matched. The model guard denied the opaque native spawn; no child started.
- `/var/tmp/bee-codex-canary-Cm0jce/evidence`: reproduction before the native-name fix. The child started because the model guard was not selected. Raw parent and child transcripts, session records, and lifecycle inputs remain here.
- `/var/tmp/bee-codex-canary-7T5XZB/evidence`: first private-home proof of patch and shell protection, with ephemeral transcript handling.

Observed tool names are `apply_patch`, `Bash`, and `collaborationspawn_agent`. Native spawn fields were `task_name`, `fork_turns`, and `message`. The message reached the hook as opaque host-wrapped bytes. Reports contain field metadata; only private raw evidence retains the opaque body. Nothing was decrypted.

The catalog now selects the exact observed native name. The shared adapter changes that name to `spawn_agent` and leaves input untouched. The existing model guard therefore denies a request whose role cannot be checked. This proves enforcement, not successful native dispatch with verifiable role metadata. cpc-2 owns that transport limitation.

Observed events include SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, Stop, and SessionEnd. The reproduction also observed SubagentStart and SubagentStop. Production SessionEnd routing remains unchanged by leader direction. The persisted reproduction set the parent's `state.json.waiting_on` to `turn-end`; its activity journal changed from working to idle only at Stop. Child completion did not add an idle transition.

Earlier failed activation evidence remains in `bee-codex-canary-7uXY7p`, `bee-codex-canary-87IuOK`, and `bee-codex-canary-xBwllE` under `/var/tmp`. Their causes were not inferred from the successful isolated run. The first advisor's empty-configuration theory was disproved by the second advisor's versioned-source trace.

Departure: use approved private authentication and deny opaque native dispatch — live inputs exposed a different name and inaccessible role metadata — hit an unforeseen obstacle.

Capture: exact native names, opaque-message refusal, and observed lifecycle policy are recorded here for cpc-4. Earlier reflections retain the failed-target, wrapper, trust, and executable-mode mistakes.

Next action: the leader checks the artifacts, records the cap, and owns private-home cleanup.
