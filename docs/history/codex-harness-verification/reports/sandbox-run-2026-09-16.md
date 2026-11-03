# Codex harness verification, sandbox run (2026-09-16)

This run drives the real `bee` binary against a throwaway onboarded sandbox and
against the real Codex CLI. It follows the same method as the Pi acceptance run:
build, onboard a disposable repo, drive the user path, read back what changed,
and keep the evidence.

| Item | Value |
|---|---|
| bee | 2.39.0 (`/home/thanhsmind/.cache/cargo-target/release/bee`) |
| Codex | codex-cli 0.154.0, direct executable, not the mise shim |
| Sandbox | `/var/tmp/bee-verify/run/20260916-101950-2802202/repo` |
| CLI evidence | `/var/tmp/bee-verify/evidence/20260916-101950-2802202` |
| Canary evidence | `/var/tmp/bee-codex-canary-w5oBep/evidence` |
| Codex home | `/var/tmp/bee-codex-auth-1qn4fO` (the isolated home authorized on 2026-09-13; no new credential copy was made) |

`control-bee doctor` reported all seven rows `ok` before any driving started.

## Proven

1. **Onboarding writes the Codex wiring.** `onboard --apply --repo-hooks
   --runtime codex --json` returned `status: "applied"`, and the installed
   `.codex/hooks.json` is byte-identical to the repository source (`cmp` exit 0).
   Evidence `006`, `007`, `008`.
2. **Static attestation.** `doctor attest --runtime codex --json` returned
   `doctor-attest/1` with hash `aca26fc0e183a8…` and `codex-cli 0.154.0`.
   Evidence `009`.
3. **The doctor fails closed.** `doctor --runtime codex --json` exits 1. Every
   wiring row is `ok` and the attestation is valid; the one FAIL is
   `hook_handler`, because a freshly onboarded repo has no vendored
   `.bee/bin/bee`. That is the expected state for a fresh repo and the remedy is
   written in `packages/bee-rs/crates/bee/src/doctor.rs:484`. Evidence `011`.
4. **Installed hooks deny and allow the right writes.** The live canary
   (`scripts/codex-parity-canary.sh`) exited 0. Real Codex tool calls were
   denied for `apply_patch` and `Bash` writes to `.bee/backlog.jsonl` (guard
   status 2 each), and allowed for `docs/history/canary/patch.txt` and
   `shell.txt` (status 0 each). `backlog.jsonl` is byte-identical to the
   before-copy; both allowed files hold their markers on disk.
5. **Native spawn reached the guard and was denied.** This run observed
   `collaborationspawn_agent` in `observed_tools`, the installed model guard
   selected it, and no `SubagentStart` followed. The 2026-09-13 closeout could
   only record "Native spawn input was not observed"; this run closes that gap.
6. **Parent turn completion is recorded.** The `Stop` event carried a persisted
   transcript, and the sandbox `.bee/state.json` holds
   `waiting_on {kind: "turn-end"}` for that session.
7. **The read-only CLI fallback really is read-only.** `dispatch prepare
   --runtime codex --kind gather --role extraction --json` returned tool `Bash`
   with command `codex exec --sandbox read-only --ephemeral -`. Executed exactly
   as returned: `readable.txt` was read (`READ_OK_CPC2`), both write attempts
   failed with `Read-only file system` (status 1 each), the script exited 0, and
   the `target.txt` hash is unchanged. Evidence `027`.
8. **The model guard's role rule.** Raw payloads to `hook model-guard --runtime
   codex`: unmarked message denied (exit 2); a marker that is not at the start
   denied (exit 2); `generation`, `code`, `review` and `advisor` denied (exit 2,
   "current spawn schema carries no sandbox field"); `ceiling` allowed (exit 0).
   The denial of read-only roles is intentional and covered by the test
   `codex_spawn_read_only_roles_refused_as_unenforceable`.
9. **The canary's own guards hold.** `scripts/codex-parity-canary-test.sh`
   printed `ALL TESTS PASSED` over its seven isolation cases.
10. **The suite is green.** `cargo test --release --no-fail-fast
    --manifest-path packages/bee-rs/Cargo.toml` exited 0: 4021 passed, 0 failed,
    20 ignored, across 36 targets. The ignored tests are not passing evidence.

## Finding: the feature map gives an onboard command that installs nothing

`.bee/verify/verify-app/features/codex-runtime.md` (line 21, last modified
2026-09-13) tells the reader to run:

```
bee onboard --repo-root <target> --apply --runtime codex --json
```

Run exactly that, and no `.codex/hooks.json` is written. `doctor attest` then
answers "`.codex/hooks.json` is missing — there is no wiring to attest" and the
whole recipe stalls. Evidence `001` is that command with no hooks installed;
evidence `006` is the same command plus `--repo-hooks`, which installs them.
`bee onboard --help` states the flag's job: "Vendor the hook wiring into the
repository." The canary script already passes `--repo-hooks`; the map does not.

The correction is to add `--repo-hooks` to that line, then re-render the copies
with `bee onboard --apply`.

## Second run (same day): the two gaps, now closed

Both items the first run left open were driven in a second sandbox, on bee
2.40.0. Sandbox `/var/tmp/bee-verify/run/20260916-130522-3521384/repo`,
evidence `/var/tmp/bee-verify/evidence/20260916-130522-3521384`.

### Codex cell dispatch is refused as documented

The first run never reached this door: it stopped at `claim_ownership` because
the sandbox's execution gate was unapproved. This run drove the whole user
chain in a throwaway sandbox — `state start-feature`, `route --set`, a
`plan.md` carrying a load-bearing claims table, `gate --preview`,
`gate --merge --approved true`, `cells add`, `cells claim` — and then asked for
the cell dispatch. Evidence `018` through `022`.

`dispatch prepare --runtime codex --kind cell --cell ccd-1 --worker w1` answers:

```json
{"ok": false, "type": "refused", "reason": "native_hook_input_opaque",
 "slot": "extraction", "worker_registered": true}
```

with the fix line "Codex 0.154.0 hides the native role message from installed
hooks. Configure an explicit herding executor for this cell role and prepare
again." The refusal the feature map documents is real, and it refuses *after*
registering the worker rather than before.

One gate note: `gate --merge` first refused because the plan had no
`## Load-bearing claims` table, naming the exact required columns and stating
"There is no waiver flag." The approval only went through once the table held
rows whose evidence this run actually holds.

### The requested model does reach Codex

With `team.codex.extraction` unset (`null`), `dispatch prepare` returns
`codex exec --sandbox read-only --ephemeral -` with `requested_model: null` and
`effective_model_status: "unverified"` (evidence `009`).

Pinning `team.codex.extraction = {"model": "gpt-5.6-sol"}` changes all three
observable things (evidence `011`, `012`, `013`):

| | Before the pin | After the pin |
|---|---|---|
| Roster transport | (empty), model `null` | `native`, model `gpt-5.6-sol` |
| Prepared command | `codex exec --sandbox read-only --ephemeral -` | `… --ephemeral --model 'gpt-5.6-sol' -` |
| `effective_model_status` | `unverified` | `declared` |
| Codex `exec` banner | `model: gpt-6-astra` (Codex's own default) | `model: gpt-5.6-sol` |

The worker returned `PROBE_OK`, so the pinned model ran the work.

Read the status vocabulary precisely: it moves `unverified` → `declared`, never
to "verified". `effective_model` stays `null` throughout. bee records the model
it asked for and never claims to have read back what the provider served — the
Codex banner is the external evidence that the request took effect, and bee
does not pretend to own that proof.
