# Pi runtime

bee integrates with the Pi runtime using an extension file, fail-closed health inspection, write protection, and session turn tracking. This recipe drives Pi runtime health checks, verifies write denial and allow paths, inspects turn completion, and checks herding transport readiness.

## Sub-features

- `pi-doctor-health` grades installed Pi runtime health fail-closed across extension file presence, `.bee/bin/bee` handler, installed skill directories, compiled extension byte equality (`.pi/extensions/bee-guard.ts`), binary freshness, and configured herding transport readiness.
- `pi-doctor-refusal-classes` drives fail-closed refusal across absent artifacts, unreadable permissions, drifted extension bytes, malformed config, missing pane environment, and stale binary versions.
- `pi-doctor-attest-refusal` refuses attestation requests because Pi has no structurally unprovable trust rows.
- `pi-early-write-denial` denies unauthorized file edits and bash tool execution before Gate 2 approval.
- `pi-turn-tracking` tracks session turn completion and records idle and turn-end state without cross-session contamination.
- `pi-herding-transport` validates agent pane transport for Pi worker execution.
- `pi-model-usage-status` aggregates active-branch assistant token totals by provider and model and renders compact new and cached token counts in Pi's statusline.
- `pi-no-pane-dispatch` drives child-process worker execution directly without tmux panes for pi binary agents, returning full output through standard mailbox reports.
- `pi-worker-verdict` registers a terminating tool mirroring `MailboxResult` that ends worker execution with a structured result file (`result-N.json`) and returns `terminate: true`.
- `pi-in-flight-worker-widget` renders an informational widget below the editor listing active in-flight workers from `.bee/result-inbox/` with progress tick glyphs, updating on drain poll ticks and vanishing when all workers complete.

## How to get to it (user POV)

- Run `bash .bee/verify/verify-app/control-bee cli -- doctor --runtime pi --json`.
- Run `bash .bee/verify/verify-app/control-bee cli -- doctor attest --runtime pi --json`.
- Execute Pi integration contracts via `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts`.
- Execute Pi model usage status contract via `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts model_usage_status`.
- Dispatch native worker with `HERDR_ENV=1 HERDR_PANE_ID=1 bash .bee/verify/verify-app/control-bee cli -- dispatch prepare --runtime pi --kind advisor --role advisor --purpose "<purpose>" --json` and run the returned command with `--no-pane`.

## Driving it with control-bee

Preconditions:

- A launched sandbox, `bash .bee/verify/verify-app/control-bee doctor` fully `ok`.
- Set `TMPDIR=/var/tmp`.
- Direct `BEE_BIN="$(bash .bee/verify/verify-app/control-bee bin)"`.
- The repository is onboarded with Pi extension present at `.pi/extensions/bee-guard.ts`.

- **Doctor reports ready when all Pi checks pass (ready outcome).**
  Drive Pi doctor in a fully configured sandbox with required herding transport variables set:
  `HERDR_ENV=1 HERDR_PANE_ID=1 bash .bee/verify/verify-app/control-bee cli -- doctor --runtime pi --json`
  Assert the command output:
  - Exit code is 0.
  - Payload field `overall_status` is `"ready"`.
  - Six mechanical rows report `"ok"`: `hooks_file`, `hook_handler`, `skills_installed`, `wiring_matches_binary`, `binary_freshness`, and `herding_transport`.

- **Doctor reports not_ok and unknown on absent files (absent outcomes).**
  Drive each absent artifact refusal case:
  1. *Missing extension:*
     `bash .bee/verify/verify-app/control-bee sh -- mv .pi/extensions/bee-guard.ts .pi/extensions/bee-guard.ts.bak`
     `HERDR_ENV=1 HERDR_PANE_ID=1 bash .bee/verify/verify-app/control-bee cli -- doctor --runtime pi --json`
     Assert: exit code is 1, `overall_status` is `"blocked"`, `hooks_file` reports `status: "not_ok"`, and `wiring_matches_binary` reports `status: "not_ok"`.
     Restore: `bash .bee/verify/verify-app/control-bee sh -- mv .pi/extensions/bee-guard.ts.bak .pi/extensions/bee-guard.ts`
  2. *Missing skills directory:*
     `bash .bee/verify/verify-app/control-bee sh -- mv .agents/skills .agents/skills.bak`
     `HERDR_ENV=1 HERDR_PANE_ID=1 bash .bee/verify/verify-app/control-bee cli -- doctor --runtime pi --json`
     Assert: exit code is 1, `overall_status` is `"blocked"`, and `skills_installed` reports `status: "not_ok"`.
     Restore: `bash .bee/verify/verify-app/control-bee sh -- mv .agents/skills.bak .agents/skills`
  3. *Missing binary handler:*
     `bash .bee/verify/verify-app/control-bee sh -- mv .bee/bin/bee .bee/bin/bee.bak`
     `HERDR_ENV=1 HERDR_PANE_ID=1 bash .bee/verify/verify-app/control-bee cli -- doctor --runtime pi --json`
     Assert: exit code is 1, `overall_status` is `"blocked"`, `hook_handler` reports `status: "not_ok"`, and `binary_freshness` reports `status: "unknown"`.
     Restore: `bash .bee/verify/verify-app/control-bee sh -- mv .bee/bin/bee.bak .bee/bin/bee`
  4. *Missing plugin manifest:*
     `bash .bee/verify/verify-app/control-bee sh -- mv .claude-plugin/plugin.json .claude-plugin/plugin.json.bak`
     `HERDR_ENV=1 HERDR_PANE_ID=1 bash .bee/verify/verify-app/control-bee cli -- doctor --runtime pi --json`
     Assert: exit code is 1, `overall_status` is `"blocked"`, and `binary_freshness` reports `status: "unknown"`.
     Restore: `bash .bee/verify/verify-app/control-bee sh -- mv .claude-plugin/plugin.json.bak .claude-plugin/plugin.json`

