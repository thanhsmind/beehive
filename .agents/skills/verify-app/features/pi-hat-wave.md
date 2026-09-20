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
- `pi-hat-no-pane-wave` runs a five-seat parallel hat wave as child processes without tmux panes. It verifies seat naming, budget limits, and drop-and-name ceiling enforcement.

## How to get to it (user POV)

- In a Pi session opened in a feature worktree, ask for the plan-step hat wave. The leader follows `.agents/skills/bee-hive/references/gates-and-delegation.md` ("Hat wave", its `On Pi:` block).
- Per seat, the leader runs `.bee/bin/bee dispatch prepare --runtime pi --kind advisor --role <hat-seat> --purpose "<open question>" --json` and then the returned command, with `--inbox-session "$PI_SESSION_ID"` appended.
- Dispatch no-pane hat seats with `.bee/bin/bee dispatch prepare --runtime pi --kind advisor --role <seat> --json`.
- Execute the returned commands with `--no-pane` in parallel.
- Verify each result envelope shows `seat: "<seat>"` and `pane_id: null`.
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
- **Each detached launch returns at once.** Each backgrounded `herding run ... --inbox-session "$PI_SESSION_ID"` prints one JSON line with `outcome detached` (`"outcome":"detached"`) and the job id. The launch shell exits with no timeout.
- **Hat panes stay in the worker column and close.** Capture the herdr pane layout (a `cli:pane:layout` result) every 15 s from before the leader starts until after it ends. While the hats run, each hat pane has the leader's `x` and sits below the leader pane, and the main pane keeps its `x`, width, and height. A hat pane is gone from the first capture after its `result-1.json` mtime. The last capture equals the first one.
- **Five-seat hat wave runs as child processes without tmux panes (no-pane-wave outcome).**
  Count open panes before launch:
  `bash .bee/verify/verify-app/control-bee cli -- herding pane list`
  Assert baseline pane count (for example, 27).
  Dispatch five hat seats in parallel with `--no-pane`, `--ceiling 600`, and their seat names:
  1. `hat-facts-gaps`
  2. `hat-alternatives`
  3. `hat-user-impact`
  4. `hat-risks`
  5. `hat-value`
  `printf "<prompt>" | bash .bee/verify/verify-app/control-bee cli -- herding run --task-file - --json --agent "pi-gpt-5.6-luna" --no-pane --seat "<seat>" --ceiling 600`
  Count open panes during parallel execution. Assert the pane count equals the baseline count.
  Assert results after all seats stop:
  - Each command exits with code 0.
  - Each envelope shows `outcome: "done"`.
  - Each envelope shows `pane_id: null` and `closed_pane: false`.
  - Each envelope shows its matching `seat` field. Five distinct seats return.
  - Each seat writes a full report file at `report_path`.
  - Final pane count equals the baseline count.
  - Total elapsed time stays below the 10-minute (600 s) budget.
- **Ceiling timeout drops and names late seat (ceiling-timeout outcome).**
  Start one seat with `--ceiling 1` and a long task:
  `printf "Write an exhaustive 5000-word history of mathematics." | bash .bee/verify/verify-app/control-bee cli -- herding run --task-file - --json --agent "pi-gpt-5.6-luna" --no-pane --seat "hat-risks" --ceiling 1`
  Assert the command output:
  - Exit code is 1.
  - Envelope shows `outcome: "timed_out_ceiling"`.
  - Envelope shows `seat: "hat-risks"`.
  - Envelope shows `pane_id: null` and `closed_pane: false`.
  - Envelope shows `retryable: false`.

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

### Evidence, second run (same sandbox, 2026-09-15, candidate with the detached runner and the worker-column split)

- Leader step log: `/tmp/bee-verify/run/20260915-180023-56873/evidence2/wave-log.md`. Run window: `evidence2/started-at.txt` 11:49:31Z to `evidence2/ended-at.txt` 11:56:55Z.
- Leader herding envelope: `/tmp/bee-verify/run/20260915-180023-56873/evidence2/leader-envelope.json`. After two `stalled` / `recovered` text lines, its JSON line has `"outcome":"done"`, `"pane_id":"w1:p9Z"`, and `"closed_pane":true`.
- Detached launches: the wave log shows three lines at 11:50:35Z, one per seat, each with `"outcome":"detached"`: `job-1789473035217-265277-1` (`hat-facts-gaps`), `job-1789473035218-265281-1` (`hat-alternatives`), `job-1789473035219-265285-1` (`hat-user-impact`). The launch shell exited with no error, and no shell command timed out.
- Hat job records: `/tmp/bee-verify/run/20260915-180023-56873/repo/.bee/mailbox/job-1789473035217-265277-1/`, `…-265281-1/`, and `…-265285-1/`. `job.json` gives `pane_id` `w1:p90`, `w1:pA1`, and `w1:pA2`, and `cwd` the worktree. Each `result-1.json` has `status` `done`. Dispatch log rows 11 to 13 of `<run>/repo/.bee/logs/dispatch.jsonl` carry the same pane ids; row 7 carries the leader pane `w1:p9Z`.
- Hat panes closed. No hat envelope carries `closed_pane`, because a detached runner sends its stdout to null. The proof is the layout captures in `/tmp/bee-verify/run/20260915-180023-56873/evidence2/layout/`, one every 15 s:
  - `06-115046.txt` to `12-115216.txt`: all three hat panes are open.
  - `w1:pA2` (`result-1.json` 11:52:21Z) is gone from `13-115231.txt`. `w1:pA1` (11:52:44Z) is gone from `15-115301.txt`. `w1:p90` (11:53:38Z) is gone from `18-115346.txt`.
  - The leader `w1:p9Z` is gone from `31-115701.txt`. `99-after.txt` is byte-identical to `00-before.txt` (`cmp`): only `w1:p3A` and `w1:p9Y` are open.
  - `herding-status-after.txt` lists the three hat jobs and the leader with `status=done`.
