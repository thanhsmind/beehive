# paseo-observe — advisor inventory (2026-10-06)

Advisor-tier read-only consult (fable). It found that no status, pane,
occupancy, interrupt, cancel or supervisor surface knows a Paseo job,
because `execute_paseo` writes `paseo_agent_id` and no `pane_id`, and no
wave-ledger row.

| Surface | Today on a Paseo job | Paseo equivalent | Lands in |
|---|---|---|---|
| `bee herding status` | `pane_id=null`; "working" even when dead or blocked | `paseo inspect --json` mapped by `parse_inspect`; blocked shows the permission tools | po-1 |
| Orphan sweep `mark_orphans` | skips the job | inspect Dead or missing from `paseo ls --label bee_job` marks interrupted; untracked agents reported only | po-1 |
| `interrupt`, `cancel` | refused `pane_missing` | `paseo stop`; cancel = mark + stop + `archive --force` | po-2 |
| Permission answer | none | `paseo permit allow|deny <agent> [req] [--all]` behind `bee herding permit` | po-2 |
| Occupancy, waves | not counted (no ledger row, herdr/tmux live set) | `record_dispatch` with the agent id; `paseo ls --label bee_job` in the live set | po-3 |
| Blocked worker in `run` | run ends, agent waits forever | keep polling while blocked, up to the idle timeout (D1) | po-3 |
| Timeout diagnosis | message says "pane kept"; no tail | `paseo logs <id> --tail 40` appended | po-3 |
| `pane read` | fails on an agent id | read a Paseo job or agent through `paseo logs --tail` | po-4 |
| Supervisor | cannot run status; blind | allowlist `bee herding status`; the broker tick files a blocked permission as a human decision of kind permission | po-4 |
| Pi workers widget | shows the worker, no state | out of scope | — |

Product questions it raised were answered by the user as D1 and D2.

Dismissed: a separate `bee herding job read` verb — `pane read` resolves a
Paseo job instead, so no second read verb is added.