- **Doctor reports unknown on unreadable files (unreadable outcomes).**
  Drive unreadable permission cases:
  1. *Unreadable extension:*
     `bash .bee/verify/verify-app/control-bee sh -- chmod 000 .pi/extensions/bee-guard.ts`
     `HERDR_ENV=1 HERDR_PANE_ID=1 bash .bee/verify/verify-app/control-bee cli -- doctor --runtime pi --json`
     Assert: exit code is 1, `overall_status` is `"blocked"`, `hooks_file` reports `status: "unknown"`, and `wiring_matches_binary` reports `status: "unknown"`.
     Restore: `bash .bee/verify/verify-app/control-bee sh -- chmod 644 .pi/extensions/bee-guard.ts`
  2. *Unreadable skills directory:*
     `bash .bee/verify/verify-app/control-bee sh -- chmod 000 .agents/skills`
     `HERDR_ENV=1 HERDR_PANE_ID=1 bash .bee/verify/verify-app/control-bee cli -- doctor --runtime pi --json`
     Assert: exit code is 1, `overall_status` is `"blocked"`, and `skills_installed` reports `status: "unknown"`.
     Restore: `bash .bee/verify/verify-app/control-bee sh -- chmod 755 .agents/skills`

- **Doctor reports not_ok on drifted extension bytes (drifted outcome).**
  Modify `.pi/extensions/bee-guard.ts` to drift from embedded extension bytes:
  `bash .bee/verify/verify-app/control-bee sh -- bash -c 'echo "// drifted line" >> .pi/extensions/bee-guard.ts'`
  `HERDR_ENV=1 HERDR_PANE_ID=1 bash .bee/verify/verify-app/control-bee cli -- doctor --runtime pi --json`
  Assert the command output:
  - Exit code is 1.
  - Payload field `overall_status` is `"blocked"`.
  - Row `hooks_file` reports `status: "ok"`.
  - Row `wiring_matches_binary` reports `status: "not_ok"` with detail stating `.pi/extensions/bee-guard.ts differs from what this bee embeds`.
  Restore: `bash .bee/verify/verify-app/control-bee sh -- git checkout -- .pi/extensions/bee-guard.ts`

- **Doctor reports unknown on invalid transport configuration (malformed-config outcome).**
  Set an invalid transport name in `.bee/config.json`:
  `bash .bee/verify/verify-app/control-bee sh -- python3 -c 'import json; p=".bee/config.json"; d=json.load(open(p)); d.setdefault("herding",{})["transport"]="invalid_transport"; json.dump(d,open(p,"w"))'`
  `HERDR_ENV=1 HERDR_PANE_ID=1 bash .bee/verify/verify-app/control-bee cli -- doctor --runtime pi --json`
  Assert the command output:
  - Exit code is 1.
  - Payload field `overall_status` is `"blocked"`.
  - Row `herding_transport` reports `status: "unknown"` with detail stating `herding.transport is "invalid_transport"`.
  Restore: `bash .bee/verify/verify-app/control-bee sh -- git checkout -- .bee/config.json`

- **Doctor reports not_ok on missing herding pane (missing-pane outcome).**
  Drive missing herding pane and environment variables:
  1. *Missing HERDR_ENV:*
     `env -u HERDR_ENV -u HERDR_PANE_ID -u TMUX bash .bee/verify/verify-app/control-bee cli -- doctor --runtime pi --json`
     Assert: exit code is 1, `overall_status` is `"blocked"`, and `herding_transport` reports `status: "not_ok"` with detail stating `HERDR_ENV is not set`.
  2. *HERDR_ENV set without HERDR_PANE_ID:*
     `env HERDR_ENV=1 -u HERDR_PANE_ID -u TMUX bash .bee/verify/verify-app/control-bee cli -- doctor --runtime pi --json`
     Assert: exit code is 1, `overall_status` is `"blocked"`, and `herding_transport` reports `status: "not_ok"` with detail stating `HERDR_PANE_ID is not set`.
  3. *Tmux transport configured without TMUX session:*
     `bash .bee/verify/verify-app/control-bee sh -- python3 -c 'import json; p=".bee/config.json"; d=json.load(open(p)); d.setdefault("herding",{})["transport"]="tmux"; json.dump(d,open(p,"w"))'`
     `env -u TMUX -u TMUX_PANE bash .bee/verify/verify-app/control-bee cli -- doctor --runtime pi --json`
     Assert: exit code is 1, `overall_status` is `"blocked"`, and `herding_transport` reports `status: "not_ok"`.
     Restore: `bash .bee/verify/verify-app/control-bee sh -- git checkout -- .bee/config.json`

