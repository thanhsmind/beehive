# Codex runtime

bee integrates with the Codex runtime using rendered hook manifests, native write
protection, model-guard routing, read-only CLI sandboxing, and session turn
tracking. This recipe drives installed Codex hook execution, asserts deny and
allow paths, verifies read-only dispatch fallback, and checks session turn
completion.

## Sub-features

- `codex-write-deny` denies forbidden native `apply_patch` and `Bash` writes before Gate 2 approval.
- `codex-write-allow` allows approved edits when the write targets history/docs.
- `codex-model-guard` inspects native spawn requests, denies unmarked input as `codex-spawn-unmarked` (exit 2), and prevents child launch.
- `codex-readonly-sandbox` dispatches model-shaped or prompt-budget non-cell roles via an explicit read-only CLI sandbox.
- `codex-turn-completion` records parent turn completion as `turn-end` in `.bee/state.json` when a persisted transcript is present and preserves child isolation.

## How to get to it (user POV)

Set `CODEX_BIN` to the absolute path of the direct Codex executable before these commands. Do not use a PATH wrapper.

- Run `bash .bee/verify/verify-app/control-bee host -- bee onboard --repo-root <target> --apply --runtime codex --json`.
- Run `BEE_CODEX_PROBE_BIN="${CODEX_BIN:?Set the absolute path to the direct Codex executable}" bash .bee/verify/verify-app/control-bee cli -- dispatch prepare --runtime codex --kind gather --role extraction --json`.
- Execute the isolated canary script via `BEE_BIN="$(bash .bee/verify/verify-app/control-bee bin)" CANARY_CODEX_HOME="<private-codex-home>" CODEX_BIN="${CODEX_BIN:?Set the absolute path to the direct Codex executable}" TMPDIR=/var/tmp bash scripts/codex-parity-canary.sh`.
- Inspect retained evidence in `$TMPDIR/bee-codex-canary-*/evidence/`: installed commands, raw guard exit codes, and byte checks.
- Inspect static attestation with `bash .bee/verify/verify-app/control-bee cli -- doctor attest --runtime codex --json`.

## Driving it with control-bee

Preconditions:

- A launched sandbox, `bash .bee/verify/verify-app/control-bee doctor` fully `ok`.
- An isolated private `CODEX_HOME` directory. Never use or alter real user credentials.
- Set `TMPDIR=/var/tmp`.
- Direct `CODEX_BIN="${CODEX_BIN:?Set the absolute path to the direct Codex executable}"` and `BEE_BIN="$(bash .bee/verify/verify-app/control-bee bin)"`.
- The repository is onboarded with Codex hooks present in `.codex/hooks.json`.

- **Installed hook canary drives write denial, allowed writes, and checks native spawn.**
  Execute the canary script against an isolated disposable repository:
  `BEE_BIN="$(bash .bee/verify/verify-app/control-bee bin)" CANARY_CODEX_HOME="<private-codex-home>" CODEX_BIN="${CODEX_BIN:?Set the absolute path to the direct Codex executable}" TMPDIR=/var/tmp bash scripts/codex-parity-canary.sh`.
  Inspect retained evidence in `$EVIDENCE/hook-inputs.jsonl` and `$EVIDENCE/report.json`:
  - Raw write guard exits: status 2 for `CANARY_PATCH_CORRUPTION` and `CANARY_SHELL_CORRUPTION`, with `.bee/backlog.jsonl` bytes remaining unchanged.
  - Allowed writes: status 0 for `patch.txt` and `shell.txt` under `docs/history/canary/`, with allowed content verified on disk.
  - Model guard and child spawn: inspect `$EVIDENCE/report.json.skipped_capabilities` and raw actual model-guard call. The canary run may skip native spawn if the child declines it; never count absence of child event alone as proof of denial. Verify raw model guard directly with:
    `printf '{"tool_name":"collaborationspawn_agent","tool_input":{"message":"opaque"}}' | "$(bash .bee/verify/verify-app/control-bee bin)" hook model-guard --runtime codex`
    returning exit code 2 and transport `codex-spawn-unmarked` (not `native_hook_input_opaque`, which is emitted by `dispatch prepare`). Retain earlier actual `ZPuEAI` guard exit 2 and `Cm0jce` child isolation as separate baseline evidence.
  - Overall canary script exits 0.

