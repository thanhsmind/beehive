# Pi harness dispatch

On Pi, a leader in an approved swarming phase on lane `small`, `standard` or
`high-risk` cannot write source. bee's write guard refuses the write, and the
stage tool set removes `edit` and `write` and adds `bee_dispatch` and
`bee_advisor`. The leader starts a worker with one `bee_dispatch` call. The
worker's result comes back to the session through the result drain.

## How to get to it (user POV)

- Open a Pi session in a feature worktree whose feature is in phase `swarming`,
  with the execution gate approved and a route lane of `small` or larger.
- Ask the leader to get an open cell done. Its tool list has `bee_dispatch`
  and no `edit` or `write`.
- Escape hatch: `/bee-tools-reopen` gives the leader all tools and lets it
  write until the stage changes. Config `pi_harness_workflow: false` turns
  the whole mechanism off.

## Driving it with control-bee

- `control-bee build`, `launch`, then vendor the candidate into the sandbox
  (`control-bee put .bee/bin/bee < "$(control-bee bin)"`, then chmod).
- In the sandbox: start a feature at `planning`, set route lane `small`, write
  `docs/history/<f>/plan.md` with a one-cell packet, `gate --preview`,
  `gate --name context`, `gate --merge`, `cells add`, set phase `swarming`,
  `worktree new`.
- Probe the hooks from the worktree: `bee hook stage-tools` returns
  `allowed_tools` without `edit`/`write` and with `bee_dispatch`; a
  `bee hook write-guard` Write with `"bee_runtime":"pi"` exits 2 with
  `bee Pi leader lock: writing "<file>" is refused`.
- Drive a Pi leader in RPC mode with cwd = the worktree and ask it to get the
  cell done. Drive from a session rooted in main or the sandbox; the
  worker-outward guard refuses `pi` from a feature-worktree session.

## Evidence (run 20261002-153242-3268921, 2026-10-02, Pi 1.0.0, leader deepseek/deepseek-flash)

Evidence dir: `/home/thanhsmind/.local/state/bee-verify/evidence/20261002-153242-3268921/`
(`rpc-events.jsonl`, `drive-summary.txt`).

- Hook probe: `allowed_tools` = read, bash, find, grep, ls, powershell,
  codemode, tool_search, bee_dispatch, bee_advisor. The write-guard probe was
  refused with the bee_dispatch hint (exit 2).
- The leader called `bee_dispatch {"kind":"cell","cell":"greet-1","worker":"greet-worker-1"}`
  and got `Worker started (job job-1790930000557-3270322-1)` at once.
- About 60 s later the drain delivered the worker result into the session.
  The worker wrote `hello.txt` (`hi`), committed `759201f` and capped the
  cell. The leader read the report and recorded a leader check.
- The leader wrote no source itself.

The first run (20261002-151421-3214679, before the p1u-5/p1u-6 fixes) found
three defects: the belt dropped the hook's notice sentences, narrowing removed
non-bee tools, and the leader set carried `verdict`. All three are fixed.

## Settle obligations (slice 3)

When a Pi leader tries to stop with bee work still owed, it gets one extra
turn. bee's `session-close` hook decides what is owed (`obligations_only`
payload) and returns each item once; the belt's `agent_before_settle`
handler shows the notice and asks Pi for one continuation. `/bee-obligation-skip
<key>` dismisses an item. The older agent_settled nudge does not fire for the
same settle.

Drive it: a sandbox feature with a claimed, uncapped cell (kind `cap`), or a
`high-risk` mode feature in `planning` with a gate preview and no advisor_ref
(kind `advisor`). Delete `.bee/runtime/settle-obligations.json` after any
probe of the hook, because a probe marks the item served.

### Evidence (2026-10-02, Pi 1.0.0, leader deepseek/deepseek-flash)

- **Cap, run 20261002-181622-3758076** (evidence
  `/home/thanhsmind/.local/state/bee-verify/evidence/20261002-181622-3758076/`, `drive-cap.txt`, `rpc-cap.jsonl`):
  after "ready" the notice `bee: work is still owed: cell demo-1 is claimed but
  not capped. One extra turn starts now to cap it.` fired; in the extra turn the
  leader found the work undone, called `bee_dispatch`, the worker wrote and
  capped the cell. A second prompt settled with no repeat.
- **Advisor, run 20261002-184508-3853826** (evidence
  `/home/thanhsmind/.local/state/bee-verify/evidence/20261002-184508-3853826/`, `drive-advisor.txt`, `rpc-advisor.jsonl`):
  the planning tool set was `read, bash, bee_advisor`; the notice named
  `5 runs, one bee_advisor call each: hat-facts-gaps, hat-risks, hat-value,
  hat-alternatives, hat-user-impact`; the leader called `bee_advisor` five
  times and read the reports. It did not approve the gate. A second prompt
  settled with no repeat.
- The first advisor run (20261002-182003-3768157, before the p1u-5 rework)
  had only `read, bash` in planning; the leader started the seats through
  bash and recorded the advisor_ref itself.

## Gotchas

- In the advisor run on the fixed build, the leader stopped before
  `bee state advisor-ref record`; the obligation fires once per plan, so it was
  not pushed again. Filed as a backlog finding.

- The small model still ran `dispatch prepare` through bash once before it
  called `bee_dispatch`. The tool call is what started the worker.
- A worker whose cwd is a feature worktree cannot record `verdict` ("No job
  mailbox directory found"); the drain still delivers the result. Filed as a
  backlog finding.
- A leader write was never attempted by the model in the live run; the refusal
  is proven by the hook probe above.
