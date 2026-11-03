# crc-1 revision required

The worker process returned, but the cell remains claimed. Continue the SAME cell and reservations. Do not claim again.

## Reproduced installed failure

Command: BEE_BIN=/home/thanhsmind/.cache/cargo-target/release/bee BEE_CODEX_PROBE_BIN=/home/thanhsmind/.local/share/mise/installs/codex/latest/bin/codex CODEX_BIN=/home/thanhsmind/.local/share/mise/installs/codex/latest/bin/codex CANARY_CODEX_HOME=/var/tmp/bee-codex-auth-1qn4fO TMPDIR=/var/tmp bash scripts/codex-parity-canary.sh
Exit 1: Refusing symlink into real user settings inside CANARY_CODEX_HOME: /var/tmp/bee-codex-auth-1qn4fO/tmp/arg0/codex-arg0uTiqwr/apply_patch -> /home/thanhsmind/.local/share/mise/installs/codex/0.154.0/bin/codex
This is Codex's legitimate executable helper link, not a credential or configuration alias. Do not delete it or read credentials. Reproduce with a harmless fixture helper link to the selected executable, then distinguish this exact executable capability from prohibited settings links.

## Other verified source gaps

- Inherited BEE_CODEX_PROBE_BIN is never replaced or cleared. The plan explicitly requires direct probe configuration. A nested bee probe can invoke an inherited host wrapper even when PATH is filtered. Test and fix this.
- Direct CANARY_CODEX_HOME beneath HOME/.codex passes the equality check. The same location reached through a symlink is rejected. Reject actual sensitive descendants and custom inherited XDG roots consistently.
- CODEX_BIN accepts relative paths, then executes the original value after fixture commands change directory. Require and execute a canonical absolute path.
- test_inherited_path_wrapper and test_hostile_subprocess_isolation discard all exit status and do not assert that the fake process ran. Add positive markers and verify the expected isolated writes. A launcher that exits before any probe must fail these tests.
- isolation.md says subprocesses can write only inside the sandbox. Environment hygiene does not provide that OS restriction. Correct this statement and do not call all of F1 resolved: historical restoration remains unverified.
- Do not silently fall back from an explicitly supplied invalid BEE_BIN to a different binary. That can test stale code.

Keep a small implementation. Do not introduce dependencies or a generic security framework. Preserve existing raw logs and append revision proof. Reserve any additional evidence path first. Avoid read-scanning the entire installed executable as a wrapper detector where possible.
Run the hermetic tests with genuine pre-fix failures, then run them after repair. The leader owns authenticated live rerun. Finish the cell through the shared control-plane CLI with exact tests, mistakes, and deviations, or explicitly name the cap refusal. One final consolidated cell commit remains the integration requirement; do not rewrite sibling commits.
