# Codex Canary Process and Environment Isolation

## Overview

This document records the fix for finding F1 from the Codex workflow compliance review (`docs/history/codex-parity-completion/reports/workflow-review-20260913.md`).
The canary entrypoint (`scripts/codex-parity-canary.sh`) previously resolved `CODEX_BIN` via `mise which codex` and inherited the host environment, allowing subprocesses to resolve user PATH wrappers and mutate user-level configuration (e.g. global `mise` configuration and real-home project trust entries).

## Isolation Guarantees

The updated entrypoint enforces these boundaries:

1. **Explicit Direct Executable**:
   - `CODEX_BIN` is strictly required. No automatic discovery via `mise` or `command -v` is permitted.
   - `CODEX_BIN` must be an explicit file path (containing `/`). Bare command names resolved via `PATH` are rejected.
   - `CODEX_BIN` must not be a shell wrapper script or shim that invokes `mise`.
   - `CODEX_BIN` must point to an existing executable binary.

2. **Controlled PATH**:
   - The execution `PATH` is sanitized.
   - Any directory containing a `codex` executable that does not match `realpath "$CODEX_BIN"` (such as host shims or `~/.local/bin/codex`) is stripped from `PATH`.
   - Subprocesses cannot resolve host wrapper scripts through inherited `PATH`.

3. **Rejection of Unsafe Homes and Aliases Before Probe Execution**:
   - `CANARY_CODEX_HOME` is validated before creating directories or launching probes.
   - The real user home (`$HOME`), real `.codex` directory, and real XDG configuration/data/cache directories are rejected.
   - Real-home aliases and symlinks pointing to real user configuration or credentials (`auth.json`) are rejected before any subprocess can execute.
   - System root directories (`/`, `/root`, `/home`, `/etc`) are rejected.

4. **Hermetic Environment Isolation**:
   - Dedicated isolated directories under `CANARY_ROOT/home` are assigned to `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME`, and `XDG_CACHE_HOME`.
   - Host sentinels remain untouched during probe execution.
   - Subprocesses can write only into the isolated sandbox environment.

5. **No Credential Access**:
   - No credentials (`auth.json`) are copied, read, or deleted by the canary script.

## Behavioral Verification

Hermetic behavioral test suite `scripts/codex-parity-canary-test.sh` exercises all isolation boundaries:
- `test_missing_codex_bin`: Confirms missing `CODEX_BIN` refuses without invoking `mise`.
- `test_wrapper_codex_bin`: Confirms bare commands and wrapper scripts in `CODEX_BIN` are rejected.
- `test_inherited_path_wrapper`: Confirms host wrappers on inherited `PATH` cannot be resolved.
- `test_unsafe_homes`: Confirms real homes, real `.codex`, symlinks to real settings, and unsafe paths are rejected before launching probes.
- `test_hostile_subprocess_isolation`: Confirms a hostile probe attempting to mutate `$HOME` and `$XDG_*` writes only to isolated sandbox paths and leaves host sentinels untouched.
- `test_syntax`: Validates `scripts/codex-parity-canary.sh` syntax with `bash -n`.

## Retained Evidence

- Red log (pre-fix failures): `docs/history/codex-reliability-closeout/isolation-red.log`
  - Showed 5 test failures: missing `CODEX_BIN` invoked `mise`, wrapper script was executed, host wrapper was resolved on inherited `PATH`, probe executed before rejecting real home, and host home sentinel was corrupted.
- Green log (post-fix passes): `docs/history/codex-reliability-closeout/isolation-green.log`
  - Shows 6 passes and 0 failures across all hermetic isolation tests.