- Worker column: in `07-115101.txt` the main pane `w1:p3A` stays at `x=0`, width 95, height 45, as in `00-before.txt`. The leader `w1:p9Z` sits at `x=95, y=23` under `w1:p9Y`. The hats `w1:pA2` (`y=26`), `w1:pA1` (`y=29`), and `w1:p90` (`y=34`) are all at `x=95`. Every `down` split after `split_1_1` starts at the leader's rect (`x=95, y=23`), so each hat pane was split from the leader worker pane, never from the main pane.
- Budget: launch 11:50:35Z, `result-1.json` at 11:52:21Z, 11:52:44Z, and 11:53:38Z. That is 3 min 3 s.

### Evidence, third run: five-seat no-pane wave and drop-and-name (run 20260919-073214-2866155, 2026-09-19, bee 2.41.3 candidate, Pi 0.85.1)

- Baseline pane count: `bash .bee/verify/verify-app/control-bee cli -- herding pane list` showed 27 panes (`evidence/024-bee-herding-pane-list.out`).
- Parallel execution: five hat seats started at the same time with `herding run --no-pane --seat <seat> --ceiling 600`.
- During execution: `herding pane list` showed 27 panes (`evidence/028-bee-herding-pane-list.out`). Zero tmux panes opened.
- Post-run pane count: 27 panes (`evidence/029-bee-herding-pane-list.out`).
- Total elapsed time: 12.62 s (budget 600 s).
- Distinct seat returns: all five envelopes showed `outcome: "done"`, `pane_id: null`, `closed_pane: false`, and their seat name:
  - `hat-facts-gaps`: job `job-1789781946966-3059014-1`, `report_path: .../job-1789781946966-3059014-1/report-1.md`
  - `hat-alternatives`: job `job-1789781946967-3059019-1`, `report_path: .../job-1789781946967-3059019-1/report-1.md`
  - `hat-user-impact`: job `job-1789781946966-3059015-1`, `report_path: .../job-1789781946966-3059015-1/report-1.md`
  - `hat-risks`: job `job-1789781946966-3059018-1`, `report_path: .../job-1789781946966-3059018-1/report-1.md`
  - `hat-value`: job `job-1789781946967-3059020-1`, `report_path: .../job-1789781946967-3059020-1/report-1.md`
- Forced ceiling timeout: job `job-1789781937693-3058161-1` started with `--seat "hat-risks" --ceiling 1`.
  - Exit code is 1.
  - Envelope: `{"job_id":"job-1789781937693-3058161-1","seat":"hat-risks","outcome":"timed_out_ceiling","pane_id":null,"closed_pane":false,"dry_run":false,"retryable":false}`.
  - The runner stopped the child at the 1-second ceiling. The runner returned `outcome: "timed_out_ceiling"` and kept `seat: "hat-risks"`.

## Gotchas

- **A launcher timeout no longer leaves a hat pane open (second run).** In the first run the leader put `& wait` after the backgrounded launches, and its 30-second bash timeout ended the launcher. Pi kills the whole process group on a timeout, so the runner died before its pane close. Now an `--inbox-session` run detaches into its own process group and prints `outcome detached` at once. In the second run no shell timed out, and every hat pane closed after its result (see the second-run evidence). A leader must still not `wait` on the detached jobs.
- **Stacked worker panes can get too small.** Each hat splits the caller's pane downward, and nothing sets a minimum pane height. In `evidence2/layout/07-115101.txt` the leader `w1:p9Z` and the hat `w1:pA2` have a height of 3 rows. Tracked in the backlog.
- **A native Claude subagent cannot drive this.** Under worktree isolation, a Claude subagent cannot type into a Pi pane in a `/tmp` sandbox. The isolation guard refuses `herdr agent prompt` with task text and `control-bee sh` scripts that aim outside the worktree. Start the leader with `bee herding run --cwd <sandbox worktree>`, or drive from a session rooted in the sandbox.
- The result-inbox folder is empty after the drain. Read the seat rows from the Pi session log, not from the marker folder.
- `result-1.json` in the mailbox carries no `seat` field. The seat rides the inbox marker, the `herding run --json` envelope, and the injected block.
- A worktree's `.bee/bin/bee` is a symlink into the main sandbox. Vendor the candidate into the main sandbox store; copying a binary into the worktree path writes through the link.
- Hats spend real model calls on the configured agents. A missing `herding.agents` entry for a `team.pi` agent refuses the run and lists the known agent names.
- A seat that exceeds the ceiling stops and returns `timed_out_ceiling`. The result envelope keeps the `seat` label.
- Running hat seats with `--no-pane` executes workers as child processes. It does not open tmux panes (`pane_id: null`).