- **Doctor reports not_ok on stale binary version (stale-binary outcome).**
  Simulate version mismatch by altering source release version in `.claude-plugin/plugin.json`:
  `bash .bee/verify/verify-app/control-bee sh -- python3 -c 'import json; p=".claude-plugin/plugin.json"; d=json.load(open(p)); d["version"]="9.9.9"; json.dump(d,open(p,"w"))'`
  `HERDR_ENV=1 HERDR_PANE_ID=1 bash .bee/verify/verify-app/control-bee cli -- doctor --runtime pi --json`
  Assert the command output:
  - Exit code is 1.
  - Payload field `overall_status` is `"blocked"`.
  - Row `binary_freshness` reports `status: "not_ok"` with detail indicating version mismatch (`9.9.9`).
  Restore: `bash .bee/verify/verify-app/control-bee sh -- git checkout -- .claude-plugin/plugin.json`

- **Doctor attest refuses attestation for Pi runtime.**
  Run doctor attest for runtime Pi:
  `bash .bee/verify/verify-app/control-bee cli -- doctor attest --runtime pi --json`
  Assert the command output:
  - Exit code is 1.
  - Output payload reports `refused` because Pi has no structurally unknown trust rows.
  - No `.bee/doctor-attest.json` file is written.

- **Automated contract suite verifies installed Pi lifecycle and write protection.**
  Execute the Pi integration test suite:
  `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts`
  Assert the suite output:
  - All contract tests pass with status `ok`.
  - Early write denial blocks unauthorized edits before Gate 2 approval.
  - Session activity and turn tracking complete cleanly.

- **Model usage status contract verifies active-branch token aggregation by provider and model.**
  Execute the Pi model usage status contract test:
  `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts model_usage_status`
  Assert the suite output:
  - The contract test passes with status `ok`.
  - Assistant message tokens on the active branch aggregate by provider and model.
  - Compact format displays thousands (`k`) and millions (`m`).
  - Turn completion refreshes totals and empty branch clears status.

- **No-pane dispatch runs worker as child process and returns report (native-dispatch outcome).**
  Configure `team.pi` with an advisor role mapped to an agent whose argv starts with `pi` in `herding.agents`.
  Run dispatch prepare to obtain the no-pane command:
  `HERDR_ENV=1 HERDR_PANE_ID=1 bash .bee/verify/verify-app/control-bee cli -- dispatch prepare --runtime pi --kind advisor --role advisor --purpose "test no-pane" --json`
  Assert the command output:
  - Exit code is 0.
  - Returned command carries `--no-pane`.
  - Field `transport_ready` is `true`.
  Drive execution through the returned command:
  `printf "Briefly summarize the purpose of this repository in under 20 words." | bash .bee/verify/verify-app/control-bee cli -- herding run --task-file - --json --agent "pi-gpt-5.6-luna" --no-pane --seat "advisor"`
  Assert the command output:
  - Exit code is 0.
  - Payload field `outcome` is `"done"`.
  - `pane_id` is `null` and `closed_pane` is `false`.
  - Full report file is written at `report_path` in `.bee/mailbox/<job-id>/report-1.md`.

## Gotchas

- Pi has no structurally unprovable trust rows. Do not expect attestation to succeed; `doctor attest --runtime pi` always refuses.
- If `.pi/extensions/bee-guard.ts` differs by even one byte from the compiled extension bytes, `wiring_matches_binary` fails. Re-install or refresh the extension to fix drift.
- Herding transport defaults to herdr. When `HERDR_ENV` or `HERDR_PANE_ID` is unset, doctor reports `herding_transport` as not ok and exits 1. When `herding.transport` is configured as tmux, `$TMUX` must be set.
- Every missing or unreadable required artifact returns `not_ok` or `unknown` and fails closed to `blocked` (exit 1).
- Direct write-guard pipes with synthetic diffs do not reproduce installed hook context; use `cargo test -p bee --test pi_plugin_contracts` to drive real installed commands.
- In-child workers execute with `BEE_HERDING_WORKER=1`. Under `packages/bee-rs/crates/bee/src/hooks/mod.rs`, that marker causes every hook except `activity` to exit 0 immediately, muting `write-guard` enforcement inside herded child subprocesses.
