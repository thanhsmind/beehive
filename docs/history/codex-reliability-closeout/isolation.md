# Codex Canary Process and Environment Isolation

## Overview

This document records the process and environment isolation fix for finding F1 from the Codex workflow compliance review (`docs/history/codex-parity-completion/reports/workflow-review-20260913.md`).
The canary entrypoint (`scripts/codex-parity-canary.sh`) previously resolved `CODEX_BIN` via `mise which codex` and inherited the host environment, allowing subprocesses to resolve user PATH wrappers and mutate user-level configuration (e.g. global `mise` configuration and real-home project trust entries).

Note on F1 scope: This implementation resolves the prevention and isolation requirements of F1. Historical restoration of pre-existing user-level configuration deltas remains unverified because no retained before-image of host global settings exists (as documented in `CONTEXT.md` and `plan.md`).

## Isolation Guarantees

The updated entrypoint enforces these boundaries:

1. **Explicit Direct Executable and Binary Inspection**:
   - `CODEX_BIN` is strictly required. No automatic discovery via `mise` or `command -v` is permitted.
   - `CODEX_BIN` must be an absolute path (starting with `/`). Bare command names and relative paths resolved via `PATH` are rejected.
   - `CODEX_BIN` must point to an existing executable file. Its canonical absolute path is resolved via `realpath` and used for all invocations.
   - Shell wrapper scripts or shims that invoke `mise` are refused. Large compiled binary executables (such as ELF binaries) avoid full-file regex scanning by inspecting headers and bounding read sizes.
   - An explicitly supplied `BEE_BIN` is strictly validated; invalid explicit paths fail immediately rather than silently falling back to a different binary.

2. **Controlled PATH and Direct Probe Configuration**:
   - `BEE_CODEX_PROBE_BIN` is explicitly exported as `CODEX_REAL`, preventing nested `bee` probes from invoking inherited host wrappers.
   - The execution `PATH` is sanitized: directories containing non-canonical `codex` binaries or `mise/shims` are stripped from `PATH`.
   - Subprocesses cannot resolve host wrapper scripts through inherited `PATH` or environment probe variables.

3. **Rejection of Unsafe Homes and Sensitive Settings Aliases**:
   - `CANARY_CODEX_HOME` is validated before directory creation or probe launch.
   - The real user home (`$HOME`), real `.codex` directory, and real XDG settings directories (`$HOME/.config`, `$HOME/.local`, `$HOME/.cache`), along with custom inherited `XDG_*` roots, and any descendants thereof are rejected.
   - Real-home aliases and symlinks pointing to real user configuration or credentials (`auth.json`) are rejected before subprocess launch.
   - System root directories (`/`, `/root`, `/home`, `/etc`) and their sensitive descendants are rejected.
   - Legitimate Codex executable helper symlinks inside `CANARY_CODEX_HOME` that point to the canonical `CODEX_REAL` executable (such as `tmp/arg0/.../apply_patch`) are permitted capabilities, distinguished from prohibited user configuration or credential links.

4. **Environment Hygiene vs. OS Sandboxing**:
   - Dedicated isolated directories under `CANARY_ROOT/home` are assigned to `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME`, and `XDG_CACHE_HOME`.
   - Host sentinels remain untouched during probe execution.
   - Subprocesses conforming to standard environment conventions operate within the isolated sandbox paths.
   - Clarification: Environment hygiene directs standard tools and well-behaved subprocesses to isolated paths; it does not provide OS-level kernel filesystem sandboxing (e.g. landlock, namespaces, containers) against arbitrary hostile path writes.

5. **No Credential Access**:
   - No credentials (`auth.json`) are copied, read, or deleted by the canary script.

## Behavioral Verification

Hermetic behavioral test suite `scripts/codex-parity-canary-test.sh` exercises all isolation boundaries:
- `test_missing_codex_bin`: Confirms missing `CODEX_BIN` refuses without invoking `mise`.
- `test_wrapper_codex_bin`: Confirms bare commands, wrapper scripts, and relative paths in `CODEX_BIN` are rejected.
- `test_bee_bin_explicit_invalid`: Confirms explicitly supplied invalid `BEE_BIN` fails immediately without silent fallback.
- `test_inherited_path_wrapper`: Confirms host wrappers on inherited `PATH` or `BEE_CODEX_PROBE_BIN` are not resolved, with positive markers asserting probe launch.
- `test_unsafe_homes`: Confirms real homes, `.codex` descendants, custom inherited XDG roots, and credential symlinks are rejected before launching probes, while legitimate helper links to `CODEX_REAL` are permitted.
- `test_hostile_subprocess_isolation`: Confirms positive probe execution and expected isolated writes while host sentinels remain untouched.
- `test_syntax`: Validates `scripts/codex-parity-canary.sh` syntax with `bash -n`.

## Retained Evidence

- Red log (pre-fix failures): `docs/history/codex-reliability-closeout/isolation-red.log`
  - Preserves initial 5 test failures and appends revision failures (relative `CODEX_BIN`, invalid `BEE_BIN` fallback, inherited `BEE_CODEX_PROBE_BIN`, nested `.codex` descendant, and legitimate helper link refusal).
- Green log (post-fix passes): `docs/history/codex-reliability-closeout/isolation-green.log`
  - Preserves initial passes and appends revision run with 7 passes and 0 failures across all hermetic isolation tests.
