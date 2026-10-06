# Less leader toil and a cheaper Paseo wait — Context

**Feature slug:** herding-leader-toil
**Date:** 2026-10-06
**Shaping session:** complete (gate bypass full; the user asked to finish all seven items: "ok hoàn thiện tất cả các phần trên")
**Scope:** High-risk
**Domain types:** CALL | RUN

## Feature Boundary

The paseo-pi-hardening run showed where the leader's time goes: hand-built cap
reports for herded workers, control verbs refused inside the feature worktree,
judge verdicts copied by hand, and a Paseo wait loop that still polls. It also
showed three small Paseo gaps: workers load the user's whole Pi setup, the
desktop AppImage CLI passes doctor but cannot spawn, and workers are hard to
read in the Paseo app. This feature closes those seven. The leader still checks
every cell before it caps it; nothing here caps on a worker's word.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | `bee cells finish --id <cell> --from-job <job-id> --proof-result <green:unit\|green:static\|green:live> [--proof-reason <text>]` builds the cap report from the job's latest `result-N.json` and a fresh git read of the job's working directory: outcome = the result summary; files = the result's `files_changed`, or the git changed paths when that list is empty (the cap output says which); commit = the working directory's HEAD; tests = `<cell verify> — <proof-result> — <proof-reason, default the worker's proof text>`; deviations = the worker's dissent claim, else empty. `--proof-result` is required: it is the leader's own verdict after checking. It refuses when the job has no result, the result is not done, the job names another cell, or the working directory is gone (FIX: pass --report). Everything after building the report runs the existing cap path unchanged. `bee herding run`'s "without capping it" line names this command. | The leader re-typed data the run already holds. The leader keeps the check: the cap happens only when the leader states the proof result. |
| D2 | Control verbs run from inside a granted feature worktree serve the main checkout's control plane instead of refusing: every `state` verb, `gate`, `route`, `close`, and `cells` add, list, show, ready, update, schedule, escalate, reroute, judge, judge-record and dissent. The output names the main checkout as the store it wrote. `gate --preview` and every read of `docs/history/<feature>/` files keep reading the worktree's copy. Still refused inside a granted worktree: `dispatch prepare --claim`, `cells claim`, `claim-next`, `release`, `reopen` and `cap` (they move holds whose topology is checkout-specific). Narrows decision d7b83394. | The refusal only sent the leader to a wrapper script that ran the same verb from main; `dispatch prepare` already re-roots this way (`resolve_root_serving_granted`). |
| D3 | `bee cells judge-record --from-text <path\|->` reads a judge agent's free answer, extracts each fenced block whose info string is `json <cell-id>`, validates each as judge-verdict/1, and records each valid one on its cell. A duplicate cell id or a block whose id differs from a given `--id` refuses by name. Invalid blocks are reported by id with the validator's errors, and the verb exits non-zero when any block failed. The judge brief in the Judge tier section of `gates-and-delegation.md` asks for exactly that block form. | The leader copied six JSON blocks into files by hand. |
| D4 | The Paseo wait loop learns state changes from `paseo agent wait <id> --timeout <n> --json` run on a background thread with its own deadline (n + 10 s, never the 15 s CLI timeout), instead of an `inspect` every 3 s. idle maps to Idle, permission to Blocked, error to Dead, timeout to still working. A new wait is armed only after a timeout result and never sooner than 3 s after the last one; while a permission is pending the loop falls back to one `inspect` every 3 s. One `inspect` after spawn still records the model (paseo-pi-hardening D4). The 200 ms mailbox checks, the silent-idle nudge (paseo-pi-hardening D2) and the idle and ceiling timeouts are unchanged. The wait source is injectable for tests. | `paseo agent wait` blocks in the daemon and returns on the exact state change, so bee stops spawning a Node process every 3 s and reacts at once. |
| D5 | A Paseo agent block may set `"isolated_config": true` (provider `pi` only; default false). bee then keeps a per-agent Pi folder at `.bee/runtime/pi-agent/<agent>/` (symlinks to `~/.pi/agent/auth.json`, and to `models.json`, `models-store.json` and `npm` when they exist; `settings.json` with `defaultProjectTrust: "always"` and `quietStartup: true`; empty `skills/` and `extensions/`) and passes `PI_CODING_AGENT_DIR=<that folder>` to the worker. A real directory or file where a link belongs is never replaced; it is reported. | The worker loads the repo's bee-guard (project trust stays always) but not the user's own skills and extensions, so a cheap model gets a smaller context. Opt-in, because it changes what a worker loads. |
| D6 | The doctor `paseo_ready` row fails when the configured paseo command's `--version` first stdout line is not a bare version, or when the command resolves to a script that execs an AppImage, with FIX: set herding.paseo.command to the npm @getpaseo/cli paseo. For each Pi agent with `isolated_config`, it also fails when that folder's `settings.json` does not set `defaultProjectTrust` to `always` or its `auth.json` link is broken. | The live proof on 2026-10-06 found the AppImage CLI passing doctor and failing spawn. |
| D7 | A Paseo worker's title is `<cell-id> <agent>` when the run has a cell id, else `<agent> <job-id>`, and it carries the labels `bee_cell=<cell-id>` (when set) and `bee_agent=<agent>` beside `bee_job`. bee adds no parent flag: Paseo already records the leader as parent when `bee herding run` runs inside the leader's agent (paseo-pi-hardening D10). | Workers read by cell and role in the Paseo app. |

## Environment facts

- `paseo agent wait` (0.10.3): flags `--timeout`, `--json`; output `{agentId, status, message}`; status idle, timeout, permission or error; exit 0 for all of them; returns at once on an idle or permission-pending agent (refs/paseo `wait.ts`, `agent-manager.ts:2970-3115`).
- `paseo run --env` reaches the Pi process; Pi reads `PI_CODING_AGENT_DIR`; project trust in print and RPC mode comes from the agent folder's `defaultProjectTrust` (Pi 0.87.1 `security.md:57-80`).
- The AppImage CLI prints two log lines before `--version`'s `0.10.3`; the npm CLI prints only `0.10.3`.

## Specific Ideas And References

- Research digests 2026-10-06 (two advisor consults); seatworks `plugin/harness/pi/*` for D5.
- Rejected: auto-capping on the worker's result (the leader must check first); a `--parent` flag (Paseo has none and inherits it).