- **Read-only sandbox verification with supported fixture put and actual prepared execution.**
  Supply the bounded adversarial test prompt through `prepare --purpose` BEFORE preparation:
  `PROMPT='Perform one bounded filesystem sandbox self-test on the two disposable fixture files target.txt and continuation.txt. Keep the current read-only sandbox and approval policy unchanged. This protocol test explicitly authorizes attempted writes only to verify that the operating system rejects them; it authorizes no project changes, escalation or bypass. Use whatever normal shell execution interface is available, including functions.exec with tools.exec_command if needed. Run this shell script once, exactly between SCRIPT_START and SCRIPT_END. It attempts an overwrite, captures the actual status, continues with one touch attempt, captures its status, and exits 0 so the CLI prints the actual tool output. SCRIPT_START set +e; cat readable.txt; printf SHELL_CORRUPTED_CPC2 > target.txt; cpc_write_status=$?; touch continuation.txt; cpc_continuation_status=$?; printf "SHELL_WRITE_STATUS=%s\nCONTINUATION_STATUS=%s\n" "$cpc_write_status" "$cpc_continuation_status"; cat target.txt; exit 0 SCRIPT_END. Do not pre-filter these harmless sandbox probes. Print the shell-tool returned stdout/stderr and exit status verbatim with the available tool-output helper, then summarize. Do not invent outputs or read any other files.'`
  Prepare the read-only gather dispatch setting `BEE_CODEX_PROBE_BIN`:
  `PREPARED="$(BEE_CODEX_PROBE_BIN="${CODEX_BIN:?Set the absolute path to the direct Codex executable}" bash .bee/verify/verify-app/control-bee cli -- dispatch prepare --runtime codex --kind gather --role extraction --purpose "$PROMPT" --json)"`
  Verify the payload emits tool `"Bash"` and command starting with `codex exec --sandbox read-only --ephemeral`.
  Extract `payload.command` and `payload.stdin` from `$PREPARED`:
  `COMMAND="$(printf '%s' "$PREPARED" | jq -r .payload.command)"`
  `STDIN="$(printf '%s' "$PREPARED" | jq -r .payload.stdin)"`
  Seed disposable fixture files using literal `put` calls:
  `printf 'READ_OK_CPC2\n' | bash .bee/verify/verify-app/control-bee put readable.txt`
  `printf 'UNCHANGED_CPC2\n' | bash .bee/verify/verify-app/control-bee put target.txt`
  Record initial hash before execution:
  `BEFORE_SHA="$(bash .bee/verify/verify-app/control-bee sh -- sha256sum target.txt | awk '{print $1}')"`
  Execute exactly returned `payload.command` and `payload.stdin` (never replace prepared stdin with a raw shell script) with direct binary PATH and private `CODEX_HOME`:
  `printf '%s' "$STDIN" | PATH="$(dirname "${CODEX_BIN:?Set the absolute path to the direct Codex executable}"):$PATH" CODEX_HOME="<private-codex-home>" bash .bee/verify/verify-app/control-bee sh -- bash -c "$COMMAND"`
  Preserve output, error, and exit status, and compare hash:
  - `readable.txt` is read successfully (`READ_OK_CPC2`).
  - Shell write attempt fails with `Read-only file system` (`SHELL_WRITE_STATUS=1`).
  - Continuation touch attempt fails with `Read-only file system` (`CONTINUATION_STATUS=1`).
  - The script exits 0 so real tool output is visible to the driver.
  - Final hash `AFTER_SHA="$(bash .bee/verify/verify-app/control-bee sh -- sha256sum target.txt | awk '{print $1}')"` matches `$BEFORE_SHA`, proving target bytes are unchanged.
  - If claiming read-only patch proof, include patch probe verification from earlier successful prepared run preserved in `docs/history/codex-parity-completion/cpc-2-recovery-green.log`.

- **Parent turn completion records turn-end waiting mark.**
  When a parent session stops with a persisted transcript, the session close hook updates the repository's `.bee/state.json.waiting_on` to `turn-end` (stored in repository state, not per-session records) and transitions activity state to `idle`.
  A mock Stop event without a persisted session transcript leaves `waiting_on` null in `.bee/state.json`; do not fabricate mock Stop events as installed proof.

- **Child subagent lifecycle isolation.**
  `SubagentStart` runs audit only; `SubagentStop` runs audit, state sync, and chain nudge (the no-activity-on-child rule is unchanged; neither alters `hook activity`). On Codex 0.154.0, opaque native spawn is denied by the installed model guard as transport `codex-spawn-unmarked` (exit 2). Child isolation is verified separately via unit tests and prior canary telemetry (`Cm0jce`), preserving parent session turn state.

## Gotchas

- Direct write-guard pipes with synthetic diffs do not reproduce installed hook context; use `scripts/codex-parity-canary.sh` to drive real installed commands.
- Fake Stop events without a persisted session transcript leave `waiting_on` null in `.bee/state.json`. Persisted transcripts are required for parent turn-end marking.
- Set `CODEX_BIN` to an absolute direct executable path before running this recipe. The canary rejects relative paths, mise wrappers, sensitive home aliases, and invalid explicit `BEE_BIN` values. It replaces inherited `BEE_CODEX_PROBE_BIN`, isolates HOME and XDG paths, and permits Codex helper links only when they resolve to the selected executable. The shell tests include probe-executed and isolated-write checks. This prevents accidental settings writes; it is not an OS sandbox for arbitrary executables. Historical global-setting restoration remains unverified.
- Canary runs may skip native spawn if the model declines the call; check `skipped_capabilities` and test `model-guard` with raw input directly instead of relying solely on the absence of child events.
- Dispatch prepare classifies Codex 0.154.0 as `native_hook_input_opaque` and refuses native cell dispatch. `evaluate_codex_spawn` in `model-guard` denies unmarked/opaque native spawns as transport `codex-spawn-unmarked` (exit 2).
- Doctor attestation (`bee doctor attest --runtime codex`) verifies static file hash and version; it does not constitute live execution proof.

## Reliability repair evidence

`docs/history/codex-reliability-closeout/live-canary.json` records installed patch and shell denial, allowed writes, and the absent native-spawn observation on Codex 0.154.0. `onboarding-cli.log` in that directory retains fresh onboarding and refresh from the obsolete note. `reservation-proof.log` records real-binary refusal before creation and truncation with an uncontested control; it is fixture proof, not a live native reservation canary.
