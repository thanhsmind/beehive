# Codex Onboarding Guidance Repair (F7)

## Context and Goal

Workflow review finding `F7` (`docs/history/codex-parity-completion/reports/workflow-review-20260913.md:120-133`) identified that generated onboarding metadata (`.bee/onboarding.json.agents_sync.codex.note`) retained obsolete runtime claims asserting universal lack of per-agent model selection and all-null defaults by design. In practice, Codex runtime integration supports configurable roles and transports (`team.codex` / `models.codex`), read-only CLI execution fallback for non-cell gather/review/advisor dispatches, and capability-dependent native dispatch without effective-model proof.

Cell `crc-3` repairs `CODEX_AGENTS_NOTE` and its associated documentation comments, adds a behavioral regression test asserting expected supported statements and the absence of obsolete universal claims, captures red/green evidence logs, and proves fresh and refreshed onboarding through the compiled CLI.

## Note Changes

### Obsolete Baseline Note

```
Codex has no per-agent model selection (DEFAULT_MODELS.codex is all-null by design) - tiers are enforced as a read budget + output cap in the worker prompt instead. No agent files are rendered under .agents/ (AO11).
```

### Corrected Note (`packages/bee-rs/crates/bee/src/onboard/templates.rs:411`)

```
Codex defaults unconfigured roles to null (enforced as prompt read-and-output budget), but supports configurable roles and transports (team.codex / models.codex), read-only CLI fallback for non-cell execution, and capability-dependent native dispatch without effective-model proof. No agent files are rendered under .agents/ (AO11).
```

### Contract Elements
1. **Unconfigured Defaults Preserved**: Unconfigured roles default to `null` and fall back to prompt read-and-output budget.
2. **Configurable Transports**: Configurable roles and transports are recognized (`team.codex` / `models.codex`).
3. **Read-Only CLI Fallback**: Non-cell execution (gather/advisor/review) uses read-only CLI sandboxing (`codex exec --sandbox read-only --ephemeral`).
4. **Capability-Dependent Native Dispatch**: Native dispatch (`spawn_agent`) is capability-dependent and subject to model-guard validation.
5. **Effective-Model Disclaimers**: Disclaims independent proof of effective backend model execution.
6. **No Native Agent Files**: Preserves `No agent files are rendered under .agents/ (AO11)`.

## Behavioral Regression Test

Added `codex_onboarding_note_describes_configurable_transports_without_obsolete_claims` in `packages/bee-rs/crates/bee/src/onboard/agents.rs`:
- Verifies absence of `"Codex has no per-agent model selection"`.
- Verifies absence of `"DEFAULT_MODELS.codex is all-null by design"`.
- Asserts presence of configurable roles/transports guidance.
- Asserts presence of read-only CLI fallback explanation.
- Asserts presence of capability-dependent native dispatch statement.
- Asserts presence of disclaimer on lack of effective-model proof.
- Asserts presence of unconfigured null defaults and AO11 preserved behavior.

## Verification Evidence

### 1. Red Regression Failure (`docs/history/codex-reliability-closeout/onboarding-red.log`)

Ran before template edit against unedited `CODEX_AGENTS_NOTE`:
```
thread 'onboard::agents::tests::codex_onboarding_note_describes_configurable_transports_without_obsolete_claims' panicked at crates/bee/src/onboard/agents.rs:589:9:
obsolete claim: note must not state Codex has no per-agent model selection (got: Codex has no per-agent model selection (DEFAULT_MODELS.codex is all-null by design) - tiers are enforced as a read budget + output cap in the worker prompt instead. No agent files are rendered under .agents/ (AO11).)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured
```

### 2. Green Test Suite (`docs/history/codex-reliability-closeout/onboarding-green.log`)

Ran after template edit:
```
$ PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" TMPDIR=/var/tmp BEE_CODEX_PROBE_BIN=/bin/false cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee onboard
test result: ok. 186 passed; 0 failed; 1 ignored; 0 measured; 3378 filtered out; finished in 0.48s
```

### 3. Disposable CLI Onboarding Probes

Executed against disposable temporary git repositories using the built release binary `/home/thanhsmind/.cache/cargo-target/release/bee`:

- **Fresh Onboarding Probe**:
  - Initialized empty repository in `/var/tmp/bee-onboard-test-*/fresh-repo`.
  - Executed `bee onboard --repo-root <fresh-repo> --apply --runtime codex --no-statusline --json`.
  - Result: `.bee/onboarding.json` created with `agents_sync.codex.note` emitting the corrected note.
- **Refresh Existing Onboarding Probe**:
  - Seeded existing repository with `.bee/onboarding.json` containing the obsolete `F7` note.
  - Executed `bee onboard --repo-root <refresh-repo> --apply --runtime codex --no-statusline --json`.
  - Result: `.bee/onboarding.json` refreshed from the binary, updating `agents_sync.codex.note` to the corrected note without manual intervention.

## Boundaries and Ownership

- In compliance with cell constraints, repository-level `.bee/onboarding.json` was NOT hand-edited.
- The integration leader owns repository-wide dev regeneration at the wave barrier.
- No changes made to role selection, model guards, or credential handling.
