# rust-port-11 — bee-write-guard port 2 of 3: Bash path

**Status:** [DONE]

**Outcome:** Ported the write-guard's Bash-command analysis to rust — `extractBashTargets` (redirects, fd-dup exclusion, quoted-segment merge, `sed -i`, bare `rm`, `git add/mv/rm` incl. bsg-1 blanket-staging broad writes), `checkGitBashCommand` with GIT SPAWN PARITY (git spawned only in the commit staged-resolution branch; git-bash fixtures run in git-initialized temp roots), the internals-reach guard, and CLI-shape check (d) against the rust-port-8 registry bridge (`validate-args.mjs` ported as `bee_core::validate_args`). Stale-registry semantics are two separate fixtures per the advisor note: rust-side stale cache skips check (d) with a `cli-shape-registry-stale` coverage-gap line (accepted rust-only allow window, asserted rust-side only); node-side registry import failure is its own separate skip proof. Conformance corpus `writeguard_bash`: 19 tests green, every deny fixture node-oracle exit-2 asserted then byte-diffed, deny-preservation red-first run recorded (14 red against the darkened pre-cell hook).

**Files:** `crates/bee-core/src/guards.rs`, `crates/bee-core/src/validate_args.rs`, `crates/bee-core/src/lib.rs`, `crates/queen-bee/src/hooks/write_guard.rs`, `crates/queen-bee/tests/writeguard_bash.rs`

Full trace and verification evidence: `.bee/cells/rust-port-11.json`.
