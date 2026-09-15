# Pi hat wave

A Pi leader session inside a feature worktree runs the plan-step hat wave. It
prepares each seat's dispatch from the worktree and launches the seats detached
with `--inbox-session "$PI_SESSION_ID"`. The session gets one injected result
per seat, named by its `seat:` row, and the leader reads each full answer at
`report_path`. The leader then records the synthesis with
`bee state advisor-ref record`, and the wave stays inside its 10-minute budget.

## Sub-features

- `pi-hat-prepare-in-worktree` returns a `bee herding run` payload for each hat seat when run inside a granted feature worktree, with `--cwd` set to that worktree, `--seat "<seat>"`, and `--ceiling 600`.
- `pi-hat-detached-launch` launches every seat in the background with `--inbox-session "$PI_SESSION_ID"`.
- `pi-hat-seat-injection` injects one `bee-result` block per seat into the Pi session, with `job_id`, `seat`, `status`, `summary`, `proof`, and `report_path` rows.
- `pi-hat-full-answer` gives the leader the full hat answer in the file at `report_path`. The injection carries only a one-line summary.
- `pi-hat-advisor-record` records the synthesis from the bound Pi leader inside the worktree.

## How to get to it (user POV)

- In a Pi session opened in a feature worktree, ask for the plan-step hat wave. The leader follows `.agents/skills/bee-hive/references/gates-and-delegation.md` ("Hat wave", its `On Pi:` block).
- Per seat, the leader runs `.bee/bin/bee dispatch prepare --runtime pi --kind advisor --role <hat-seat> --purpose "<open question>" --json` and then the returned command, with `--inbox-session "$PI_SESSION_ID"` appended.
- The leader records the synthesis with `.bee/bin/bee state advisor-ref record --advisor "<name>" --digest-file <path>`.

## Driving it with control-bee

Preconditions:

- Build the candidate with `bash .bee/verify/verify-app/control-bee build`. Then run `bash .bee/verify/verify-app/control-bee launch` and `bash .bee/verify/verify-app/control-bee doctor`, and require all `ok`.
- Vendor the candidate into the sandbox store with `bash .bee/verify/verify-app/control-bee put .bee/bin/bee < "$(bash .bee/verify/verify-app/control-bee bin)"`, then `bash .bee/verify/verify-app/control-bee sh -- chmod +x .bee/bin/bee`. A worktree's `.bee/bin/bee` is a symlink to this file.
- The sandbox `.bee/config.json` needs a `team.pi` block that names the hat seats and a `herding.agents` entry for every agent it names, for example `pi-gpt-5.6-luna` = `["pi","-a","--model","openai-codex/gpt-5.6-luna:high"]`. Write the whole file with `control-bee put .bee/config.json`.
- Start a feature and its worktree with `bash .bee/verify/verify-app/control-bee cli -- state start-feature --feature hat-demo --mode standard --json` and `bash .bee/verify/verify-app/control-bee cli -- worktree new --feature hat-demo --json`.
- Put a plan only in the worktree: `VERIFY_CWD=repo--wt--hat-demo bash .bee/verify/verify-app/control-bee put docs/history/hat-demo/plan.md < plan.md`. A hat that runs in main cannot see this file.

- **Start the Pi leader in the worktree.** Use bee's own herding transport, not a herdr prompt from a Claude subagent (see Gotchas). Run `bee herding run --task-file - --json --cwd "<run>/repo--wt--hat-demo" --agent pi-gpt-5.6-luna` from the sandbox main checkout, with the leader task on stdin. Tell the task to: echo `PI_SESSION_ID`; read the Hat wave section; prepare the three default seats; run the returned commands detached with the note's flag appended; read each `report_path`; write a synthesis; run `advisor-ref record`; and log every command and its exact output to an evidence file.
  Observable: a Pi session log appears under `~/.pi/agent/sessions/--<run path with dashes>--wt--hat-demo--/`. Its file name ends in the same id that `echo $PI_SESSION_ID` prints.
- **Each prepare succeeds from the worktree.** Each `dispatch prepare --runtime pi --kind advisor --role <seat>` returns `"tool": "Bash"`, `transport_ready: true`, and a command that carries `--cwd "<run>/repo--wt--hat-demo"`, `--seat "<seat>"`, and `--ceiling 600`. It returns no granted-worktree refusal. The stdin opens with `Seat: <seat>.` and names the Hat wave row.
- **Each hat runs in the worktree.** `<run>/repo/.bee/mailbox/job-*/job.json` shows `cwd` = `<run>/repo--wt--hat-demo` for each seat.
- **Each result is injected with its seat row.** The Pi session log holds one drain block per seat in this shape:
  ```
  job_id: job-1789470560590-91086-1
  seat: hat-facts-gaps
  status: done
  summary: Completed the facts-and-gaps audit of the greeting-command plan.
  report_path: <run>/repo/.bee/mailbox/job-1789470560590-91086-1/report-1.md
  ```
  The markers live in `<run>/repo/.bee/result-inbox/<PI_SESSION_ID>/`. The drain removes them after injection, so an empty folder after the turn is expected. The folder name equals the bash `$PI_SESSION_ID`, and a block arrived for each seat. Together these prove that the bash token equals the drain token.
