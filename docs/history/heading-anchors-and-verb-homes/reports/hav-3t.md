# hav-3t: proof that the verb doctrine-home test goes red

**Cell:** hav-3t (test role, tester rule)
**Contract checked:** verb-doctrine-home (D2, D3), on hav-3 commit `649815cdd926084a62bcdec53d2185a075ff5c13`
**Files under test:** `packages/bee-rs/crates/bee/tests/verb_doctrine_home.rs`, `packages/bee-rs/crates/bee/tests/verb_doctrine_home_allowlist.txt`
**Verify:** `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test verb_doctrine_home`

I did not read the hav-3 cap report. Each mutation edited only the allowlist file. After each one I ran the verify, then restored the file with `git checkout HEAD -- <allowlist>`. `git status --short` was empty after every restore.

## Runs

| # | Mutation | Failure line | Result line |
|---|----------|--------------|-------------|
| 0 | none (baseline before mutating) | none | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s` |
| 1 | deleted the row `timings report :: …` | ``` `bee timings report` is named in no doctrine home and is not in the allowlist ``` | `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s` |
| 2 | emptied the reason: `perf log ::` | ``allowlist row `perf log` has an empty reason`` | `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s` |
| 3 | appended `cells frobnicate :: made-up command to prove the stale-row check` | ``allowlist row `cells frobnicate` names a command that no longer exists`` | `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s` |
| 4 | restored from HEAD (clean run) | none | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s` |

Each failure names the command that was mutated, and nothing else fails. Mutation 1 shows that `timings report` is named in no doctrine home, so its allowlist row is doing real work.

## Spot-check: five allowlist reasons against `bee <verb> --help`

| Command | Allowlist reason | What `--help` says | Holds? |
|---------|------------------|--------------------|--------|
| `reservations sweep` | TTL cleanup that the binary runs on its own, with release as the agent-facing verb | "Release every TTL-expired reservation that was never explicitly released." The help does not cover the "runs on its own" half, so I checked the code: `lease_store::sweep_expired_leases` is called from `verbs/status_full/orient.rs:331` and `integration_queue.rs:376`. | Yes |
| `intent show` | read by the PreCompact and resume hooks to render the anchor | "Show the current intent anchor, or the rendered PreCompact/resume block for it", with `--render precompact|resume`. | Yes |
| `decisions render` | renders docs/decisions/index.md, a generated view run by the binary and scripts | "Render docs/decisions/index.md from the active decision store … regenerated only, never hand-edited", with a `--check` drift mode. | Yes |
| `cells archive` | housekeeping that moves a fully terminal feature's cells out of the hot scan path | "Move a fully-terminal feature's cells … out of the hot .bee/cells/ scan path … the pressure valve for the hot read path". | Yes |
| `supervisor consent-sweep` | opt-in tick run by the supervisor loop, never an agent step | "Run the deterministic tick of silence-is-consent … OPT-IN AND OFF BY DEFAULT". | Yes |

## Observation (not a defect filed)

The home check is a plain substring match (`corpus.contains(cmd)`), so a two-word command counts as named whenever its text appears anywhere in the corpus. For example, `perf log` would match inside `perf logs`. This makes the test lenient, never wrongly red. I did not check whether any current command passes only through such an accidental match.
