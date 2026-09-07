# install-fetch-reason — plan

Lane: tiny. One file of behavior, one file of test.

## Shape

`scripts/install.sh` falls back to a multi-minute source build without saying
why. `scripts/install.ps1` already names the reason on both of its fallback
lines. Give the POSIX installer the same answer, and stop a single network blip
from reaching the fallback at all.

Three changes, all inside the `# ---------- published binary (preferred)`
block of `scripts/install.sh`:

1. `fetch()` captures the transport's own stderr into `FETCH_ERR` instead of
   discarding it. Nothing else about its contract changes: still curl-then-wget,
   still returns 1 when neither exists, still non-fatal to the caller.
2. Both fallback lines interpolate that captured reason, mirroring the wording
   `install.ps1` already ships:
   - `could not resolve a published release (<reason>) — building from source`
   - `no downloadable asset at <tag> (<reason>) — building from source`
   With no captured reason the parenthetical is omitted, so a genuinely absent
   asset still reads cleanly.
3. `curl` gains `--retry 3 --retry-connrefused --connect-timeout 10`; `wget`
   gains `--tries=3 --timeout=10`. A blip costs three seconds, not a build.

## Not in this slice

`install.ps1` retry — recorded in CONTEXT.md § Out of scope. Its 5.1 support
floor has no `-MaximumRetryCount`, and inventing a retry loop there is a
different change with a different risk.

## Proof

`bee-principle-red-before-green`: the assertion is written and watched fail
before the script changes.

1. Widen `installer_contracts.rs::install_ps1_names_why_the_published_binary_was_skipped`
   into `both_installers_name_why_the_published_binary_was_skipped`, adding the
   two `install.sh` assertions beside the two `install.ps1` ones it already
   makes. Run it — RED on the two new assertions, for the reason CONTEXT.md
   names.
2. Apply the three changes. Re-run — GREEN.
3. Run the whole `installer_contracts` + `installer_invocations` +
   `instruction_laws` set, which is the file's blast radius: `install.sh` is
   scanned for control bytes, sed captures, ASCII, tool preflight and binary
   naming by tests that a wording change can break.
4. Drive the real fallback once: run the block with `RELEASES` pointed at an
   unresolvable host and confirm the printed line carries curl's own message.

## Files

- `scripts/install.sh`
- `packages/bee-rs/crates/bee/tests/installer_contracts.rs`