- **The leader reads each full answer.** The leader's next tool calls read each `report_path`. The injection header says the report body is not in the block.
- **The wave keeps its budget.** Compare the launch time with the newest `report-1.md` mtime. Budget: 10 minutes.
- **The advisor record succeeds from the bound leader.** `.bee/bin/bee state advisor-ref record --advisor "hat-wave:live-check" --digest-file <synthesis>` prints `Recorded advisor_ref (advisor "hat-wave:live-check", feature "hat-demo").`

### Evidence (run 20260915-180023-56873, 2026-09-15, bee 2.37.3 candidate, Pi 0.85.1)

- Leader step log: `/tmp/bee-verify/run/20260915-180023-56873/evidence/wave-log.md`. Synthesis: `/tmp/bee-verify/run/20260915-180023-56873/evidence/synthesis.md`.
- Leader Pi session log: `/home/thanhsmind/.pi/agent/sessions/--tmp-bee-verify-run-20260915-180023-56873-repo--wt--hat-demo--/2026-09-15T11-05-34-675Z_01a0a4be-5ed3-77fc-9a7f-928b7df1850a.jsonl`.
- Token: `echo "PI_SESSION_ID=$PI_SESSION_ID"` printed `PI_SESSION_ID=01a0a4be-5ed3-77fc-9a7f-928b7df1850a`. That is the session log id and the result-inbox folder `<run>/repo/.bee/result-inbox/01a0a4be-5ed3-77fc-9a7f-928b7df1850a/`.
- Prepares: all three seats (`hat-facts-gaps`, `hat-alternatives`, `hat-user-impact`) returned payloads from the worktree with `--seat "<seat>" --ceiling 600` and `--cwd` of the worktree.
- Hats ran in the worktree: each `job.json` in `job-1789470560590-9108{6,8,9}-1` has `cwd` `/tmp/bee-verify/run/20260915-180023-56873/repo--wt--hat-demo`.
- Injections in the session log carried `seat: hat-facts-gaps` (job `…91086-1`), `seat: hat-alternatives` (job `…91088-1`), and `seat: hat-user-impact` (job `…91089-1`), all `status: done`.
- Reports read: `/tmp/bee-verify/run/20260915-180023-56873/repo/.bee/mailbox/job-1789470560590-91086-1/report-1.md`, `…-91088-1/report-1.md`, and `…-91089-1/report-1.md`.
- Budget: launch 11:09:20Z, report completions at 11:11:32Z, 11:12:01Z, and 11:12:43Z. That is 3 min 23 s, and no seat was dropped.
- Advisor record, exact output:
  ```
  $ .bee/bin/bee state advisor-ref record --advisor "hat-wave:live-check" --digest-file /tmp/bee-verify/run/20260915-180023-56873/evidence/synthesis.md
  Recorded advisor_ref (advisor "hat-wave:live-check", feature "hat-demo").
  [bee] state advisor-ref record 1ms
  ```
- Bash default timeout: not observed. All 14 bash tool calls in the session log passed an explicit `timeout` (8 at 10 s, 6 at 30 s), so this run cannot show whether Pi applies a default.

## Gotchas

- **Never `wait` on the detached jobs.** The leader put `& wait` after the three backgrounded `herding run` commands. So the launcher shell waited on them, and the leader's own 30-second bash timeout ended it (`Command timed out after 30 seconds`). The hats still finished and were injected. The skill `On Pi:` block says "keep working — never wait in the foreground". It does not yet say "never `wait` on the detached jobs", and a leader can read `& wait` as allowed.
- **A native Claude subagent cannot drive this.** Under worktree isolation, a Claude subagent cannot type into a Pi pane in a `/tmp` sandbox. The isolation guard refuses `herdr agent prompt` with task text and `control-bee sh` scripts that aim outside the worktree. Start the leader with `bee herding run --cwd <sandbox worktree>`, or drive from a session rooted in the sandbox.
- The result-inbox folder is empty after the drain. Read the seat rows from the Pi session log, not from the marker folder.
- `result-1.json` in the mailbox carries no `seat` field. The seat rides the inbox marker, the `herding run --json` envelope, and the injected block.
- A worktree's `.bee/bin/bee` is a symlink into the main sandbox. Vendor the candidate into the main sandbox store; copying a binary into the worktree path writes through the link.
- Hats spend real model calls on the configured agents. A missing `herding.agents` entry for a `team.pi` agent refuses the run and lists the known agent names.
