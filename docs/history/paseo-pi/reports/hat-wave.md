# paseo-pi — plan-step hat wave synthesis

Date: 2026-10-05. Five seats, all returned inside the budget: hat-facts-gaps
(opus), hat-risks (fable), hat-alternatives (opus), hat-value
(agy-flash), hat-user-impact (agy-flash). The leader synthesized.

## What changed the plan

- **Shape (alternatives, value, facts-gaps).** No third `PaneTransport`.
  Paseo gets its own executor shaped like `execute_no_pane`
  (`herding/run.rs:3005`). Half the trait methods have no Paseo meaning.
- **Per-agent selection (facts-gaps E, D1).** The choice rides a `paseo`
  block on `herding.agents.<name>`, not a new `TransportKind`. That keeps
  the cockpit, the waves, `allowed_tools_for` and `job_verbs` untouched.
- **Fail closed (risks 1, 7; user-impact 1).** A failed inspect is Unknown
  and never alive. A version below 0.10.3 and a failed `paseo run` refuse
  with FIX lines.
- **Orphans and archive (risks 5, 6).** `paseo_agent_id` goes into job.json
  at once, every run carries `--label bee_job=<job id>`, archive runs only
  under `should_close_pane` and never on the caller's own `PASEO_AGENT_ID`.
- **The result source (risks 2; alternatives fact 2).** The result comes
  only from `result-N.json`; Paseo's finish is not read as done.
- **No interrupt (risks 3).** Slice 1 has no `paseo send` call at all.
- **Permission mode (facts-gaps G).** The block carries an optional `mode`.
- **Status values (facts-gaps A).** Probed live: `Status` is running, idle,
  error or closed; `PendingPermissions` marks blocked.

## Carried to later slices (Open Questions in plan.md)

- Slice 2: leader-only tick guard, a per-tick drain lock, the heartbeat
  turn exempt from the continuation nudge, a minimal and quiet empty turn
  (risks 8, 9; user-impact 2).
- Slice 3: the answer goes to the same idle agent by `paseo wait` then
  `paseo send`; Pi workers keep the `steer-N.json` drain; a Node WebSocket
  helper only for mid-turn steering of a non-Pi worker; the steer outcome is
  recorded before any fallback (alternatives; risks 3; facts-gaps D;
  value).
- An orphan sweep by the `bee_job` label (risks 5).

## Dismissed

- hat-alternatives "add `TransportKind::Paseo`" — dismissed: a repo-wide
  kind breaks the herdr-only callers facts-gaps E lists, and D1 asks for a
  per-team-config choice.
- hat-value "defer slice 3" — dismissed as a deferral, kept as a smaller
  slice 3: D5 is a locked decision, and the alternatives seat showed a
  shape without a Rust WebSocket client.
- hat-user-impact "show `PASEO_AGENT_ID` in the workers widget" —
  dismissed for slice 1: job.json and the run envelope carry the id; the
  widget is a cosmetic change no locked decision asks for.
- hat-user-impact "print 'Steering not supported; restarted as round
  N+1'" — dismissed: slice 3 sends to the same agent, so no restart
  message applies.
- hat-risks "re-create after delete with the same job id" — dismissed:
  job ids are unique per run (`job-<ms>-<pid>-<n>`), so a new run never
  reuses an archived agent's label.
